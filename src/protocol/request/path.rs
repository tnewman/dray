use crate::error::Error;
use crate::try_buf::TryBuf;

use bytes::Bytes;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Path {
    pub id: u32,
    pub path: String,
}

impl Path {
    pub fn to_normalized_path(&self) -> String {
        let mut normalized_components: Vec<&str> = vec![];
        let mut components_to_skip: usize = 0;

        for path_component in self.path.rsplit("/") {
            match path_component {
                "" => {}
                "." => {}
                ".." => components_to_skip = components_to_skip + 1,
                _ => {
                    if components_to_skip > 0 {
                        components_to_skip = components_to_skip - 1;
                    } else {
                        normalized_components.push(path_component);
                    }
                }
            }
        }

        if normalized_components.len() > 0 {
            normalized_components.push("");
            normalized_components.reverse();
            normalized_components.join("/")
        } else {
            "/".to_owned()
        }
    }
}

impl TryFrom<&mut Bytes> for Path {
    type Error = Error;

    fn try_from(path_bytes: &mut Bytes) -> Result<Self, Self::Error> {
        let id = path_bytes.try_get_u32()?;
        let path = path_bytes.try_get_string()?;

        Ok(Path { id, path })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::try_buf::TryBufMut;

    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_parse_path() {
        let mut path_bytes = BytesMut::new();

        path_bytes.put_u32(0x01); // id
        path_bytes.try_put_str("/filename").unwrap(); // filename

        assert_eq!(
            Path::try_from(&mut path_bytes.freeze()),
            Ok(Path {
                id: 0x01,
                path: String::from("/filename")
            })
        );
    }

    #[test]
    fn test_parse_path_with_invalid_id() {
        let mut path_bytes = BytesMut::new();

        path_bytes.put_u8(0x01); // invalid id

        assert_eq!(
            Path::try_from(&mut path_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_parse_path_with_invalid_filename() {
        let mut path_bytes = BytesMut::new();

        path_bytes.put_u32(0x01); // id
        path_bytes.put_u32(0x10); // bad length

        assert_eq!(
            Path::try_from(&mut path_bytes.freeze()),
            Err(Error::BadMessage)
        );
    }

    #[test]
    fn test_normalize_path_skips_normalized_path() {
        let path = create_path("/sample/path");

        assert_eq!("/sample/path", path.to_normalized_path());
    }

    #[test]
    fn test_normalize_path_converts_relative_path() {
        let path = create_path("sample/path");

        assert_eq!("/sample/path", path.to_normalized_path());
    }

    #[test]
    fn test_normalize_path_strips_trailing_slash() {
        let path = create_path("/sample/path/");

        assert_eq!("/sample/path", path.to_normalized_path());
    }

    #[test]
    fn test_normalize_path_handles_single_dot() {
        let path = create_path("/sample/./path");

        assert_eq!("/sample/path", path.to_normalized_path());
    }

    #[test]
    fn test_normalize_path_pops_component_with_double_dot() {
        let path = create_path("/sample/../path");

        assert_eq!("/path", path.to_normalized_path());
    }

    #[test]
    fn test_normalize_returns_root_with_no_components_remaining() {
        let path = create_path("/../..");

        assert_eq!("/", path.to_normalized_path());
    }

    #[test]
    fn test_normalize_strips_extra_slashes() {
        let path = create_path("//////sample///////path////");

        assert_eq!("/sample/path", path.to_normalized_path());
    }

    fn create_path(path: &str) -> Path {
        Path {
            id: 1,
            path: path.to_owned(),
        }
    }
}
