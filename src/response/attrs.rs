use crate::file_attributes::FileAttributes;
use bytes::BufMut;
use std::convert::From;

pub struct Attrs {
    pub id: u32,
    pub file_attributes: FileAttributes,
}

impl From<&Attrs> for Vec<u8> {
    fn from(item: &Attrs) -> Self {
        let mut attrs_bytes: Vec<u8> = vec![];

        attrs_bytes.put_u32(item.id);
        attrs_bytes.put_slice(&Vec::from(&item.file_attributes));

        attrs_bytes
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use bytes::Buf;

    #[test]
    fn test_from_creates_attrs_bytes() {
        let attrs = Attrs {
            id: 0x01,
            file_attributes: FileAttributes {
                size: Some(1000),
                uid: Some(100),
                gid: Some(200),
                permissions: Some(777),
                atime: Some(300),
                mtime: Some(400),
            }
        };

        let mut attrs_bytes: &[u8] = &Vec::from(&attrs);

        assert_eq!(0x01, attrs_bytes.get_u32());
        assert_eq!(0x0F, attrs_bytes.get_u32()); // check attributes bitmask
        assert_eq!(true, attrs_bytes.has_remaining());
    }
}
