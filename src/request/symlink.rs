use crate::error::Error;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Symlink {
    id: u32,
    link_path: String,
    target_path: String,
}

impl TryFrom<&[u8]> for Symlink {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut symlink_bytes = item;

        let id = symlink_bytes.try_get_u32()?;
        let link_path = symlink_bytes.try_get_string()?;
        let target_path = symlink_bytes.try_get_string()?;

        Ok(Symlink {
            id,
            link_path,
            target_path,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use bytes::BufMut;
    use std::{convert::TryInto, vec};

    #[test]
    fn test_parse_symlink() {
        let mut symlink_bytes: Vec<u8> = vec![];

        symlink_bytes.put_u32(0x01);

        let link_path = "/linkpath".as_bytes();
        symlink_bytes.put_u32(link_path.len().try_into().unwrap()); // link path length
        symlink_bytes.put_slice(link_path); // link path

        let target_path = "/targetpath".as_bytes();
        symlink_bytes.put_u32(target_path.len().try_into().unwrap()); // target path length
        symlink_bytes.put_slice(target_path); // target path

        assert_eq!(
            Symlink::try_from(symlink_bytes.as_slice()),
            Ok(Symlink {
                id: 0x01,
                link_path: String::from("/linkpath"),
                target_path: String::from("/targetpath"),
            })
        );
    }

    #[test]
    fn test_parse_symlink_with_invalid_id() {
        let mut symlink_bytes: Vec<u8> = vec![];

        symlink_bytes.put_u8(0x01);

        assert_eq!(
            Symlink::try_from(symlink_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_symlink_with_invalid_link_path() {
        let mut symlink_bytes: Vec<u8> = vec![];

        symlink_bytes.put_u32(0x01);

        symlink_bytes.put_u32(0x01); // invalid link path length

        assert_eq!(
            Symlink::try_from(symlink_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_symlink_with_invalid_target_path() {
        let mut symlink_bytes: Vec<u8> = vec![];

        symlink_bytes.put_u32(0x01);

        let link_path = "/linkpath".as_bytes();
        symlink_bytes.put_u32(link_path.len().try_into().unwrap()); // link path length
        symlink_bytes.put_slice(link_path); // link path

        symlink_bytes.put_u32(0x01); // invalid target path length

        assert_eq!(
            Symlink::try_from(symlink_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }
}
