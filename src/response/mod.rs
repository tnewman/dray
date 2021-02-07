pub mod attrs;
pub mod data;
pub mod handle;
pub mod name;
pub mod status;
pub mod version;

#[derive(Debug, PartialEq)]
pub enum Response {
    Version(version::Version),
    Status(status::Status),
    Handle(handle::Handle),
    Data(data::Data),
    Name(name::Name),
    Attrs(attrs::Attrs),
}

impl From<&Response> for Vec<u8> {
    fn from(item: &Response) -> Self {
        match item {
            Response::Version(version) => version.into(),
            Response::Status(status) => status.into(),
            Response::Handle(handle) => handle.into(),
            Response::Data(data) => data.into(),
            Response::Name(name) => name.into(),
            Response::Attrs(attrs) => attrs.into(),
        }
    }
}

#[cfg(test)]
mod test {

    use crate::file_attributes::FileAttributes;

    use super::*;

    use bytes::Buf;

    #[test]
    fn test_from_creates_version_bytes() {
        let version = Response::Version(version::Version { version: 0x01 });

        let mut version_bytes: &[u8] = &Vec::from(&version);

        assert_eq!(0x01, version_bytes.get_u32());
    }

    #[test]
    fn test_from_creates_status_bytes() {
        let status = Response::Status(status::Status {
            id: 0x01,
            status_code: status::StatusCode::Ok,
            error_message: String::from("OK"),
        });

        let mut status_bytes: &[u8] = &Vec::from(&status);

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

        let mut handle_bytes: &[u8] = &Vec::from(&handle);

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

        let mut data_bytes: &[u8] = &Vec::from(&data);

        assert_eq!(0x01, data_bytes.get_u32());
        assert_eq!(0x02, data_bytes.get_u32()); // data length
        assert_eq!(&[0x02, 0x03], &data_bytes.copy_to_bytes(2)[..]); // data
    }

    #[test]
    fn test_from_creates_name_bytes() {
        let file_attributes = get_file_attributes();
        let file_attributes_bytes = &Vec::from(&file_attributes);

        let name = Response::Name(name::Name {
            id: 0x01,
            files: vec![name::File {
                file_name: String::from("file"),
                long_name: String::from("long"),
                file_attributes: file_attributes,
            }],
        });

        let mut name_bytes: &[u8] = &Vec::from(&name);

        assert_eq!(0x01, name_bytes.get_u32());
        assert_eq!(0x01, name_bytes.get_u32());
        assert_eq!(0x04, name_bytes.get_u32()); // file length
        assert_eq!(&[0x66, 0x69, 0x6C, 0x65], &name_bytes.copy_to_bytes(4)[..]); // file
        assert_eq!(0x04, name_bytes.get_u32()); // long length
        assert_eq!(&[0x6C, 0x6F, 0x6E, 0x67], &name_bytes.copy_to_bytes(4)[..]); // long
        assert_eq!(file_attributes_bytes, &name_bytes[..]);
    }

    #[test]
    fn test_from_creates_attrs_bytes() {
        let file_attributes = get_file_attributes();
        let file_attributes_bytes = &Vec::from(&file_attributes);

        let attrs = attrs::Attrs {
            id: 0x01,
            file_attributes,
        };

        let mut attrs_bytes: &[u8] = &Vec::from(&attrs);

        assert_eq!(0x01, attrs_bytes.get_u32());
        assert_eq!(file_attributes_bytes, &attrs_bytes[..]);
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
