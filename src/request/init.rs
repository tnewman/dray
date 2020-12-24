use std::convert::TryFrom;

use crate::error::Error;
use crate::try_buf::TryBuf;

#[derive(Debug, PartialEq)]
pub struct Init {
    pub version: u8,
}

impl TryFrom<&[u8]> for Init {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut bytes = item;

        Ok(Init {
            version: bytes.try_get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bytes::Buf;
    use bytes::BufMut;

    #[test]
    fn test_parse_init_message() {
        let mut init_bytes = vec![];
        init_bytes.put_u8(0x03);

        assert_eq!(Init::try_from(&init_bytes[..]), Ok(Init { version: 0x03 }));
    }

    #[test]
    fn test_parse_invalid_message() {
        let init_bytes: &[u8] = &[];

        assert_eq!(Init::try_from(init_bytes), Err(Error::BadMessage));
    }
}
