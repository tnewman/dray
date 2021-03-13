use anyhow::Result;
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_s3::{
    CommonPrefix, GetObjectRequest, ListObjectsV2Output, ListObjectsV2Request, Object, S3Client, S3,
};
use serde::Deserialize;
use tokio::io::AsyncReadExt;

use super::ObjectStorage;
use crate::protocol::file_attributes::FileAttributes;
use crate::protocol::response::name::File;
use crate::ssh_keys;

#[derive(Clone)]
pub struct S3ObjectStorage {
    s3_client: S3Client,
    bucket: String,
}

#[derive(Clone, Deserialize, Debug)]
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
        format!("/home/{}", user)
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

    async fn has_permission(
        &self,
        user: String,
        path: String,
        permission: super::Permission,
    ) -> Result<bool> {
        todo!()
    }

    async fn list_prefix(
        &self,
        prefix: String,
        continuation_token: Option<String>,
        max_results: Option<i64>,
    ) -> Result<Vec<File>> {
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

        Ok(map_list_objects_to_files(objects))
    }

    async fn create_prefix(&self, prefix: String) {
        todo!()
    }

    async fn rename_prefix(&self, current: String, new: String) {
        todo!()
    }

    async fn remove_prefix(&self, prefix: String) {
        todo!()
    }

    async fn open_object_read_stream(&self, key: String) {
        todo!()
    }

    async fn open_object_write_stream(&self, key: String) {
        todo!()
    }

    async fn rename_object(&self, current: String, new: String) {
        todo!()
    }

    async fn remove_object(&self, key: String) {
        todo!()
    }
}

fn map_list_objects_to_files(list_objects: ListObjectsV2Output) -> Vec<File> {
    let files = list_objects.contents.unwrap_or(vec![]);

    let directories = list_objects.common_prefixes.unwrap_or(vec![]);

    let mapped_files = files.iter().map(|object| map_object_to_file(object));

    let mapped_dirs = directories.iter().map(|prefix| map_prefix_to_file(prefix));

    mapped_dirs.chain(mapped_files).collect()
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
            size: object.size.map_or(None, |size| Some(size as u64)),
            uid: None,
            gid: None,
            permissions: Some(0700),
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
            permissions: Some(0700),
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
            ..Default::default()
        };

        let result = map_list_objects_to_files(list_objects);

        assert_eq!(2, result.len());
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
            result[0]
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
            result[1]
        );
    }

    #[test]
    fn test_map_list_objects_to_files_with_missing_data() {
        let list_objects = ListObjectsV2Output {
            ..Default::default()
        };

        assert_eq!(0, map_list_objects_to_files(list_objects).len());
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