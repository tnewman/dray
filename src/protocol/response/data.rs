use bytes::{BufMut, Bytes, BytesMut};
use std::convert::From;
use std::convert::TryInto;
use std::fmt::Debug;
use tracing::Level;

#[derive(PartialEq, Eq)]
pub struct Data {
    pub id: u32,
    pub data: Vec<u8>,
}

impl From<&Data> for Bytes {
    #[tracing::instrument(level = Level::TRACE)]
    fn from(data: &Data) -> Self {
        let mut data_bytes = BytesMut::new();

        data_bytes.put_u32(data.id);
        data_bytes.put_u32(data.data.len().try_into().unwrap());
        data_bytes.put_slice(&data.data);

        data_bytes.freeze()
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("id", &self.id)
            .field("len", &self.data.len())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use bytes::Buf;

    #[test]
    fn test_from_creates_data_bytes() {
        let data = Data {
            id: 0x01,
            data: vec![0x02, 0x03],
        };

        let data_bytes = &mut Bytes::from(&data);

        assert_eq!(0x01, data_bytes.get_u32());
        assert_eq!(0x02, data_bytes.get_u32());
        assert_eq!(0x02, data_bytes.get_u8());
        assert_eq!(0x03, data_bytes.get_u8());
    }

    #[test]
    fn test_debug_formats_efficient_debug_string() {
        let data = Data {
            id: 0x01,
            data: vec![0x02, 0x03],
        };

        assert_eq!("Data { id: 1, len: 2 }", format!("{:?}", data));
    }
}
