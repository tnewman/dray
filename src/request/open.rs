use crate::error::Error;
use crate::file_attributes::FileAttributes;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

const READ: u32 = 0x00000001;
const WRITE: u32 = 0x00000002;
const APPEND: u32 = 0x00000004;
const CREAT: u32 = 0x00000008;
const TRUNC: u32 = 0x00000010;
const EXCL: u32 = 0x00000020;

#[derive(Debug, PartialEq)]
pub struct Open {
    pub id: u32,
    pub filename: String,
    pub file_attributes: FileAttributes,
    pub open_options: OpenOptions,
}

impl TryFrom<&[u8]> for Open {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut bytes = item;

        let id = bytes.try_get_u32()?;
        let filename = bytes.try_get_string()?;
        let open_options = OpenOptions::try_from(bytes)?;
        let file_attributes = FileAttributes::try_from(bytes)?;

        Ok(Open {
            id,
            filename,
            file_attributes,
            open_options,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct OpenOptions {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub create_new_only: bool,
    pub append: bool,
    pub truncate: bool,
}

impl TryFrom<&[u8]> for OpenOptions {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut bytes = item;

        let file_attributes = bytes.try_get_u32()?;

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

    use bytes::BufMut;
    use std::convert::TryInto;

    #[test]
    fn test_parse_open() {
        let mut open_bytes = vec![];

        open_bytes.put_u32(0x01); // id
        open_bytes.try_put_str("/file/path").unwrap(); // filename
        open_bytes.put_u32(0x00); // pflags

        let file_attributes = get_file_attributes();

        open_bytes.put_slice(Vec::from(&file_attributes).as_slice()); // file attributes

        assert_eq!(
            Open::try_from(open_bytes.as_slice()),
            Ok(Open {
                id: 0x01,
                filename: String::from("/file/path"),
                open_options: get_open_options(),
                file_attributes
            })
        );
    }

    #[test]
    fn test_parse_open_with_empty_data() {
        assert_eq!(Open::try_from(&vec![][..]), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_open_with_invalid_id() {
        let mut open_bytes = vec![];
        open_bytes.put_u8(0x00);

        assert_eq!(Open::try_from(&open_bytes[..]), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_open_with_invalid_filename() {
        let mut open_bytes = vec![];
        open_bytes.put_u32(0x01); // id

        open_bytes.put_u32(1); // filename length

        assert_eq!(Open::try_from(&open_bytes[..]), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_open_with_invalid_open_options() {
        let mut open_bytes = vec![];

        open_bytes.put_u32(0x01); // id

        let filename = "/file/path".as_bytes();
        open_bytes.put_u32(filename.len().try_into().unwrap()); // filename length
        open_bytes.put_slice(filename); // filename

        assert_eq!(
            Open::try_from(open_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_open_with_invalid_file_attributes() {
        let mut open_bytes = vec![];

        open_bytes.put_u32(0x01); // id

        let filename = "/file/path".as_bytes();
        open_bytes.put_u32(filename.len().try_into().unwrap()); // filename length
        open_bytes.put_slice(filename); // filename

        open_bytes.put_u32(0x00); // pflags

        assert_eq!(
            Open::try_from(open_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_open_options_with_no_flags() {
        let mut open_bytes = vec![];
        open_bytes.put_u32(0x00);

        assert_eq!(
            OpenOptions::try_from(&open_bytes[..]),
            Ok(get_open_options())
        );
    }

    #[test]
    fn test_parse_open_options_with_read_flag() {
        let mut open_bytes = vec![];
        open_bytes.put_u32(0x01);

        assert_eq!(
            OpenOptions::try_from(&open_bytes[..]),
            Ok(OpenOptions {
                read: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_open_options_with_write_flag() {
        let mut open_bytes = vec![];
        open_bytes.put_u32(0x02);

        assert_eq!(
            OpenOptions::try_from(&open_bytes[..]),
            Ok(OpenOptions {
                write: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_open_options_with_create_flag() {
        let mut open_bytes = vec![];
        open_bytes.put_u32(0x08);

        assert_eq!(
            OpenOptions::try_from(&open_bytes[..]),
            Ok(OpenOptions {
                create: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_open_options_with_create_new_only_flag() {
        let mut open_bytes = vec![];
        open_bytes.put_u32(0x20);

        assert_eq!(
            OpenOptions::try_from(&open_bytes[..]),
            Ok(OpenOptions {
                create_new_only: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_open_options_with_append_flag() {
        let mut open_bytes = vec![];
        open_bytes.put_u32(0x04);

        assert_eq!(
            OpenOptions::try_from(&open_bytes[..]),
            Ok(OpenOptions {
                append: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_open_options_with_truncate_flag() {
        let mut open_bytes = vec![];
        open_bytes.put_u32(0x10);

        assert_eq!(
            OpenOptions::try_from(&open_bytes[..]),
            Ok(OpenOptions {
                truncate: true,
                ..get_open_options()
            })
        );
    }

    #[test]
    fn test_parse_invalid_open_options() {
        assert_eq!(OpenOptions::try_from(&vec![][..]), Err(Error::BadMessage));
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
