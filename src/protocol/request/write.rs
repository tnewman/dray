use bytes::{Buf, Bytes};
use tracing::Level;

use crate::error::Error;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;
use std::fmt::Debug;

use super::RequestId;

#[derive(PartialEq, Eq)]
pub struct Write {
    pub id: u32,
    pub handle: String,
    pub offset: u64,
    pub data: Bytes,
}

impl RequestId for Write {
    fn get_request_id(&self) -> u32 {
        self.id
    }
}

impl TryFrom<&mut Bytes> for Write {
    type Error = Error;

    #[tracing::instrument(level = Level::DEBUG)]
    fn try_from(write_bytes: &mut Bytes) -> Result<Self, Self::Error> {
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

impl Debug for Write {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Write")
            .field("id", &self.id)
            .field("handle", &self.handle)
            .field("offset", &self.offset)
            .field("len", &self.data.len())
            .finish()
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
        let mut write_bytes = BytesMut::new();

        write_bytes.put_u32(0x01); // id
        write_bytes.try_put_str("handle").unwrap(); // handle
        write_bytes.put_u64(0x02); // offset

        let data = vec![0x01, 0x02];
        write_bytes.put_u32(data.len().try_into().unwrap()); // data length
        write_bytes.put_slice(data.as_slice()); // data

        assert_eq!(
            Write::try_from(&mut write_bytes.freeze()),
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
        assert_eq!(Write::try_from(&mut Bytes::new()), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_write_with_invalid_id() {
        let mut write_bytes = BytesMut::new();

        write_bytes.put_u8(0x01); // id

        assert_eq!(
            Write::try_from(&mut write_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_write_with_invalid_handle() {
        let mut write_bytes = BytesMut::new();

        write_bytes.put_u32(0x01); // id
        write_bytes.put_u8(0x02); // invalid handle

        assert_eq!(
            Write::try_from(&mut write_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_write_with_invalid_offset() {
        let mut write_bytes = BytesMut::new();

        write_bytes.put_u32(0x01); // id

        let handle = "handle".as_bytes();
        write_bytes.put_u32(handle.len().try_into().unwrap()); // handle length
        write_bytes.put_slice(handle); // handle

        write_bytes.put_u8(0x02); // invalid offset

        assert_eq!(
            Write::try_from(&mut write_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_write_with_invalid_data_length() {
        let mut write_bytes = BytesMut::new();

        write_bytes.put_u32(0x01); // id

        let handle = "handle".as_bytes();
        write_bytes.put_u32(handle.len().try_into().unwrap()); // handle length
        write_bytes.put_slice(handle); // handle

        write_bytes.put_u64(0x02); // offset
        write_bytes.put_u8(0x01); // invalid data length

        assert_eq!(
            Write::try_from(&mut write_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_write_with_invalid_data() {
        let mut write_bytes = BytesMut::new();

        write_bytes.put_u32(0x01); // id

        let handle = "handle".as_bytes();
        write_bytes.put_u32(handle.len().try_into().unwrap()); // handle length
        write_bytes.put_slice(handle); // handle

        write_bytes.put_u64(0x02); // offset
        write_bytes.put_u32(0x02); // data length
        write_bytes.put_u8(0x01); // invalid data

        assert_eq!(
            Write::try_from(&mut write_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_debug_formats_efficient_debug_string() {
        let write = Write {
            id: 0x01,
            handle: "handle".to_string(),
            offset: 0x02,
            data: Bytes::from(vec![0x01, 0x02, 0x03]),
        };

        assert_eq!(
            "Write { id: 1, handle: \"handle\", offset: 2, len: 3 }",
            format!("{:?}", write)
        );
    }

    #[test]
    fn test_get_request_id() {
        let write = Write {
            id: 1000,
            handle: String::from("handle"),
            offset: 0,
            data: Bytes::from(vec![]),
        };

        assert_eq!(1000, write.get_request_id());
    }
}
