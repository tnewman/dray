use anyhow::Result;
use thrussh_keys::key::PublicKey;

pub enum Permission {
    READ,
    WRITE,
}

/// An implementation of an Object Storage backend, such as AWS S3.
///
/// Implementers of this trait are providing low level access to objects, allowing
/// the high level framework to deal with generic concerns, such as associating
/// streams with SFTP handles and creating SFTP messages.
pub trait ObjectStorage {
    /// Retrieves the home directory of a user.
    fn get_home(user: String) -> Result<String>;

    /// Retrieves the authorized keys for a user that will be compared against a
    /// supplied key to determine if a user is allowed to log in.
    ///
    /// # Warning
    /// An empty list of keys should be returned for missing users instead of an error
    /// to prevent clients from determining whether or not a user exists.
    fn get_authorized_keys(user: String) -> Result<Vec<PublicKey>>;

    /// Checks if a user has permission to perform an operation on a path.
    ///
    /// # Warning
    /// The permission check must account for:
    /// - Preventing the home directory from being deleted or renamed
    /// - Denying reads and write to paths outside of the user's home directory
    fn has_permission(user: String, path: String, permission: Permission) -> Result<bool>;

    /// Lists objects under a prefix. The list will start at `continuation_token` if
    /// provided and return up the smaller of `max_results` or the backend max limit.
    fn list_prefix(prefix: String, continuation_token: Option<String>, max_results: Option<i64>);

    /// Creates a prefix.
    fn create_prefix(prefix: String);

    /// Renames a prefix.
    fn rename_prefix(current: String, new: String);

    /// Removes a prefix.
    fn remove_prefix(prefix: String);

    /// Creates a read stream for an object.
    fn open_object_read_stream(key: String);

    /// Creates a write stream for an object.
    fn open_object_write_stream(key: String);

    /// Renames an object.
    fn rename_object(current: String, new: String);

    /// Removes an object.
    fn remove_object(key: String);
}
