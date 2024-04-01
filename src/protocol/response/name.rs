use crate::protocol::file_attributes::FileAttributes;

use bytes::{BufMut, Bytes, BytesMut};

use chrono::DateTime;

use std::convert::From;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct File {
    pub file_name: String,
    pub file_attributes: FileAttributes,
}

impl File {
    pub fn get_long_name(&self) -> String {
        let permissions = self.decode_permissions();
        let size = self.file_attributes.size.unwrap_or(0);
        let uid = self.file_attributes.uid.unwrap_or(0);
        let gid = self.file_attributes.gid.unwrap_or(0);

        let datetime = DateTime::from_timestamp(self.file_attributes.mtime.unwrap_or(0) as i64, 0)
            .unwrap_or_default();
        let datetime = datetime.format("%b %d %Y %H:%M");

        format!(
            "{} 0 {} {} {} {} {}",
            permissions, uid, gid, size, datetime, self.file_name
        )
    }

    fn decode_permissions(&self) -> String {
        let permissions = self.file_attributes.permissions.unwrap_or(0);

        let owner = File::decode_permission((permissions >> 6) & 0x7);
        let group = File::decode_permission((permissions >> 3) & 0x7);
        let other = File::decode_permission(permissions & 0x7);

        let directory = if (permissions >> 14) & 0x1 == 0x01 {
            "d"
        } else {
            "-"
        };

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
    #[tracing::instrument]
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
            },
        };

        assert_eq!(
            "---------- 0 0 0 0 Jan 01 1970 00:00 file",
            file.get_long_name()
        );
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_700_file() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                permissions: Some(0o700),
                ..Default::default()
            },
        };

        assert_eq!(
            "-rwx------ 0 0 0 0 Jan 01 1970 00:00 file",
            file.get_long_name()
        );
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_070_file() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                permissions: Some(0o070),
                ..Default::default()
            },
        };

        assert_eq!(
            "----rwx--- 0 0 0 0 Jan 01 1970 00:00 file",
            file.get_long_name()
        );
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_007_file() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                permissions: Some(0o007),
                ..Default::default()
            },
        };

        assert_eq!(
            "-------rwx 0 0 0 0 Jan 01 1970 00:00 file",
            file.get_long_name()
        );
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_500_file() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                permissions: Some(0o500),
                ..Default::default()
            },
        };

        assert_eq!(
            "-r-x------ 0 0 0 0 Jan 01 1970 00:00 file",
            file.get_long_name()
        );
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_777_directory() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                permissions: Some(0o40777),
                ..Default::default()
            },
        };

        assert_eq!(
            "drwxrwxrwx 0 0 0 0 Jan 01 1970 00:00 file",
            file.get_long_name()
        );
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_filesize() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                size: Some(1000),
                ..Default::default()
            },
        };

        assert_eq!(
            "---------- 0 0 0 1000 Jan 01 1970 00:00 file",
            file.get_long_name()
        );
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_uid_gid() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                uid: Some(1000),
                gid: Some(2000),
                ..Default::default()
            },
        };

        assert_eq!(
            "---------- 0 1000 2000 0 Jan 01 1970 00:00 file",
            file.get_long_name()
        );
    }

    #[test]
    fn test_get_long_name_creates_long_name_with_mtime() {
        let file = File {
            file_name: String::from("file"),
            file_attributes: FileAttributes {
                mtime: Some(1000000000),
                ..Default::default()
            },
        };

        assert_eq!(
            "---------- 0 0 0 0 Sep 09 2001 01:46 file",
            file.get_long_name()
        );
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
        let long_name = "---------- 0 0 0 0 Jan 01 1970 00:00 file";
        assert_eq!(long_name.len() as u32, file_bytes.get_u32());
        assert_eq!(
            long_name.as_bytes(),
            &file_bytes.copy_to_bytes(long_name.len())[..]
        );
        assert!(file_bytes.has_remaining()); // has file attributes
    }
}
