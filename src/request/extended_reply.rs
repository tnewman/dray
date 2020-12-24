use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct ExtendedReply {}

impl ExtendedReply {
    pub fn parse_bytes(byte: &[u8]) -> Result<ExtendedReply, Error> {
        Err(Error::Failure)
    }
}
