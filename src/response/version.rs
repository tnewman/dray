use bytes::BufMut;
use std::convert::From;

#[derive(Debug, PartialEq)]
pub struct Version {
    pub version: u32,
}

impl From<&Version> for Vec<u8> {
    fn from(item: &Version) -> Self {
        let mut status_bytes: Vec<u8> = vec![];

        status_bytes.put_u32(item.version);

        status_bytes
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use bytes::Buf;

    #[test]
    fn test_from_creates_version_bytes() {
        let version = Version { version: 0x03 };

        let mut version_bytes: &[u8] = &Vec::from(&version);

        assert_eq!(0x03, version_bytes.get_u32());
    }
}
