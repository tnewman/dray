use crate::error::Error;
use crate::file_attributes::FileAttributes;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Mkdir {
    id: u32,
    path: String,
    file_attributes: FileAttributes,
}

impl TryFrom<&[u8]> for Mkdir {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut mkdir_bytes = item;

        let id = mkdir_bytes.try_get_u32()?;
        let path = mkdir_bytes.try_get_string()?;
        let file_attributes = FileAttributes::try_from(mkdir_bytes)?;

        Ok(Mkdir {
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
    fn test_parse_mkdir() {
        let mut mkdir_bytes = vec![];

        mkdir_bytes.put_u32(0x01); // id

        let filename = "/file/path".as_bytes();
        mkdir_bytes.put_u32(filename.len().try_into().unwrap()); // filename length
        mkdir_bytes.put_slice(filename); // filename

        let file_attributes = get_file_attributes();
        mkdir_bytes.put_slice(Vec::from(&file_attributes).as_slice()); // file attributes

        assert_eq!(
            Mkdir::try_from(mkdir_bytes.as_slice()),
            Ok(Mkdir {
                id: 0x01,
                path: String::from("/file/path"),
                file_attributes
            })
        );
    }

    #[test]
    fn test_parse_mkdir_with_invalid_id() {
        let mut mkdir_bytes = vec![];

        mkdir_bytes.put_u8(0x01); // invalid id

        assert_eq!(
            Mkdir::try_from(mkdir_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_mkdir_with_invalid_path() {
        let mut mkdir_bytes = vec![];

        mkdir_bytes.put_u32(0x01); // id

        mkdir_bytes.put_u32(0x01); // invalid filename length

        assert_eq!(
            Mkdir::try_from(mkdir_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_mkdir_with_invalid_file_attributes() {
        let mut mkdir_bytes = vec![];

        mkdir_bytes.put_u32(0x01); // id

        let filename = "/file/path".as_bytes();
        mkdir_bytes.put_u32(filename.len().try_into().unwrap()); // filename length
        mkdir_bytes.put_slice(filename); // filename

        mkdir_bytes.put_u8(0x01);

        assert_eq!(
            Mkdir::try_from(mkdir_bytes.as_slice()),
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
