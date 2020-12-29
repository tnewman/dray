use crate::error::Error;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Handle {
    id: u32,
    handle: String,
}

impl TryFrom<&[u8]> for Handle {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut handle_bytes = item;

        let id = handle_bytes.try_get_u32()?;
        let handle = handle_bytes.try_get_string()?;

        Ok(Handle { id, handle })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use bytes::BufMut;
    use std::convert::TryInto;

    #[test]
    fn test_parse_handle() {
        let mut handle_bytes: Vec<u8> = vec![];

        handle_bytes.put_u32(0x01); // id

        let handle = "HANDLE".as_bytes();
        handle_bytes.put_u32(handle.len().try_into().unwrap()); // handle length
        handle_bytes.put_slice(handle); // handle

        assert_eq!(
            Handle::try_from(handle_bytes.as_slice()),
            Ok(Handle {
                id: 0x01,
                handle: String::from("HANDLE")
            })
        )
    }

    #[test]
    fn test_parse_handle_with_invalid_id() {
        let mut handle_bytes: Vec<u8> = vec![];

        handle_bytes.put_u8(0x01); // bad id

        assert_eq!(
            Handle::try_from(handle_bytes.as_slice()),
            Err(Error::BadMessage)
        )
    }

    #[test]
    fn test_parse_handle_with_invalid_handle() {
        let mut handle_bytes: Vec<u8> = vec![];

        handle_bytes.put_u32(0x01); // id
        handle_bytes.put_u32(0x01); // bad handle length

        assert_eq!(
            Handle::try_from(handle_bytes.as_slice()),
            Err(Error::BadMessage)
        )
    }
}
