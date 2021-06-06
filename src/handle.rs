use std::collections::HashMap;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite};
use uuid::Uuid;

pub struct HandleManager {
    read_handles: HashMap<String, ReadHandle>,
    write_handles: HashMap<String, WriteHandle>,
    dir_handles: HashMap<String, DirHandle>,
}

impl HandleManager {
    pub fn new() -> HandleManager {
        HandleManager {
            read_handles: HashMap::new(),
            write_handles: HashMap::new(),
            dir_handles: HashMap::new(),
        }
    }

    pub fn create_dir_handle(&mut self, continuation_token: Option<String>) -> String {
        let dir_handle = DirHandle::new(continuation_token);
        let handle_id = dir_handle.get_handle_id_string();

        self.dir_handles
            .insert(dir_handle.get_handle_id().to_string(), dir_handle);

        handle_id
    }

    pub fn create_read_handle(&mut self, async_read: Pin<Box<dyn AsyncRead>>) -> String {
        let read_handle = ReadHandle::new(async_read);
        let handle_id = read_handle.get_handle_id_string();

        self.read_handles
            .insert(read_handle.get_handle_id().to_string(), read_handle);

        handle_id
    }

    pub fn create_write_handle(&mut self, async_write: Pin<Box<dyn AsyncWrite>>) -> String {
        let write_handle = WriteHandle::new(async_write);
        let handle_id = write_handle.get_handle_id_string();

        self.write_handles
            .insert(write_handle.get_handle_id().to_string(), write_handle);

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

trait Handle {
    fn get_handle_id(&self) -> &str;

    fn get_handle_id_string(&self) -> String {
        self.get_handle_id().to_string()
    }
}

pub struct DirHandle {
    id: String,
    continuation_token: Option<String>,
}

impl DirHandle {
    pub fn new(continuation_token: Option<String>) -> DirHandle {
        DirHandle {
            id: generate_handle_id(),
            continuation_token,
        }
    }

    pub fn get_continuation_token(&self) -> Option<&str> {
        match &self.continuation_token {
            Some(token) => Option::Some(token),
            None => None,
        }
    }
}

impl Handle for DirHandle {
    fn get_handle_id(&self) -> &str {
        &self.id
    }
}

pub struct ReadHandle {
    id: String,
    async_read: Pin<Box<dyn AsyncRead>>,
}

impl ReadHandle {
    pub fn new(async_read: Pin<Box<dyn AsyncRead>>) -> ReadHandle {
        ReadHandle {
            id: generate_handle_id(),
            async_read,
        }
    }

    pub fn get_async_read(&mut self) -> &mut Pin<Box<dyn AsyncRead>> {
        &mut self.async_read
    }
}

impl Handle for ReadHandle {
    fn get_handle_id(&self) -> &str {
        &self.id
    }
}

pub struct WriteHandle {
    id: String,
    async_write: Pin<Box<dyn AsyncWrite>>,
}

impl WriteHandle {
    pub fn new(async_write: Pin<Box<dyn AsyncWrite>>) -> WriteHandle {
        WriteHandle {
            id: generate_handle_id(),
            async_write,
        }
    }

    pub fn get_async_write(&mut self) -> &mut Pin<Box<dyn AsyncWrite>> {
        &mut self.async_write
    }
}

impl Handle for WriteHandle {
    fn get_handle_id(&self) -> &str {
        &self.id
    }
}

fn generate_handle_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod test {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    #[tokio::test]
    async fn test_handle_manager_dir_handle_create_get() {
        let mut handle_manager = HandleManager::new();

        let handle_id = handle_manager.create_dir_handle(Option::Some(String::from("token")));

        assert_eq!(
            handle_id,
            handle_manager
                .get_dir_handle(&handle_id)
                .unwrap()
                .get_handle_id_string()
        );
    }

    #[tokio::test]
    async fn test_handle_manager_dir_handle_delete() {
        let mut handle_manager = HandleManager::new();

        let handle_id = handle_manager.create_dir_handle(Option::Some(String::from("token")));
        assert!(handle_manager.get_dir_handle(&handle_id).is_some());

        handle_manager.remove_handle(&handle_id);
        assert!(handle_manager.get_dir_handle(&handle_id).is_none());
    }

    #[test]
    fn test_handle_manager_get_missing_dir_handle() {
        let mut handle_manager = HandleManager::new();

        assert!(handle_manager.get_dir_handle("missing_handle").is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_read_handle_create_get() {
        let mut handle_manager = HandleManager::new();
        let (client, _server) = tokio::io::duplex(1);

        let handle_id = handle_manager.create_read_handle(Pin::from(Box::new(client)));

        assert_eq!(
            handle_id,
            handle_manager
                .get_read_handle(&handle_id)
                .unwrap()
                .get_handle_id_string()
        );
    }

    #[tokio::test]
    async fn test_handle_manager_read_handle_delete() {
        let mut handle_manager = HandleManager::new();
        let (client, _server) = tokio::io::duplex(1);

        let handle_id = handle_manager.create_read_handle(Pin::from(Box::new(client)));
        assert!(handle_manager.get_read_handle(&handle_id).is_some());

        handle_manager.remove_handle(&handle_id);
        assert!(handle_manager.get_read_handle(&handle_id).is_none());
    }

    #[test]
    fn test_handle_manager_get_missing_read_handle() {
        let mut handle_manager = HandleManager::new();

        assert!(handle_manager.get_read_handle("missing_handle").is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_write_handle_create_get() {
        let mut handle_manager = HandleManager::new();
        let (client, _server) = tokio::io::duplex(1);

        let handle_id = handle_manager.create_write_handle(Pin::from(Box::new(client)));

        assert_eq!(
            handle_id,
            handle_manager
                .get_write_handle(&handle_id)
                .unwrap()
                .get_handle_id_string()
        );
    }

    #[tokio::test]
    async fn test_handle_manager_write_handle_delete() {
        let mut handle_manager = HandleManager::new();
        let (client, _server) = tokio::io::duplex(1);

        let handle_id = handle_manager.create_write_handle(Pin::from(Box::new(client)));
        assert!(handle_manager.get_write_handle(&handle_id).is_some());

        handle_manager.remove_handle(&handle_id);
        assert!(handle_manager.get_write_handle(&handle_id).is_none());
    }

    #[test]
    fn test_handle_manager_get_missing_write_handle() {
        let mut handle_manager = HandleManager::new();

        assert!(handle_manager.get_write_handle("missing_handle").is_none());
    }

    #[test]
    fn test_handle_manager_remove_missing_handle() {
        let mut handle_manager = HandleManager::new();

        handle_manager.remove_handle("missing_handle");
    }

    #[tokio::test]
    async fn test_new_read_handle_creates_read_handle() {
        let (client, mut server) = tokio::io::duplex(1);
        server.write_u8(0x01).await.unwrap();

        let mut read_handle = ReadHandle::new(Pin::from(Box::new(client)));

        let mut buf: [u8; 1] = [0x00];
        read_handle
            .get_async_read()
            .read_exact(&mut buf)
            .await
            .unwrap();

        assert_ne!(0, read_handle.get_handle_id().len());
        assert_eq!([0x01], buf);
    }

    #[tokio::test]
    async fn test_new_write_handle_creates_write_handle() {
        let (client, mut server) = tokio::io::duplex(1);

        let mut write_handle = WriteHandle::new(Pin::from(Box::new(client)));

        write_handle.get_async_write().write(&[0x01]).await.unwrap();

        let mut buf: [u8; 1] = [0x00];
        server.read_exact(&mut buf).await.unwrap();

        assert_ne!(0, write_handle.get_handle_id().len());
        assert_eq!([0x01], buf);
    }

    #[test]
    fn test_new_dir_handle_creates_dir_handle() {
        let dir_handle = DirHandle::new(Option::Some(String::from("token")));

        assert_ne!(0, dir_handle.get_handle_id().len());
        assert_eq!("token", dir_handle.get_continuation_token().unwrap());
    }

    #[test]
    fn test_new_dir_handle_creates_dir_handle_without_continuation_token() {
        let dir_handle = DirHandle::new(Option::None);

        assert_ne!(0, dir_handle.get_handle_id().len());
        assert_eq!(true, dir_handle.get_continuation_token().is_none());
    }

    #[test]
    fn test_generate_handle_id_creates_uuid() {
        let handle = generate_handle_id();

        assert_eq!(true, handle.len() > 0);
    }
}
