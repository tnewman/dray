mod handle;
pub mod s3;

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use russh::keys::PublicKey;

use crate::{error::Error, protocol::response::name::File};

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
    /// Initializes the storage backend, such as creating a bucket in an object
    /// storage backend if it does not already exist.
    async fn init(&self) -> Result<(), Error>;

    /// Retrieves the expected home directory of a user.
    ///
    /// # Note
    /// - The calculation of the home directory must always succeed.
    /// - The framework will check the if home exists if required.
    /// - The framework will sanitize the user.
    fn get_home(&self, user: &str) -> String;

    /// Checks if storage is available. An error will be returned if  storage
    /// operations cannot be performed.
    async fn health_check(&self) -> Result<(), Error>;

    /// Retrieves the authorized keys for a user that will be compared against the
    /// user-supplied key to determine if a user is allowed to log in.
    ///
    /// # Warning
    /// An empty list of keys should be returned for missing users instead of an error
    /// to prevent clients from determining whether or not a user exists.
    async fn get_authorized_keys(&self, user: &str) -> Result<Vec<PublicKey>, Error>;

    // Opens a directory handle for a prefix.
    async fn open_dir_handle(&self, dir_name: String) -> Result<String, Error>;

    /// Creates a directory.
    async fn create_dir(&self, dir_name: String) -> Result<(), Error>;

    // Reads a file listing from the prefix associated with a given handle.
    async fn read_dir(&self, handle: &str) -> Result<Vec<File>, Error>;

    /// Removes a directory.
    async fn remove_dir(&self, dir_name: String) -> Result<(), Error>;

    /// Retrieves an file's metadata.
    async fn get_file_metadata(&self, file_name: String) -> Result<File, Error>;

    /// Retrieves a handle's metadata.
    async fn get_handle_metadata(&self, handle: &str) -> Result<File, Error>;

    /// Creates a read handle for a file.
    async fn open_read_handle(&self, file_name: String) -> Result<String, Error>;

    /// Reads up to len bytes of data data from a file associated with a given handle.
    async fn read_data(&self, handle: &str, len: u32) -> Result<Vec<u8>, Error>;

    /// Creates a write handle for a file.
    async fn open_write_handle(&self, file_name: String) -> Result<String, Error>;

    /// Writes data to a file associated with a given handle.
    async fn write_data(&self, handle: &str, data: Bytes) -> Result<(), Error>;

    /// Removes a file.
    async fn remove_file(&self, key: String) -> Result<(), Error>;

    // Closes a handle.
    async fn close_handle(&self, handle: &str) -> Result<(), Error>;

    /// Renames a file or directory.
    async fn rename(&self, current: String, new: String) -> Result<(), Error>;
}
