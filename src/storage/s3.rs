use super::handle::HandleManager;
use super::Storage;
use super::StorageFactory;
use crate::error::Error;
use crate::protocol::file_attributes::FileAttributes;
use crate::protocol::response::name::File;
use crate::ssh_keys;
use anyhow::Result;
use async_trait::async_trait;
use bytes::BufMut;
use chrono::{DateTime, TimeZone, Utc};
use rusoto_core::ByteStream;
use rusoto_core::Region;
use rusoto_s3::CompleteMultipartUploadRequest;
use rusoto_s3::CompletedMultipartUpload;
use rusoto_s3::CompletedPart;
use rusoto_s3::CopyObjectRequest;
use rusoto_s3::CreateMultipartUploadOutput;
use rusoto_s3::CreateMultipartUploadRequest;
use rusoto_s3::DeleteObjectRequest;
use rusoto_s3::HeadBucketRequest;
use rusoto_s3::UploadPartRequest;
use rusoto_s3::{
    CommonPrefix, GetObjectRequest, HeadObjectOutput, ListObjectsV2Output, ListObjectsV2Request,
    Object, S3Client, S3,
};
use rusoto_s3::{HeadObjectError, HeadObjectRequest};
use serde::Deserialize;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

#[derive(Deserialize, Debug)]
pub struct S3Config {
    #[serde(rename(deserialize = "s3_endpoint_name"))]
    pub endpoint_name: Option<String>,

    #[serde(default = "get_default_endpoint_region")]
    pub endpoint_region: String,

    #[serde(rename(deserialize = "s3_bucket"))]
    pub bucket: String,
}

pub struct S3StorageFactory {
    s3_client: S3Client,
    bucket: String,
}

impl S3StorageFactory {
    pub fn new(s3_config: &S3Config) -> S3StorageFactory {
        let region = match &s3_config.endpoint_name {
            Some(endpoint_name) => Region::Custom {
                name: s3_config.endpoint_region.clone(),
                endpoint: endpoint_name.clone(),
            },
            None => Region::default(),
        };

        S3StorageFactory {
            s3_client: S3Client::new(region),
            bucket: s3_config.bucket.clone(),
        }
    }
}

#[async_trait]
impl StorageFactory for S3StorageFactory {
    fn create_storage(&self) -> Arc<dyn Storage> {
        Arc::new(S3Storage::new(self.s3_client.clone(), self.bucket.clone()))
    }
}

pub struct S3Storage {
    s3_client: S3Client,
    bucket: String,
    handle_manager: HandleManager<Pin<Box<dyn AsyncRead + Send + Sync>>, WriteHandle, DirHandle>,
}

impl S3Storage {
    pub fn new(s3_client: S3Client, bucket: String) -> S3Storage {
        S3Storage {
            s3_client,
            bucket,
            handle_manager: HandleManager::new(),
        }
    }

    async fn complete_part_upload(
        &self,
        write_handle: &mut tokio::sync::MutexGuard<'_, WriteHandle>,
    ) -> Result<()> {
        let part_number = (write_handle.completed_parts.len() as i64) + 1;

        let upload_part_response = self
            .s3_client
            .upload_part(UploadPartRequest {
                bucket: self.bucket.clone(),
                key: write_handle.key.clone(),
                upload_id: write_handle.upload_id.clone(),
                part_number,
                body: Some(ByteStream::from(write_handle.buffer.clone())),
                ..Default::default()
            })
            .await?;

        write_handle.completed_parts.push(CompletedPart {
            e_tag: upload_part_response.e_tag,
            part_number: Some(part_number),
        });

        write_handle.buffer.clear();

        Ok(())
    }
}

#[async_trait]
impl Storage for S3Storage {
    fn get_home(&self, user: &str) -> String {
        get_home(user)
    }

    async fn health_check(&self) -> Result<()> {
        self.s3_client
            .head_bucket(HeadBucketRequest {
                bucket: self.bucket.clone(),
                ..Default::default()
            })
            .await?;

        Ok(())
    }

    async fn get_authorized_keys_fingerprints(&self, user: &str) -> Result<Vec<String>> {
        let authorized_keys_key = format!(".ssh/{}/authorized_keys", user);

        let object = self
            .s3_client
            .get_object(GetObjectRequest {
                bucket: self.bucket.clone(),
                key: authorized_keys_key,
                ..Default::default()
            })
            .await?;

        let body = match object.body {
            Some(body) => body,
            None => return Ok(vec![]),
        };

        let mut buffer = String::new();
        body.into_async_read().read_to_string(&mut buffer).await?;

        Ok(ssh_keys::parse_authorized_keys(&buffer))
    }

    async fn open_dir_handle(&self, dir_name: String) -> Result<String> {
        Ok(self
            .handle_manager
            .create_dir_handle(DirHandle {
                prefix: dir_name,
                continuation_token: None,
                is_eof: false,
            })
            .await)
    }

    async fn read_dir(&self, handle: &str) -> Result<Vec<File>> {
        let dir_handle = match self.handle_manager.get_dir_handle(&handle).await {
            Some(dir_handle) => dir_handle,
            None => return Err(anyhow::anyhow!("Missing directory handle.")),
        };

        let mut dir_handle = dir_handle.lock().await;

        if dir_handle.is_eof {
            return Ok(Vec::new());
        }

        let prefix = get_s3_prefix(dir_handle.prefix.clone());

        let objects = self
            .s3_client
            .list_objects_v2(ListObjectsV2Request {
                bucket: self.bucket.clone(),
                prefix: Some(prefix),
                continuation_token: dir_handle.continuation_token.clone(),
                delimiter: Some("/".to_owned()),
                ..Default::default()
            })
            .await?;

        dir_handle.continuation_token = objects.next_continuation_token.clone();
        dir_handle.is_eof = objects.next_continuation_token.is_none();

        Ok(map_list_objects_to_files(objects))
    }

    async fn create_dir(&self, _prefix: String) -> Result<()> {
        /*
            S3 does not support creating empty prefixes. The prefix is created when the
            first object is added to it. This operation is a NO-OP to allow GUI-based
            SFTP clients to make it appear that a directory has been created.
        */
        Ok(())
    }

    async fn rename_dir(&self, current: String, new: String) {
        todo!("TODO: Rename prefix {} to {}", current, new)
    }

    async fn remove_dir(&self, prefix: String) {
        todo!("TODO: Remove prefix {}", prefix)
    }

    async fn file_exists(&self, key: String) -> Result<bool> {
        let head_object_response = self
            .s3_client
            .head_object(HeadObjectRequest {
                bucket: self.bucket.clone(),
                key: get_s3_key(key),
                ..Default::default()
            })
            .await;

        match head_object_response {
            Ok(_) => Ok(true),
            Err(error) => match error {
                rusoto_core::RusotoError::Unknown(http_response) => {
                    if 404 == http_response.status.as_u16() {
                        Ok(false)
                    } else {
                        Err(anyhow::Error::from(rusoto_core::RusotoError::<
                            HeadObjectError,
                        >::Unknown(
                            http_response
                        )))
                    }
                }
                _ => Err(anyhow::Error::from(error)),
            },
        }
    }

    async fn get_file_metadata(&self, file_name: String) -> Result<File> {
        let head_object = self
            .s3_client
            .head_object(HeadObjectRequest {
                bucket: self.bucket.clone(),
                key: file_name.clone(),
                ..Default::default()
            })
            .await?;

        Ok(map_head_object_to_file(&file_name, &head_object))
    }

    async fn open_read_handle(&self, file_name: String) -> Result<String> {
        let read_response = self
            .s3_client
            .get_object(GetObjectRequest {
                bucket: self.bucket.clone(),
                key: file_name,
                ..Default::default()
            })
            .await?;

        let read_stream = read_response
            .body
            .ok_or(Error::ServerError)?
            .into_async_read();

        Ok(self
            .handle_manager
            .create_read_handle(Box::pin(read_stream))
            .await)
    }

    async fn read_data(&self, handle: &str, len: u32) -> Result<Vec<u8>> {
        let read_handle = match self.handle_manager.get_read_handle(&handle).await {
            Some(dir_handle) => dir_handle,
            None => return Err(anyhow::anyhow!("Missing read handle.")),
        };

        let mut buffer = Vec::with_capacity(len as usize);

        read_handle
            .lock()
            .await
            .as_mut()
            .take(len as u64)
            .read_to_end(&mut buffer)
            .await?;

        Ok(buffer)
    }

    async fn open_write_handle(&self, file_name: String) -> Result<String> {
        let multipart_response = self
            .s3_client
            .create_multipart_upload(CreateMultipartUploadRequest {
                bucket: self.bucket.clone(),
                key: file_name,
                ..Default::default()
            })
            .await?;

        let write_handle = map_create_multipart_response_to_write_handle(multipart_response)?;

        Ok(self.handle_manager.create_write_handle(write_handle).await)
    }

    async fn write_data(&self, handle: &str, data: bytes::Bytes) -> Result<()> {
        let write_handle = match self.handle_manager.get_write_handle(&handle).await {
            Some(dir_handle) => dir_handle,
            None => return Err(anyhow::anyhow!("Missing write handle.")),
        };

        let mut write_handle = write_handle.lock().await;

        write_handle.buffer.put(data);

        if write_handle.buffer.len() > 10000000 {
            self.complete_part_upload(&mut write_handle).await?;
            write_handle.buffer.clear();
        };

        Ok(())
    }

    async fn close_handle(&self, handle: &str) -> Result<()> {
        if let Some(write_handle) = self.handle_manager.get_write_handle(handle).await {
            let mut write_handle = write_handle.lock().await;

            self.complete_part_upload(&mut write_handle).await?;

            self.s3_client
                .complete_multipart_upload(CompleteMultipartUploadRequest {
                    bucket: self.bucket.clone(),
                    key: write_handle.key.clone(),
                    upload_id: write_handle.upload_id.clone(),
                    multipart_upload: Some(CompletedMultipartUpload {
                        parts: Some(write_handle.completed_parts.clone()),
                    }),
                    ..Default::default()
                })
                .await?;
        }

        self.handle_manager.remove_handle(handle).await;
        Ok(())
    }

    async fn rename_file(&self, current: String, new: String) -> Result<()> {
        self.s3_client
            .copy_object(CopyObjectRequest {
                bucket: self.bucket.clone(),
                copy_source: get_s3_copy_source(&self.bucket, &current),
                key: new,
                ..Default::default()
            })
            .await?;

        self.remove_file(current).await?;

        Ok(())
    }

    async fn remove_file(&self, file_name: String) -> Result<()> {
        self.s3_client
            .delete_object(DeleteObjectRequest {
                bucket: self.bucket.clone(),
                key: file_name,
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}

struct DirHandle {
    prefix: String,
    continuation_token: Option<String>,
    is_eof: bool,
}

struct WriteHandle {
    key: String,
    upload_id: String,
    completed_parts: Vec<CompletedPart>,
    buffer: Vec<u8>,
}

fn get_home(user: &str) -> String {
    format!("/home/{}", user)
}

fn get_s3_key(key: String) -> String {
    key[1..key.len()].to_string()
}

fn get_s3_prefix(dir_name: String) -> String {
    let prefix = match "".eq(&dir_name) {
        true => String::from("/"),
        false => format!("{}/", dir_name[1..dir_name.len()].to_string()),
    };
    prefix
}

fn get_s3_copy_source(bucket: &str, key: &str) -> String {
    format!("{}/{}", bucket, key)
}

fn map_list_objects_to_files(list_objects: ListObjectsV2Output) -> Vec<File> {
    let files = list_objects.contents.unwrap_or_default();

    let directories = list_objects.common_prefixes.unwrap_or_default();

    let mapped_files = files.iter().map(|object| map_object_to_file(object));

    let mapped_dirs = directories.iter().map(|prefix| map_prefix_to_file(prefix));

    mapped_dirs.chain(mapped_files).collect()
}

fn map_object_to_file(object: &Object) -> File {
    let key = match &object.key {
        Some(key) => key,
        None => "",
    };

    let mut key_pieces = key.rsplit('/');
    let file_name = key_pieces.next().unwrap_or("");

    File {
        file_name: file_name.to_string(),
        file_attributes: FileAttributes {
            size: object.size.map(|size| size as u64),
            uid: None,
            gid: None,
            permissions: Some(0o100777),
            atime: None,
            mtime: map_rfc3339_to_epoch(object.last_modified.as_ref()),
        },
    }
}

fn map_prefix_to_file(prefix: &CommonPrefix) -> File {
    let prefix = match prefix.prefix {
        Some(ref prefix) => {
            let mut prefix = prefix.to_string();
            prefix.pop(); // strip trailing /
            format!("/{}", prefix)
        }
        None => "".to_owned(),
    };

    let mut prefix_pieces = prefix.rsplit('/');
    let file_name = prefix_pieces.next().unwrap_or("");

    File {
        file_name: file_name.to_string(),
        file_attributes: FileAttributes {
            size: None,
            uid: None,
            gid: None,
            permissions: Some(0o40777),
            atime: None,
            mtime: None,
        },
    }
}

fn map_head_object_to_file(key: &str, head_object: &HeadObjectOutput) -> File {
    let mut key_pieces = key.rsplit('/');
    let file_name = key_pieces.next().unwrap_or("");

    File {
        file_name: file_name.to_string(),
        file_attributes: FileAttributes {
            size: head_object
                .content_length
                .map(|content_length| content_length as u64),
            uid: None,
            gid: None,
            permissions: Some(0o100777),
            atime: None,
            mtime: None,
        },
    }
}

fn map_rfc3339_to_epoch(rfc3339: Option<&String>) -> Option<u32> {
    rfc3339.map(|last_modified| {
        last_modified
            .parse::<DateTime<Utc>>()
            .unwrap_or_else(|_e| Utc.timestamp(0, 0))
            .timestamp() as u32
    })
}

fn map_create_multipart_response_to_write_handle(
    create_multipart_response: CreateMultipartUploadOutput,
) -> Result<WriteHandle> {
    let upload_id = match create_multipart_response.upload_id {
        Some(upload_id) => Ok(upload_id),
        None => Err(anyhow::anyhow!("Missing upload id.")),
    }?;

    let key = match create_multipart_response.key {
        Some(key) => Ok(key),
        None => Err(anyhow::anyhow!("Missing key.")),
    }?;

    Ok(WriteHandle {
        key,
        upload_id,
        completed_parts: Vec::new(),
        buffer: Vec::with_capacity(5000000),
    })
}

fn get_default_endpoint_region() -> String {
    String::from("custom")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_home_returns_users_home_directory() {
        assert_eq!("/home/test", get_home("test"));
    }

    #[test]
    fn test_get_default_endpoint_region() {
        assert_eq!("custom", get_default_endpoint_region());
    }

    #[test]
    fn test_get_s3_key_converts_unix_absolute_directory() {
        assert_eq!(
            String::from("test/file1.txt"),
            get_s3_key(String::from("/test/file1.txt"))
        )
    }

    #[test]
    fn test_get_s3_prefix_converts_unix_absolute_directory() {
        assert_eq!(String::from("test/"), get_s3_prefix(String::from("/test")))
    }

    #[test]
    fn test_get_s3_copy_source() {
        assert_eq!("bucket/key", get_s3_copy_source("bucket", "key"))
    }

    #[test]
    fn test_get_s3_prefix_converts_blank_directory() {
        assert_eq!(String::from("/"), get_s3_prefix(String::from("")))
    }

    #[test]
    fn test_map_list_objects_to_files() {
        let list_objects = ListObjectsV2Output {
            common_prefixes: Some(vec![CommonPrefix {
                prefix: Some("users/test/subfolder/".to_owned()),
                ..Default::default()
            }]),
            contents: Some(vec![Object {
                key: Some("users/test/file.txt".to_owned()),
                size: Some(1),
                last_modified: Some(String::from("2014-11-28T12:00:09Z")),
                ..Default::default()
            }]),
            continuation_token: Some(String::from("token")),
            ..Default::default()
        };

        let result = map_list_objects_to_files(list_objects);

        assert_eq!(2, result.len());
        assert_eq!(
            File {
                file_name: "subfolder".to_owned(),
                file_attributes: FileAttributes {
                    size: None,
                    gid: None,
                    uid: None,
                    permissions: Some(0o40777),
                    atime: None,
                    mtime: None,
                }
            },
            result[0]
        );
        assert_eq!(
            File {
                file_name: "file.txt".to_owned(),
                file_attributes: FileAttributes {
                    size: Some(1),
                    gid: None,
                    uid: None,
                    permissions: Some(0o100777),
                    atime: None,
                    mtime: Some(1417176009),
                }
            },
            result[1]
        );
    }

    #[test]
    fn test_map_list_objects_to_files_with_missing_data() {
        let list_objects = ListObjectsV2Output {
            ..Default::default()
        };

        let result = map_list_objects_to_files(list_objects);

        assert_eq!(0, result.len());
    }

    #[test]
    fn test_map_object_to_file_with_missing_data() {
        let object = Object {
            ..Default::default()
        };

        assert_eq!(
            File {
                file_name: "".to_owned(),
                file_attributes: FileAttributes {
                    size: None,
                    gid: None,
                    uid: None,
                    permissions: Some(0o100777),
                    atime: None,
                    mtime: None,
                }
            },
            map_object_to_file(&object)
        );
    }

    #[test]
    fn test_map_prefix_to_file_with_missing_data() {
        let prefix = CommonPrefix { prefix: None };

        assert_eq!(
            File {
                file_name: "".to_owned(),
                file_attributes: FileAttributes {
                    size: None,
                    gid: None,
                    uid: None,
                    permissions: Some(0o40777),
                    atime: None,
                    mtime: None,
                }
            },
            map_prefix_to_file(&prefix)
        );
    }

    #[test]
    fn test_map_head_object_to_file() {
        let head_object = HeadObjectOutput {
            ..Default::default()
        };

        assert_eq!(
            File {
                file_name: "file".to_owned(),
                file_attributes: FileAttributes {
                    size: None,
                    gid: None,
                    uid: None,
                    permissions: Some(0o100777),
                    atime: None,
                    mtime: None,
                }
            },
            map_head_object_to_file("file", &head_object)
        );
    }

    #[test]
    fn test_map_rfc3339_to_epoch_maps_valid_date() {
        assert_eq!(
            Some(1417176009 as u32),
            map_rfc3339_to_epoch(Some(String::from("2014-11-28T12:00:09Z")).as_ref())
        );
    }

    #[test]
    fn test_map_rfc3339_to_epoch_maps_none_to_unix_epoch() {
        assert_eq!(None, map_rfc3339_to_epoch(None));
    }

    #[test]
    fn test_map_rfc3339_to_epoch_maps_invalid_date_to_unix_epoch() {
        assert_eq!(
            Some(0 as u32),
            map_rfc3339_to_epoch(Some(String::from("invalid")).as_ref())
        );
    }

    #[test]
    fn test_map_create_multipart_response_to_write_handle() {
        let multipart_response = CreateMultipartUploadOutput {
            upload_id: Some(String::from("id")),
            key: Some(String::from("key")),
            ..Default::default()
        };

        let write_handle =
            map_create_multipart_response_to_write_handle(multipart_response).unwrap();

        assert_eq!("id", &write_handle.upload_id);
        assert_eq!("key", &write_handle.key);
        assert_eq!(0, write_handle.completed_parts.len());
        assert_eq!(5000000, write_handle.buffer.capacity());
    }

    #[test]
    fn test_map_create_multipart_response_to_write_handle_with_missing_multipart_id() {
        let multipart_response = CreateMultipartUploadOutput {
            key: Some(String::from("key")),
            ..Default::default()
        };

        assert!(map_create_multipart_response_to_write_handle(multipart_response).is_err());
    }

    #[test]
    fn test_map_create_multipart_response_to_write_handle_with_missing_key() {
        let multipart_response = CreateMultipartUploadOutput {
            upload_id: Some(String::from("id")),
            ..Default::default()
        };

        assert!(map_create_multipart_response_to_write_handle(multipart_response).is_err());
    }
}
