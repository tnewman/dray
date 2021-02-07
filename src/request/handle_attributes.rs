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

impl TryFrom<&[u8]> for HandleAttributes {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut handle_attributes_bytes = item;

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

    use bytes::BufMut;

    #[test]
    fn test_parse_handle_attributes() {
        let mut handle_attributes_bytes = vec![];

        handle_attributes_bytes.put_u32(0x01); // id
        handle_attributes_bytes.try_put_str("handle").unwrap(); // handle

        let file_attributes = get_file_attributes();
        handle_attributes_bytes.put_slice(Vec::from(&file_attributes).as_slice()); // file attributes

        assert_eq!(
            HandleAttributes::try_from(handle_attributes_bytes.as_slice()),
            Ok(HandleAttributes {
                id: 0x01,
                handle: String::from("handle"),
                file_attributes
            })
        );
    }

    #[test]
    fn test_parse_handle_attributes_with_invalid_id() {
        let mut handle_attributes_bytes = vec![];

        handle_attributes_bytes.put_u8(0x01); // invalid id

        assert_eq!(
            HandleAttributes::try_from(handle_attributes_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_handle_attributes_with_invalid_handle() {
        let mut handle_attributes_bytes = vec![];

        handle_attributes_bytes.put_u32(0x01); // id

        handle_attributes_bytes.put_u32(0x01); // invalid filename length

        assert_eq!(
            HandleAttributes::try_from(handle_attributes_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_handle_attributes_with_invalid_file_attributes() {
        let mut handle_attributes_bytes = vec![];

        handle_attributes_bytes.put_u32(0x01); // id
        handle_attributes_bytes.try_put_str("handle").unwrap(); // handle

        handle_attributes_bytes.put_u8(0x01);

        assert_eq!(
            HandleAttributes::try_from(handle_attributes_bytes.as_slice()),
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
