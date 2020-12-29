use crate::error::Error;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Rename {
    id: u32,
    old_path: String,
    new_path: String,
}

impl TryFrom<&[u8]> for Rename {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut rename_bytes = item;

        let id = rename_bytes.try_get_u32()?;
        let old_path = rename_bytes.try_get_string()?;
        let new_path = rename_bytes.try_get_string()?;

        Ok(Rename {
            id,
            old_path,
            new_path,
        })
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_parse_rename() {

    }

    #[test]
    fn test_parse_rename_with_invalid_id() {

    }

    #[test]
    fn test_parse_rename_with_invalid_old_path() {

    }

    #[test]
    fn test_parse_rename_with_invalid_new_path() {
        
    }
}