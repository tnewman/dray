use super::error::Error;

pub enum Message {
    Init,
    Version,
    Open,
    Close,
    Read,
    Write,
    Lstat,
    Setstat,
    Fsetstat,
    Opendir,
    Readdir,
    Remove,
    Mkdir,
    Rmdir,
    Realpath,
    Stat,
    Rename,
    Readlink,
    Symlink,
    Status,
    Handle,
    Data,
    Name,
    Attrs,
    Extended,
    ExtendedReply,
}

impl Message {
    pub fn parse_bytes(bytes: &[u8]) -> Result<Message, Error> {
        Ok(Message::Init)
    }
}
