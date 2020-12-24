use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Data {}

impl Data {
    pub fn parse_bytes(byte: &[u8]) -> Result<Data, Error> {
        Err(Error::Failure)
    }
}
