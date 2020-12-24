use crate::error::Error;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Read {}

impl TryFrom<&[u8]> for Read {
    type Error = Error;

    fn try_from(_item: &[u8]) -> Result<Self, Self::Error> {
        Err(Error::Unimplemented)
    }
}
