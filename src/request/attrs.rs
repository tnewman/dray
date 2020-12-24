use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Attrs {}

impl Attrs {
    pub fn parse_bytes(byte: &[u8]) -> Result<Attrs, Error> {
        Err(Error::Failure)
    }
}
