use crate::error::Error;
use crate::try_buf::TryBuf;

use bytes::Bytes;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Close {
    pub id: u32,
    pub handle: String,
}

impl TryFrom<&mut Bytes> for Close {
    type Error = Error;

    fn try_from(close_bytes: &mut Bytes) -> Result<Self, Self::Error> {
        let id = close_bytes.try_get_u32()?;
        let handle = close_bytes.try_get_string()?;

        Ok(Close { id, handle })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::try_buf::TryBufMut;

    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_parse_close() {
        let mut close_bytes = BytesMut::new();

        close_bytes.put_u32(0x01); // id
        close_bytes.try_put_str("handle").unwrap(); // handle

        assert_eq!(
            Close::try_from(&mut close_bytes.freeze()),
            Ok(Close {
                id: 0x01,
                handle: String::from("handle")
            })
        );
    }

    #[test]
    fn test_parse_close_with_empty_data() {
        assert_eq!(Close::try_from(&mut Bytes::new()), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_close_with_invalid_id() {
        let mut close_bytes = BytesMut::new();

        close_bytes.put_u8(0x01); // id

        assert_eq!(
            Close::try_from(&mut close_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_close_with_invalid_handle() {
        let mut close_bytes = BytesMut::new();

        close_bytes.put_u32(0x01); // id
        close_bytes.put_u8(0x01); // handle

        assert_eq!(
            Close::try_from(&mut close_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }
}
