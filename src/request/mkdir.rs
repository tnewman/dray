use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Mkdir {}

impl Mkdir {
    pub fn parse_bytes(byte: &[u8]) -> Result<Mkdir, Error> {
        Err(Error::Failure)
    }
}
