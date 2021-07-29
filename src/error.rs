use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Bad message received from client.")]
    BadMessage,

    #[error("Configuration error: {}", .0)]
    Configuration(String),

    #[error("{}", .0)]
    Failure(String),

    #[error("File not found.")]
    NoSuchFile,

    #[error("Permission denied.")]
    PermissionDenied,

    #[error("An error occurred with the storage backend: {}", .0)]
    Storage(String),

    #[error("SFTP request not implemented.")]
    Unimplemented,
}

impl From<envy::Error> for Error {
    fn from(envy_error: envy::Error) -> Self {
        Error::Configuration(envy_error.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(io_error: std::io::Error) -> Self {
        Error::Storage(io_error.to_string())
    }
}

impl From<thrussh::Error> for Error {
    fn from(thrussh_error: thrussh::Error) -> Self {
        Error::Failure(thrussh_error.to_string())
    }
}

impl From<thrussh_keys::Error> for Error {
    fn from(thrussh_error: thrussh_keys::Error) -> Self {
        Error::Failure(thrussh_error.to_string())
    }
}
