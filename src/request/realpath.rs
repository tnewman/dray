use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Realpath {}

impl Realpath {
    pub fn parse_bytes(byte: &[u8]) -> Result<Realpath, Error> {
        Err(Error::Failure)
    }
}
