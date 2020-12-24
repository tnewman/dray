use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Fsetstat {}

impl Fsetstat {
    pub fn parse_bytes(byte: &[u8]) -> Result<Fsetstat, Error> {
        Err(Error::Failure)
    }
}
