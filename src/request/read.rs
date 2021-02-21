use crate::error::Error;
use crate::try_buf::TryBuf;

use bytes::Bytes;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Read {
    pub id: u32,
    pub handle: String,
    pub offset: u64,
    pub len: u32,
}

impl TryFrom<&Bytes> for Read {
    type Error = Error;

    fn try_from(read_bytes: &Bytes) -> Result<Self, Self::Error> {
        let id = read_bytes.try_get_u32()?;
        let handle = read_bytes.try_get_string()?;
        let offset = read_bytes.try_get_u64()?;
        let len = read_bytes.try_get_u32()?;

        Ok(Read {
            id,
            handle,
            offset,
            len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::try_buf::TryBufMut;

    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_parse_read() {
        let read_bytes = BytesMut::new();

        read_bytes.put_u32(0x01); // id
        read_bytes.try_put_str("handle").unwrap(); // handle
        read_bytes.put_u64(0x02); // offset
        read_bytes.put_u32(0x03); // length

        assert_eq!(
            Read::try_from(&read_bytes.freeze()),
            Ok(Read {
                id: 0x01,
                handle: String::from("handle"),
                offset: 0x02,
                len: 0x03
            })
        )
    }

    #[test]
    fn test_parse_read_with_invalid_data() {
        assert_eq!(Read::try_from(&Bytes::new()), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_read_with_invalid_id() {
        let read_bytes = BytesMut::new();

        read_bytes.put_u8(0x01); // id

        assert_eq!(Read::try_from(&read_bytes.freeze()), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_read_with_invalid_handle() {
        let read_bytes = BytesMut::new();

        read_bytes.put_u32(0x01); // id
        read_bytes.put_u8(0x02); // invalid handle
    }

    #[test]
    fn test_parse_read_with_invalid_offset() {
        let read_bytes = BytesMut::new();

        read_bytes.put_u32(0x01); // id
        read_bytes.try_put_str("handle").unwrap(); // handle
        read_bytes.put_u8(0x02); // invalid offset

        assert_eq!(Read::try_from(&read_bytes.freeze()), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_read_with_invalid_len() {
        let read_bytes = BytesMut::new();

        read_bytes.put_u32(0x01); // id
        read_bytes.try_put_str("handle").unwrap(); // handle
        read_bytes.put_u64(0x02); // offset
        read_bytes.put_u8(0x03); // invalid length

        assert_eq!(Read::try_from(&read_bytes.freeze()), Err(Error::BadMessage));
    }
}
