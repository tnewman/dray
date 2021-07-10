use std::{collections::HashMap, pin::Pin, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use uuid::Uuid;

pub struct HandleManager<ReadHandle, WriteHandle, DirHandle> {
    read_handles: HashMap<String, ReadHandle>,
    write_handles: HashMap<String, WriteHandle>,
    dir_handles: HashMap<String, DirHandle>,
}

impl<ReadHandle, WriteHandle, DirHandle> HandleManager<ReadHandle, WriteHandle, DirHandle> {
    pub fn new() -> HandleManager<ReadHandle, WriteHandle, DirHandle> {
        HandleManager {
            read_handles: HashMap::new(),
            write_handles: HashMap::new(),
            dir_handles: HashMap::new(),
        }
    }

    pub fn create_dir_handle(&mut self, dir_handle: DirHandle) -> String {
        let handle_id = generate_handle_id();

        self.dir_handles.insert(handle_id.clone(), dir_handle);

        handle_id
    }

    pub fn create_read_handle(&mut self, read_handle: ReadHandle) -> String {
        let handle_id = generate_handle_id();

        self.read_handles.insert(handle_id.clone(), read_handle);

        handle_id
    }

    pub fn create_write_handle(&mut self, write_handle: WriteHandle) -> String {
        let handle_id = generate_handle_id();

        self.write_handles.insert(handle_id.clone(), write_handle);

        handle_id
    }

    pub fn get_dir_handle(&mut self, handle_id: &str) -> Option<&mut DirHandle> {
        self.dir_handles.get_mut(handle_id)
    }

    pub fn get_read_handle(&mut self, handle_id: &str) -> Option<&mut ReadHandle> {
        self.read_handles.get_mut(handle_id)
    }

    pub fn get_write_handle(&mut self, handle_id: &str) -> Option<&mut WriteHandle> {
        self.write_handles.get_mut(handle_id)
    }

    pub fn remove_handle(&mut self, handle: &str) {
        self.dir_handles.remove(handle);
        self.read_handles.remove(handle);
        self.write_handles.remove(handle);
    }
}

pub trait Handle {
    fn get_handle_id(&self) -> &str;

    fn get_handle_id_string(&self) -> String {
        self.get_handle_id().to_string()
    }
}

pub struct DirHandle {
    id: String,
    prefix: String,
    continuation_token: Option<String>,
    eof: bool,
}

impl DirHandle {
    pub fn new(
        id: Option<String>,
        prefix: String,
        continuation_token: Option<String>,
        eof: bool,
    ) -> DirHandle {
        DirHandle {
            id: id.unwrap_or_else(generate_handle_id),
            prefix,
            continuation_token,
            eof,
        }
    }

    pub fn get_prefix(&self) -> &str {
        &self.prefix
    }

    pub fn get_continuation_token(&self) -> Option<&str> {
        match &self.continuation_token {
            Some(token) => Option::Some(token),
            None => None,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.eof
    }
}

impl Handle for DirHandle {
    fn get_handle_id(&self) -> &str {
        &self.id
    }
}

fn generate_handle_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_handle_manager_dir_handle_create_get() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager.create_dir_handle(String::from("dir"));

        assert!(handle_manager.get_dir_handle(&handle_id).is_some())
    }

    #[test]
    fn test_handle_manager_dir_handle_delete() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager.create_dir_handle(String::from("dir"));
        assert!(handle_manager.get_dir_handle(&handle_id).is_some());

        handle_manager.remove_handle(&handle_id);
        assert!(handle_manager.get_dir_handle(&handle_id).is_none());
    }

    #[test]
    fn test_handle_manager_get_missing_dir_handle() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        assert!(handle_manager.get_dir_handle("missing_handle").is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_read_handle_create_get() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager.create_read_handle(String::from("read"));

        let handle = handle_manager.get_read_handle(&handle_id).unwrap();

        assert_eq!(&mut "read", &handle);
    }

    #[test]
    fn test_handle_manager_read_handle_delete() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager.create_read_handle(String::from("read"));
        assert!(handle_manager.get_read_handle(&handle_id).is_some());

        handle_manager.remove_handle(&handle_id);
        assert!(handle_manager.get_read_handle(&handle_id).is_none());
    }

    #[test]
    fn test_handle_manager_get_missing_read_handle() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        assert!(handle_manager.get_read_handle("missing_handle").is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_write_handle_create_get() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager.create_write_handle(String::from("write"));

        let handle = handle_manager.get_write_handle(&handle_id).unwrap();

        assert_eq!(&mut "write", &handle)
    }

    #[test]
    fn test_handle_manager_write_handle_delete() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager.create_write_handle(String::from("write"));
        assert!(handle_manager.get_write_handle(&handle_id).is_some());

        handle_manager.remove_handle(&handle_id);
        assert!(handle_manager.get_write_handle(&handle_id).is_none());
    }

    #[test]
    fn test_handle_manager_get_missing_write_handle() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        assert!(handle_manager.get_write_handle("missing_handle").is_none());
    }

    #[test]
    fn test_handle_manager_remove_missing_handle() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        handle_manager.remove_handle("missing_handle");

        assert!(handle_manager.get_write_handle("missing_handle").is_none());
    }

    #[test]
    fn test_generate_handle_id_creates_uuid() {
        let handle = generate_handle_id();

        assert_eq!(true, handle.len() > 0);
    }
}
