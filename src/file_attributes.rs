use super::error::Error;
use bytes::Buf;
use bytes::BufMut;
use std::convert::From;
use std::convert::TryFrom;

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

impl TryFrom<&[u8]> for FileAttributes {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut item = item;

        if item.remaining() != 32 {
            return Err(Error::BadMessage);
        }

        let attributes = item.get_u32();
        let size = item.get_u64();
        let uid = item.get_u32();
        let gid = item.get_u32();
        let permissions = item.get_u32();
        let atime = item.get_u32();
        let mtime = item.get_u32();

        Ok(FileAttributes {
            size: if attributes & SIZE != 0 {
                Some(size)
            } else {
                None
            },
            uid: if attributes & UIDGID != 0 {
                Some(uid)
            } else {
                None
            },
            gid: if attributes & UIDGID != 0 {
                Some(gid)
            } else {
                None
            },
            permissions: if attributes & PERMISSIONS != 0 {
                Some(permissions)
            } else {
                None
            },
            atime: if attributes & ACMODTIME != 0 {
                Some(atime)
            } else {
                None
            },
            mtime: if attributes & ACMODTIME != 0 {
                Some(mtime)
            } else {
                None
            },
        })
    }
}

impl From<&FileAttributes> for Vec<u8> {
    fn from(item: &FileAttributes) -> Self {
        let mut attributes: u32 = 0;

        if item.size.is_some() {
            attributes |= SIZE;
        }

        if item.uid.is_some() || item.gid.is_some() {
            attributes |= UIDGID;
        }

        if item.permissions.is_some() {
            attributes |= PERMISSIONS;
        }

        if item.atime.is_some() || item.mtime.is_some() {
            attributes |= ACMODTIME;
        }

        let mut attribute_bytes = vec![];

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

    #[test]
    fn test_try_from_vector_creates_file_attributes_with_set_fields() {
        let mut file_attributes_bytes = vec![];

        file_attributes_bytes.put_u32(0x0000000F);
        file_attributes_bytes.put_u64(1000);
        file_attributes_bytes.put_u32(100);
        file_attributes_bytes.put_u32(200);
        file_attributes_bytes.put_u32(777);
        file_attributes_bytes.put_u32(1608671340);
        file_attributes_bytes.put_u32(1608671341);

        let file_attributes = FileAttributes::try_from(&*file_attributes_bytes).unwrap();

        assert_eq!(
            FileAttributes {
                size: Some(1000),
                uid: Some(100),
                gid: Some(200),
                permissions: Some(777),
                atime: Some(1608671340),
                mtime: Some(1608671341)
            },
            file_attributes
        );
    }

    #[test]
    fn test_try_from_vector_creates_file_attributes_with_unset_fields() {
        let mut file_attributes_bytes = vec![];

        file_attributes_bytes.put_u32(0x00000000);
        file_attributes_bytes.put_u64(1000);
        file_attributes_bytes.put_u32(100);
        file_attributes_bytes.put_u32(200);
        file_attributes_bytes.put_u32(777);
        file_attributes_bytes.put_u32(1608671340);
        file_attributes_bytes.put_u32(1608671341);

        let file_attributes = FileAttributes::try_from(&*file_attributes_bytes).unwrap();

        assert_eq!(
            FileAttributes {
                size: None,
                uid: None,
                gid: None,
                permissions: None,
                atime: None,
                mtime: None
            },
            file_attributes
        );
    }

    #[test]
    fn test_try_from_vector_returns_error_with_missing_data() {
        let file_attributes_bytes: &[u8] = &vec![0; 27];

        assert_eq!(
            Error::BadMessage,
            FileAttributes::try_from(file_attributes_bytes).unwrap_err()
        );
    }
}
