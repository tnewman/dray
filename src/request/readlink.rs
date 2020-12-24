use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Readlink {}

impl Readlink {
    pub fn parse_bytes(byte: &[u8]) -> Result<Readlink, Error> {
        Err(Error::Failure)
    }
}
