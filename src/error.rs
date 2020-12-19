use std::error::Error as StdError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub enum Error {
    NoSuchFile,
    PermissionDenied,
    Failure,
    BadMessage,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        todo!()
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        todo!()
    }
}
