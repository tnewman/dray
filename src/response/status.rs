use bytes::BufMut;
use std::convert::From;
use std::convert::TryInto;

pub struct Status {
    pub id: u32,
    pub status_code: StatusCode,
    pub error_message: String,
}

#[derive(Copy, Clone)]
pub enum StatusCode {
    Ok = 0,
    EOF = 1,
    NoSuchFile = 2,
    PermissionDenied = 3,
    Failure = 4,
    BadMessage = 5,
    NoConnection = 6,
    ConnectionLost = 7,
    OperationUnsupported = 8,
}

impl From<&Status> for Vec<u8> {
    fn from(item: &Status) -> Self {
        let mut status_bytes: Vec<u8> = vec![];

        status_bytes.put_u32(item.id);
        status_bytes.put_u32(item.status_code as u32);

        let error_message_bytes = item.error_message.as_bytes();
        status_bytes.put_u32(error_message_bytes.len().try_into().unwrap());
        status_bytes.put_slice(error_message_bytes);

        let language_tag_bytes = b"en-us";
        status_bytes.put_u32(language_tag_bytes.len().try_into().unwrap());
        status_bytes.put_slice(language_tag_bytes);

        status_bytes
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use bytes::Buf;

    #[test]
    fn test_from_creates_status_bytes() {
        let status = Status {
            id: 0x01,
            status_code: StatusCode::Failure,
            error_message: String::from("Sample"),
        };

        let mut status_bytes: &[u8] = &Vec::from(&status);

        assert_eq!(0x01, status_bytes.get_u32());
        assert_eq!(0x04, status_bytes.get_u32());
        assert_eq!(0x06, status_bytes.get_u32()); // Error message length
        assert_eq!(
            &[0x53, 0x61, 0x6D, 0x70, 0x6C, 0x65],
            &status_bytes.copy_to_bytes(6)[..]
        ); // Error message
        assert_eq!(0x05, status_bytes.get_u32()); // Language length
        assert_eq!(
            &[0x65, 0x6E, 0x2D, 0x75, 0x73],
            &status_bytes.copy_to_bytes(5)[..]
        ); // Language
    }
}
