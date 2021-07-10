mod handle;
pub mod s3;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;

use crate::protocol::response::name::File;

/// Builds an instance of an Object Storage backend, such as AWS S3.
///
/// A new instance of Object Storage is created for each SSH session,
/// so data that is shared bewtween SSH sessions should be injected
/// by the factory.
pub trait ObjectStorageFactory: Send + Sync {
    fn create_object_storage(&self) -> Arc<dyn ObjectStorage>;
}

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
    async fn create_prefix(&self, prefix: String) -> Result<()>;

    /// Renames a prefix.
    async fn rename_prefix(&self, current: String, new: String);

    /// Removes a prefix.
    async fn remove_prefix(&self, prefix: String);

    /// Checks if an object exists.
    async fn object_exists(&self, key: String) -> Result<bool>;

    /// Retrieves an object's metadata.
    async fn get_object_metadata(&self, key: String) -> Result<File>;

    /// Creates a read handle for an object.
    async fn open_read_handle(&self, key: String) -> Result<String>;

    /// Writes data to an object associated with a given handle.
    async fn write_data(&self, handle: &str, data: Bytes) -> Result<()>;

    /// Creates a write handle for an object.
    async fn open_write_handle(&self, key: String) -> Result<String>;

    /// Reads data from an object associated with a given handle.
    async fn read_data(&self, handle: &str) -> Result<Vec<u8>>;

    /// Renames an object.
    async fn rename_object(&self, current: String, new: String);

    /// Removes an object.
    async fn remove_object(&self, key: String);

    // Closes a handle.
    async fn close_handle(&self, handle: &str) -> Result<()>;
}

/// A list of objects under a prefix along with a continuation token to retrieve
/// the next objects if the current result is incomplete.
pub struct ListPrefixResult {
    /// A list of objects under a prefix and continuation token.
    pub objects: Vec<File>,

    /// The continuation token to retrieve the next list of objects under the prefix.
    pub continuation_token: Option<String>,
}
