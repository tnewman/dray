use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Readdir {}

impl Readdir {
    pub fn parse_bytes(byte: &[u8]) -> Result<Readdir, Error> {
        Err(Error::Failure)
    }
}
