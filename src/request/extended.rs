use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Extended {}

impl Extended {
    pub fn parse_bytes(byte: &[u8]) -> Result<Extended, Error> {
        Err(Error::Failure)
    }
}
