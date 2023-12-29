use super::handle::HandleManager;
use super::Storage;
use super::StorageFactory;
use crate::error::Error;
use crate::protocol::file_attributes::FileAttributes;
use crate::protocol::response::name::File;
use crate::ssh_keys;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::CompletedMultipartUpload;
use aws_sdk_s3::types::CompletedPart;
use bytes::BufMut;
use chrono::{DateTime, TimeZone, Utc};
use log::{error, info};
use rusoto_core::Region;
use rusoto_core::RusotoError;
use rusoto_s3::CreateBucketRequest;
use rusoto_s3::CreateMultipartUploadOutput;
use rusoto_s3::CreateMultipartUploadRequest;
use rusoto_s3::HeadBucketRequest;
use rusoto_s3::HeadObjectRequest;
use rusoto_s3::PutObjectRequest;
use rusoto_s3::{
    CommonPrefix, GetObjectRequest, HeadObjectOutput, ListObjectsV2Output, ListObjectsV2Request,
    Object, S3Client, S3,
};
use serde::Deserialize;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

#[derive(Clone, Deserialize, Debug)]
pub struct S3Config {
    #[serde(rename(deserialize = "s3_endpoint_name"))]
    pub endpoint_name: Option<String>,

    #[serde(default = "get_default_endpoint_region")]
    pub endpoint_region: String,

    #[serde(rename(deserialize = "s3_bucket"))]
    pub bucket: String,
}

pub struct S3StorageFactory {
    legacy_s3_client: S3Client,
    s3_client: aws_sdk_s3::Client,
    bucket: String,
}

impl S3StorageFactory {
    pub async fn new(s3_config: &S3Config) -> S3StorageFactory {
        let legacy_region = match &s3_config.endpoint_name {
            Some(endpoint_name) => Region::Custom {
                name: s3_config.endpoint_region.clone(),
                endpoint: endpoint_name.clone(),
            },
            None => Region::default(),
        };

        let mut config_loader = aws_config::defaults(BehaviorVersion::latest());

        if let Some(endpoint_name) = &s3_config.endpoint_name {
            config_loader = config_loader.endpoint_url(endpoint_name);
        };

        let config = config_loader.load().await;

        let s3_client_builder = aws_sdk_s3::config::Builder::new();

        if config.endpoint_url().is_some() {
            s3_client_builder.force_path_style(true);
        }

        let mut s3_sdk_config = aws_sdk_s3::config::Builder::from(&config);

        if config.endpoint_url().is_some() {
            s3_sdk_config = s3_sdk_config.force_path_style(true);
        }

        let s3_client = aws_sdk_s3::Client::from_conf(s3_sdk_config.build());

        S3StorageFactory {
            s3_client,
            legacy_s3_client: S3Client::new(legacy_region),
            bucket: s3_config.bucket.clone(),
        }
    }
}

#[async_trait]
impl StorageFactory for S3StorageFactory {
    fn create_storage(&self) -> Arc<dyn Storage> {
        Arc::new(S3Storage::new(
            self.s3_client.clone(),
            self.legacy_s3_client.clone(),
            self.bucket.clone(),
        ))
    }
}

pub struct S3Storage {
    s3_client: aws_sdk_s3::Client,
    legacy_s3_client: S3Client,
    bucket: String,
    handle_manager: HandleManager<ReadHandle, WriteHandle, DirHandle>,
}

impl S3Storage {
    pub fn new(
        s3_client: aws_sdk_s3::Client,
        legacy_s3_client: S3Client,
        bucket: String,
    ) -> S3Storage {
        S3Storage {
            s3_client,
            legacy_s3_client,
            bucket,
            handle_manager: HandleManager::new(),
        }
    }

    async fn complete_part_upload(
        &self,
        write_handle: &mut tokio::sync::MutexGuard<'_, WriteHandle>,
    ) -> Result<(), Error> {
        let part_number = (write_handle.completed_parts.len() as i32) + 1;

        let upload_part_response = self
            .s3_client
            .upload_part()
            .bucket(&self.bucket)
            .key(&write_handle.key)
            .upload_id(&write_handle.upload_id)
            .part_number(part_number)
            .body(ByteStream::from(write_handle.buffer.clone()))
            .send()
            .await
            .map_err(aws_sdk_s3::Error::from)
            .map_err(map_err)?;

        write_handle.completed_parts.push(
            CompletedPart::builder()
                .e_tag(upload_part_response.e_tag().unwrap_or_default())
                .part_number(part_number)
                .build(),
        );

        write_handle.buffer.clear();

        Ok(())
    }

    async fn get_directory_metadata(&self, folder_name: &str) -> Result<File, Error> {
        let list_objects_output = self
            .s3_client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(get_s3_prefix(folder_name))
            .delimiter("/")
            .send()
            .await
            .map_err(aws_sdk_s3::Error::from)
            .map_err(map_err)?;

        map_list_objects_to_directory(list_objects_output)
    }

    async fn rename_file(&self, current: String, new: String) -> Result<(), Error> {
        self.s3_client
            .copy_object()
            .bucket(&self.bucket)
            .copy_source(get_s3_copy_source(&self.bucket, &current))
            .key(&new)
            .send()
            .await
            .map_err(aws_sdk_s3::Error::from)
            .map_err(map_err)?;

        self.remove_file(current).await?;

        Ok(())
    }

    async fn rename_dir(&self, current: String, new: String) -> Result<(), Error> {
        let current_prefix = get_s3_prefix(&current);
        let new_prefix = get_s3_prefix(&new);

        let mut continuation_token = None;

        loop {
            let objects = self
                .legacy_s3_client
                .list_objects_v2(ListObjectsV2Request {
                    bucket: self.bucket.clone(),
                    prefix: Some(current_prefix.clone()),
                    continuation_token: continuation_token.clone(),
                    delimiter: None,
                    ..Default::default()
                })
                .await
                .map_err(map_legacy_err)?;

            continuation_token = objects.continuation_token;

            if let Some(contents) = objects.contents {
                let keys = contents.into_iter().filter_map(|content| content.key);

                for key in keys {
                    let destination = key.replace(&current_prefix, &new_prefix);

                    self.rename_file(key, destination).await?;
                }
            }

            if continuation_token.is_none() {
                break;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn init(&self) -> Result<(), Error> {
        let head_response = self
            .legacy_s3_client
            .head_bucket(HeadBucketRequest {
                bucket: self.bucket.clone(),
                ..Default::default()
            })
            .await;

        match head_response {
            Ok(_) => Ok(()),
            Err(_) => {
                self.legacy_s3_client
                    .create_bucket(CreateBucketRequest {
                        bucket: self.bucket.clone(),
                        ..Default::default()
                    })
                    .await
                    .map_err(map_legacy_err)?;
                Ok(())
            }
        }
    }

    fn get_home(&self, user: &str) -> String {
        get_home(user)
    }

    async fn health_check(&self) -> Result<(), Error> {
        info!("Running health check for S3 Bucket {}", self.bucket);

        let result = self
            .legacy_s3_client
            .head_bucket(HeadBucketRequest {
                bucket: self.bucket.clone(),
                ..Default::default()
            })
            .await
            .map_err(map_legacy_err);

        match result {
            Ok(_) => {
                info!(
                    "Successfully completed health check for S3 Bucket {}",
                    self.bucket
                );
                Ok(())
            }
            Err(error) => {
                error!(
                    "Failed to complete health check for S3 Bucket {}: {}",
                    self.bucket, error
                );
                Err(error)
            }
        }
    }

    async fn get_authorized_keys_fingerprints(&self, user: &str) -> Result<Vec<String>, Error> {
        let authorized_keys_key = format!(".ssh/{}/authorized_keys", user);

        let object = self
            .legacy_s3_client
            .get_object(GetObjectRequest {
                bucket: self.bucket.clone(),
                key: authorized_keys_key,
                ..Default::default()
            })
            .await
            .map_err(map_legacy_err)?;

        let body = match object.body {
            Some(body) => body,
            None => return Ok(vec![]),
        };

        let mut buffer = String::new();
        body.into_async_read().read_to_string(&mut buffer).await?;

        Ok(ssh_keys::parse_authorized_keys(&buffer))
    }

    async fn open_dir_handle(&self, dir_name: String) -> Result<String, Error> {
        let prefix = get_s3_prefix(&dir_name);

        self.handle_manager
            .create_dir_handle(DirHandle {
                prefix,
                continuation_token: None,
                is_eof: false,
            })
            .await
    }

    async fn read_dir(&self, handle: &str) -> Result<Vec<File>, Error> {
        let dir_handle = match self.handle_manager.get_dir_handle(handle).await {
            Some(dir_handle) => dir_handle,
            None => return Err(Error::Failure("Missing directory handle.".to_string())),
        };

        let mut dir_handle = dir_handle.lock().await;

        if dir_handle.is_eof {
            return Ok(Vec::new());
        }

        let prefix = get_s3_prefix(&dir_handle.prefix);

        let objects = self
            .legacy_s3_client
            .list_objects_v2(ListObjectsV2Request {
                bucket: self.bucket.clone(),
                prefix: Some(prefix),
                continuation_token: dir_handle.continuation_token.clone(),
                delimiter: Some("/".to_owned()),
                ..Default::default()
            })
            .await
            .map_err(map_legacy_err)?;

        dir_handle.continuation_token = objects.next_continuation_token.clone();
        dir_handle.is_eof = objects.next_continuation_token.is_none();

        Ok(map_list_objects_to_files(objects))
    }

    async fn create_dir(&self, dir_name: String) -> Result<(), Error> {
        /*
            S3 does not support creating empty prefixes. A marker file must be added to preserve empty
            directories until the directories are explicitly deleted.
        */
        self.legacy_s3_client
            .put_object(PutObjectRequest {
                bucket: self.bucket.clone(),
                key: get_s3_folder_marker(&dir_name),
                ..Default::default()
            })
            .await
            .map_err(map_legacy_err)?;

        Ok(())
    }

    async fn remove_dir(&self, dir_name: String) -> Result<(), Error> {
        let prefix = get_s3_prefix(&dir_name);
        let mut continuation_token = None;

        loop {
            let objects = self
                .legacy_s3_client
                .list_objects_v2(ListObjectsV2Request {
                    bucket: self.bucket.clone(),
                    prefix: Some(prefix.clone()),
                    continuation_token: continuation_token.clone(),
                    delimiter: None,
                    ..Default::default()
                })
                .await
                .map_err(map_legacy_err)?;

            continuation_token = objects.continuation_token;

            if let Some(contents) = objects.contents {
                let keys = contents.into_iter().filter_map(|content| content.key);

                for key in keys {
                    self.remove_file(key).await?;
                }
            }

            if continuation_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    async fn get_file_metadata(&self, file_name: String) -> Result<File, Error> {
        let head_object_response = self
            .legacy_s3_client
            .head_object(HeadObjectRequest {
                bucket: self.bucket.clone(),
                key: file_name.clone(),
                ..Default::default()
            })
            .await
            .map_err(map_legacy_err);

        match head_object_response {
            Ok(head_object_response) => {
                Ok(map_head_object_to_file(&file_name, &head_object_response))
            }
            Err(err) => match err {
                Error::NoSuchFile => self.get_directory_metadata(&file_name).await,
                _ => Err(err),
            },
        }
    }

    async fn get_handle_metadata(&self, handle: &str) -> Result<File, Error> {
        if let Some(read_handle) = self.handle_manager.get_read_handle(handle).await {
            let read_handle = read_handle.lock().await;
            self.get_file_metadata(read_handle.key.to_string()).await
        } else if let Some(write_handle) = self.handle_manager.get_write_handle(handle).await {
            let write_handle = write_handle.lock().await;
            self.get_file_metadata(write_handle.key.to_string()).await
        } else if let Some(dir_handle) = self.handle_manager.get_dir_handle(handle).await {
            let dir_handle = dir_handle.lock().await;
            self.get_file_metadata(dir_handle.prefix.to_string()).await
        } else {
            Err(Error::Failure(format!("Handle {} does not exist!", handle)))
        }
    }

    async fn open_read_handle(&self, file_name: String) -> Result<String, Error> {
        let read_response = self
            .legacy_s3_client
            .get_object(GetObjectRequest {
                bucket: self.bucket.clone(),
                key: file_name.clone(),
                ..Default::default()
            })
            .await
            .map_err(map_legacy_err)?;

        let read_stream = read_response
            .body
            .ok_or_else(|| Error::Storage("Read stream body is not available.".to_string()))?
            .into_async_read();

        self.handle_manager
            .create_read_handle(ReadHandle::new(file_name, Box::pin(read_stream)))
            .await
    }

    async fn read_data(&self, handle: &str, len: u32) -> Result<Vec<u8>, Error> {
        let read_handle = match self.handle_manager.get_read_handle(handle).await {
            Some(dir_handle) => dir_handle,
            None => return Err(Error::Storage("Missing read handle.".to_string())),
        };

        let mut buffer = Vec::with_capacity(len as usize);

        read_handle
            .lock()
            .await
            .async_read
            .as_mut()
            .take(len as u64)
            .read_to_end(&mut buffer)
            .await?;

        Ok(buffer)
    }

    async fn open_write_handle(&self, file_name: String) -> Result<String, Error> {
        let multipart_response = self
            .legacy_s3_client
            .create_multipart_upload(CreateMultipartUploadRequest {
                bucket: self.bucket.clone(),
                key: file_name,
                ..Default::default()
            })
            .await
            .map_err(map_legacy_err)?;

        let write_handle = map_create_multipart_response_to_write_handle(multipart_response)?;

        self.handle_manager.create_write_handle(write_handle).await
    }

    async fn write_data(&self, handle: &str, data: bytes::Bytes) -> Result<(), Error> {
        let write_handle = match self.handle_manager.get_write_handle(handle).await {
            Some(dir_handle) => dir_handle,
            None => return Err(Error::Storage("Missing write handle.".to_string())),
        };

        let mut write_handle = write_handle.lock().await;

        write_handle.buffer.put(data);

        if write_handle.buffer.len() > 10000000 {
            self.complete_part_upload(&mut write_handle).await?;
            write_handle.buffer.clear();
        };

        Ok(())
    }

    async fn close_handle(&self, handle: &str) -> Result<(), Error> {
        if let Some(write_handle) = self.handle_manager.get_write_handle(handle).await {
            let mut write_handle = write_handle.lock().await;

            self.complete_part_upload(&mut write_handle).await?;

            let complete_multipart_upload = CompletedMultipartUpload::builder()
                .set_parts(Some(write_handle.completed_parts.clone()))
                .build();

            self.s3_client
                .complete_multipart_upload()
                .bucket(&self.bucket)
                .key(&write_handle.key)
                .multipart_upload(complete_multipart_upload)
                .upload_id(&write_handle.upload_id)
                .send()
                .await
                .map_err(aws_sdk_s3::Error::from)
                .map_err(map_err)?;
        }

        self.handle_manager.remove_handle(handle).await;
        Ok(())
    }

    async fn remove_file(&self, file_name: String) -> Result<(), Error> {
        self.s3_client
            .delete_object()
            .bucket(&self.bucket)
            .key(&file_name)
            .send()
            .await
            .map_err(aws_sdk_s3::Error::from)
            .map_err(map_err)?;

        Ok(())
    }

    async fn rename(&self, current: String, new: String) -> Result<(), Error> {
        let file = self.get_file_metadata(current.clone()).await?;

        match file.file_attributes.is_dir() {
            true => self.rename_dir(current, new).await,
            false => self.rename_file(current, new).await,
        }?;

        Ok(())
    }
}

struct DirHandle {
    prefix: String,
    continuation_token: Option<String>,
    is_eof: bool,
}

struct ReadHandle {
    key: String,
    async_read: Pin<Box<dyn AsyncRead + Send>>,
}

impl ReadHandle {
    fn new(key: String, async_read: Pin<Box<dyn AsyncRead + Send>>) -> ReadHandle {
        ReadHandle { key, async_read }
    }
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

fn get_s3_prefix(dir_name: &str) -> String {
    if "".eq(dir_name) {
        return String::from("/");
    }

    let prefix_builder = match dir_name.starts_with('/') {
        true => &dir_name[1..dir_name.len()],
        false => dir_name,
    };

    match prefix_builder.ends_with('/') {
        true => prefix_builder.to_string(),
        false => format!("{}/", prefix_builder),
    }
}

fn get_s3_copy_source(bucket: &str, key: &str) -> String {
    format!("{}/{}", bucket, key)
}

fn get_s3_folder_marker(dir_name: &str) -> String {
    let prefix = get_s3_prefix(dir_name);
    format!("{}_$folder$", prefix)
}

fn map_list_objects_to_files(list_objects: ListObjectsV2Output) -> Vec<File> {
    let files = list_objects.contents.unwrap_or_default();

    let directories = list_objects.common_prefixes.unwrap_or_default();

    let mapped_files = files
        .iter()
        .map(map_object_to_file)
        .filter(|file| !file.file_name.ends_with("_$folder$"));

    let mapped_dirs = directories.iter().map(map_prefix_to_file);

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

fn map_list_objects_to_directory(
    list_objects: aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output,
) -> Result<File, Error> {
    let contents = list_objects.contents.unwrap_or_default();

    let prefix = match list_objects.prefix {
        Some(prefix) => prefix,
        None => return Err(Error::NoSuchFile),
    };

    if contents.is_empty() {
        Err(Error::NoSuchFile)
    } else {
        Ok(map_prefix_to_file(&CommonPrefix {
            prefix: Some(prefix),
        }))
    }
}

fn map_legacy_list_objects_to_directory(list_objects: ListObjectsV2Output) -> Result<File, Error> {
    let contents = list_objects.contents.unwrap_or_default();

    let prefix = match list_objects.prefix {
        Some(prefix) => prefix,
        None => return Err(Error::NoSuchFile),
    };

    match contents.is_empty() {
        true => Err(Error::NoSuchFile),
        false => Ok(map_prefix_to_file(&CommonPrefix {
            prefix: Some(prefix),
        })),
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
            .unwrap_or_else(|_e| match Utc.timestamp_opt(0, 0) {
                chrono::LocalResult::Single(timestamp) => timestamp,
                _ => DateTime::<Utc>::MIN_UTC,
            })
            .timestamp() as u32
    })
}

fn map_create_multipart_response_to_write_handle(
    create_multipart_response: CreateMultipartUploadOutput,
) -> Result<WriteHandle, Error> {
    let upload_id = match create_multipart_response.upload_id {
        Some(upload_id) => Ok(upload_id),
        None => Err(Error::Storage("Missing upload id.".to_string())),
    }?;

    let key = match create_multipart_response.key {
        Some(key) => Ok(key),
        None => Err(Error::Storage("Missing key.".to_string())),
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

fn map_err(s3_sdk_error: aws_sdk_s3::Error) -> Error {
    match s3_sdk_error {
        aws_sdk_s3::Error::NoSuchKey(_) => Error::NoSuchFile,
        _ => Error::Storage(s3_sdk_error.to_string()),
    }
}

fn map_legacy_err<E: std::error::Error + 'static>(rusoto_error: RusotoError<E>) -> Error {
    match rusoto_error {
        rusoto_core::RusotoError::Service(error) => {
            if "The specified key does not exist." == error.to_string() {
                Error::NoSuchFile
            } else {
                Error::Storage(error.to_string())
            }
        }
        rusoto_core::RusotoError::Unknown(http_response) => {
            if 404 == http_response.status.as_u16() {
                Error::NoSuchFile
            } else {
                Error::Storage(format!(
                    "{} - {}",
                    http_response.status,
                    http_response.body_as_str()
                ))
            }
        }
        _ => Error::Storage(rusoto_error.to_string()),
    }
}

#[cfg(test)]
mod test {
    use bytes::Bytes;
    use rusoto_core::request::BufferedHttpResponse;
    use rusoto_s3::{GetObjectError, UploadPartError};

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
    fn test_get_s3_prefix_converts_unix_absolute_directory() {
        assert_eq!(String::from("test/"), get_s3_prefix("/test"))
    }

    #[test]
    fn test_get_s3_prefix_converts_unix_absolute_directory_with_trailing_slash() {
        assert_eq!(String::from("test/"), get_s3_prefix("/test/"))
    }

    #[test]
    fn test_get_s3_prefix_converts_blank_directory() {
        assert_eq!("/", get_s3_prefix(""))
    }

    #[test]
    fn test_get_s3_copy_source() {
        assert_eq!("bucket/key", get_s3_copy_source("bucket", "key"))
    }

    #[test]
    fn test_get_s3_folder_marker_appends_folder_marker() {
        assert_eq!(
            "folder/subfolder/_$folder$",
            get_s3_folder_marker("/folder/subfolder")
        );
    }

    #[test]
    fn test_map_list_objects_to_files() {
        let list_objects = ListObjectsV2Output {
            common_prefixes: Some(vec![CommonPrefix {
                prefix: Some("users/test/subfolder/".to_owned()),
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
    fn test_map_list_objects_to_files_with_directory_marker() {
        let list_objects = ListObjectsV2Output {
            contents: Some(vec![
                Object {
                    key: Some("users/test/file.txt".to_owned()),
                    ..Default::default()
                },
                Object {
                    key: Some("users/test/_$folder$".to_owned()),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        let result = map_list_objects_to_files(list_objects);

        assert_eq!(1, result.len());
        assert_eq!("file.txt", &result[0].file_name);
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
    fn test_map_list_objects_to_directory() {
        let directory = map_legacy_list_objects_to_directory(ListObjectsV2Output {
            prefix: Some("directory/subdirectory/".to_string()),
            contents: Some(vec![Object {
                ..Default::default()
            }]),
            ..Default::default()
        });

        assert_eq!(
            Ok(File {
                file_name: "subdirectory".to_string(),
                file_attributes: FileAttributes {
                    size: None,
                    gid: None,
                    uid: None,
                    permissions: Some(0o40777),
                    atime: None,
                    mtime: None,
                }
            }),
            directory
        );
    }

    #[test]
    fn test_map_list_objects_to_directory_with_none_contents() {
        let directory = map_legacy_list_objects_to_directory(ListObjectsV2Output {
            prefix: Some("directory/subdirectory/".to_string()),
            contents: None,
            ..Default::default()
        });

        assert_eq!(Err(Error::NoSuchFile), directory);
    }

    #[test]
    fn test_map_list_objects_to_directory_with_0_contents() {
        let directory = map_legacy_list_objects_to_directory(ListObjectsV2Output {
            prefix: Some("directory/subdirectory/".to_string()),
            contents: Some(vec![]),
            ..Default::default()
        });

        assert_eq!(Err(Error::NoSuchFile), directory);
    }

    #[test]
    fn test_map_list_objects_to_directory_with_no_prefix() {
        let directory = map_legacy_list_objects_to_directory(ListObjectsV2Output {
            prefix: None,
            contents: Some(vec![]),
            ..Default::default()
        });

        assert_eq!(Err(Error::NoSuchFile), directory);
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
            Some(1417176009),
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
            Some(0),
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

    #[test]
    fn test_map_err_maps_404_to_no_such_file() {
        let not_found_error = RusotoError::Unknown::<UploadPartError>(BufferedHttpResponse {
            status: http::StatusCode::NOT_FOUND,
            body: Bytes::from(&b"test"[..]),
            headers: http::HeaderMap::<String>::with_capacity(0),
        });

        assert_eq!(Error::NoSuchFile, map_legacy_err(not_found_error));
    }

    #[test]
    fn test_map_err_maps_missing_key_to_no_such_file() {
        let not_found_error = RusotoError::Service::<GetObjectError>(GetObjectError::NoSuchKey(
            "The specified key does not exist.".to_string(),
        ));

        assert_eq!(Error::NoSuchFile, map_legacy_err(not_found_error));
    }

    #[test]
    fn test_map_error_maps_error_code_to_storage_error() {
        let internal_server_error = RusotoError::Unknown::<UploadPartError>(BufferedHttpResponse {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            body: Bytes::from(&b"test"[..]),
            headers: http::HeaderMap::<String>::with_capacity(0),
        });

        assert_eq!(
            Error::Storage(String::from("500 Internal Server Error - test")),
            map_legacy_err(internal_server_error)
        );
    }

    #[test]
    fn test_map_err_maps_unknown_service_error_to_storage_error() {
        let not_found_error = RusotoError::Service::<GetObjectError>(
            GetObjectError::InvalidObjectState("Unknown".to_string()),
        );

        assert_eq!(
            Error::Storage(String::from("Unknown")),
            map_legacy_err(not_found_error)
        );
    }

    #[test]
    fn test_map_error_maps_generic_error_to_storage_error() {
        assert_eq!(
            Error::Storage(String::from("parse error")),
            map_legacy_err(RusotoError::ParseError::<UploadPartError>(String::from(
                "parse error"
            )))
        );
    }
}
