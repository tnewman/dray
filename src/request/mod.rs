use std::convert::TryFrom;

use crate::error::Error;
use crate::try_buf::TryBuf;

pub mod close;
pub mod handle;
pub mod handle_attributes;
pub mod init;
pub mod open;
pub mod path;
pub mod path_attributes;
pub mod read_write;
pub mod rename;
pub mod symlink;

#[derive(Debug, PartialEq)]
pub enum Request {
    Init(init::Init),
    Open(open::Open),
    Close(handle::Handle),
    Read(read_write::ReadWrite),
    Write(read_write::ReadWrite),
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
    Rename(rename::Rename),
    Readlink(path::Path),
    Symlink(symlink::Symlink),
    Handle(handle::Handle),
}

impl TryFrom<&[u8]> for Request {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut bytes = item;

        let data_length = bytes.try_get_u32()?;

        let data_length = match usize::try_from(data_length) {
            Ok(data_length) => data_length,
            Err(_) => return Err(Error::BadMessage),
        };

        let data_type = bytes.try_get_u8()?;
        let data_payload: &[u8] = &bytes.try_get_bytes(data_length)?;

        let message = match data_type {
            1 => Request::Init(init::Init::try_from(data_payload)?),
            3 => Request::Open(open::Open::try_from(data_payload)?),
            4 => Request::Close(handle::Handle::try_from(data_payload)?),
            5 => Request::Read(read_write::ReadWrite::try_from(data_payload)?),
            6 => Request::Write(read_write::ReadWrite::try_from(data_payload)?),
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
            18 => Request::Rename(rename::Rename::try_from(data_payload)?),
            19 => Request::Readlink(path::Path::try_from(data_payload)?),
            20 => Request::Symlink(symlink::Symlink::try_from(data_payload)?),
            102 => Request::Handle(handle::Handle::try_from(data_payload)?),
            _ => return Err(Error::BadMessage),
        };

        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_message() {
        let message: &[u8] = &[];

        assert_eq!(Request::try_from(message), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_invalid_message() {
        let message: &[u8] = &[0x00];

        assert_eq!(Request::try_from(message), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_init_message() {
        let message: &[u8] = &[
            0x00, 0x00, 0x00, 0x01, // Payload Length 1
            0x01, // Init Message
            0x03, // Protocol Version 3
        ];

        assert_eq!(
            Request::try_from(message),
            Ok(Request::Init(init::Init { version: 0x03 }))
        );
    }
}
