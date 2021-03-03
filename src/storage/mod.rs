pub mod s3;

use anyhow::Result;
use async_trait::async_trait;
use thrussh_keys::key::PublicKey;

use crate::protocol::response::name::File;

/// Object Storage permissions.
pub enum Permission {
    /// Permission to read a prefix or object.
    READ,

    /// Permission to write a prefix or object.
    WRITE,
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

    /// Retrieves the authorized key fingerprints for a user that will be compared
    /// against the fingerprint of the user-supplied key to determine if a user is
    /// allowed to log in.
    ///
    /// # Warning
    /// An empty list of keys should be returned for missing users instead of an error
    /// to prevent clients from determining whether or not a user exists.
    async fn get_authorized_keys_fingerprints(&self, user: &str) -> Result<Vec<String>>;

    /// Checks if a user has permission to perform an operation on a path.
    ///
    /// # Warning
    /// The permission check must account for:
    /// - Preventing the home directory from being deleted or renamed
    /// - Denying reads and write to paths outside of the user's home directory
    async fn has_permission(
        &self,
        user: String,
        path: String,
        permission: Permission,
    ) -> Result<bool>;

    /// Lists objects under a prefix. The list will start at `continuation_token` if
    /// provided and return up the smaller of `max_results` or the backend max limit.
    async fn list_prefix(
        &self,
        prefix: String,
        continuation_token: Option<String>,
        max_results: Option<i64>,
    ) -> Result<Vec<File>>;

    /// Creates a prefix.
    async fn create_prefix(&self, prefix: String);

    /// Renames a prefix.
    async fn rename_prefix(&self, current: String, new: String);

    /// Removes a prefix.
    async fn remove_prefix(&self, prefix: String);

    /// Creates a read stream for an object.
    async fn open_object_read_stream(&self, key: String);

    /// Creates a write stream for an object.
    async fn open_object_write_stream(&self, key: String);

    /// Renames an object.
    async fn rename_object(&self, current: String, new: String);

    /// Removes an object.
    async fn remove_object(&self, key: String);
}
