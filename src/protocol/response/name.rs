use crate::protocol::file_attributes::FileAttributes;

use bytes::{BufMut, Bytes, BytesMut};
use std::convert::From;
use std::convert::TryInto;

#[derive(Debug, PartialEq)]
pub struct Name {
    pub id: u32,
    pub files: Vec<File>,
}

impl From<&Name> for Bytes {
    fn from(name: &Name) -> Self {
        let mut name_bytes = BytesMut::new();

        name_bytes.put_u32(name.id);
        name_bytes.put_u32(name.files.len().try_into().unwrap());

        for file in &name.files {
            name_bytes.put_slice(&Bytes::from(file));
        }

        name_bytes.freeze()
    }
}

#[derive(Debug, PartialEq)]
pub struct File {
    pub file_name: String,
    pub file_attributes: FileAttributes,
}

impl File {
    pub fn get_long_name(&self) -> String {
        String::from("---------- 0 nobody nobody 0 Jan  1  1970")
    }
}

impl From<&File> for Bytes {
    fn from(item: &File) -> Self {
        let mut file_bytes = BytesMut::new();

        let file_name_bytes = item.file_name.as_bytes();
        file_bytes.put_u32(file_name_bytes.len().try_into().unwrap());
        file_bytes.put_slice(file_name_bytes);

        let long_name = item.get_long_name();
        file_bytes.put_u32(long_name.len().try_into().unwrap());
        file_bytes.put_slice(long_name.as_bytes());

        file_bytes.put_slice(&Bytes::from(&item.file_attributes));

        file_bytes.freeze()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use bytes::Buf;

    #[test]
    fn test_get_long_name_creates_long_name_with_missing_fields() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                size: None,
                uid: None,
                gid: None,
                atime: None,
                mtime: None,
                permissions: None,
            }
        };

        assert_eq!("---------- 0 nobody nobody 0 Jan  1  1970", file.get_long_name());
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_700_file() {

    }

    #[test]
    fn test_get_long_name_creates_long_name_with_070_file() {

    }

    #[test]
    fn test_get_long_name_creates_long_name_with_007_file() {

    }

    #[test]
    fn test_get_long_name_creates_long_name_with_500_file() {

    }

    #[test]
    fn test_get_long_name_creates_long_name_with_700_directory() {

    }

    #[test]
    fn test_from_creates_file_bytes() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                size: None,
                uid: None,
                gid: None,
                permissions: None,
                atime: None,
                mtime: None,
            },
        };

        let file_bytes = &mut Bytes::from(&file);

        assert_eq!(0x04, file_bytes.get_u32());
        assert_eq!(&[0x66, 0x69, 0x6C, 0x65], &file_bytes.copy_to_bytes(4)[..]);
        let long_name = "---------- 0 nobody nobody 0 Jan  1  1970";
        assert_eq!(long_name.len() as u32, file_bytes.get_u32());
        assert_eq!(long_name.as_bytes(), &file_bytes.copy_to_bytes(long_name.len())[..]);
        assert_eq!(true, file_bytes.has_remaining()); // has file attributes
    }
}
