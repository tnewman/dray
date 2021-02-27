use crate::error::Error;
use crate::try_buf::TryBuf;

use bytes::Bytes;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Symlink {
    pub id: u32,
    pub link_path: String,
    pub target_path: String,
}

impl TryFrom<&mut Bytes> for Symlink {
    type Error = Error;

    fn try_from(symlink_bytes: &mut Bytes) -> Result<Self, Self::Error> {
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

    use crate::try_buf::TryBufMut;

    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_parse_symlink() {
        let mut symlink_bytes = BytesMut::new();

        symlink_bytes.put_u32(0x01);
        symlink_bytes.try_put_str("/linkpath").unwrap();
        symlink_bytes.try_put_str("/targetpath").unwrap();

        assert_eq!(
            Symlink::try_from(&mut symlink_bytes.freeze()),
            Ok(Symlink {
                id: 0x01,
                link_path: String::from("/linkpath"),
                target_path: String::from("/targetpath"),
            })
        );
    }

    #[test]
    fn test_parse_symlink_with_invalid_id() {
        let mut symlink_bytes = BytesMut::new();

        symlink_bytes.put_u8(0x01);

        assert_eq!(
            Symlink::try_from(&mut symlink_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_symlink_with_invalid_link_path() {
        let mut symlink_bytes = BytesMut::new();

        symlink_bytes.put_u32(0x01);
        symlink_bytes.put_u32(0x01); // invalid link path length

        assert_eq!(
            Symlink::try_from(&mut symlink_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_symlink_with_invalid_target_path() {
        let mut symlink_bytes = BytesMut::new();

        symlink_bytes.put_u32(0x01);
        symlink_bytes.try_put_str("/linkpath").unwrap();
        symlink_bytes.put_u32(0x01); // invalid target path length

        assert_eq!(
            Symlink::try_from(&mut symlink_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }
}
