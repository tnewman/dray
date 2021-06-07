pub mod s3;

use anyhow::Result;
use async_trait::async_trait;

use crate::protocol::response::name::File;

/// An implementation of an Object Storage backend, such as AWS S3.
///
/// Implementers of this trait are providing low level access to objects, allowing
/// the high level framework to deal with generic concerns, such as associating
/// streams with SFTP handles and creating SFTP messages.
#[async_trait]
pub trait ObjectStorage: Send + Sync {
    /// Retrieves the expected home directory of a user.
    ///
    /// # Note
    /// - The calculation of the home directory must always succeed.
    /// - The framework will check the if home exists if required.
    /// - The framework will sanitize the user.
    fn get_home(&self, user: &str) -> String;

    /// Checks if object storage is available. An error will be returned if object
    /// storage operations cannot be performed.
    async fn health_check(&self) -> Result<()>;

    /// Retrieves the authorized key fingerprints for a user that will be compared
    /// against the fingerprint of the user-supplied key to determine if a user is
    /// allowed to log in.
    ///
    /// # Warning
    /// An empty list of keys should be returned for missing users instead of an error
    /// to prevent clients from determining whether or not a user exists.
    async fn get_authorized_keys_fingerprints(&self, user: &str) -> Result<Vec<String>>;

    /// Lists objects under a prefix. The list will start at `continuation_token` if
    /// provided and return up the smaller of `max_results` or the backend max limit.
    async fn list_prefix(
        &self,
        prefix: String,
        continuation_token: Option<String>,
        max_results: Option<i64>,
    ) -> Result<ListPrefixResult>;

    /// Creates a prefix.
    async fn create_prefix(&self, prefix: String);

    /// Renames a prefix.
    async fn rename_prefix(&self, current: String, new: String);

    /// Removes a prefix.
    async fn remove_prefix(&self, prefix: String);

    /// Creates a read stream for an object.
    async fn read_object(&self, key: String, offset: u64, len: u32) -> Result<Vec<u8>>;

    /// Creates a multipart upload for an object.
    async fn create_multipart_upload(&self, key: String) -> Result<String>;

    /// Writes a part to an existing multipart upload for an object.
    async fn write_object_part(&self, multipart_upload_id: String, offset: u64, data: Vec<u8>);

    /// Renames an object.
    async fn rename_object(&self, current: String, new: String);

    /// Removes an object.
    async fn remove_object(&self, key: String);
}

/// A list of objects under a prefix along with a continuation token to retrieve
/// the next objects if the current result is incomplete.
pub struct ListPrefixResult {
    /// A list of objects under a prefix and continuation token.
    objects: Vec<File>,

    /// The continuation token to retrieve the next list of objects under the prefix.
    continuation_token: Option<String>,
}
