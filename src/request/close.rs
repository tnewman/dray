use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Close {}

impl Close {
    pub fn parse_bytes(byte: &[u8]) -> Result<Close, Error> {
        Err(Error::Failure)
    }
}
