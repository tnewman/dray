pub mod attrs;
pub mod data;
pub mod handle;
pub mod name;
pub mod status;
pub mod version;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::debug;
use log::log_enabled;
use log::Level::Debug;
use std::convert::TryFrom;

use crate::error::Error;

const DATA_TYPE_LENGTH: u32 = 1;

#[derive(Debug, PartialEq)]
pub enum Response {
    Version(version::Version),
    Status(status::Status),
    Handle(handle::Handle),
    Data(data::Data),
    Name(name::Name),
    Attrs(attrs::Attrs),
}

impl Response {
    pub fn build_error_response(id: u32, error: Error) -> Self {
        match error {
            Error::BadMessage => Response::build_status(
                id,
                status::StatusCode::BadMessage,
                "The client sent a bad message.",
            ),
            Error::NoSuchFile => Response::build_status(
                id,
                status::StatusCode::NoSuchFile,
                "The requested file was not found.",
            ),
            Error::PermissionDenied => Response::build_status(
                id,
                status::StatusCode::PermissionDenied,
                "The client has insufficient privileges to perform the requested operation.",
            ),
            Error::Unimplemented => Response::build_status(
                id,
                status::StatusCode::OperationUnsupported,
                "The requested operation is unsupported.",
            ),
            _ => Response::build_status(
                id,
                status::StatusCode::Failure,
                "An error occurred on the server.",
            ),
        }
    }

    fn build_status(id: u32, status_code: status::StatusCode, error_message: &str) -> Response {
        Response::Status(status::Status {
            id,
            status_code,
            error_message: error_message.to_string(),
        })
    }
}

impl From<&Response> for Bytes {
    fn from(response: &Response) -> Self {
        let data_type: u8 = match response {
            Response::Version(_) => 2,
            Response::Status(_) => 101,
            Response::Handle(_) => 102,
            Response::Data(_) => 103,
            Response::Name(_) => 104,
            Response::Attrs(_) => 105,
        };

        let data_payload: Bytes = match response {
            Response::Version(version) => version.into(),
            Response::Status(status) => status.into(),
            Response::Handle(handle) => handle.into(),
            Response::Data(data) => data.into(),
            Response::Name(name) => name.into(),
            Response::Attrs(attrs) => attrs.into(),
        };

        let data_length = DATA_TYPE_LENGTH + u32::try_from(data_payload.remaining()).unwrap();

        let mut response_bytes = BytesMut::new();
        response_bytes.put_u32(data_length);
        response_bytes.put_u8(data_type);
        response_bytes.put_slice(&data_payload);

        let response_bytes = response_bytes.freeze();

        if log_enabled!(Debug) {
            debug!("Response bytes: {}", hex::encode(&response_bytes));
        }

        response_bytes
    }
}

#[cfg(test)]
mod test {

    use crate::protocol::file_attributes::FileAttributes;

    use super::*;

    use bytes::Buf;

    #[test]
    fn test_from_creates_version_bytes() {
        let version = Response::Version(version::Version { version: 0x01 });

        let version_bytes = &mut Bytes::from(&version);

        assert_eq!(5, version_bytes.get_u32());
        assert_eq!(2, version_bytes.get_u8());
        assert_eq!(0x01, version_bytes.get_u32());
    }

    #[test]
    fn test_from_creates_status_bytes() {
        let status = Response::Status(status::Status {
            id: 0x01,
            status_code: status::StatusCode::Ok,
            error_message: String::from("OK"),
        });

        let status_bytes = &mut Bytes::from(&status);

        assert_eq!(24, status_bytes.get_u32());
        assert_eq!(101, status_bytes.get_u8());
        assert_eq!(0x01, status_bytes.get_u32());
        assert_eq!(0x00, status_bytes.get_u32());
        assert_eq!(0x02, status_bytes.get_u32()); // OK length
        assert_eq!(&[0x4F, 0x4B], &status_bytes.copy_to_bytes(2)[..]); // OK bytes
        assert_eq!(0x05, status_bytes.get_u32()); // en-US length
        assert_eq!(
            &[0x65, 0x6E, 0x2D, 0x55, 0x53],
            &status_bytes.copy_to_bytes(5)[..]
        ) // OK bytes
    }

    #[test]
    fn test_from_creates_handle_bytes() {
        let handle = Response::Handle(handle::Handle {
            id: 0x01,
            handle: String::from("handle"),
        });

        let handle_bytes = &mut Bytes::from(&handle);

        assert_eq!(15, handle_bytes.get_u32());
        assert_eq!(102, handle_bytes.get_u8());
        assert_eq!(0x01, handle_bytes.get_u32());
        assert_eq!(0x06, handle_bytes.get_u32()); // handle length
        assert_eq!(
            &[0x68, 0x61, 0x6E, 0x64, 0x6C, 0x65],
            &handle_bytes.copy_to_bytes(6)[..]
        ); // handle bytes
    }

    #[test]
    fn test_from_creates_data_bytes() {
        let data = Response::Data(data::Data {
            id: 0x01,
            data: vec![0x02, 0x03],
        });

        let data_bytes = &mut Bytes::from(&data);

        assert_eq!(11, data_bytes.get_u32());
        assert_eq!(103, data_bytes.get_u8());
        assert_eq!(0x01, data_bytes.get_u32());
        assert_eq!(0x02, data_bytes.get_u32()); // data length
        assert_eq!(&[0x02, 0x03], &data_bytes.copy_to_bytes(2)[..]); // data
    }

    #[test]
    fn test_from_creates_name_bytes() {
        let file_attributes = get_file_attributes();
        let file_attributes_bytes = &mut Bytes::from(&file_attributes);

        let name = Response::Name(name::Name {
            id: 0x01,
            files: vec![name::File {
                file_name: String::from("file"),
                file_attributes: file_attributes,
            }],
        });

        let name_bytes = &mut Bytes::from(&name);

        assert_eq!(74, name_bytes.get_u32());
        assert_eq!(104, name_bytes.get_u8());
        assert_eq!(0x01, name_bytes.get_u32());
        assert_eq!(0x01, name_bytes.get_u32());
        assert_eq!(0x04, name_bytes.get_u32()); // file length
        assert_eq!(&[0x66, 0x69, 0x6C, 0x65], &name_bytes.copy_to_bytes(4)[..]); // file

        let long = "---------- 0 2 3 0 Jan 01 1970 00:00 file";
        assert_eq!(long.len() as u32, name_bytes.get_u32()); // long length
        assert_eq!(long.as_bytes(), &name_bytes.copy_to_bytes(long.len())[..]); // long
        assert_eq!(file_attributes_bytes, &name_bytes[..]);
    }

    #[test]
    fn test_from_creates_attrs_bytes() {
        let file_attributes = get_file_attributes();
        let file_attributes_bytes = &mut Bytes::from(&file_attributes);

        let attrs = Response::Attrs(attrs::Attrs {
            id: 0x01,
            file_attributes,
        });

        let attrs_bytes = &mut Bytes::from(&attrs);

        assert_eq!(17, attrs_bytes.get_u32());
        assert_eq!(105, attrs_bytes.get_u8());
        assert_eq!(0x01, attrs_bytes.get_u32());
        assert_eq!(file_attributes_bytes, &attrs_bytes[..]);
    }

    #[test]
    fn test_map_error_response_maps_bad_message() {
        let expected_status = Response::Status(status::Status {
            id: 1000,
            status_code: status::StatusCode::BadMessage,
            error_message: String::from("The client sent a bad message."),
        });

        assert_eq!(
            expected_status,
            Response::build_error_response(1000, Error::BadMessage)
        );
    }

    #[test]
    fn test_map_error_response_maps_no_such_file() {
        let expected_status = Response::Status(status::Status {
            id: 1000,
            status_code: status::StatusCode::NoSuchFile,
            error_message: String::from("The requested file was not found."),
        });

        assert_eq!(
            expected_status,
            Response::build_error_response(1000, Error::NoSuchFile)
        );
    }

    #[test]
    fn test_map_error_response_maps_permission_denied() {
        let expected_status = Response::Status(status::Status {
            id: 1000,
            status_code: status::StatusCode::PermissionDenied,
            error_message: String::from(
                "The client has insufficient privileges to perform the requested operation.",
            ),
        });

        assert_eq!(
            expected_status,
            Response::build_error_response(1000, Error::PermissionDenied)
        );
    }

    #[test]
    fn test_map_error_response_maps_unimplemented() {
        let expected_status = Response::Status(status::Status {
            id: 1000,
            status_code: status::StatusCode::OperationUnsupported,
            error_message: String::from("The requested operation is unsupported."),
        });

        assert_eq!(
            expected_status,
            Response::build_error_response(1000, Error::Unimplemented)
        );
    }

    #[test]
    fn test_map_error_response_maps_other_error() {
        let expected_status = Response::Status(status::Status {
            id: 1000,
            status_code: status::StatusCode::Failure,
            error_message: String::from("An error occurred on the server."),
        });

        assert_eq!(
            expected_status,
            Response::build_error_response(
                1000,
                Error::Storage(String::from("Storage is unavailable."))
            )
        );
    }

    fn get_file_attributes() -> FileAttributes {
        FileAttributes {
            size: None,
            uid: Some(0x02),
            gid: Some(0x03),
            permissions: None,
            atime: None,
            mtime: None,
        }
    }
}
