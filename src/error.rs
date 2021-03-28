use std::error::Error as StdError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, PartialEq)]
pub enum Error {
    BadMessage,
    Unimplemented,
    ServerError,
}

impl Display for Error {
    fn fmt(&self, _formatter: &mut Formatter) -> Result {
        todo!()
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        todo!()
    }
}
