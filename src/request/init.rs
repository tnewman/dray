use crate::error::Error;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub struct Init {
    version: u8,
}

impl Init {
    pub fn parse_bytes(mut bytes: &[u8]) -> Result<Init, Error> {
        if bytes.remaining() < 1 {
            return Err(Error::BadMessage);
        }

        Ok(Init {
            version: bytes.get_u8(),
        })
    }
}
