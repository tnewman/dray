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
        let permissions = self.decode_permissions();

        format!("{} 0 nobody nobody 0 Jan  1  1970", permissions)
    }

    fn decode_permissions(&self) -> String {
        let permissions = self.file_attributes.permissions.unwrap_or(0);

        let owner = File::decode_permission((permissions >> 6) & 0x7);
        let group = File::decode_permission((permissions >> 3) & 0x7);
        let other = File::decode_permission(permissions & 0x7);

        let directory = if (permissions >> 14) & 0x1 == 0x01 { "d" } else { "-" };

        format!("{}{}{}{}", directory, owner, group, other)
    }

    fn decode_permission(permission: u32) -> String {
        let read = (permission >> 2) & 0x1;
        let write = (permission >> 1) & 0x1;
        let execute = permission & 0x1;

        let read = if read == 0x1 { "r" } else { "-" };
        let write = if write == 0x01 { "w" } else { "-" };
        let execute = if execute == 0x01 { "x" } else { "-" };
        
        format!("{}{}{}", read, write, execute)
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
                ..Default::default()
            }
        };

        assert_eq!("---------- 0 nobody nobody 0 Jan  1  1970", file.get_long_name());
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_700_file() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                permissions: Some(0o700),
                ..Default::default()
            }
        };

        assert_eq!("-rwx------ 0 nobody nobody 0 Jan  1  1970", file.get_long_name());
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_070_file() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                permissions: Some(0o070),
                ..Default::default()
            }
        };

        assert_eq!("----rwx--- 0 nobody nobody 0 Jan  1  1970", file.get_long_name());
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_007_file() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                permissions: Some(0o007),
                ..Default::default()
            }
        };

        assert_eq!("-------rwx 0 nobody nobody 0 Jan  1  1970", file.get_long_name());
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_500_file() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                permissions: Some(0o500),
                ..Default::default()
            }
        };

        assert_eq!("-r-x------ 0 nobody nobody 0 Jan  1  1970", file.get_long_name());
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_700_directory() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                permissions: Some(0o40777),
                ..Default::default()
            }
        };

        assert_eq!("drwxrwxrwx 0 nobody nobody 0 Jan  1  1970", file.get_long_name());
    }

    #[test]
    fn test_from_creates_file_bytes() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                ..Default::default()
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
