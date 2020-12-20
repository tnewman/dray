use super::error::Error;

pub enum Message {
    Init(Init),
    Version(Version),
    Open(Open),
    Close(Close),
    Read(Read),
    Write(Write),
    Lstat(Lstat),
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

impl Message {
    pub fn parse_bytes(bytes: &[u8]) -> Result<Message, Error> {
        Ok(Message::Init(Init {}))
    }
}

pub struct Init {}

pub struct Version {}

pub struct Open {}

pub struct Close {}

pub struct Read {}

pub struct Write {}

pub struct Lstat {}

pub struct Setstat {}

pub struct Fsetstat {}

pub struct Opendir {}

pub struct Readdir {}

pub struct Remove {}

pub struct Mkdir {}

pub struct Rmdir {}

pub struct Realpath {}

pub struct Stat {}

pub struct Rename {}

pub struct Readlink {}

pub struct Symlink {}

pub struct Status {}

pub struct Handle {}

pub struct Data {}

pub struct Name {}

pub struct Attrs {}

pub struct Extended {}

pub struct ExtendedReply {}
