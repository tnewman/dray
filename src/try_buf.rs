use crate::error::Error;
use bytes::Buf;
use bytes::BufMut;
use bytes::Bytes;
use std::convert::TryFrom;
use std::convert::TryInto;

pub trait TryBuf: Buf {
    fn try_get_bytes(&mut self, len: u32) -> Result<Bytes, Error>;

    fn try_get_string(&mut self) -> Result<String, Error>;
}

impl<T: Buf> TryBuf for T {
    fn try_get_bytes(&mut self, len: u32) -> Result<Bytes, Error> {
        let len = match len.try_into() {
            Ok(len) => len,
            Err(_) => return Err(Error::BadMessage),
        };

        if self.remaining() < len {
            return Err(Error::BadMessage);
        }

        Ok(self.copy_to_bytes(len))
    }

    fn try_get_string(&mut self) -> Result<String, Error> {
        let len = match self.try_get_u32() {
            Ok(len) => len,
            Err(_) => return Err(Error::BadMessage),
        };

        let string_bytes = self.try_get_bytes(len)?;

        let string = match String::from_utf8(string_bytes.to_vec()) {
            Ok(string) => string,
            Err(_) => return Err(Error::BadMessage),
        };

        Ok(string)
    }
}

pub trait TryBufMut: BufMut {
    fn try_put_str(&mut self, str: &str) -> Result<(), Error>;
}

impl<T: BufMut> TryBufMut for T {
    fn try_put_str(&mut self, str: &str) -> Result<(), Error> {
        let len = str.len();

        let len = match u32::try_from(len) {
            Ok(len) => len,
            Err(_) => return Err(Error::BadMessage),
        };

        self.put_u32(len);
        self.put(str.as_bytes());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_get_bytes() {
        let bytes: Vec<u8> = vec![0x00, 0x01];

        assert_eq!(
            bytes.as_slice().try_get_bytes(1),
            Ok(Bytes::from(vec![0x00]))
        );
    }

    #[test]
    fn test_try_get_bytes_with_overrun() {
        let bytes: Vec<u8> = vec![0x00];

        assert_eq!(bytes.as_slice().try_get_bytes(2), Err(Error::BadMessage));
    }

    #[test]
    fn test_try_get_string() {
        let string: Vec<u8> = vec![0x00, 0x00, 0x00, 0x04, 0x54, 0x45, 0x53, 0x54]; // TEST

        assert_eq!(string.as_slice().try_get_string(), Ok(String::from("TEST")))
    }

    #[test]
    fn test_try_get_string_with_invalid_length() {
        let string: Vec<u8> = vec![0x00, 0x01]; // length must be 4 bytes (uint32)

        assert_eq!(string.as_slice().try_get_string(), Err(Error::BadMessage))
    }

    #[test]
    fn test_try_get_string_with_mismatched_length() {
        let string: Vec<u8> = vec![0x00, 0x00, 0x00, 0x08, 0x54, 0x45, 0x53, 0x54]; // TEST with length 8

        assert_eq!(string.as_slice().try_get_string(), Err(Error::BadMessage))
    }

    #[test]
    fn test_try_get_string_with_invalid_utf8() {
        let string: Vec<u8> = vec![0x00, 0x00, 0x00, 0x01, 0xFF];

        assert_eq!(string.as_slice().try_get_string(), Err(Error::BadMessage))
    }

    #[test]
    fn test_try_put_string() {
        let string = "TEST";

        let mut bytes: Vec<u8> = Vec::new();
        let result = bytes.try_put_str(string);

        assert_eq!(result, Ok(()));
        assert_eq!(
            bytes.as_slice(),
            &[0x00, 0x00, 0x00, 0x04, 0x54, 0x45, 0x53, 0x54]
        ); // TEST with length 4
    }
}
