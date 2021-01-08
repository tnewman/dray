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
