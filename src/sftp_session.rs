use crate::protocol::{
    file_attributes::FileAttributes,
    request::{self, Request},
    response::{self, Response},
};
use crate::storage::Storage;
use anyhow::Result;
use log::error;
use log::info;
use std::{sync::Arc, time::Duration};

pub struct SftpSession {
    object_storage: Arc<dyn Storage>,
    user: String,
}

impl SftpSession {
    pub fn new(object_storage: Arc<dyn Storage>, user: String) -> Self {
        SftpSession {
            object_storage,
            user,
        }
    }

    pub async fn handle_request(&self, request: Request) -> Response {
        info!("Received request: {:?}", request);

        let response = match request {
            Request::Init(init_request) => self.handle_init_request(init_request),
            Request::Open(open_request) => self.handle_open_request(open_request).await,
            Request::Close(close_request) => self.handle_close_request(close_request).await,
            Request::Read(read_request) => self.handle_read_request(read_request).await,
            Request::Write(write_request) => self.handle_write_request(write_request).await,
            Request::Lstat(lstat_request) => self.handle_lstat_request(lstat_request),
            Request::Fstat(fstat_request) => self.handle_fstat_request(fstat_request),
            Request::Setstat(setstat_request) => self.handle_setstat_request(setstat_request),
            Request::Fsetstat(fsetstat_request) => self.handle_fsetstat_request(fsetstat_request),
            Request::Opendir(opendir_request) => self.handle_opendir_request(opendir_request).await,
            Request::Readdir(readdir_request) => self.handle_readdir_request(readdir_request).await,
            Request::Remove(remove_request) => self.handle_remove_request(remove_request),
            Request::Mkdir(mkdir_request) => self.handle_mkdir_request(mkdir_request).await,
            Request::Rmdir(rmdir_request) => self.handle_rmdir_request(rmdir_request),
            Request::Realpath(realpath_request) => self.handle_realpath_request(realpath_request),
            Request::Stat(stat_request) => self.handle_stat_request(stat_request).await,
            Request::Rename(rename_request) => self.handle_rename_request(rename_request),
            Request::Readlink(readlink_request) => self.handle_readlink_request(readlink_request),
            Request::Symlink(symlink_request) => self.handle_symlink_request(symlink_request),
        };

        let response = match response {
            Ok(response) => response,
            Err(error) => {
                error!("Received error while processing request: {}", error);

                // TODO: Move error handling into individual handlers to get the id right
                Response::Status(response::status::Status {
                    id: 0,
                    status_code: response::status::StatusCode::BadMessage,
                    error_message: String::from("Internal server error."),
                })
            }
        };

        info!("Sending response: {:?}", response);
        response
    }

    fn handle_init_request(&self, _init_request: request::init::Init) -> Result<Response> {
        Ok(Response::Version(response::version::Version { version: 3 }))
    }

    async fn handle_open_request(&self, open_request: request::open::Open) -> Result<Response> {
        let handle = if open_request.open_options.create {
            self.object_storage
                .open_write_handle(open_request.filename)
                .await?
        } else if open_request.open_options.read {
            self.object_storage
                .open_read_handle(open_request.filename)
                .await?
        } else {
            return Ok(Response::Status(response::status::Status {
                id: open_request.id,
                status_code: response::status::StatusCode::Failure,
                error_message: String::from("Unsupported file open mode."),
            }));
        };

        Ok(Response::Handle(response::handle::Handle {
            id: open_request.id,
            handle,
        }))
    }

    async fn handle_close_request(
        &self,
        close_request: request::handle::Handle,
    ) -> Result<Response> {
        self.object_storage
            .close_handle(&close_request.handle)
            .await?;

        Ok(Response::Status(response::status::Status {
            id: close_request.id,
            status_code: response::status::StatusCode::Ok,
            error_message: String::from(""),
        }))
    }

    async fn handle_read_request(&self, read_request: request::read::Read) -> Result<Response> {
        let data = self
            .object_storage
            .read_data(&read_request.handle, read_request.len)
            .await?;

        if data.is_empty() {
            Ok(Response::Status(response::status::Status {
                id: read_request.id,
                status_code: response::status::StatusCode::Eof,
                error_message: String::from("End of file."),
            }))
        } else {
            Ok(Response::Data(response::data::Data {
                id: read_request.id,
                data,
            }))
        }
    }

    async fn handle_write_request(&self, write_request: request::write::Write) -> Result<Response> {
        self.object_storage
            .write_data(&write_request.handle, write_request.data)
            .await?;

        // TODO: This is a hack to prevent Filezilla from running out of request ids.
        // Refactor the handle manager to lock the entire handle manager, so only
        // one request will proceed at a time.
        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(Response::Status(response::status::Status {
            id: write_request.id,
            status_code: response::status::StatusCode::Ok,
            error_message: String::from("Bytes written."),
        }))
    }

    fn handle_lstat_request(&self, lstat_request: request::path::Path) -> Result<Response> {
        Ok(SftpSession::build_not_supported_response(lstat_request.id))
    }

    fn handle_fstat_request(&self, fstat_request: request::path::Path) -> Result<Response> {
        Ok(SftpSession::build_not_supported_response(fstat_request.id))
    }

    fn handle_setstat_request(
        &self,
        setstat_request: request::path_attributes::PathAttributes,
    ) -> Result<Response> {
        Ok(SftpSession::build_not_supported_response(
            setstat_request.id,
        ))
    }

    fn handle_fsetstat_request(
        &self,
        fsetstat_request: request::handle_attributes::HandleAttributes,
    ) -> Result<Response> {
        Ok(SftpSession::build_not_supported_response(
            fsetstat_request.id,
        ))
    }

    async fn handle_opendir_request(
        &self,
        opendir_request: request::path::Path,
    ) -> Result<Response> {
        let handle = self
            .object_storage
            .open_dir_handle(opendir_request.path)
            .await?;

        Ok(Response::Handle(response::handle::Handle {
            id: opendir_request.id,
            handle,
        }))
    }

    async fn handle_readdir_request(
        &self,
        readdir_request: request::handle::Handle,
    ) -> Result<Response> {
        let files = self
            .object_storage
            .read_dir(&readdir_request.handle)
            .await?;

        match files.is_empty() {
            true => Ok(Response::Status(response::status::Status {
                id: readdir_request.id,
                status_code: response::status::StatusCode::Eof,
                error_message: String::from("End of file."),
            })),
            false => Ok(Response::Name(response::name::Name {
                id: readdir_request.id,
                files,
            })),
        }
    }

    fn handle_remove_request(&self, remove_request: request::path::Path) -> Result<Response> {
        Ok(SftpSession::build_not_supported_response(remove_request.id))
    }

    async fn handle_mkdir_request(
        &self,
        mkdir_request: request::path_attributes::PathAttributes,
    ) -> Result<Response> {
        self.object_storage.create_dir(mkdir_request.path).await?;

        Ok(Response::Status(response::status::Status {
            id: mkdir_request.id,
            status_code: response::status::StatusCode::Ok,
            error_message: String::from("Successfully created directory."),
        }))
    }

    fn handle_rmdir_request(&self, rmdir_request: request::path::Path) -> Result<Response> {
        Ok(SftpSession::build_not_supported_response(rmdir_request.id))
    }

    fn handle_realpath_request(&self, realpath_request: request::path::Path) -> Result<Response> {
        let path = if realpath_request.path == "." {
            self.object_storage.get_home(&self.user)
        } else {
            realpath_request.to_normalized_path()
        };

        Ok(Response::Name(response::name::Name {
            id: realpath_request.id,
            files: vec![response::name::File {
                file_name: path,
                file_attributes: FileAttributes {
                    permissions: Some(0o40777),
                    size: None,
                    uid: None,
                    gid: None,
                    atime: None,
                    mtime: None,
                },
            }],
        }))
    }

    async fn handle_stat_request(&self, stat_request: request::path::Path) -> Result<Response> {
        let file_attributes = match self
            .object_storage
            .file_exists(stat_request.path.clone())
            .await?
        {
            true => {
                self.object_storage
                    .get_file_metadata(stat_request.path)
                    .await?
                    .file_attributes
            }
            false => FileAttributes {
                permissions: Some(0o40777),
                size: None,
                uid: None,
                gid: None,
                atime: None,
                mtime: None,
            },
        };

        Ok(Response::Attrs(response::attrs::Attrs {
            id: stat_request.id,
            file_attributes,
        }))
    }

    fn handle_rename_request(&self, rename_request: request::rename::Rename) -> Result<Response> {
        Ok(SftpSession::build_not_supported_response(rename_request.id))
    }

    fn handle_readlink_request(&self, readlink_request: request::path::Path) -> Result<Response> {
        Ok(SftpSession::build_not_supported_response(
            readlink_request.id,
        ))
    }

    fn handle_symlink_request(
        &self,
        symlink_request: request::symlink::Symlink,
    ) -> Result<Response> {
        Ok(SftpSession::build_not_supported_response(
            symlink_request.id,
        ))
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
