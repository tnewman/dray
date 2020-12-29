use std::convert::TryFrom;

use crate::error::Error;
use crate::try_buf::TryBuf;

pub mod attrs;
pub mod close;
pub mod data;
pub mod extended;
pub mod extended_reply;
pub mod fsetstat;
pub mod fstat;
pub mod handle;
pub mod init;
pub mod lstat;
pub mod mkdir;
pub mod name;
pub mod open;
pub mod path;
pub mod read_write;
pub mod readlink;
pub mod realpath;
pub mod rename;
pub mod setstat;
pub mod stat;
pub mod status;
pub mod symlink;

#[derive(Debug, PartialEq)]
pub enum Request {
    Init(init::Init),
    Open(open::Open),
    Close(close::Close),
    Read(read_write::ReadWrite),
    Write(read_write::ReadWrite),
    Lstat(lstat::Lstat),
    Fstat(fstat::Fstat),
    Setstat(setstat::Setstat),
    Fsetstat(fsetstat::Fsetstat),
    Opendir(path::Path),
    Readdir(handle::Handle),
    Remove(path::Path),
    Mkdir(mkdir::Mkdir),
    Rmdir(path::Path),
    Realpath(realpath::Realpath),
    Stat(stat::Stat),
    Rename(rename::Rename),
    Readlink(readlink::Readlink),
    Symlink(symlink::Symlink),
    Status(status::Status),
    Handle(handle::Handle),
    Data(data::Data),
    Name(name::Name),
    Attrs(attrs::Attrs),
    Extended(extended::Extended),
    ExtendedReply(extended_reply::ExtendedReply),
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
            4 => Request::Close(close::Close::try_from(data_payload)?),
            5 => Request::Read(read_write::ReadWrite::try_from(data_payload)?),
            6 => Request::Write(read_write::ReadWrite::try_from(data_payload)?),
            7 => Request::Lstat(lstat::Lstat::try_from(data_payload)?),
            8 => Request::Fstat(fstat::Fstat::try_from(data_payload)?),
            9 => Request::Setstat(setstat::Setstat::try_from(data_payload)?),
            10 => Request::Fsetstat(fsetstat::Fsetstat::try_from(data_payload)?),
            11 => Request::Opendir(path::Path::try_from(data_payload)?),
            12 => Request::Readdir(handle::Handle::try_from(data_payload)?),
            13 => Request::Remove(path::Path::try_from(data_payload)?),
            14 => Request::Mkdir(mkdir::Mkdir::try_from(data_payload)?),
            15 => Request::Rmdir(path::Path::try_from(data_payload)?),
            16 => Request::Realpath(realpath::Realpath::try_from(data_payload)?),
            17 => Request::Stat(stat::Stat::try_from(data_payload)?),
            18 => Request::Rename(rename::Rename::try_from(data_payload)?),
            19 => Request::Readlink(readlink::Readlink::try_from(data_payload)?),
            20 => Request::Symlink(symlink::Symlink::try_from(data_payload)?),
            101 => Request::Status(status::Status::try_from(data_payload)?),
            102 => Request::Handle(handle::Handle::try_from(data_payload)?),
            103 => Request::Data(data::Data::try_from(data_payload)?),
            104 => Request::Name(name::Name::try_from(data_payload)?),
            105 => Request::Attrs(attrs::Attrs::try_from(data_payload)?),
            200 => Request::Extended(extended::Extended::try_from(data_payload)?),
            201 => Request::ExtendedReply(extended_reply::ExtendedReply::try_from(data_payload)?),
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
