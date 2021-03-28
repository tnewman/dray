use std::convert::TryFrom;

use bytes::Bytes;

use crate::error::Error;
use crate::try_buf::TryBuf;

pub mod handle;
pub mod handle_attributes;
pub mod init;
pub mod open;
pub mod path;
pub mod path_attributes;
pub mod read;
pub mod rename;
pub mod symlink;
pub mod write;

const DATA_TYPE_LENGTH: u32 = 1;

#[derive(Debug, PartialEq)]
pub enum Request {
    Init(init::Init),
    Open(open::Open),
    Close(handle::Handle),
    Read(read::Read),
    Write(write::Write),
    Lstat(path::Path),
    Fstat(path::Path),
    Setstat(path_attributes::PathAttributes),
    Fsetstat(handle_attributes::HandleAttributes),
    Opendir(path::Path),
    Readdir(handle::Handle),
    Remove(path::Path),
    Mkdir(path_attributes::PathAttributes),
    Rmdir(path::Path),
    Realpath(path::Path),
    Stat(path::Path),
    Rename(rename::Rename),
    Readlink(path::Path),
    Symlink(symlink::Symlink),
}

impl TryFrom<&mut Bytes> for Request {
    type Error = Error;

    fn try_from(request_bytes: &mut Bytes) -> Result<Self, Self::Error> {
        let data_length = request_bytes.try_get_u32()?;
        let data_type = request_bytes.try_get_u8()?;
        let data_payload = &mut request_bytes.try_get_bytes(data_length - DATA_TYPE_LENGTH)?;

        let message = match data_type {
            1 => Request::Init(init::Init::try_from(data_payload)?),
            3 => Request::Open(open::Open::try_from(data_payload)?),
            4 => Request::Close(handle::Handle::try_from(data_payload)?),
            5 => Request::Read(read::Read::try_from(data_payload)?),
            6 => Request::Write(write::Write::try_from(data_payload)?),
            7 => Request::Lstat(path::Path::try_from(data_payload)?),
            8 => Request::Fstat(path::Path::try_from(data_payload)?),
            9 => Request::Setstat(path_attributes::PathAttributes::try_from(data_payload)?),
            10 => Request::Fsetstat(handle_attributes::HandleAttributes::try_from(data_payload)?),
            11 => Request::Opendir(path::Path::try_from(data_payload)?),
            12 => Request::Readdir(handle::Handle::try_from(data_payload)?),
            13 => Request::Remove(path::Path::try_from(data_payload)?),
            14 => Request::Mkdir(path_attributes::PathAttributes::try_from(data_payload)?),
            15 => Request::Rmdir(path::Path::try_from(data_payload)?),
            16 => Request::Realpath(path::Path::try_from(data_payload)?),
            17 => Request::Stat(path::Path::try_from(data_payload)?),
            18 => Request::Rename(rename::Rename::try_from(data_payload)?),
            19 => Request::Readlink(path::Path::try_from(data_payload)?),
            20 => Request::Symlink(symlink::Symlink::try_from(data_payload)?),
            _ => return Err(Error::BadMessage),
        };

        Ok(message)
    }
}

impl TryFrom<&[u8]> for Request {
    type Error = Error;

    fn try_from(request_bytes: &[u8]) -> Result<Self, Self::Error> {
        Request::try_from(&mut Bytes::copy_from_slice(request_bytes))
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use bytes::{BufMut, BytesMut};

    use crate::protocol::file_attributes::FileAttributes;
    use crate::try_buf::TryBufMut;

    use super::*;

    #[test]
    fn test_parse_empty_message() {
        let mut message = Bytes::new();

        assert_eq!(Request::try_from(&mut message), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_invalid_message() {
        let mut message = BytesMut::new();
        message.put_u8(0);

        assert_eq!(
            Request::try_from(&mut message.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_init_message() {
        let mut init_payload = BytesMut::new();
        init_payload.put_u8(3); // Protocol Version 3

        assert_eq!(
            Request::try_from(&mut build_message(1, init_payload)),
            Ok(Request::Init(init::Init { version: 0x03 }))
        );
    }

    #[test]
    fn test_parse_invalid_init_message() {
        assert_invalid_message(1);
    }

    #[test]
    fn test_parse_open_message() {
        let mut open_payload = BytesMut::new();
        open_payload.put_u32(0x01); // Id
        open_payload.try_put_str("filename").unwrap();
        open_payload.put_u32(0x00000001); // Read Flag
        open_payload.put_slice(&Bytes::from(&get_file_attrs()));

        assert_eq!(
            Request::try_from(&mut build_message(3, open_payload)),
            Ok(Request::Open(open::Open {
                id: 0x01,
                filename: String::from("filename"),
                file_attributes: get_file_attrs(),
                open_options: open::OpenOptions {
                    read: true,
                    write: false,
                    create: false,
                    create_new_only: false,
                    append: false,
                    truncate: false,
                },
            })),
        )
    }

    #[test]
    fn test_parse_invalid_open_message() {
        assert_invalid_message(3);
    }

    #[test]
    fn test_parse_close_message() {
        let mut close_payload = BytesMut::new();

        close_payload.put_u32(1); // Id
        close_payload.try_put_str("handle").unwrap(); // Handle

        assert_eq!(
            Request::try_from(&mut build_message(4, close_payload)),
            Ok(Request::Close(handle::Handle {
                id: 1,
                handle: String::from("handle"),
            })),
        )
    }

    #[test]
    fn test_parse_invalid_close_message() {
        assert_invalid_message(4);
    }

    #[test]
    fn test_parse_read_message() {
        let mut read_payload = BytesMut::new();

        read_payload.put_u32(1); // Id
        read_payload.try_put_str("handle").unwrap();
        read_payload.put_u64(2); // Offset
        read_payload.put_u32(3); // Length

        assert_eq!(
            Request::try_from(&mut build_message(5, read_payload)),
            Ok(Request::Read(read::Read {
                id: 1,
                handle: String::from("handle"),
                offset: 2,
                len: 3
            })),
        )
    }

    #[test]
    fn test_parse_invalid_read_message() {
        assert_invalid_message(5);
    }
    #[test]
    fn test_parse_write_message() {
        let mut write_payload = BytesMut::new();

        write_payload.put_u32(1); // Id
        write_payload.try_put_str("handle").unwrap();
        write_payload.put_u64(2); // Offset
        write_payload.try_put_str("test").unwrap();

        assert_eq!(
            Request::try_from(&mut build_message(6, write_payload)),
            Ok(Request::Write(write::Write {
                id: 1,
                handle: String::from("handle"),
                offset: 2,
                data: Bytes::from("test"),
            })),
        );
    }

    #[test]
    fn test_parse_invalid_write_message() {
        assert_invalid_message(6);
    }

    #[test]
    fn test_parse_lstat_message() {
        let mut lstat_payload = BytesMut::new();

        lstat_payload.put_u32(1); // Id
        lstat_payload.try_put_str("path").unwrap(); // Path

        assert_eq!(
            Request::try_from(&mut build_message(7, lstat_payload)),
            Ok(Request::Lstat(path::Path {
                id: 1,
                path: String::from("path"),
            }))
        )
    }

    #[test]
    fn test_parse_invalid_lstat_message() {
        assert_invalid_message(7);
    }

    #[test]
    fn test_parse_fstat_message() {
        let mut fstat_payload = BytesMut::new();

        fstat_payload.put_u32(1); // Id
        fstat_payload.try_put_str("path").unwrap(); // Path

        assert_eq!(
            Request::try_from(&mut build_message(8, fstat_payload)),
            Ok(Request::Fstat(path::Path {
                id: 1,
                path: String::from("path"),
            }))
        )
    }

    #[test]
    fn test_parse_invalid_fstat_message() {
        assert_invalid_message(8);
    }

    #[test]
    fn test_parse_setstat_message() {
        let mut setstat_payload = BytesMut::new();

        setstat_payload.put_u32(1); // Id
        setstat_payload.try_put_str("path").unwrap(); // Path
        setstat_payload.put_slice(&Bytes::from(&get_file_attrs()));

        assert_eq!(
            Request::try_from(&mut build_message(9, setstat_payload)),
            Ok(Request::Setstat(path_attributes::PathAttributes {
                id: 1,
                path: String::from("path"),
                file_attributes: get_file_attrs(),
            })),
        );
    }

    #[test]
    fn test_parse_invalid_setstat_message() {
        assert_invalid_message(9);
    }

    #[test]
    fn test_parse_fsetstat_message() {
        let mut fsetstat_payload = BytesMut::new();

        fsetstat_payload.put_u32(1); // Id
        fsetstat_payload.try_put_str("handle").unwrap();
        fsetstat_payload.put_slice(&Bytes::from(&get_file_attrs()));

        assert_eq!(
            Request::try_from(&mut build_message(10, fsetstat_payload)),
            Ok(Request::Fsetstat(handle_attributes::HandleAttributes {
                id: 1,
                handle: String::from("handle"),
                file_attributes: get_file_attrs(),
            }))
        );
    }

    #[test]
    fn test_parse_invalid_fsetstat_message() {
        assert_invalid_message(10);
    }

    #[test]
    fn test_parse_opendir_message() {
        let mut opendir_payload = BytesMut::new();

        opendir_payload.put_u32(1); // Id
        opendir_payload.try_put_str("path").unwrap(); // Path

        assert_eq!(
            Request::try_from(&mut build_message(11, opendir_payload)),
            Ok(Request::Opendir(path::Path {
                id: 1,
                path: String::from("path"),
            }))
        );
    }

    #[test]
    fn test_parse_invalid_opendir_message() {
        assert_invalid_message(11);
    }

    #[test]
    fn test_parse_readdir_message() {
        let mut readdir_payload = BytesMut::new();

        readdir_payload.put_u32(1); // Id
        readdir_payload.try_put_str("handle").unwrap(); // Handle

        assert_eq!(
            Request::try_from(&mut build_message(12, readdir_payload)),
            Ok(Request::Readdir(handle::Handle {
                id: 1,
                handle: String::from("handle")
            })),
        );
    }

    #[test]
    fn test_parse_invalid_readdir_message() {
        assert_invalid_message(12);
    }

    #[test]
    fn test_parse_remove_message() {
        let mut remove_payload = BytesMut::new();

        remove_payload.put_u32(1); // Id
        remove_payload.try_put_str("filename").unwrap(); // Filename

        assert_eq!(
            Request::try_from(&mut build_message(13, remove_payload)),
            Ok(Request::Remove(path::Path {
                id: 1,
                path: String::from("filename")
            }))
        );
    }

    #[test]
    fn test_parse_invalid_remove_message() {
        assert_invalid_message(13);
    }

    #[test]
    fn test_parse_mkdir_message() {
        let mut mkdir_payload = BytesMut::new();

        mkdir_payload.put_u32(1); // Id
        mkdir_payload.try_put_str("path").unwrap(); // Path
        mkdir_payload.put_slice(&Bytes::from(&get_file_attrs()));

        assert_eq!(
            Request::try_from(&mut build_message(14, mkdir_payload)),
            Ok(Request::Mkdir(path_attributes::PathAttributes {
                id: 1,
                path: String::from("path"),
                file_attributes: get_file_attrs(),
            }))
        );
    }

    #[test]
    fn test_parse_invalid_mkdir_message() {
        assert_invalid_message(14);
    }

    #[test]
    fn test_parse_rmdir_message() {
        let mut rmdir_payload = BytesMut::new();

        rmdir_payload.put_u32(1); // Id
        rmdir_payload.try_put_str("path").unwrap(); // Path

        assert_eq!(
            Request::try_from(&mut build_message(15, rmdir_payload)),
            Ok(Request::Rmdir(path::Path {
                id: 1,
                path: String::from("path"),
            }))
        );
    }

    #[test]
    fn test_parse_invalid_rmdir_message() {
        assert_invalid_message(15);
    }

    #[test]
    fn test_parse_realpath_message() {
        let mut realpath_payload = BytesMut::new();

        realpath_payload.put_u32(1); // Id
        realpath_payload.try_put_str("path").unwrap(); // Path

        assert_eq!(
            Request::try_from(&mut build_message(16, realpath_payload)),
            Ok(Request::Realpath(path::Path {
                id: 1,
                path: String::from("path"),
            }))
        );
    }

    #[test]
    fn test_parse_invalid_realpath_message() {
        assert_invalid_message(16);
    }

    #[test]
    fn test_parse_stat_message() {
        let mut stat_payload = BytesMut::new();

        stat_payload.put_u32(1); // Id
        stat_payload.try_put_str("path").unwrap(); // Path

        assert_eq!(
            Request::try_from(&mut build_message(17, stat_payload)),
            Ok(Request::Stat(path::Path {
                id: 1,
                path: String::from("path"),
            }))
        );
    }

    #[test]
    fn test_parse_invalid_stat_message() {
        assert_invalid_message(17);
    }

    #[test]
    fn test_parse_rename_message() {
        let mut rename_payload = BytesMut::new();

        rename_payload.put_u32(1); // Id
        rename_payload.try_put_str("oldpath").unwrap();
        rename_payload.try_put_str("newpath").unwrap();

        assert_eq!(
            Request::try_from(&mut build_message(18, rename_payload)),
            Ok(Request::Rename(rename::Rename {
                id: 1,
                old_path: String::from("oldpath"),
                new_path: String::from("newpath"),
            }))
        );
    }

    #[test]
    fn test_parse_invalid_rename_message() {
        assert_invalid_message(18);
    }

    #[test]
    fn test_parse_readlink_message() {
        let mut readlink_payload = BytesMut::new();

        readlink_payload.put_u32(1); // Id
        readlink_payload.try_put_str("path").unwrap(); // Path

        assert_eq!(
            Request::try_from(&mut build_message(19, readlink_payload)),
            Ok(Request::Readlink(path::Path {
                id: 1,
                path: String::from("path")
            })),
        );
    }

    #[test]
    fn test_parse_invalid_readlink_mesage() {
        assert_invalid_message(19);
    }

    #[test]
    fn test_parse_symlink_message() {
        let mut symlink_payload = BytesMut::new();

        symlink_payload.put_u32(1);
        symlink_payload.try_put_str("linkpath").unwrap();
        symlink_payload.try_put_str("targetpath").unwrap();

        assert_eq!(
            Request::try_from(&mut build_message(20, symlink_payload)),
            Ok(Request::Symlink(symlink::Symlink {
                id: 1,
                link_path: String::from("linkpath"),
                target_path: String::from("targetpath"),
            }))
        );
    }

    #[test]
    fn test_parse_invalid_symlink_message() {
        assert_invalid_message(20);
    }

    fn assert_invalid_message(message_type: u8) {
        let payload = BytesMut::new();

        assert_eq!(
            Request::try_from(&mut build_message(message_type, payload)),
            Err(Error::BadMessage),
        )
    }

    fn build_message(message_type: u8, payload: BytesMut) -> Bytes {
        let mut message = BytesMut::new();

        // The message type is included in the length
        let length = payload.len() + 1;

        message.put_u32(length.try_into().unwrap());
        message.put_u8(message_type);
        message.put_slice(&payload.freeze());

        message.freeze()
    }

    fn get_file_attrs() -> FileAttributes {
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
