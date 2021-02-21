use bytes::{BufMut, Bytes, BytesMut};
use std::convert::From;
use std::convert::TryInto;

#[derive(Debug, PartialEq)]
pub struct Data {
    pub id: u32,
    pub data: Vec<u8>,
}

impl From<&Data> for Bytes {
    fn from(data: &Data) -> Self {
        let mut data_bytes = BytesMut::new();

        data_bytes.put_u32(data.id);
        data_bytes.put_u32(data.data.len().try_into().unwrap());
        data_bytes.put_slice(&data.data);

        data_bytes.freeze()
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
}
