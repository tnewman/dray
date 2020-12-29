use crate::error::Error;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct ReadWrite {
    id: u32,
    handle: String,
    offset: u64,
    len: u32,
}

impl TryFrom<&[u8]> for ReadWrite {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut read_write_bytes = item;

        let id = read_write_bytes.try_get_u32()?;
        let handle = read_write_bytes.try_get_string()?;
        let offset = read_write_bytes.try_get_u64()?;
        let len = read_write_bytes.try_get_u32()?;

        Ok(ReadWrite {
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

    use bytes::BufMut;
    use std::{convert::TryInto, vec};

    #[test]
    fn test_parse_read_write() {
        let mut read_write_bytes = vec![];

        read_write_bytes.put_u32(0x01); // id

        let handle = "handle".as_bytes();
        read_write_bytes.put_u32(handle.len().try_into().unwrap()); // handle length
        read_write_bytes.put_slice(handle); // handle

        read_write_bytes.put_u64(0x02); // offset
        read_write_bytes.put_u32(0x03); // length

        assert_eq!(
            ReadWrite::try_from(read_write_bytes.as_slice()),
            Ok(ReadWrite {
                id: 0x01,
                handle: String::from("handle"),
                offset: 0x02,
                len: 0x03
            })
        )
    }

    #[test]
    fn test_parse_read_write_with_invalid_data() {
        assert_eq!(ReadWrite::try_from(&vec![][..]), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_read_write_with_invalid_id() {
        let mut read_write_bytes = vec![];

        read_write_bytes.put_u8(0x01); // id

        assert_eq!(
            ReadWrite::try_from(read_write_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_read_write_with_invalid_handle() {
        let mut read_write_bytes = vec![];

        read_write_bytes.put_u32(0x01); // id
        read_write_bytes.put_u8(0x02); // invalid handle
    }

    #[test]
    fn test_parse_read_write_with_invalid_offset() {
        let mut read_write_bytes = vec![];

        read_write_bytes.put_u32(0x01); // id

        let handle = "handle".as_bytes();
        read_write_bytes.put_u32(handle.len().try_into().unwrap()); // handle length
        read_write_bytes.put_slice(handle); // handle

        read_write_bytes.put_u8(0x02); // invalid offset

        assert_eq!(
            ReadWrite::try_from(read_write_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_read_write_with_invalid_len() {
        let mut read_write_bytes = vec![];

        read_write_bytes.put_u32(0x01); // id

        let handle = "handle".as_bytes();
        read_write_bytes.put_u32(handle.len().try_into().unwrap()); // handle length
        read_write_bytes.put_slice(handle); // handle

        read_write_bytes.put_u64(0x02); // offset
        read_write_bytes.put_u8(0x03); // invalid length

        assert_eq!(
            ReadWrite::try_from(read_write_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }
}
