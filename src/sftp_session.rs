use crate::storage::Storage;
use crate::{
    error::Error,
    protocol::{
        file_attributes::FileAttributes,
        request::{self, Request, RequestId},
        response::{self, Response},
    },
};
use tracing::error;
use tracing::Level;

use std::sync::Arc;

pub struct SftpSession {
    object_storage: Arc<dyn Storage>,
    user: String,
    user_home: String,
}

impl SftpSession {
    pub fn new(object_storage: Arc<dyn Storage>, user: String) -> Self {
        let user_home = object_storage.get_home(&user);

        SftpSession {
            object_storage,
            user,
            user_home,
        }
    }

    #[tracing::instrument(skip(self), level = Level::DEBUG)]
    pub async fn handle_request(&self, request: Request) -> Response {
        let request_id = request.get_request_id();

        let response = match request {
            Request::Init(init_request) => self.handle_init_request(init_request),
            Request::Open(open_request) => self.handle_open_request(open_request).await,
            Request::Close(close_request) => self.handle_close_request(close_request).await,
            Request::Read(read_request) => self.handle_read_request(read_request).await,
            Request::Write(write_request) => self.handle_write_request(write_request).await,
            Request::Lstat(lstat_request) => self.handle_lstat_request(lstat_request).await,
            Request::Fstat(fstat_request) => self.handle_fstat_request(fstat_request).await,
            Request::Setstat(setstat_request) => self.handle_setstat_request(setstat_request),
            Request::Fsetstat(fsetstat_request) => self.handle_fsetstat_request(fsetstat_request),
            Request::Opendir(opendir_request) => self.handle_opendir_request(opendir_request).await,
            Request::Readdir(readdir_request) => self.handle_readdir_request(readdir_request).await,
            Request::Remove(remove_request) => self.handle_remove_request(remove_request).await,
            Request::Mkdir(mkdir_request) => self.handle_mkdir_request(mkdir_request).await,
            Request::Rmdir(rmdir_request) => self.handle_rmdir_request(rmdir_request).await,
            Request::Realpath(realpath_request) => self.handle_realpath_request(realpath_request),
            Request::Stat(stat_request) => self.handle_stat_request(stat_request).await,
            Request::Rename(rename_request) => self.handle_rename_request(rename_request).await,
            Request::Readlink(readlink_request) => self.handle_readlink_request(readlink_request),
            Request::Symlink(symlink_request) => self.handle_symlink_request(symlink_request),
        };

        match response {
            Ok(response) => response,
            Err(error) => {
                error!("Received error while processing request: {}", error);
                Response::build_error_response(request_id, error)
            }
        }
    }

    fn handle_init_request(&self, _init_request: request::init::Init) -> Result<Response, Error> {
        Ok(Response::Version(response::version::Version { version: 3 }))
    }

    #[tracing::instrument(skip(self))]
    async fn handle_open_request(
        &self,
        open_request: request::open::Open,
    ) -> Result<Response, Error> {
        self.check_permission(&open_request.filename)?;

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

    #[tracing::instrument(skip(self))]
    async fn handle_close_request(
        &self,
        close_request: request::handle::Handle,
    ) -> Result<Response, Error> {
        self.object_storage
            .close_handle(&close_request.handle)
            .await?;

        Ok(SftpSession::build_successful_response(close_request.id))
    }

    #[tracing::instrument(skip(self), level = Level::DEBUG)]
    async fn handle_read_request(
        &self,
        read_request: request::read::Read,
    ) -> Result<Response, Error> {
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

    #[tracing::instrument(skip(self), level = Level::DEBUG)]
    async fn handle_write_request(
        &self,
        write_request: request::write::Write,
    ) -> Result<Response, Error> {
        self.object_storage
            .write_data(&write_request.handle, write_request.data)
            .await?;

        Ok(SftpSession::build_successful_response(write_request.id))
    }

    #[tracing::instrument(skip(self))]
    async fn handle_lstat_request(
        &self,
        lstat_request: request::path::Path,
    ) -> Result<Response, Error> {
        self.check_permission(&lstat_request.path)?;

        // Object storage does not have symbolic links, so stat and lstat will operate the same
        // because there are no symbolic links to follow.
        self.handle_stat_request(lstat_request).await
    }

    #[tracing::instrument(skip(self))]
    async fn handle_fstat_request(
        &self,
        fstat_request: request::handle::Handle,
    ) -> Result<Response, Error> {
        let file_attributes = self
            .object_storage
            .get_handle_metadata(&fstat_request.handle)
            .await?
            .file_attributes;

        Ok(Response::Attrs(response::attrs::Attrs {
            id: fstat_request.id,
            file_attributes,
        }))
    }

    #[tracing::instrument(skip(self))]
    fn handle_setstat_request(
        &self,
        setstat_request: request::path_attributes::PathAttributes,
    ) -> Result<Response, Error> {
        Ok(SftpSession::build_not_supported_response(
            setstat_request.id,
        ))
    }

    #[tracing::instrument(skip(self))]
    fn handle_fsetstat_request(
        &self,
        fsetstat_request: request::handle_attributes::HandleAttributes,
    ) -> Result<Response, Error> {
        Ok(SftpSession::build_not_supported_response(
            fsetstat_request.id,
        ))
    }

    #[tracing::instrument(skip(self))]
    async fn handle_opendir_request(
        &self,
        opendir_request: request::path::Path,
    ) -> Result<Response, Error> {
        self.check_permission(&opendir_request.path)?;

        let handle = self
            .object_storage
            .open_dir_handle(opendir_request.path)
            .await?;

        Ok(Response::Handle(response::handle::Handle {
            id: opendir_request.id,
            handle,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn handle_readdir_request(
        &self,
        readdir_request: request::handle::Handle,
    ) -> Result<Response, Error> {
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

    #[tracing::instrument(skip(self))]
    async fn handle_remove_request(
        &self,
        remove_request: request::path::Path,
    ) -> Result<Response, Error> {
        self.check_permission(&remove_request.path)?;

        self.object_storage.remove_file(remove_request.path).await?;

        Ok(SftpSession::build_successful_response(remove_request.id))
    }

    #[tracing::instrument(skip(self))]
    async fn handle_mkdir_request(
        &self,
        mkdir_request: request::path_attributes::PathAttributes,
    ) -> Result<Response, Error> {
        self.check_permission(&mkdir_request.path)?;

        self.object_storage.create_dir(mkdir_request.path).await?;

        Ok(SftpSession::build_successful_response(mkdir_request.id))
    }

    #[tracing::instrument(skip(self))]
    async fn handle_rmdir_request(
        &self,
        rmdir_request: request::path::Path,
    ) -> Result<Response, Error> {
        self.check_permission(&rmdir_request.path)?;

        self.object_storage.remove_dir(rmdir_request.path).await?;

        Ok(SftpSession::build_successful_response(rmdir_request.id))
    }

    #[tracing::instrument(skip(self))]
    fn handle_realpath_request(
        &self,
        realpath_request: request::path::Path,
    ) -> Result<Response, Error> {
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

    #[tracing::instrument(skip(self))]
    async fn handle_stat_request(
        &self,
        stat_request: request::path::Path,
    ) -> Result<Response, Error> {
        self.check_permission(&stat_request.path)?;

        let file_attributes = self
            .object_storage
            .get_file_metadata(stat_request.path.clone())
            .await?
            .file_attributes;

        Ok(Response::Attrs(response::attrs::Attrs {
            id: stat_request.id,
            file_attributes,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn handle_rename_request(
        &self,
        rename_request: request::rename::Rename,
    ) -> Result<Response, Error> {
        self.check_permission(&rename_request.new_path)?;
        self.check_permission(&rename_request.old_path)?;

        self.object_storage
            .rename(rename_request.old_path, rename_request.new_path)
            .await?;

        Ok(SftpSession::build_successful_response(rename_request.id))
    }

    #[tracing::instrument(skip(self))]
    fn handle_readlink_request(
        &self,
        readlink_request: request::path::Path,
    ) -> Result<Response, Error> {
        Ok(SftpSession::build_not_supported_response(
            readlink_request.id,
        ))
    }

    #[tracing::instrument(skip(self))]
    fn handle_symlink_request(
        &self,
        symlink_request: request::symlink::Symlink,
    ) -> Result<Response, Error> {
        Ok(SftpSession::build_not_supported_response(
            symlink_request.id,
        ))
    }

    #[tracing::instrument]
    fn build_successful_response(id: u32) -> Response {
        Response::Status(response::status::Status {
            id,
            status_code: response::status::StatusCode::Ok,
            error_message: String::from(""),
        })
    }

    #[tracing::instrument]
    pub fn build_invalid_request_message_response() -> Response {
        Response::Status(response::status::Status {
            id: 0,
            status_code: response::status::StatusCode::BadMessage,
            error_message: String::from("The request message is invalid."),
        })
    }

    #[tracing::instrument]
    fn build_not_supported_response(id: u32) -> Response {
        Response::Status(response::status::Status {
            id,
            status_code: response::status::StatusCode::OperationUnsupported,
            error_message: String::from("Operation Unsupported!"),
        })
    }

    fn check_permission(&self, path: &str) -> Result<(), Error> {
        match path.starts_with(&self.user_home) {
            true => Ok(()),
            false => Err(Error::PermissionDenied),
        }
    }
}
