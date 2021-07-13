mod handle;
pub mod s3;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;

use crate::protocol::response::name::File;

/// Builds an instance of a Storage backend, such as AWS S3.
///
/// A new instance of Storage is created for each SSH session, so data that is
/// shared bewtween SSH sessions should be injected by the factory.
pub trait StorageFactory: Send + Sync {
    fn create_storage(&self) -> Arc<dyn Storage>;
}

/// An implementation of a Storage backend, such as AWS S3.
#[async_trait]
pub trait Storage: Send + Sync {
    /// Retrieves the expected home directory of a user.
    ///
    /// # Note
    /// - The calculation of the home directory must always succeed.
    /// - The framework will check the if home exists if required.
    /// - The framework will sanitize the user.
    fn get_home(&self, user: &str) -> String;

    /// Checks if storage is available. An error will be returned if  storage
    /// operations cannot be performed.
    async fn health_check(&self) -> Result<()>;

    /// Retrieves the authorized key fingerprints for a user that will be compared
    /// against the fingerprint of the user-supplied key to determine if a user is
    /// allowed to log in.
    ///
    /// # Warning
    /// An empty list of keys should be returned for missing users instead of an error
    /// to prevent clients from determining whether or not a user exists.
    async fn get_authorized_keys_fingerprints(&self, user: &str) -> Result<Vec<String>>;

    // Opens a directory handle for a prefix.
    async fn open_dir_handle(&self, dir_name: String) -> Result<String>;

    /// Creates a directory.
    async fn create_dir(&self, dir_name: String) -> Result<()>;

    // Reads a file listing from the prefix associated with a given handle.
    async fn read_dir(&self, handle: &str) -> Result<Vec<File>>;

    /// Removes a directory.
    async fn remove_dir(&self, dir_name: String) -> Result<()>;

    /// Retrieves an file's metadata.
    async fn get_file_metadata(&self, file_name: String) -> Result<File>;

    /// Creates a read handle for a file.
    async fn open_read_handle(&self, file_name: String) -> Result<String>;

    /// Reads up to len bytes of data data from a file associated with a given handle.
    async fn read_data(&self, handle: &str, len: u32) -> Result<Vec<u8>>;

    /// Creates a write handle for a file.
    async fn open_write_handle(&self, file_name: String) -> Result<String>;

    /// Writes data to a file associated with a given handle.
    async fn write_data(&self, handle: &str, data: Bytes) -> Result<()>;

    /// Removes a file.
    async fn remove_file(&self, key: String) -> Result<()>;

    // Closes a handle.
    async fn close_handle(&self, handle: &str) -> Result<()>;

    /// Renames a file or directory.
    async fn rename(&self, current: String, new: String) -> Result<()>;
}
