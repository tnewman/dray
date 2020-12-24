use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Remove {}

impl Remove {
    pub fn parse_bytes(byte: &[u8]) -> Result<Remove, Error> {
        Err(Error::Failure)
    }
}
