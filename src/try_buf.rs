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
