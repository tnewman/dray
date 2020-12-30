use crate::file_attributes::FileAttributes;

pub struct Name {
    pub id: u32,
    pub files: File,
}

pub struct File {
    pub file_name: String,
    pub long_name: String,
    pub file_attributes: FileAttributes,
}
