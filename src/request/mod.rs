use bytes::Buf;
use std::convert::TryFrom;

use super::error::Error;

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
pub mod opendir;
pub mod read;
pub mod readdir;
pub mod readlink;
pub mod realpath;
pub mod remove;
pub mod rename;
pub mod rmdir;
pub mod setstat;
pub mod stat;
pub mod status;
pub mod symlink;
pub mod write;

#[derive(Debug, PartialEq)]
pub enum Request {
    Init(init::Init),
    Open(open::Open),
    Close(close::Close),
    Read(read::Read),
    Write(write::Write),
    Lstat(lstat::Lstat),
    Fstat(fstat::Fstat),
    Setstat(setstat::Setstat),
    Fsetstat(fsetstat::Fsetstat),
    Opendir(opendir::Opendir),
    Readdir(readdir::Readdir),
    Remove(remove::Remove),
    Mkdir(mkdir::Mkdir),
    Rmdir(rmdir::Rmdir),
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

impl Request {
    pub fn parse_bytes(mut bytes: &[u8]) -> Result<Request, Error> {
        if bytes.remaining() < 4 {
            return Err(Error::BadMessage);
        }

        let data_length = match usize::try_from(bytes.get_u32()) {
            Ok(data_length) => data_length,
            Err(_) => return Err(Error::BadMessage),
        };

        if bytes.remaining() < 1 {
            return Err(Error::BadMessage);
        }

        let data_type = bytes.get_u8();

        if bytes.remaining() != data_length {
            return Err(Error::BadMessage);
        }

        let data_payload = bytes.bytes();

        let message = match data_type {
            1 => Request::Init(init::Init::parse_bytes(data_payload)?),
            3 => Request::Open(open::Open::parse_bytes(data_payload)?),
            4 => Request::Close(close::Close::parse_bytes(data_payload)?),
            5 => Request::Read(read::Read::parse_bytes(data_payload)?),
            6 => Request::Write(write::Write::parse_bytes(data_payload)?),
            7 => Request::Lstat(lstat::Lstat::parse_bytes(data_payload)?),
            8 => Request::Fstat(fstat::Fstat::parse_bytes(data_payload)?),
            9 => Request::Setstat(setstat::Setstat::parse_bytes(data_payload)?),
            10 => Request::Fsetstat(fsetstat::Fsetstat::parse_bytes(data_payload)?),
            11 => Request::Opendir(opendir::Opendir::parse_bytes(data_payload)?),
            12 => Request::Readdir(readdir::Readdir::parse_bytes(data_payload)?),
            13 => Request::Remove(remove::Remove::parse_bytes(data_payload)?),
            14 => Request::Mkdir(mkdir::Mkdir::parse_bytes(data_payload)?),
            15 => Request::Rmdir(rmdir::Rmdir::parse_bytes(data_payload)?),
            16 => Request::Realpath(realpath::Realpath::parse_bytes(data_payload)?),
            17 => Request::Stat(stat::Stat::parse_bytes(data_payload)?),
            18 => Request::Rename(rename::Rename::parse_bytes(data_payload)?),
            19 => Request::Readlink(readlink::Readlink::parse_bytes(data_payload)?),
            20 => Request::Symlink(symlink::Symlink::parse_bytes(data_payload)?),
            101 => Request::Status(status::Status::parse_bytes(data_payload)?),
            102 => Request::Handle(handle::Handle::parse_bytes(data_payload)?),
            103 => Request::Data(data::Data::parse_bytes(data_payload)?),
            104 => Request::Name(name::Name::parse_bytes(data_payload)?),
            105 => Request::Attrs(attrs::Attrs::parse_bytes(data_payload)?),
            200 => Request::Extended(extended::Extended::parse_bytes(data_payload)?),
            201 => {
                Request::ExtendedReply(extended_reply::ExtendedReply::parse_bytes(data_payload)?)
            }
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
        assert_eq!(Request::parse_bytes(&[]), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_invalid_message() {
        assert_eq!(Request::parse_bytes(&[0x00]), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_init_message() {
        assert_eq!(
            Request::parse_bytes(&[
                0x00, 0x00, 0x00, 0x01, // Payload Length 1
                0x01, // Init Message
                0x03  // Protocol Version 3
            ]),
            Ok(Request::Init(Init { version: 0x03 }))
        );
    }

    #[test]
    fn test_parse_init_message_with_missing_protocol() {
        assert_eq!(
            Request::parse_bytes(&[
                0x00, 0x00, 0x00, 0x00, // Payload Length 0
                0x01  // Init Message
                      // Missing Protocol Version
            ]),
            Err(Error::BadMessage)
        );
    }
}
