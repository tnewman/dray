use bytes::Bytes;

use crate::error::Error;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Write {
    pub id: u32,
    pub handle: String,
    pub offset: u64,
    pub data: Bytes,
}

impl TryFrom<&Bytes> for Write {
    type Error = Error;

    fn try_from(write_bytes: &Bytes) -> Result<Self, Self::Error> {
        let id = write_bytes.try_get_u32()?;
        let handle = write_bytes.try_get_string()?;
        let offset = write_bytes.try_get_u64()?;
        let data_length = write_bytes.try_get_u32()?;
        let data = write_bytes.try_get_bytes(data_length)?;

        Ok(Write {
            id,
            handle,
            offset,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::try_buf::TryBufMut;

    use bytes::{BufMut, BytesMut};

    use std::convert::TryInto;

    #[test]
    fn test_parse_write() {
        let write_bytes = BytesMut::new();

        write_bytes.put_u32(0x01); // id
        write_bytes.try_put_str("handle").unwrap(); // handle
        write_bytes.put_u64(0x02); // offset

        let data = vec![0x01, 0x02];
        write_bytes.put_u32(data.len().try_into().unwrap()); // data length
        write_bytes.put_slice(data.as_slice()); // data

        assert_eq!(
            Write::try_from(&write_bytes.freeze()),
            Ok(Write {
                id: 0x01,
                handle: String::from("handle"),
                offset: 0x02,
                data: Bytes::from(data),
            })
        )
    }

    #[test]
    fn test_parse_write_with_empty_data() {
        assert_eq!(Write::try_from(&Bytes::new()), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_write_with_invalid_id() {
        let write_bytes = BytesMut::new();

        write_bytes.put_u8(0x01); // id

        assert_eq!(
            Write::try_from(&write_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_write_with_invalid_handle() {
        let write_bytes = BytesMut::new();

        write_bytes.put_u32(0x01); // id
        write_bytes.put_u8(0x02); // invalid handle

        assert_eq!(
            Write::try_from(&write_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_write_with_invalid_offset() {
        let write_bytes = BytesMut::new();

        write_bytes.put_u32(0x01); // id

        let handle = "handle".as_bytes();
        write_bytes.put_u32(handle.len().try_into().unwrap()); // handle length
        write_bytes.put_slice(handle); // handle

        write_bytes.put_u8(0x02); // invalid offset

        assert_eq!(
            Write::try_from(&write_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_write_with_invalid_data_length() {
        let write_bytes = BytesMut::new();

        write_bytes.put_u32(0x01); // id

        let handle = "handle".as_bytes();
        write_bytes.put_u32(handle.len().try_into().unwrap()); // handle length
        write_bytes.put_slice(handle); // handle

        write_bytes.put_u64(0x02); // offset
        write_bytes.put_u8(0x01); // invalid data length

        assert_eq!(
            Write::try_from(&write_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_write_with_invalid_data() {
        let write_bytes = BytesMut::new();

        write_bytes.put_u32(0x01); // id

        let handle = "handle".as_bytes();
        write_bytes.put_u32(handle.len().try_into().unwrap()); // handle length
        write_bytes.put_slice(handle); // handle

        write_bytes.put_u64(0x02); // offset
        write_bytes.put_u32(0x02); // data length
        write_bytes.put_u8(0x01); // invalid data

        assert_eq!(
            Write::try_from(&write_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }
}
