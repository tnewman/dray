use std::pin::Pin;
use std::sync::Arc;

use super::handle::HandleManager;
use super::Storage;
use super::StorageFactory;
use crate::protocol::file_attributes::FileAttributes;
use crate::protocol::response::name::File;
use crate::ssh_keys;
use anyhow::Result;
use async_trait::async_trait;

use chrono::{DateTime, TimeZone, Utc};
use rusoto_core::Region;
use rusoto_s3::HeadBucketRequest;
use rusoto_s3::{
    CommonPrefix, GetObjectRequest, HeadObjectOutput, ListObjectsV2Output, ListObjectsV2Request,
    Object, S3Client, S3,
};
use rusoto_s3::{HeadObjectError, HeadObjectRequest};
use serde::Deserialize;
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
    handle_manager: HandleManager<Pin<Box<dyn AsyncRead + Send + Sync>>, String, DirHandle>,
}

impl S3Storage {
    pub fn new(s3_client: S3Client, bucket: String) -> S3Storage {
        S3Storage {
            s3_client,
            bucket,
            handle_manager: HandleManager::new(),
        }
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

    async fn get_file_metadata(&self, key: String) -> Result<File> {
        let head_object = self
            .s3_client
            .head_object(HeadObjectRequest {
                bucket: self.bucket.clone(),
                key: key.clone(),
                ..Default::default()
            })
            .await?;

        Ok(map_head_object_to_file(&key, &head_object))
    }

    async fn open_read_handle(&self, key: String) -> Result<String> {
        todo!()
    }

    async fn read_data(&self, handle: &str) -> Result<Vec<u8>> {
        todo!()
    }

    async fn open_write_handle(&self, key: String) -> Result<String> {
        todo!()
    }

    async fn write_data(&self, handle: &str, data: bytes::Bytes) -> Result<()> {
        todo!()
    }

    async fn open_dir_handle(&self, prefix: String) -> Result<String> {
        Ok(self
            .handle_manager
            .create_dir_handle(DirHandle {
                prefix,
                continuation_token: None,
                is_eof: false,
            })
            .await)
    }

    async fn close_handle(&self, handle: &str) -> Result<()> {
        self.handle_manager.remove_handle(handle).await;
        Ok(())
    }

    async fn rename_file(&self, current: String, new: String) {
        todo!("TODO: Rename object {} to {}", current, new)
    }

    async fn remove_file(&self, key: String) {
        todo!("TODO: Remove object {}", key)
    }
}

pub struct DirHandle {
    prefix: String,
    continuation_token: Option<String>,
    is_eof: bool,
}

fn get_home(user: &str) -> String {
    format!("/home/{}", user)
}

fn get_s3_key(key: String) -> String {
    let key = key[1..key.len()].to_string();
    key
}

fn get_s3_prefix(prefix: String) -> String {
    let prefix = match "".eq(&prefix) {
        true => String::from("/"),
        false => format!("{}/", prefix[1..prefix.len()].to_string()),
    };
    prefix
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
}
