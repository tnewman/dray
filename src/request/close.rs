use crate::error::Error;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Close {
    pub id: u32,
    pub handle: String,
}

impl TryFrom<&[u8]> for Close {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut close_bytes = item;

        let id = close_bytes.try_get_u32()?;
        let handle = close_bytes.try_get_string()?;

        Ok(Close { id, handle })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::try_buf::TryBufMut;

    use bytes::BufMut;

    #[test]
    fn test_parse_close() {
        let mut close_bytes = vec![];

        close_bytes.put_u32(0x01); // id
        close_bytes.try_put_str("handle").unwrap(); // handle

        assert_eq!(
            Close::try_from(close_bytes.as_slice()),
            Ok(Close {
                id: 0x01,
                handle: String::from("handle")
            })
        );
    }

    #[test]
    fn test_parse_close_with_empty_data() {
        assert_eq!(Close::try_from(&vec![][..]), Err(Error::BadMessage));
    }

    #[test]
    fn test_parse_close_with_invalid_id() {
        let mut close_bytes = vec![];

        close_bytes.put_u8(0x01); // id

        assert_eq!(
            Close::try_from(close_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_close_with_invalid_handle() {
        let mut close_bytes = vec![];

        close_bytes.put_u32(0x01); // id
        close_bytes.put_u8(0x01); // handle

        assert_eq!(
            Close::try_from(close_bytes.as_slice()),
            Err(Error::BadMessage)
        );
    }
}
