use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Lstat {}

impl Lstat {
    pub fn parse_bytes(byte: &[u8]) -> Result<Lstat, Error> {
        Err(Error::Failure)
    }
}
