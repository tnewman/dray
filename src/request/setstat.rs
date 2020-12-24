use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Setstat {}

impl Setstat {
    pub fn parse_bytes(byte: &[u8]) -> Result<Setstat, Error> {
        Err(Error::Failure)
    }
}
