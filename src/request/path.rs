use crate::error::Error;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Path {
    id: u32,
    path: String,
}

impl TryFrom<&[u8]> for Path {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut path_bytes = item;

        let id = path_bytes.try_get_u32()?;
        let path = path_bytes.try_get_string()?;

        Ok(Path { id, path })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use bytes::BufMut;
    use std::{convert::TryInto, vec};

    #[test]
    fn test_parse_path() {
        let mut path_bytes = vec![];

        path_bytes.put_u32(0x01); // id
        let filename = "/filename".as_bytes();
        path_bytes.put_u32(filename.len().try_into().unwrap()); // filename length
        path_bytes.put_slice(filename); // filename

        assert_eq!(
            Path::try_from(path_bytes.as_slice()),
            Ok(Path {
                id: 0x01,
                path: String::from("/filename")
            })
        );
    }

    #[test]
    fn test_parse_path_with_invalid_id() {
        let mut path_bytes = vec![];

        path_bytes.put_u8(0x01); // invalid id

        assert_eq!(
            Path::try_from(path_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_path_with_invalid_filename() {
        let mut path_bytes = vec![];

        path_bytes.put_u32(0x01); // id
        path_bytes.put_u32(0x10); // bad length

        assert_eq!(
            Path::try_from(path_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }
}
