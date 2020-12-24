use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Read {}

impl Read {
    pub fn parse_bytes(byte: &[u8]) -> Result<Read, Error> {
        Err(Error::Failure)
    }
}
