use anyhow::Result;
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_s3::{
    CommonPrefix, GetObjectRequest, ListObjectsV2Output, ListObjectsV2Request, Object, S3Client, S3,
};
use rusoto_s3::{HeadObjectError, HeadObjectRequest};
use serde::Deserialize;
use std::error::Error as StdError;
use tokio::io::AsyncReadExt;

use super::ListPrefixResult;
use super::ObjectStorage;
use crate::protocol::response::name::File;
use crate::ssh_keys;
use crate::{error::Error, protocol::file_attributes::FileAttributes};

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

    async fn create_prefix(&self, prefix: String) {
        todo!("TODO: Create prefix {}", prefix)
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
                key,
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

    async fn read_object(&self, key: String, offset: u64, len: u32) -> Result<Vec<u8>> {
        let get_object_response = self.s3_client.get_object(GetObjectRequest {
            bucket: self.bucket.clone(),
            key,
            range: Option::Some(get_range(offset, len)),
            ..Default::default()
        });

        let result = get_object_response.await?;

        let body = result.body.ok_or(Error::ServerError)?;
        let mut data = Vec::new();
        body.into_async_read().read_to_end(&mut data).await?;

        Ok(data)
    }

    async fn create_multipart_upload(&self, key: String) -> Result<String> {
        todo!("TODO: Create multipart upload {}", key);
    }

    async fn write_object_part(&self, multipart_upload_id: String, offset: u64, data: Vec<u8>) {
        todo!(
            "TODO: Write object part {} {} {}",
            multipart_upload_id,
            offset,
            data.len()
        );
    }

    async fn complete_multipart_upload(&self, multipart_upload_id: String) -> Result<()> {
        todo!("TODO: Complete multipart upload {}", multipart_upload_id);
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
    format!("bytes={}-{}", offset, len)
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
    let file_name = match object.key {
        Some(ref key) => String::from(key),
        None => "".to_owned(),
    };

    File {
        file_name: file_name.clone(),
        long_name: file_name,
        file_attributes: FileAttributes {
            size: object.size.map(|size| size as u64),
            uid: None,
            gid: None,
            permissions: Some(700),
            atime: None,
            mtime: None,
        },
    }
}

fn map_prefix_to_file(prefix: &CommonPrefix) -> File {
    let prefix = match prefix.prefix {
        Some(ref prefix) => String::from(prefix),
        None => "".to_owned(),
    };

    File {
        file_name: prefix.clone(),
        long_name: prefix,
        file_attributes: FileAttributes {
            size: None,
            uid: None,
            gid: None,
            permissions: Some(700),
            atime: None,
            mtime: None,
        },
    }
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
        assert_eq!("bytes=0-1024", get_range(0, 1024));
    }

    #[test]
    fn test_get_default_endpoint_region() {
        assert_eq!("custom", get_default_endpoint_region());
    }

    #[test]
    fn test_map_list_objects_to_files() {
        let list_objects = ListObjectsV2Output {
            common_prefixes: Some(vec![CommonPrefix {
                prefix: Some("/users/test/subfolder".to_owned()),
                ..Default::default()
            }]),
            contents: Some(vec![Object {
                key: Some("/users/test/file.txt".to_owned()),
                size: Some(1),
                ..Default::default()
            }]),
            continuation_token: Some(String::from("token")),
            ..Default::default()
        };

        let result = map_list_objects_to_list_prefix_result(list_objects);

        assert_eq!(2, result.objects.len());
        assert_eq!(
            File {
                file_name: "/users/test/subfolder".to_owned(),
                long_name: "/users/test/subfolder".to_owned(),
                file_attributes: FileAttributes {
                    size: None,
                    gid: None,
                    uid: None,
                    permissions: Some(700),
                    atime: None,
                    mtime: None,
                }
            },
            result.objects[0]
        );
        assert_eq!(
            File {
                file_name: "/users/test/file.txt".to_owned(),
                long_name: "/users/test/file.txt".to_owned(),
                file_attributes: FileAttributes {
                    size: Some(1),
                    gid: None,
                    uid: None,
                    permissions: Some(700),
                    atime: None,
                    mtime: None,
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
                long_name: "".to_owned(),
                file_attributes: FileAttributes {
                    size: None,
                    gid: None,
                    uid: None,
                    permissions: Some(700),
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
                long_name: "".to_owned(),
                file_attributes: FileAttributes {
                    size: None,
                    gid: None,
                    uid: None,
                    permissions: Some(700),
                    atime: None,
                    mtime: None,
                }
            },
            map_prefix_to_file(&prefix)
        );
    }
}
