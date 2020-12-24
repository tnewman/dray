use crate::error::Error;
use crate::file_attributes::FileAttributes;
use crate::try_buf::TryBuf;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Open {
    pub id: u32,
    pub filename: String,
    pub file_attributes: FileAttributes,
    pub open_options: OpenOptions,
}

impl TryFrom<&[u8]> for Open {
    type Error = Error;

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        let mut bytes = item;

        let id = bytes.try_get_u32()?;
        let filename = bytes.try_get_string()?;
        let file_attributes = FileAttributes::try_from(item)?;

        Ok(Open {
            id,
            filename,
            file_attributes,
            open_options: OpenOptions {
                read: false,
                write: false,
                create: false,
                create_new: false,
                truncate: false,
            },
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct OpenOptions {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub create_new: bool,
    pub truncate: bool,
}
