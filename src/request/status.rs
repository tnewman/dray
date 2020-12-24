use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Status {}

impl Status {
    pub fn parse_bytes(byte: &[u8]) -> Result<Status, Error> {
        Err(Error::Failure)
    }
}
