use crate::error::Error;
use bytes::Buf;
use bytes::Bytes;
use std::convert::TryInto;

pub trait TryBuf: Buf {
    fn try_get_bytes(&mut self, len: usize) -> Result<Bytes, Error>;

    fn try_get_string(&mut self) -> Result<String, Error>;

    fn try_get_u8(&mut self) -> Result<u8, Error>;

    fn try_get_u32(&mut self) -> Result<u32, Error>;

    fn try_get_u64(&mut self) -> Result<u64, Error>;
}

impl<T: Buf> TryBuf for T {
    fn try_get_u8(&mut self) -> Result<u8, Error> {
        if self.remaining() < std::mem::size_of::<u8>() {
            return Err(Error::BadMessage);
        }

        Ok(self.get_u8())
    }

    fn try_get_u32(&mut self) -> Result<u32, Error> {
        if self.remaining() < std::mem::size_of::<u32>() {
            return Err(Error::BadMessage);
        }

        Ok(self.get_u32())
    }

    fn try_get_u64(&mut self) -> Result<u64, Error> {
        if self.remaining() < std::mem::size_of::<u64>() {
            return Err(Error::BadMessage);
        }

        Ok(self.get_u64())
    }

    fn try_get_bytes(&mut self, len: usize) -> Result<Bytes, Error> {
        if self.remaining() < len {
            return Err(Error::BadMessage);
        }

        Ok(self.copy_to_bytes(len))
    }

    fn try_get_string(&mut self) -> Result<String, Error> {
        let length = self.try_get_u32()?;

        let length = match length.try_into() {
            Ok(length) => length,
            Err(_) => return Err(Error::BadMessage),
        };

        let string_bytes = self.try_get_bytes(length)?;

        let string = match String::from_utf8((&string_bytes).to_vec()) {
            Ok(string) => string,
            Err(_) => return Err(Error::BadMessage),
        };

        Ok(string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_get_u8() {
        let u8_bytes: Vec<u8> = vec![0x01, 0x02];

        assert_eq!(u8_bytes.as_slice().try_get_u8(), Ok(0x01));
    }

    #[test]
    fn test_try_get_u8_with_invalid_data() {
        assert_eq!(vec![].as_slice().try_get_u8(), Err(Error::BadMessage));
    }

    #[test]
    fn test_try_get_u32() {
        let u32_bytes: Vec<u8> = vec![0x00, 0x00, 0x00, 0x01];

        assert_eq!(u32_bytes.as_slice().try_get_u32(), Ok(0x01));
    }

    #[test]
    fn test_try_get_u32_with_invalid_data() {
        let u32_bytes: Vec<u8> = vec![0x00, 0x00, 0x01];

        assert_eq!(u32_bytes.as_slice().try_get_u32(), Err(Error::BadMessage));
    }

    #[test]
    fn test_try_get_u64() {
        let u64_bytes: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];

        assert_eq!(u64_bytes.as_slice().try_get_u64(), Ok(0x01));
    }

    #[test]
    fn test_try_get_u64_with_invalid_data() {
        let u64_bytes: Vec<u8> = vec![0x00];

        assert_eq!(u64_bytes.as_slice().try_get_u64(), Err(Error::BadMessage));
    }

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
}
