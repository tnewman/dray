use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Stat {}

impl Stat {
    pub fn parse_bytes(byte: &[u8]) -> Result<Stat, Error> {
        Err(Error::Failure)
    }
}
