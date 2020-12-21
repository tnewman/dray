use bytes::Buf;
use std::convert::TryFrom;

use super::error::Error;

#[derive(Debug, PartialEq)]
pub enum Request {
    Init(Init),
    Open(Open),
    Close(Close),
    Read(Read),
    Write(Write),
    Lstat(Lstat),
    Fstat(Fstat),
    Setstat(Setstat),
    Fsetstat(Fsetstat),
    Opendir(Opendir),
    Readdir(Readdir),
    Remove(Remove),
    Mkdir(Mkdir),
    Rmdir(Rmdir),
    Realpath(Realpath),
    Stat(Stat),
    Rename(Rename),
    Readlink(Readlink),
    Symlink(Symlink),
    Status(Status),
    Handle(Handle),
    Data(Data),
    Name(Name),
    Attrs(Attrs),
    Extended(Extended),
    ExtendedReply(ExtendedReply),
}

impl Request {
    pub fn parse_bytes(mut bytes: &[u8]) -> Result<Message, Error> {
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
            1 => Message::Init(Init::parse_bytes(data_payload)?),
            2 => Message::Version(Version::parse_bytes(data_payload)?),
            3 => Message::Open(Open::parse_bytes(data_payload)?),
            4 => Message::Close(Close::parse_bytes(data_payload)?),
            5 => Message::Read(Read::parse_bytes(data_payload)?),
            6 => Message::Write(Write::parse_bytes(data_payload)?),
            7 => Message::Lstat(Lstat::parse_bytes(data_payload)?),
            8 => Message::Fstat(Fstat::parse_bytes(data_payload)?),
            9 => Message::Setstat(Setstat::parse_bytes(data_payload)?),
            10 => Message::Fsetstat(Fsetstat::parse_bytes(data_payload)?),
            11 => Message::Opendir(Opendir::parse_bytes(data_payload)?),
            12 => Message::Readdir(Readdir::parse_bytes(data_payload)?),
            13 => Message::Remove(Remove::parse_bytes(data_payload)?),
            14 => Message::Mkdir(Mkdir::parse_bytes(data_payload)?),
            15 => Message::Rmdir(Rmdir::parse_bytes(data_payload)?),
            16 => Message::Realpath(Realpath::parse_bytes(data_payload)?),
            17 => Message::Stat(Stat::parse_bytes(data_payload)?),
            18 => Message::Rename(Rename::parse_bytes(data_payload)?),
            19 => Message::Readlink(Readlink::parse_bytes(data_payload)?),
            20 => Message::Symlink(Symlink::parse_bytes(data_payload)?),
            101 => Message::Status(Status::parse_bytes(data_payload)?),
            102 => Message::Handle(Handle::parse_bytes(data_payload)?),
            103 => Message::Data(Data::parse_bytes(data_payload)?),
            104 => Message::Name(Name::parse_bytes(data_payload)?),
            105 => Message::Attrs(Attrs::parse_bytes(data_payload)?),
            200 => Message::Extended(Extended::parse_bytes(data_payload)?),
            201 => Message::ExtendedReply(ExtendedReply::parse_bytes(data_payload)?),
            _ => return Err(Error::BadMessage),
        };

        Ok(message)
    }
}

#[derive(Debug, PartialEq)]
pub struct Init {
    version: u8,
}

impl Init {
    fn parse_bytes(mut bytes: &[u8]) -> Result<Init, Error> {
        if bytes.remaining() < 1 {
            return Err(Error::BadMessage);
        }

        Ok(Init {
            version: bytes.get_u8(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Version {}

impl Version {
    fn parse_bytes(byte: &[u8]) -> Result<Version, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Open {}

impl Open {
    fn parse_bytes(byte: &[u8]) -> Result<Open, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Close {}

impl Close {
    fn parse_bytes(byte: &[u8]) -> Result<Close, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Read {}

impl Read {
    fn parse_bytes(byte: &[u8]) -> Result<Read, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Write {}

impl Write {
    fn parse_bytes(byte: &[u8]) -> Result<Write, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Lstat {}

impl Lstat {
    fn parse_bytes(byte: &[u8]) -> Result<Lstat, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Fstat {}

impl Fstat {
    fn parse_bytes(byte: &[u8]) -> Result<Fstat, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Setstat {}

impl Setstat {
    fn parse_bytes(byte: &[u8]) -> Result<Setstat, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Fsetstat {}

impl Fsetstat {
    fn parse_bytes(byte: &[u8]) -> Result<Fsetstat, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Opendir {}

impl Opendir {
    fn parse_bytes(byte: &[u8]) -> Result<Opendir, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Readdir {}

impl Readdir {
    fn parse_bytes(byte: &[u8]) -> Result<Readdir, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Remove {}

impl Remove {
    fn parse_bytes(byte: &[u8]) -> Result<Remove, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Mkdir {}

impl Mkdir {
    fn parse_bytes(byte: &[u8]) -> Result<Mkdir, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Rmdir {}

impl Rmdir {
    fn parse_bytes(byte: &[u8]) -> Result<Rmdir, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Realpath {}

impl Realpath {
    fn parse_bytes(byte: &[u8]) -> Result<Realpath, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Stat {}

impl Stat {
    fn parse_bytes(byte: &[u8]) -> Result<Stat, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Rename {}

impl Rename {
    fn parse_bytes(byte: &[u8]) -> Result<Rename, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Readlink {}

impl Readlink {
    fn parse_bytes(byte: &[u8]) -> Result<Readlink, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Symlink {}

impl Symlink {
    fn parse_bytes(byte: &[u8]) -> Result<Symlink, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Status {}

impl Status {
    fn parse_bytes(byte: &[u8]) -> Result<Status, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Handle {}

impl Handle {
    fn parse_bytes(byte: &[u8]) -> Result<Handle, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Data {}

impl Data {
    fn parse_bytes(byte: &[u8]) -> Result<Data, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Name {}

impl Name {
    fn parse_bytes(byte: &[u8]) -> Result<Name, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Attrs {}

impl Attrs {
    fn parse_bytes(byte: &[u8]) -> Result<Attrs, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct Extended {}

impl Extended {
    fn parse_bytes(byte: &[u8]) -> Result<Extended, Error> {
        Err(Error::Failure)
    }
}

#[derive(Debug, PartialEq)]
pub struct ExtendedReply {}

impl ExtendedReply {
    fn parse_bytes(byte: &[u8]) -> Result<ExtendedReply, Error> {
        Err(Error::Failure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_message() {
        assert_eq!(Message::parse_bytes(&[]), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_invalid_message() {
        assert_eq!(Message::parse_bytes(&[0x00]), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_init_message() {
        assert_eq!(
            Message::parse_bytes(&[
                0x00, 0x00, 0x00, 0x01, // Payload Length 1
                0x01, // Init Message
                0x03  // Protocol Version 3
            ]),
            Ok(Message::Init(Init { version: 0x03 }))
        );
    }

    #[test]
    fn test_parse_init_message_with_missing_protocol() {
        assert_eq!(
            Message::parse_bytes(&[
                0x00, 0x00, 0x00, 0x00, // Payload Length 0
                0x01  // Init Message
                      // Missing Protocol Version
            ]),
            Err(Error::BadMessage)
        );
    }
}
