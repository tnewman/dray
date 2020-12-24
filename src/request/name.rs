use crate::error::Error;
use bytes::Buf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Name {}

impl TryFrom<&[u8]> for Name {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut bytes = item;

        if bytes.remaining() < 1 {
            return Err(Error::BadMessage);
        }

        Ok(Name {})
    }
}

impl Name {
    pub fn parse_bytes(byte: &[u8]) -> Result<Name, Error> {
        Err(Error::Failure)
    }
}
