use bytes::Bytes;

use crate::error::Error;
use crate::file_attributes::FileAttributes;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct HandleAttributes {
    pub id: u32,
    pub handle: String,
    pub file_attributes: FileAttributes,
}

impl TryFrom<&mut Bytes> for HandleAttributes {
    type Error = Error;

    fn try_from(handle_attributes_bytes: &mut Bytes) -> Result<Self, Self::Error> {
        let id = handle_attributes_bytes.try_get_u32()?;
        let handle = handle_attributes_bytes.try_get_string()?;
        let file_attributes = FileAttributes::try_from(handle_attributes_bytes)?;

        Ok(HandleAttributes {
            id,
            handle,
            file_attributes,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::try_buf::TryBufMut;

    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_parse_handle_attributes() {
        let mut handle_attributes_bytes = BytesMut::new();

        handle_attributes_bytes.put_u32(0x01); // id
        handle_attributes_bytes.try_put_str("handle").unwrap(); // handle

        let file_attributes = get_file_attributes();
        handle_attributes_bytes.put_slice(&mut Bytes::from(&file_attributes)); // file attributes

        assert_eq!(
            HandleAttributes::try_from(&mut handle_attributes_bytes.freeze()),
            Ok(HandleAttributes {
                id: 0x01,
                handle: String::from("handle"),
                file_attributes
            })
        );
    }

    #[test]
    fn test_parse_handle_attributes_with_invalid_id() {
        let mut handle_attributes_bytes = BytesMut::new();

        handle_attributes_bytes.put_u8(0x01); // invalid id

        assert_eq!(
            HandleAttributes::try_from(&mut handle_attributes_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_handle_attributes_with_invalid_handle() {
        let mut handle_attributes_bytes = BytesMut::new();

        handle_attributes_bytes.put_u32(0x01); // id
        handle_attributes_bytes.put_u32(0x01); // invalid filename length

        assert_eq!(
            HandleAttributes::try_from(&mut handle_attributes_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_handle_attributes_with_invalid_file_attributes() {
        let mut handle_attributes_bytes = BytesMut::new();

        handle_attributes_bytes.put_u32(0x01); // id
        handle_attributes_bytes.try_put_str("handle").unwrap(); // handle

        handle_attributes_bytes.put_u8(0x01);

        assert_eq!(
            HandleAttributes::try_from(&mut handle_attributes_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    fn get_file_attributes() -> FileAttributes {
        FileAttributes {
            size: None,
            uid: None,
            gid: None,
            permissions: None,
            atime: None,
            mtime: None,
        }
    }
}
