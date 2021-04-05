use std::sync::Arc;

use log::debug;

use crate::protocol::{
    file_attributes::FileAttributes,
    request::{self, Request},
    response::{self, Response},
};
use crate::storage::ObjectStorage;

pub struct SftpSession {
    object_storage: Arc<dyn ObjectStorage>,
    user: String,
}

impl SftpSession {
    pub fn new(object_storage: Arc<dyn ObjectStorage>, user: String) -> Self {
        SftpSession {
            object_storage,
            user,
        }
    }

    pub async fn handle_request(&self, request: Request) -> Response {
        debug!("Received request: {:?}", request);

        let response = match request {
            Request::Init(init_request) => self.handle_init_request(init_request),
            Request::Open(open_request) => self.handle_open_request(open_request),
            Request::Close(close_request) => self.handle_close_request(close_request),
            Request::Read(read_request) => self.handle_read_request(read_request),
            Request::Write(write_request) => self.handle_write_request(write_request),
            Request::Lstat(lstat_request) => self.handle_lstat_request(lstat_request),
            Request::Fstat(fstat_request) => self.handle_fstat_request(fstat_request),
            Request::Setstat(setstat_request) => self.handle_setstat_request(setstat_request),
            Request::Fsetstat(fsetstat_request) => self.handle_fsetstat_request(fsetstat_request),
            Request::Opendir(opendir_request) => self.handle_opendir_request(opendir_request),
            Request::Readdir(readdir_request) => self.handle_readdir_request(readdir_request),
            Request::Remove(remove_request) => self.handle_remove_request(remove_request),
            Request::Mkdir(mkdir_request) => self.handle_mkdir_request(mkdir_request),
            Request::Rmdir(rmdir_request) => self.handle_rmdir_request(rmdir_request),
            Request::Realpath(realpath_request) => self.handle_realpath_request(realpath_request),
            Request::Stat(stat_request) => self.handle_stat_request(stat_request),
            Request::Rename(rename_request) => self.handle_rename_request(rename_request),
            Request::Readlink(readlink_request) => self.handle_readlink_request(readlink_request),
            Request::Symlink(symlink_request) => self.handle_symlink_request(symlink_request),
        };

        debug!("Sending response: {:?}", response);
        response
    }

    fn handle_init_request(&self, _init_request: request::init::Init) -> Response {
        Response::Version(response::version::Version { version: 3 })
    }

    fn handle_open_request(&self, open_request: request::open::Open) -> Response {
        SftpSession::build_not_supported_response(open_request.id)
    }

    fn handle_close_request(&self, close_request: request::handle::Handle) -> Response {
        SftpSession::build_not_supported_response(close_request.id)
    }

    fn handle_read_request(&self, read_request: request::read::Read) -> Response {
        SftpSession::build_not_supported_response(read_request.id)
    }

    fn handle_write_request(&self, write_request: request::write::Write) -> Response {
        SftpSession::build_not_supported_response(write_request.id)
    }

    fn handle_lstat_request(&self, lstat_request: request::path::Path) -> Response {
        SftpSession::build_not_supported_response(lstat_request.id)
    }

    fn handle_fstat_request(&self, fstat_request: request::path::Path) -> Response {
        SftpSession::build_not_supported_response(fstat_request.id)
    }

    fn handle_setstat_request(
        &self,
        setstat_request: request::path_attributes::PathAttributes,
    ) -> Response {
        SftpSession::build_not_supported_response(setstat_request.id)
    }

    fn handle_fsetstat_request(
        &self,
        fsetstat_request: request::handle_attributes::HandleAttributes,
    ) -> Response {
        SftpSession::build_not_supported_response(fsetstat_request.id)
    }

    fn handle_opendir_request(&self, opendir_request: request::path::Path) -> Response {
        SftpSession::build_not_supported_response(opendir_request.id)
    }

    fn handle_readdir_request(&self, readdir_request: request::handle::Handle) -> Response {
        SftpSession::build_not_supported_response(readdir_request.id)
    }

    fn handle_remove_request(&self, remove_request: request::path::Path) -> Response {
        SftpSession::build_not_supported_response(remove_request.id)
    }

    fn handle_mkdir_request(
        &self,
        mkdir_request: request::path_attributes::PathAttributes,
    ) -> Response {
        SftpSession::build_not_supported_response(mkdir_request.id)
    }

    fn handle_rmdir_request(&self, rmdir_request: request::path::Path) -> Response {
        SftpSession::build_not_supported_response(rmdir_request.id)
    }

    fn handle_realpath_request(&self, realpath_request: request::path::Path) -> Response {
        let path = if realpath_request.path == "." {
            self.object_storage.get_home(&self.user)
        } else {
            realpath_request.to_normalized_path()
        };

        Response::Name(response::name::Name {
            id: realpath_request.id,
            files: vec![response::name::File {
                file_name: path.clone(),
                long_name: path,
                file_attributes: FileAttributes {
                    ..Default::default()
                },
            }],
        })
    }

    fn handle_stat_request(&self, stat_request: request::path::Path) -> Response {
        SftpSession::build_not_supported_response(stat_request.id)
    }

    fn handle_rename_request(&self, rename_request: request::rename::Rename) -> Response {
        SftpSession::build_not_supported_response(rename_request.id)
    }

    fn handle_readlink_request(&self, readlink_request: request::path::Path) -> Response {
        SftpSession::build_not_supported_response(readlink_request.id)
    }

    fn handle_symlink_request(&self, symlink_request: request::symlink::Symlink) -> Response {
        SftpSession::build_not_supported_response(symlink_request.id)
    }

    pub fn build_invalid_request_message_response() -> Response {
        Response::Status(response::status::Status {
            id: 0,
            status_code: response::status::StatusCode::BadMessage,
            error_message: String::from("The request message is invalid."),
        })
    }

    fn build_not_supported_response(id: u32) -> Response {
        Response::Status(response::status::Status {
            id,
            status_code: response::status::StatusCode::OperationUnsupported,
            error_message: String::from("Operation Unsupported!"),
        })
    }
}
