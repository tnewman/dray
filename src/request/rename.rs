use crate::error::Error;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Rename {
    pub id: u32,
    pub old_path: String,
    pub new_path: String,
}

impl TryFrom<&[u8]> for Rename {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut rename_bytes = item;

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

    use bytes::BufMut;
    use std::{convert::TryInto, vec};

    #[test]
    fn test_parse_rename() {
        let mut rename_bytes: Vec<u8> = vec![];

        rename_bytes.put_u32(0x01);

        let old_path = "/oldpath".as_bytes();
        rename_bytes.put_u32(old_path.len().try_into().unwrap()); // old path length
        rename_bytes.put_slice(old_path); // old path

        let new_path = "/newpath".as_bytes();
        rename_bytes.put_u32(new_path.len().try_into().unwrap()); // new path length
        rename_bytes.put_slice(new_path); // new path

        assert_eq!(
            Rename::try_from(rename_bytes.as_slice()),
            Ok(Rename {
                id: 0x01,
                old_path: String::from("/oldpath"),
                new_path: String::from("/newpath"),
            })
        );
    }

    #[test]
    fn test_parse_rename_with_invalid_id() {
        let mut rename_bytes: Vec<u8> = vec![];

        rename_bytes.put_u8(0x01);

        assert_eq!(
            Rename::try_from(rename_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_rename_with_invalid_old_path() {
        let mut rename_bytes: Vec<u8> = vec![];

        rename_bytes.put_u32(0x01);

        rename_bytes.put_u32(1); // invalid old path length

        assert_eq!(
            Rename::try_from(rename_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_rename_with_invalid_new_path() {
        let mut rename_bytes: Vec<u8> = vec![];

        rename_bytes.put_u32(0x01);

        let old_path = "/oldpath".as_bytes();
        rename_bytes.put_u32(old_path.len().try_into().unwrap()); // old path length
        rename_bytes.put_slice(old_path); // old path

        rename_bytes.put_u32(1); // invalid new path length

        assert_eq!(
            Rename::try_from(rename_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }
}
