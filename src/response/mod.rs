pub mod attrs;
pub mod data;
pub mod handle;
pub mod name;
pub mod status;
pub mod version;

#[derive(Debug, PartialEq)]
pub enum Response {
    Version,
    Status,
    Handle,
    Data,
    Name,
    Attrs,
}
