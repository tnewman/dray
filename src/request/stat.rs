use crate::error::Error;
use bytes::Buf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Stat {}

impl TryFrom<&[u8]> for Stat {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut bytes = item;

        if bytes.remaining() < 1 {
            return Err(Error::BadMessage);
        }

        Ok(Stat {})
    }
}

impl Stat {
    pub fn parse_bytes(byte: &[u8]) -> Result<Stat, Error> {
        Err(Error::Failure)
    }
}
