use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Open {}

impl Open {
    pub fn parse_bytes(byte: &[u8]) -> Result<Open, Error> {
        Err(Error::Failure)
    }
}
