use std::convert::TryFrom;

use bytes::{Buf, Bytes};
use tracing::Level;

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

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Init(init::Init),
    Open(open::Open),
    Close(handle::Handle),
    Read(read::Read),
    Write(write::Write),
    Lstat(path::Path),
    Fstat(handle::Handle),
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

pub trait RequestId {
    fn get_request_id(&self) -> u32;
}

impl RequestId for Request {
    fn get_request_id(&self) -> u32 {
        match self {
            Request::Init(init) => init.get_request_id(),
            Request::Open(open) => open.get_request_id(),
            Request::Close(close) => close.get_request_id(),
            Request::Read(read) => read.get_request_id(),
            Request::Write(write) => write.get_request_id(),
            Request::Lstat(lstat) => lstat.get_request_id(),
            Request::Fstat(fstat) => fstat.get_request_id(),
            Request::Setstat(setstat) => setstat.get_request_id(),
            Request::Fsetstat(fsetstat) => fsetstat.get_request_id(),
            Request::Opendir(opendir) => opendir.get_request_id(),
            Request::Readdir(readdir) => readdir.get_request_id(),
            Request::Remove(remove) => remove.get_request_id(),
            Request::Mkdir(mkdir) => mkdir.get_request_id(),
            Request::Rmdir(rmdir) => rmdir.get_request_id(),
            Request::Realpath(realpath) => realpath.get_request_id(),
            Request::Stat(stat) => stat.get_request_id(),
            Request::Rename(rename) => rename.get_request_id(),
            Request::Readlink(readlink) => readlink.get_request_id(),
            Request::Symlink(symlink) => symlink.get_request_id(),
        }
    }
}

impl TryFrom<&mut Bytes> for Request {
    type Error = Error;

    #[tracing::instrument(level = Level::DEBUG, fields(result))]
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
            8 => Request::Fstat(handle::Handle::try_from(data_payload)?),
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
                open_options: get_open_options()
            })),
        );
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
        fstat_payload.try_put_str("handle").unwrap(); // Handle

        assert_eq!(
            Request::try_from(&mut build_message(8, fstat_payload)),
            Ok(Request::Fstat(handle::Handle {
                id: 1,
                handle: String::from("handle"),
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

    #[test]
    fn test_init_get_request_id() {
        let init_request = Request::Init(super::init::Init { version: 3 });

        assert_eq!(0, init_request.get_request_id());
    }

    #[test]
    fn test_open_get_request_id() {
        let open_request = Request::Open(super::open::Open {
            id: 1000,
            filename: String::from("filename"),
            file_attributes: get_file_attrs(),
            open_options: get_open_options(),
        });

        assert_eq!(1000, open_request.get_request_id());
    }

    #[test]
    fn test_close_get_request_id() {
        let close_request = Request::Close(super::handle::Handle {
            id: 1000,
            handle: String::from("handle"),
        });

        assert_eq!(1000, close_request.get_request_id());
    }

    #[test]
    fn test_read_get_request_id() {
        let read_request = Request::Read(super::read::Read {
            id: 1000,
            handle: String::from("handle"),
            offset: 0,
            len: 0,
        });

        assert_eq!(1000, read_request.get_request_id());
    }

    #[test]
    fn test_write_get_request_id() {
        let write_request = Request::Write(super::write::Write {
            id: 1000,
            handle: String::from("handle"),
            offset: 0,
            data: Bytes::from(vec![]),
        });

        assert_eq!(1000, write_request.get_request_id());
    }

    #[test]
    fn test_lstat_get_request_id() {
        let lstat_request = Request::Lstat(super::path::Path {
            id: 1000,
            path: String::from("path"),
        });

        assert_eq!(1000, lstat_request.get_request_id());
    }

    #[test]
    fn test_fstat_get_request_id() {
        let fstat_request = Request::Fstat(super::handle::Handle {
            id: 1000,
            handle: String::from("handle"),
        });

        assert_eq!(1000, fstat_request.get_request_id());
    }

    #[test]
    fn test_setstat_get_request_id() {
        let setstat_request = Request::Setstat(super::path_attributes::PathAttributes {
            id: 1000,
            path: String::from("path"),
            file_attributes: get_file_attrs(),
        });

        assert_eq!(1000, setstat_request.get_request_id());
    }

    #[test]
    fn test_fsetstat_get_request_id() {
        let fsetstat_request = Request::Fsetstat(super::handle_attributes::HandleAttributes {
            id: 1000,
            handle: String::from("handle"),
            file_attributes: get_file_attrs(),
        });

        assert_eq!(1000, fsetstat_request.get_request_id());
    }

    #[test]
    fn test_opendir_get_request_id() {
        let opendir_request = Request::Opendir(super::path::Path {
            id: 1000,
            path: String::from("path"),
        });

        assert_eq!(1000, opendir_request.get_request_id());
    }

    #[test]
    fn test_readdir_get_request_id() {
        let readdir_request = Request::Readdir(super::handle::Handle {
            id: 1000,
            handle: String::from("handle"),
        });

        assert_eq!(1000, readdir_request.get_request_id());
    }

    #[test]
    fn test_remove_get_request_id() {
        let remove_request = Request::Remove(super::path::Path {
            id: 1000,
            path: String::from("path"),
        });

        assert_eq!(1000, remove_request.get_request_id());
    }

    #[test]
    fn test_mkdir_get_request_id() {
        let mkdir_request = Request::Mkdir(super::path_attributes::PathAttributes {
            id: 1000,
            path: String::from("path"),
            file_attributes: get_file_attrs(),
        });

        assert_eq!(1000, mkdir_request.get_request_id());
    }

    #[test]
    fn test_rmdir_get_request_id() {
        let rmdir_request = Request::Rmdir(super::path::Path {
            id: 1000,
            path: String::from("path"),
        });

        assert_eq!(1000, rmdir_request.get_request_id());
    }

    #[test]
    fn test_realpath_get_request_id() {
        let realpath_request = Request::Realpath(super::path::Path {
            id: 1000,
            path: String::from("path"),
        });

        assert_eq!(1000, realpath_request.get_request_id());
    }

    #[test]
    fn test_stat_get_request_id() {
        let stat_request = Request::Stat(super::path::Path {
            id: 1000,
            path: String::from("path"),
        });

        assert_eq!(1000, stat_request.get_request_id());
    }

    #[test]
    fn test_rename_get_request_id() {
        let rename_request = Request::Rename(super::rename::Rename {
            id: 1000,
            old_path: String::from("old"),
            new_path: String::from("new"),
        });

        assert_eq!(1000, rename_request.get_request_id());
    }

    #[test]
    fn test_readlink_get_request_id() {
        let readlink_request = Request::Readlink(super::path::Path {
            id: 1000,
            path: String::from("path"),
        });

        assert_eq!(1000, readlink_request.get_request_id());
    }

    #[test]
    fn test_symlink_get_request_id() {
        let symlink_request = Request::Symlink(super::symlink::Symlink {
            id: 1000,
            link_path: String::from("link"),
            target_path: String::from("target"),
        });

        assert_eq!(1000, symlink_request.get_request_id());
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

    fn get_open_options() -> open::OpenOptions {
        open::OpenOptions {
            read: true,
            write: false,
            create: false,
            create_new_only: false,
            append: false,
            truncate: false,
        }
    }
}
