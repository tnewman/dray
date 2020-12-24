use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Name {}

impl Name {
    pub fn parse_bytes(byte: &[u8]) -> Result<Name, Error> {
        Err(Error::Failure)
    }
}
