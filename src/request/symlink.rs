use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Symlink {}

impl Symlink {
    pub fn parse_bytes(byte: &[u8]) -> Result<Symlink, Error> {
        Err(Error::Failure)
    }
}
