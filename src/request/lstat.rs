use crate::error::Error;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Lstat {}

impl TryFrom<&[u8]> for Lstat {
    type Error = Error;

    fn try_from(_item: &[u8]) -> Result<Self, Self::Error> {
        Err(Error::Unimplemented)
    }
}
