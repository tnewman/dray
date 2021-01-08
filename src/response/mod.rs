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
