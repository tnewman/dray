use bytes::BufMut;
use std::convert::From;
use std::convert::TryInto;

#[derive(Debug, PartialEq)]
pub struct Handle {
    pub id: u32,
    pub handle: String,
}

impl From<&Handle> for Vec<u8> {
    fn from(item: &Handle) -> Self {
        let mut handle_bytes: Vec<u8> = vec![];

        handle_bytes.put_u32(item.id);
        handle_bytes.put_u32(item.handle.len().try_into().unwrap());
        handle_bytes.put_slice(item.handle.as_bytes());

        handle_bytes
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use bytes::Buf;

    #[test]
    fn test_from_creates_data_bytes() {
        let data = Handle {
            id: 0x01,
            handle: String::from("handle"),
        };

        let mut data_bytes: &[u8] = &Vec::from(&data);

        assert_eq!(0x01, data_bytes.get_u32());
        assert_eq!(0x06, data_bytes.get_u32());
        assert_eq!(&[0x68, 0x61, 0x6e, 0x64, 0x6c, 0x65], data_bytes.bytes());
    }
}
