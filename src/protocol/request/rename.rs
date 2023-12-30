use crate::error::Error;
use crate::try_buf::TryBuf;

use bytes::Bytes;
use std::convert::TryFrom;

use super::RequestId;

#[derive(Debug, PartialEq, Eq)]
pub struct Rename {
    pub id: u32,
    pub old_path: String,
    pub new_path: String,
}

impl RequestId for Rename {
    fn get_request_id(&self) -> u32 {
        self.id
    }
}

impl TryFrom<&mut Bytes> for Rename {
    type Error = Error;

    #[tracing::instrument]
    fn try_from(rename_bytes: &mut Bytes) -> Result<Self, Self::Error> {
        let id = rename_bytes.try_get_u32()?;
        let old_path = rename_bytes.try_get_string()?;
        let new_path = rename_bytes.try_get_string()?;

        Ok(Rename {
            id,
            old_path,
            new_path,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::try_buf::TryBufMut;

    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_parse_rename() {
        let mut rename_bytes = BytesMut::new();

        rename_bytes.put_u32(0x01);
        rename_bytes.try_put_str("/oldpath").unwrap(); // old path
        rename_bytes.try_put_str("/newpath").unwrap(); // new path

        assert_eq!(
            Rename::try_from(&mut rename_bytes.freeze()),
            Ok(Rename {
                id: 0x01,
                old_path: String::from("/oldpath"),
                new_path: String::from("/newpath"),
            })
        );
    }

    #[test]
    fn test_parse_rename_with_invalid_id() {
        let mut rename_bytes = BytesMut::new();

        rename_bytes.put_u8(0x01);

        assert_eq!(
            Rename::try_from(&mut rename_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_rename_with_invalid_old_path() {
        let mut rename_bytes = BytesMut::new();

        rename_bytes.put_u32(0x01);
        rename_bytes.put_u32(1); // invalid old path length

        assert_eq!(
            Rename::try_from(&mut rename_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_rename_with_invalid_new_path() {
        let mut rename_bytes = BytesMut::new();

        rename_bytes.put_u32(0x01);
        rename_bytes.try_put_str("/oldpath").unwrap(); // old path
        rename_bytes.put_u32(1); // invalid new path length

        assert_eq!(
            Rename::try_from(&mut rename_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_get_request_id() {
        let rename = Rename {
            id: 1000,
            old_path: String::from("old"),
            new_path: String::from("new"),
        };

        assert_eq!(1000, rename.get_request_id());
    }
}
