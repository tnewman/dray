use super::ObjectStorage;
use anyhow::Result;
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_s3::{S3, S3Client};
use thrussh_keys::key::PublicKey;

pub struct S3ObjectStorage {
    s3_client: S3Client,
    bucket: String,
}

impl S3ObjectStorage {
    pub fn new() -> S3ObjectStorage {
        S3ObjectStorage {
            s3_client: S3Client::new(Region::Custom {
                name: "us-east-1".to_owned(),
                endpoint: "http://localhost:8000".to_owned(),
            }),
            bucket: "test".to_owned(),
        }
    }
}

#[async_trait]
impl ObjectStorage for S3ObjectStorage {
    fn get_home(&self, user: String) -> String {
        format!("/home/{}", user)
    }

    async fn get_authorized_keys(&self, user: String) -> Result<Vec<PublicKey>> {
        let authorized_keys_key = format!("/.ssh/{}/authorized_keys", user);

        Ok(vec![])
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
    ) {
        todo!()
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
