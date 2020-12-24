use crate::error::Error;
use bytes::Buf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct ExtendedReply {}

impl TryFrom<&[u8]> for ExtendedReply {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut bytes = item;

        if bytes.remaining() < 1 {
            return Err(Error::BadMessage);
        }

        Ok(ExtendedReply {})
    }
}

impl ExtendedReply {
    pub fn parse_bytes(byte: &[u8]) -> Result<ExtendedReply, Error> {
        Err(Error::Failure)
    }
}
