use bytes::Buf;
use bytes::BufMut;
use std::convert::From;

const SIZE: u32 = 0x00000001;
const UIDGID: u32 = 0x00000002;
const PERMISSIONS: u32 = 0x00000004;
const ACMODTIME: u32 = 0x00000008;

#[derive(Debug, PartialEq)]
pub struct FileAttributes {
    size: Option<u64>,
    uid: Option<u32>,
    gid: Option<u32>,
    permissions: Option<u32>,
    atime: Option<u32>,
    mtime: Option<u32>,
}

impl From<&mut [u8]> for FileAttributes {
    fn from(item: &mut [u8]) -> Self {
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

impl From<&FileAttributes> for Vec<u8> {
    fn from(item: &FileAttributes) -> Self {
        let mut attributes: u32 = 0;

        if item.size.is_some() {
            attributes = attributes | SIZE;
        }

        if item.uid.is_some() {
            attributes = attributes | UIDGID;
        }

        if item.gid.is_some() {
            attributes = attributes | UIDGID;
        }

        if item.permissions.is_some() {
            attributes = attributes | PERMISSIONS;
        }

        if item.atime.is_some() {
            attributes = attributes | ACMODTIME;
        }

        if item.mtime.is_some() {
            attributes = attributes | ACMODTIME;
        }

        let mut attribute_bytes = vec![];

        println!("{}", attributes);

        attribute_bytes.put_u32(attributes);
        attribute_bytes.put_u64(item.size.unwrap_or(0));
        attribute_bytes.put_u32(item.uid.unwrap_or(0));
        attribute_bytes.put_u32(item.gid.unwrap_or(0));
        attribute_bytes.put_u32(item.permissions.unwrap_or(0));
        attribute_bytes.put_u32(item.atime.unwrap_or(0));
        attribute_bytes.put_u32(item.mtime.unwrap_or(0));

        attribute_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_file_attributes_creates_vector_with_set_fields() {
        let file_attributes = FileAttributes {
            size: Some(1000),
            uid: Some(100),
            gid: Some(200),
            permissions: Some(777),
            atime: Some(1608671340),
            mtime: Some(1608671341),
        };

        let mut file_attributes_bytes: &[u8] = &Vec::from(&file_attributes);

        assert_eq!(0x0000000F, file_attributes_bytes.get_u32());
        assert_eq!(1000, file_attributes_bytes.get_u64());
        assert_eq!(100, file_attributes_bytes.get_u32());
        assert_eq!(200, file_attributes_bytes.get_u32());
        assert_eq!(777, file_attributes_bytes.get_u32());
        assert_eq!(1608671340, file_attributes_bytes.get_u32());
        assert_eq!(1608671341, file_attributes_bytes.get_u32());
    }

    #[test]
    fn test_from_file_attributes_creates_vector_with_empty_fields() {
        let file_attributes = FileAttributes {
            size: None,
            uid: None,
            gid: None,
            permissions: None,
            atime: None,
            mtime: None,
        };

        let mut file_attributes_bytes: &[u8] = &Vec::from(&file_attributes);

        assert_eq!(0x00000000, file_attributes_bytes.get_u32());
        assert_eq!(0, file_attributes_bytes.get_u64());
        assert_eq!(0, file_attributes_bytes.get_u32());
        assert_eq!(0, file_attributes_bytes.get_u32());
        assert_eq!(0, file_attributes_bytes.get_u32());
        assert_eq!(0, file_attributes_bytes.get_u32());
        assert_eq!(0, file_attributes_bytes.get_u32());
    }
}
