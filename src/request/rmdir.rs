use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Rmdir {}

impl Rmdir {
    pub fn parse_bytes(byte: &[u8]) -> Result<Rmdir, Error> {
        Err(Error::Failure)
    }
}
