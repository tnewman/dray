use crate::file_attributes::FileAttributes;
use bytes::BufMut;
use std::convert::From;
use std::convert::TryInto;

pub struct Name {
    pub id: u32,
    pub files: Vec<File>,
}

impl From<&Name> for Vec<u8> {
    fn from(item: &Name) -> Self {
        let mut name_bytes: Vec<u8> = vec![];

        name_bytes.put_u32(item.id);
        name_bytes.put_u32(item.files.len().try_into().unwrap());

        for file in &item.files {
            name_bytes.put_slice(&Vec::from(file));
        }

        name_bytes
    }
}

pub struct File {
    pub file_name: String,
    pub long_name: String,
    pub file_attributes: FileAttributes,
}

impl From<&File> for Vec<u8> {
    fn from(item: &File) -> Self {
        let mut file_bytes: Vec<u8> = vec![];

        let file_name_bytes = item.file_name.as_bytes();
        file_bytes.put_u32(file_name_bytes.len().try_into().unwrap());
        file_bytes.put_slice(file_name_bytes);

        let long_name_bytes = item.long_name.as_bytes();
        file_bytes.put_u32(long_name_bytes.len().try_into().unwrap());
        file_bytes.put_slice(long_name_bytes);

        file_bytes.put_slice(&Vec::from(&item.file_attributes));

        file_bytes
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use bytes::Buf;

    #[test]
    fn test_from_creates_file_bytes() {
        let file = File {
            file_name: String::from("file"),
            long_name: String::from("long"),
            file_attributes: FileAttributes {
                size: None,
                uid: None,
                gid: None,
                permissions: None,
                atime: None,
                mtime: None,
            },
        };

        let mut file_bytes: &[u8] = &Vec::from(&file);

        assert_eq!(0x04, file_bytes.get_u32());
        assert_eq!(&[0x66, 0x69, 0x6C, 0x65], &file_bytes.copy_to_bytes(4)[..]);
        assert_eq!(0x04, file_bytes.get_u32());
        assert_eq!(&[0x6C, 0x6F, 0x6E, 0x67], &file_bytes.copy_to_bytes(4)[..]);
        assert_eq!(true, file_bytes.has_remaining()); // has file attributes
    }
}
