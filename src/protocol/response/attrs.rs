use crate::protocol::file_attributes::FileAttributes;
use bytes::{BufMut, Bytes, BytesMut};
use std::convert::From;

#[derive(Debug, PartialEq)]
pub struct Attrs {
    pub id: u32,
    pub file_attributes: FileAttributes,
}

impl From<&Attrs> for Bytes {
    fn from(attrs: &Attrs) -> Self {
        let mut attrs_bytes = BytesMut::new();

        attrs_bytes.put_u32(attrs.id);
        attrs_bytes.put_slice(&Bytes::from(&attrs.file_attributes));

        attrs_bytes.freeze()
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
            },
        };

        let mut attrs_bytes = Bytes::from(&attrs);

        assert_eq!(0x01, attrs_bytes.get_u32());
        assert_eq!(0x0F, attrs_bytes.get_u32()); // check attributes bitmask
        assert_eq!(true, attrs_bytes.has_remaining());
    }
}
