use bytes::{BufMut, Bytes, BytesMut};
use std::convert::From;

#[derive(Debug, PartialEq, Eq)]
pub struct Version {
    pub version: u32,
}

impl From<&Version> for Bytes {
    fn from(status: &Version) -> Self {
        let mut status_bytes = BytesMut::new();

        status_bytes.put_u32(status.version);

        status_bytes.freeze()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use bytes::Buf;

    #[test]
    fn test_from_creates_version_bytes() {
        let version = Version { version: 0x03 };

        let version_bytes = &mut Bytes::from(&version);

        assert_eq!(0x03, version_bytes.get_u32());
    }
}
