use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Write {}

impl Write {
    pub fn parse_bytes(byte: &[u8]) -> Result<Write, Error> {
        Err(Error::Failure)
    }
}
