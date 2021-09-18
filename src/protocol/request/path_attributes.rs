use crate::error::Error;
use crate::protocol::file_attributes::FileAttributes;
use crate::try_buf::TryBuf;

use bytes::Bytes;
use std::convert::TryFrom;

use super::RequestId;

#[derive(Debug, PartialEq)]
pub struct PathAttributes {
    pub id: u32,
    pub path: String,
    pub file_attributes: FileAttributes,
}

impl RequestId for PathAttributes {
    fn get_request_id(&self) -> u32 {
        self.id
    }
}

impl TryFrom<&mut Bytes> for PathAttributes {
    type Error = Error;

    fn try_from(path_attributes_bytes: &mut Bytes) -> Result<Self, Self::Error> {
        let id = path_attributes_bytes.try_get_u32()?;
        let path = path_attributes_bytes.try_get_string()?;
        let file_attributes = FileAttributes::try_from(path_attributes_bytes)?;

        Ok(PathAttributes {
            id,
            path,
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
    fn test_parse_path_attributes() {
        let mut path_attributes_bytes = BytesMut::new();

        path_attributes_bytes.put_u32(0x01); // id
        path_attributes_bytes.try_put_str("/file/path").unwrap(); // filename

        let file_attributes = get_file_attributes();
        path_attributes_bytes.put_slice(&Bytes::from(&file_attributes)); // file attributes

        assert_eq!(
            PathAttributes::try_from(&mut path_attributes_bytes.freeze()),
            Ok(PathAttributes {
                id: 0x01,
                path: String::from("/file/path"),
                file_attributes
            })
        );
    }

    #[test]
    fn test_parse_path_attributes_with_invalid_id() {
        let mut path_attributes_bytes = BytesMut::new();

        path_attributes_bytes.put_u8(0x01); // invalid id

        assert_eq!(
            PathAttributes::try_from(&mut path_attributes_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_path_attributes_with_invalid_path() {
        let mut path_attributes_bytes = BytesMut::new();

        path_attributes_bytes.put_u32(0x01); // id
        path_attributes_bytes.put_u32(0x01); // invalid filename length

        assert_eq!(
            PathAttributes::try_from(&mut path_attributes_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_path_attributes_with_invalid_file_attributes() {
        let mut path_attributes_bytes = BytesMut::new();

        path_attributes_bytes.put_u32(0x01); // id
        path_attributes_bytes.try_put_str("/file/path").unwrap(); // filename

        path_attributes_bytes.put_u8(0x01); // invalid attributes

        assert_eq!(
            PathAttributes::try_from(&mut path_attributes_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_get_request_id() {
        let path_attributes = PathAttributes {
            id: 1000,
            path: String::from("path"),
            file_attributes: get_file_attributes(),
        };

        assert_eq!(1000, path_attributes.get_request_id());
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
