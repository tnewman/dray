use crate::error::Error;

use bytes::Buf;
use bytes::{BufMut, Bytes, BytesMut};
use std::convert::From;
use std::convert::TryFrom;

const SIZE: u32 = 0x00000001;
const UIDGID: u32 = 0x00000002;
const PERMISSIONS: u32 = 0x00000004;
const ACMODTIME: u32 = 0x00000008;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct FileAttributes {
    pub size: Option<u64>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub permissions: Option<u32>,
    pub atime: Option<u32>,
    pub mtime: Option<u32>,
}

impl FileAttributes {
    pub fn is_dir(&self) -> bool {
        match self.permissions {
            Some(permissions) => (permissions >> 14) & 0x1 == 0x01,
            None => false,
        }
    }
}

impl TryFrom<&mut Bytes> for FileAttributes {
    type Error = Error;

    fn try_from(file_attributes_bytes: &mut Bytes) -> Result<Self, Self::Error> {
        let attributes = file_attributes_bytes.try_get_u32()?;

        Ok(FileAttributes {
            size: if attributes & SIZE != 0 {
                Some(file_attributes_bytes.try_get_u64()?)
            } else {
                None
            },
            uid: if attributes & UIDGID != 0 {
                Some(file_attributes_bytes.try_get_u32()?)
            } else {
                None
            },
            gid: if attributes & UIDGID != 0 {
                Some(file_attributes_bytes.try_get_u32()?)
            } else {
                None
            },
            permissions: if attributes & PERMISSIONS != 0 {
                Some(file_attributes_bytes.try_get_u32()?)
            } else {
                None
            },
            atime: if attributes & ACMODTIME != 0 {
                Some(file_attributes_bytes.try_get_u32()?)
            } else {
                None
            },
            mtime: if attributes & ACMODTIME != 0 {
                Some(file_attributes_bytes.try_get_u32()?)
            } else {
                None
            },
        })
    }
}

impl From<&FileAttributes> for Bytes {
    fn from(file_attributes: &FileAttributes) -> Self {
        let mut attributes: u32 = 0;

        if file_attributes.size.is_some() {
            attributes |= SIZE;
        }

        if file_attributes.uid.is_some() || file_attributes.gid.is_some() {
            attributes |= UIDGID;
        }

        if file_attributes.permissions.is_some() {
            attributes |= PERMISSIONS;
        }

        if file_attributes.atime.is_some() || file_attributes.mtime.is_some() {
            attributes |= ACMODTIME;
        }

        let mut attribute_bytes = BytesMut::new();

        attribute_bytes.put_u32(attributes);

        if file_attributes.size.is_some() {
            attribute_bytes.put_u64(file_attributes.size.unwrap_or(0));
        }

        if file_attributes.uid.is_some() || file_attributes.gid.is_some() {
            attribute_bytes.put_u32(file_attributes.uid.unwrap_or(0));
            attribute_bytes.put_u32(file_attributes.gid.unwrap_or(0));
        }

        if file_attributes.permissions.is_some() {
            attribute_bytes.put_u32(file_attributes.permissions.unwrap_or(0));
        }

        if file_attributes.atime.is_some() || file_attributes.mtime.is_some() {
            attribute_bytes.put_u32(file_attributes.atime.unwrap_or(0));
            attribute_bytes.put_u32(file_attributes.mtime.unwrap_or(0));
        }

        attribute_bytes.freeze()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bytes::Buf;

    #[test]
    fn test_file_attributes_provides_default_values() {
        let file_attributes = FileAttributes {
            ..Default::default()
        };

        assert_eq!(
            FileAttributes {
                size: None,
                uid: None,
                gid: None,
                permissions: None,
                atime: None,
                mtime: None,
            },
            file_attributes
        )
    }

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

        let mut file_attributes_bytes = Bytes::from(&file_attributes);

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

        let mut file_attributes_bytes = Bytes::from(&file_attributes);

        assert_eq!(0x00000000, file_attributes_bytes.get_u32());
        assert_eq!(0, file_attributes_bytes.remaining());
    }

    #[test]
    fn test_try_from_vector_creates_file_attributes_with_set_fields() {
        let mut file_attributes_bytes = BytesMut::new();

        file_attributes_bytes.put_u32(0x0000000F);
        file_attributes_bytes.put_u64(1000);
        file_attributes_bytes.put_u32(100);
        file_attributes_bytes.put_u32(200);
        file_attributes_bytes.put_u32(777);
        file_attributes_bytes.put_u32(1608671340);
        file_attributes_bytes.put_u32(1608671341);

        let file_attributes =
            FileAttributes::try_from(&mut file_attributes_bytes.freeze()).unwrap();

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
        let mut file_attributes_bytes = BytesMut::new();

        file_attributes_bytes.put_u32(0x00000000);
        file_attributes_bytes.put_u64(1000);
        file_attributes_bytes.put_u32(100);
        file_attributes_bytes.put_u32(200);
        file_attributes_bytes.put_u32(777);
        file_attributes_bytes.put_u32(1608671340);
        file_attributes_bytes.put_u32(1608671341);

        let file_attributes =
            FileAttributes::try_from(&mut file_attributes_bytes.freeze()).unwrap();

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
        let mut file_attributes_bytes = BytesMut::new();

        file_attributes_bytes.put_slice(&[0x01]);

        assert_eq!(
            Error::BadMessage,
            FileAttributes::try_from(&mut file_attributes_bytes.freeze()).unwrap_err()
        );
    }

    #[test]
    fn test_is_dir_returns_true_with_directory_permission_bit_set() {
        assert!(FileAttributes {
            permissions: Some(0o40777),
            ..Default::default()
        }
        .is_dir());
    }

    #[test]
    fn test_is_dir_returns_false_with_directory_permission_bit_unset() {
        assert!(!FileAttributes {
            permissions: Some(0o00777),
            ..Default::default()
        }
        .is_dir());
    }

    #[test]
    fn test_is_dir_returns_false_with_missing_permissions() {
        assert!(!FileAttributes {
            permissions: None,
            ..Default::default()
        }
        .is_dir());
    }
}
