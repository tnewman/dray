use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Handle {}

impl Handle {
    pub fn parse_bytes(byte: &[u8]) -> Result<Handle, Error> {
        Err(Error::Failure)
    }
}
