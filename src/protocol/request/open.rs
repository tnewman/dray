use crate::error::Error;
use crate::protocol::file_attributes::FileAttributes;
use crate::try_buf::TryBuf;

use bytes::Bytes;
use std::convert::TryFrom;

use super::RequestId;

const READ: u32 = 0x00000001;
const WRITE: u32 = 0x00000002;
const APPEND: u32 = 0x00000004;
const CREAT: u32 = 0x00000008;
const TRUNC: u32 = 0x00000010;
const EXCL: u32 = 0x00000020;

#[derive(Debug, PartialEq, Eq)]
pub struct Open {
    pub id: u32,
    pub filename: String,
    pub file_attributes: FileAttributes,
    pub open_options: OpenOptions,
}

impl RequestId for Open {
    fn get_request_id(&self) -> u32 {
        self.id
    }
}

impl TryFrom<&mut Bytes> for Open {
    type Error = Error;

    #[tracing::instrument]
    fn try_from(open_bytes: &mut Bytes) -> Result<Self, Self::Error> {
        let id = open_bytes.try_get_u32()?;
        let filename = open_bytes.try_get_string()?;

        let open_options = OpenOptions::try_from(&mut *open_bytes)?;
        let file_attributes = FileAttributes::try_from(&mut *open_bytes)?;

        Ok(Open {
            id,
            filename,
            file_attributes,
            open_options,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct OpenOptions {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub create_new_only: bool,
    pub append: bool,
    pub truncate: bool,
}

impl TryFrom<&mut Bytes> for OpenOptions {
    type Error = Error;

    fn try_from(open_options_bytes: &mut Bytes) -> Result<Self, Self::Error> {
        let file_attributes = open_options_bytes.try_get_u32()?;

        Ok(OpenOptions {
            read: file_attributes & READ == READ,
            write: file_attributes & WRITE == WRITE,
            create: file_attributes & CREAT == CREAT,
            create_new_only: file_attributes & EXCL == EXCL,
            append: file_attributes & APPEND == APPEND,
            truncate: file_attributes & TRUNC == TRUNC,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::try_buf::TryBufMut;

    use bytes::{BufMut, BytesMut};
    use std::convert::TryInto;

    #[test]
    fn test_parse_open() {
        let mut open_bytes = BytesMut::new();

        open_bytes.put_u32(0x01); // id
        open_bytes.try_put_str("/file/path").unwrap(); // filename
        open_bytes.put_u32(0x01); // read flag

        let file_attributes = FileAttributes {
            uid: Some(100),
            ..get_file_attributes()
        };

        open_bytes.put_slice(&Bytes::from(&file_attributes)); // file attributes

        assert_eq!(
            Open::try_from(&mut open_bytes.freeze()),
            Ok(Open {
                id: 0x01,
                filename: String::from("/file/path"),
                open_options: OpenOptions {
                    read: true,
                    ..get_open_options()
                },
                file_attributes: FileAttributes {
                    uid: Some(100),
                    gid: Some(0),
                    ..get_file_attributes()
                }
            })
        );
    }

    #[test]
    fn test_parse_open_with_empty_data() {
        assert_eq!(Open::try_from(&mut Bytes::new()), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_open_with_invalid_id() {
        let mut open_bytes = BytesMut::new();

        open_bytes.put_u8(0x00);

        assert_eq!(
            Open::try_from(&mut open_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_open_with_invalid_filename() {
        let mut open_bytes = BytesMut::new();

        open_bytes.put_u32(0x01); // id

        open_bytes.put_u32(1); // filename length

        assert_eq!(
            Open::try_from(&mut open_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_open_with_invalid_open_options() {
        let mut open_bytes = BytesMut::new();

        open_bytes.put_u32(0x01); // id

        let filename = "/file/path".as_bytes();
        open_bytes.put_u32(filename.len().try_into().unwrap()); // filename length
        open_bytes.put_slice(filename); // filename

        assert_eq!(
            Open::try_from(&mut open_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_open_with_invalid_file_attributes() {
        let mut open_bytes = BytesMut::new();

        open_bytes.put_u32(0x01); // id

        let filename = "/file/path".as_bytes();
        open_bytes.put_u32(filename.len().try_into().unwrap()); // filename length
        open_bytes.put_slice(filename); // filename

        open_bytes.put_u32(0x00); // pflags

        assert_eq!(
            Open::try_from(&mut open_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_open_options_with_no_flags() {
        let mut open_options = BytesMut::new();

        open_options.put_u32(0x00);

        assert_eq!(
            OpenOptions::try_from(&mut open_options.freeze()),
            Ok(get_open_options())
        );
    }

    #[test]
    fn test_parse_open_options_with_read_flag() {
        let mut open_options_bytes = BytesMut::new();

        open_options_bytes.put_u32(0x01);

        assert_eq!(
            OpenOptions::try_from(&mut open_options_bytes.freeze()),
            Ok(OpenOptions {
                read: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_open_options_with_write_flag() {
        let mut open_options_bytes = BytesMut::new();

        open_options_bytes.put_u32(0x02);

        assert_eq!(
            OpenOptions::try_from(&mut open_options_bytes.freeze()),
            Ok(OpenOptions {
                write: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_open_options_with_create_flag() {
        let mut open_options_bytes = BytesMut::new();

        open_options_bytes.put_u32(0x08);

        assert_eq!(
            OpenOptions::try_from(&mut open_options_bytes.freeze()),
            Ok(OpenOptions {
                create: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_open_options_with_create_new_only_flag() {
        let mut open_options_bytes = BytesMut::new();

        open_options_bytes.put_u32(0x20);

        assert_eq!(
            OpenOptions::try_from(&mut open_options_bytes.freeze()),
            Ok(OpenOptions {
                create_new_only: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_open_options_with_append_flag() {
        let mut open_options_bytes = BytesMut::new();

        open_options_bytes.put_u32(0x04);

        assert_eq!(
            OpenOptions::try_from(&mut open_options_bytes.freeze()),
            Ok(OpenOptions {
                append: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_open_options_with_truncate_flag() {
        let mut open_options_bytes = BytesMut::new();

        open_options_bytes.put_u32(0x10);

        assert_eq!(
            OpenOptions::try_from(&mut open_options_bytes.freeze()),
            Ok(OpenOptions {
                truncate: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_invalid_open_options() {
        assert_eq!(
            OpenOptions::try_from(&mut Bytes::new()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_get_request_id() {
        let open = Open {
            id: 1000,
            filename: String::from("file"),
            file_attributes: get_file_attributes(),
            open_options: get_open_options(),
        };

        assert_eq!(1000, open.get_request_id());
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

    fn get_open_options() -> OpenOptions {
        OpenOptions {
            read: false,
            write: false,
            create: false,
            create_new_only: false,
            append: false,
            truncate: false,
        }
    }
}
