use super::ListPrefixResult;
use super::ObjectStorage;
use crate::error::Error;
use crate::protocol::file_attributes::FileAttributes;
use crate::protocol::response::name::File;
use crate::ssh_keys;
use anyhow::Result;
use async_trait::async_trait;
use bytes::BytesMut;
use chrono::{DateTime, TimeZone, Utc};
use rusoto_core::{ByteStream, Region};
use rusoto_s3::{
    CommonPrefix, GetObjectRequest, HeadObjectOutput, ListObjectsV2Output, ListObjectsV2Request,
    Object, PutObjectRequest, S3Client, S3,
};
use rusoto_s3::{HeadObjectError, HeadObjectRequest};
use serde::Deserialize;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct S3ObjectStorage {
    s3_client: S3Client,
    bucket: String,
}

#[derive(Deserialize, Debug)]
pub struct S3Config {
    #[serde(rename(deserialize = "s3_endpoint_name"))]
    pub endpoint_name: Option<String>,

    #[serde(default = "get_default_endpoint_region")]
    pub endpoint_region: String,

    #[serde(rename(deserialize = "s3_bucket"))]
    pub bucket: String,
}

impl S3ObjectStorage {
    pub fn new(s3_config: &S3Config) -> S3ObjectStorage {
        let region = match &s3_config.endpoint_name {
            Some(endpoint_name) => Region::Custom {
                name: s3_config.endpoint_region.clone(),
                endpoint: endpoint_name.clone(),
            },
            None => Region::default(),
        };

        S3ObjectStorage {
            s3_client: S3Client::new(region),
            bucket: s3_config.bucket.clone(),
        }
    }
}

#[async_trait]
impl ObjectStorage for S3ObjectStorage {
    fn get_home(&self, user: &str) -> String {
        get_home(user)
    }

    async fn health_check(&self) -> Result<()> {
        self.list_prefix(String::from(""), None, None).await?;
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

    async fn list_prefix(
        &self,
        prefix: String,
        continuation_token: Option<String>,
        max_results: Option<i64>,
    ) -> Result<ListPrefixResult> {
        let prefix = get_s3_prefix(prefix);

        let objects = self
            .s3_client
            .list_objects_v2(ListObjectsV2Request {
                bucket: self.bucket.clone(),
                prefix: Some(prefix),
                continuation_token,
                max_keys: max_results,
                delimiter: Some("/".to_owned()),
                ..Default::default()
            })
            .await?;

        Ok(map_list_objects_to_list_prefix_result(objects))
    }

    async fn create_prefix(&self, _prefix: String) -> Result<()> {
        /*
            S3 does not support creating empty prefixes. The prefix is created when the
            first object is added to it. This operation is a NO-OP to allow GUI-based
            SFTP clients to make it appear that a directory has been created.
        */
        Ok(())
    }

    async fn rename_prefix(&self, current: String, new: String) {
        todo!("TODO: Rename prefix {} to {}", current, new)
    }

    async fn remove_prefix(&self, prefix: String) {
        todo!("TODO: Remove prefix {}", prefix)
    }

    async fn object_exists(&self, key: String) -> Result<bool> {
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

    async fn get_object_metadata(&self, key: String) -> Result<File> {
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

    /// Creates a read stream for an object.
    async fn read_object(
        &self,
        key: String,
    ) -> Result<Arc<Mutex<Pin<Box<dyn AsyncRead + Send + Sync>>>>> {
        let read_response = self
            .s3_client
            .get_object(GetObjectRequest {
                bucket: self.bucket.clone(),
                key,
                ..Default::default()
            })
            .await?;

        let read_stream = read_response
            .body
            .ok_or(Error::ServerError)?
            .into_async_read();
        Ok(Arc::new(Mutex::new(Box::pin(read_stream))))
    }

    /// Creates a write stream for an object.
    async fn write_object(
        &self,
        key: String,
    ) -> Result<Arc<Mutex<Pin<Box<dyn AsyncWrite + Send + Sync>>>>> {
        // Writing needs to be reworked to support buffering chunks in memory and writing a single chunk at a time as a multipart upload
        todo!("add multipart support")
    }

    async fn rename_object(&self, current: String, new: String) {
        todo!("TODO: Rename object {} to {}", current, new)
    }

    async fn remove_object(&self, key: String) {
        todo!("TODO: Remove object {}", key)
    }
}

fn get_home(user: &str) -> String {
    format!("/home/{}", user)
}

fn get_range(offset: u64, len: u32) -> String {
    format!("bytes={}-{}", offset, offset + len as u64 - 1)
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

fn map_list_objects_to_list_prefix_result(list_objects: ListObjectsV2Output) -> ListPrefixResult {
    let files = list_objects.contents.unwrap_or_default();

    let directories = list_objects.common_prefixes.unwrap_or_default();

    let mapped_files = files.iter().map(|object| map_object_to_file(object));

    let mapped_dirs = directories.iter().map(|prefix| map_prefix_to_file(prefix));

    ListPrefixResult {
        objects: mapped_dirs.chain(mapped_files).collect(),
        continuation_token: list_objects.continuation_token,
    }
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
    fn test_get_range_returns_range_string() {
        assert_eq!("bytes=0-1023", get_range(0, 1024));
    }

    #[test]
    fn test_get_range_returns_range_string_with_offset() {
        assert_eq!("bytes=1024-2023", get_range(1024, 1000));
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

        let result = map_list_objects_to_list_prefix_result(list_objects);

        assert_eq!(2, result.objects.len());
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
            result.objects[0]
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
            result.objects[1]
        );
        assert_eq!(String::from("token"), result.continuation_token.unwrap());
    }

    #[test]
    fn test_map_list_objects_to_files_with_missing_data() {
        let list_objects = ListObjectsV2Output {
            ..Default::default()
        };

        let result = map_list_objects_to_list_prefix_result(list_objects);

        assert_eq!(0, result.objects.len());
        assert!(result.continuation_token.is_none());
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
