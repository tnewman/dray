use bytes::BufMut;
use std::convert::From;

pub struct Data {
    pub id: u32,
    pub data: Vec<u8>,
}

impl From<&Data> for Vec<u8> {
    fn from(item: &Data) -> Self {
        let mut data_bytes: Vec<u8> = vec![];

        data_bytes.put_u32(item.id);
        data_bytes.put_slice(&item.data);

        data_bytes
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

        let mut data_bytes: &[u8] = &Vec::from(&data);

        assert_eq!(0x01, data_bytes.get_u32());
        assert_eq!(0x02, data_bytes.get_u8());
        assert_eq!(0x03, data_bytes.get_u8());
    }
}
