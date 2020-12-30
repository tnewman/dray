use crate::error::Error;
use crate::file_attributes::FileAttributes;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct PathAttributes {
    pub id: u32,
    pub path: String,
    pub file_attributes: FileAttributes,
}

impl TryFrom<&[u8]> for PathAttributes {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut path_attributes_bytes = item;

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

    use bytes::BufMut;
    use std::convert::TryInto;

    #[test]
    fn test_parse_path_attributes() {
        let mut path_attributes_bytes = vec![];

        path_attributes_bytes.put_u32(0x01); // id

        let filename = "/file/path".as_bytes();
        path_attributes_bytes.put_u32(filename.len().try_into().unwrap()); // filename length
        path_attributes_bytes.put_slice(filename); // filename

        let file_attributes = get_file_attributes();
        path_attributes_bytes.put_slice(Vec::from(&file_attributes).as_slice()); // file attributes

        assert_eq!(
            PathAttributes::try_from(path_attributes_bytes.as_slice()),
            Ok(PathAttributes {
                id: 0x01,
                path: String::from("/file/path"),
                file_attributes
            })
        );
    }

    #[test]
    fn test_parse_path_attributes_with_invalid_id() {
        let mut path_attributes_bytes = vec![];

        path_attributes_bytes.put_u8(0x01); // invalid id

        assert_eq!(
            PathAttributes::try_from(path_attributes_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_path_attributes_with_invalid_path() {
        let mut path_attributes_bytes = vec![];

        path_attributes_bytes.put_u32(0x01); // id

        path_attributes_bytes.put_u32(0x01); // invalid filename length

        assert_eq!(
            PathAttributes::try_from(path_attributes_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_path_attributes_with_invalid_file_attributes() {
        let mut path_attributes_bytes = vec![];

        path_attributes_bytes.put_u32(0x01); // id

        let filename = "/file/path".as_bytes();
        path_attributes_bytes.put_u32(filename.len().try_into().unwrap()); // filename length
        path_attributes_bytes.put_slice(filename); // filename

        path_attributes_bytes.put_u8(0x01);

        assert_eq!(
            PathAttributes::try_from(path_attributes_bytes.as_slice()),
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
