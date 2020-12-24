use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Fstat {}

impl Fstat {
    pub fn parse_bytes(byte: &[u8]) -> Result<Fstat, Error> {
        Err(Error::Failure)
    }
}
