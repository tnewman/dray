use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Rename {}

impl Rename {
    pub fn parse_bytes(byte: &[u8]) -> Result<Rename, Error> {
        Err(Error::Failure)
    }
}
