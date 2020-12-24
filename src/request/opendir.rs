use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Opendir {}

impl Opendir {
    pub fn parse_bytes(byte: &[u8]) -> Result<Opendir, Error> {
        Err(Error::Failure)
    }
}
