use std::io::ErrorKind;

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Bad message received from client.")]
    BadMessage,

    #[error("Configuration error: {}", .0)]
    Configuration(String),

    #[error("End of file.")]
    EndOfFile,

    #[error("{}", .0)]
    Failure(String),

    #[error("IO Error: {}", .0)]
    IOError(std::io::ErrorKind),

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
        match io_error.kind() {
            ErrorKind::UnexpectedEof => Error::EndOfFile,
            _ => Error::IOError(io_error.kind()),
        }
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(addr_parse_error: std::net::AddrParseError) -> Self {
        Error::Configuration(addr_parse_error.to_string())
    }
}

impl From<russh::Error> for Error {
    fn from(russh_error: russh::Error) -> Self {
        Error::Failure(russh_error.to_string())
    }
}

impl From<russh_keys::Error> for Error {
    fn from(russh_error: russh_keys::Error) -> Self {
        Error::Failure(russh_error.to_string())
    }
}

impl From<bytes::TryGetError> for Error {
    fn from(_: bytes::TryGetError) -> Self {
        Error::BadMessage
    }
}