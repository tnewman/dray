use crate::storage::ObjectStorage;
use crate::{
    handle::Handle,
    handle::HandleManager,
    protocol::{
        file_attributes::FileAttributes,
        request::{self, Request},
        response::{self, Response},
    },
};
use anyhow::Result;
use log::error;
use log::info;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

pub struct SftpSession {
    object_storage: Arc<dyn ObjectStorage>,
    handle_manager: Arc<Mutex<HandleManager>>,
    user: String,
}

impl SftpSession {
    pub fn new(object_storage: Arc<dyn ObjectStorage>, user: String) -> Self {
        SftpSession {
            object_storage,
            handle_manager: Arc::new(Mutex::new(HandleManager::new())),
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
            let write_stream = self
                .object_storage
                .write_object(open_request.filename)
                .await?;
            self.handle_manager
                .lock()
                .await
                .create_write_handle(write_stream)
        } else if open_request.open_options.read {
            let read_stream = self
                .object_storage
                .read_object(open_request.filename)
                .await?;
            self.handle_manager
                .lock()
                .await
                .create_read_handle(read_stream)
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
        self.handle_manager
            .lock()
            .await
            .remove_handle(&close_request.handle);

        Ok(Response::Status(response::status::Status {
            id: close_request.id,
            status_code: response::status::StatusCode::Ok,
            error_message: String::from("Successfully closed handle."),
        }))
    }

    async fn handle_read_request(&self, read_request: request::read::Read) -> Result<Response> {
        let read_handle = {
            self.handle_manager
                .lock()
                .await
                .get_read_handle(&read_request.handle)
        };

        let read_handle = match read_handle {
            Some(read_handle) => read_handle,
            None => {
                return Ok(Response::Status(response::status::Status {
                    id: read_request.id,
                    status_code: response::status::StatusCode::Failure,
                    error_message: String::from("Invalid handle."),
                }))
            }
        };

        let mut buffer = Vec::with_capacity(read_request.len as usize);
        read_handle
            .lock()
            .await
            .as_mut()
            .take(read_request.len as u64)
            .read_to_end(&mut buffer)
            .await?;

        Ok(Response::Data(response::data::Data {
            id: read_request.id,
            data: buffer.to_vec(),
        }))
    }

    async fn handle_write_request(
        &self,
        mut write_request: request::write::Write,
    ) -> Result<Response> {
        let write_handle = {
            self.handle_manager
                .lock()
                .await
                .get_write_handle(&write_request.handle)
        };

        let write_handle = match write_handle {
            Some(write_handle) => write_handle,
            None => {
                return Ok(Response::Status(response::status::Status {
                    id: write_request.id,
                    status_code: response::status::StatusCode::Failure,
                    error_message: String::from("Invalid handle."),
                }))
            }
        };

        write_handle
            .lock()
            .await
            .write_all_buf(&mut write_request.data)
            .await?;

        Ok(Response::Status(response::status::Status {
            id: write_request.id,
            status_code: response::status::StatusCode::Ok,
            error_message: String::from("Successfully wrote data to file."),
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
        let handle = self.handle_manager.lock().await.create_dir_handle(
            None,
            opendir_request.path,
            None,
            false,
        );

        Ok(Response::Handle(response::handle::Handle {
            id: opendir_request.id,
            handle,
        }))
    }

    async fn handle_readdir_request(
        &self,
        readdir_request: request::handle::Handle,
    ) -> Result<Response> {
        let (handle_id, prefix, continuation_token, eof) = {
            match self
                .handle_manager
                .lock()
                .await
                .get_dir_handle(&readdir_request.handle)
            {
                Some(handle) => (
                    handle.get_handle_id().to_string(),
                    handle.get_prefix().to_string(),
                    handle.get_continuation_token().map(|str| str.to_string()),
                    handle.is_eof(),
                ),
                None => {
                    return Ok(Response::Status(response::status::Status {
                        id: readdir_request.id,
                        status_code: response::status::StatusCode::BadMessage,
                        error_message: String::from("File not found."),
                    }));
                }
            }
        };

        if eof {
            return Ok(Response::Status(response::status::Status {
                id: readdir_request.id,
                status_code: response::status::StatusCode::Eof,
                error_message: String::from("No more files available to list."),
            }));
        }

        let result = self
            .object_storage
            .list_prefix(prefix.clone(), continuation_token, Option::None)
            .await?;

        let eof = result.continuation_token.is_none();

        self.handle_manager.lock().await.create_dir_handle(
            Some(handle_id),
            prefix,
            result.continuation_token,
            eof,
        );

        Ok(Response::Name(response::name::Name {
            id: readdir_request.id,
            files: result.objects,
        }))
    }

    fn handle_remove_request(&self, remove_request: request::path::Path) -> Result<Response> {
        Ok(SftpSession::build_not_supported_response(remove_request.id))
    }

    async fn handle_mkdir_request(
        &self,
        mkdir_request: request::path_attributes::PathAttributes,
    ) -> Result<Response> {
        self.object_storage
            .create_prefix(mkdir_request.path)
            .await?;

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
                file_name: path.clone(),
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
            .object_exists(stat_request.path.clone())
            .await?
        {
            true => {
                self.object_storage
                    .get_object_metadata(stat_request.path)
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
