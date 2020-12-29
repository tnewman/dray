use crate::error::Error;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Remove {
    id: u32,
    path: String,
}

impl TryFrom<&[u8]> for Remove {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut remove_bytes = item;

        let id = remove_bytes.try_get_u32()?;
        let path = remove_bytes.try_get_string()?;

        Ok(Remove { id, path })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use bytes::BufMut;
    use std::{convert::TryInto, vec};

    #[test]
    fn test_parse_remove() {
        let mut remove_bytes = vec![];

        remove_bytes.put_u32(0x01); // id
        let filename = "/filename".as_bytes();
        remove_bytes.put_u32(filename.len().try_into().unwrap()); // filename length
        remove_bytes.put_slice(filename); // filename

        assert_eq!(
            Remove::try_from(remove_bytes.as_slice()),
            Ok(Remove {
                id: 0x01,
                path: String::from("/filename")
            })
        );
    }

    #[test]
    fn test_parse_remove_with_invalid_id() {
        let mut remove_bytes = vec![];

        remove_bytes.put_u8(0x01); // invalid id

        assert_eq!(
            Remove::try_from(remove_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_remove_with_invalid_filename() {
        let mut remove_bytes = vec![];

        remove_bytes.put_u32(0x01); // id
        remove_bytes.put_u32(0x10); // bad length

        assert_eq!(
            Remove::try_from(remove_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }
}
