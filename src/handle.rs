use std::{collections::HashMap, pin::Pin, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use uuid::Uuid;

pub struct HandleManager {
    read_handles: HashMap<String, Arc<Mutex<Pin<Box<dyn AsyncRead + Send + Sync>>>>>,
    write_handles: HashMap<String, Arc<Mutex<Pin<Box<dyn AsyncWrite + Send + Sync>>>>>,
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

    pub fn create_dir_handle(
        &mut self,
        id: Option<String>,
        prefix: String,
        continuation_token: Option<String>,
        eof: bool,
    ) -> String {
        let dir_handle = DirHandle::new(id, prefix, continuation_token, eof);
        let handle_id = dir_handle.get_handle_id_string();

        self.dir_handles
            .insert(dir_handle.get_handle_id().to_string(), dir_handle);

        handle_id
    }

    pub fn create_read_handle(
        &mut self,
        read_stream: Arc<Mutex<Pin<Box<dyn AsyncRead + Send + Sync>>>>,
    ) -> String {
        let handle_id = generate_handle_id();

        self.read_handles.insert(handle_id.clone(), read_stream);

        handle_id
    }

    pub fn create_write_handle(
        &mut self,
        write_stream: Arc<Mutex<Pin<Box<dyn AsyncWrite + Send + Sync>>>>,
    ) -> String {
        let handle_id = generate_handle_id();

        self.write_handles.insert(handle_id.clone(), write_stream);

        handle_id
    }

    pub fn get_dir_handle(&mut self, handle_id: &str) -> Option<&mut DirHandle> {
        self.dir_handles.get_mut(handle_id)
    }

    pub fn get_read_handle(
        &mut self,
        handle_id: &str,
    ) -> Option<Arc<Mutex<Pin<Box<dyn AsyncRead + Send + Sync>>>>> {
        self.read_handles
            .get(handle_id)
            .map(|read_handle| read_handle.clone())
    }

    pub fn get_write_handle(
        &mut self,
        handle_id: &str,
    ) -> Option<Arc<Mutex<Pin<Box<dyn AsyncWrite + Send + Sync>>>>> {
        self.write_handles
            .get(handle_id)
            .map(|write_handle| write_handle.clone())
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
        let mut handle_manager = HandleManager::new();

        let handle_id = handle_manager.create_dir_handle(
            Option::None,
            String::from("prefix"),
            Option::Some(String::from("token")),
            true,
        );
        let handle = handle_manager.get_dir_handle(&handle_id).unwrap();

        assert_eq!(handle_id, handle.get_handle_id_string());
        assert_eq!("prefix", handle.get_prefix());
        assert_eq!("token", handle.get_continuation_token().unwrap());
        assert_eq!(true, handle.is_eof());
    }

    #[test]
    fn test_handle_manager_dir_handle_create_preserves_provided_handle_id() {
        let mut handle_manager = HandleManager::new();

        let handle_id = handle_manager.create_dir_handle(
            Option::Some(String::from("handle")),
            String::from("prefix"),
            Option::Some(String::from("token")),
            true,
        );
        let handle = handle_manager.get_dir_handle(&handle_id).unwrap();

        assert_eq!(handle_id, handle.get_handle_id_string());
        assert_eq!(handle_id, String::from("handle"));
    }

    #[test]
    fn test_handle_manager_dir_handle_delete() {
        let mut handle_manager = HandleManager::new();

        let handle_id = handle_manager.create_dir_handle(
            Option::None,
            String::from("prefix"),
            Option::Some(String::from("token")),
            true,
        );
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

        let read_stream = Cursor::new(vec![0x01, 0x02]);
        let handle_id =
            handle_manager.create_read_handle(Arc::new(Mutex::new(Box::pin(read_stream))));

        let handle = handle_manager.get_read_handle(&handle_id).unwrap();

        let mut buffer = vec![];
        handle.lock().await.read_to_end(&mut buffer).await.unwrap();

        assert_eq!(vec![0x01, 0x02], buffer);
    }

    #[test]
    fn test_handle_manager_read_handle_delete() {
        let mut handle_manager = HandleManager::new();

        let read_stream = Cursor::new(vec![0x01, 0x02]);
        let handle_id =
            handle_manager.create_read_handle(Arc::new(Mutex::new(Box::pin(read_stream))));
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

        let mut write_stream = Cursor::new(vec![0x01, 0x02]);
        let handle_id = handle_manager
            .create_write_handle(Arc::new(Mutex::new(Box::pin(write_stream.clone()))));
        let handle = handle_manager.get_write_handle(&handle_id).unwrap();

        handle.lock().await.write_u8(0x01).await.unwrap();
        handle.lock().await.write_u8(0x02).await.unwrap();

        let mut buffer = vec![];
        write_stream.read_to_end(&mut buffer).await.unwrap();
        assert_eq!(vec![0x01, 0x02], buffer);
    }

    #[test]
    fn test_handle_manager_write_handle_delete() {
        let mut handle_manager = HandleManager::new();

        let write_stream = Cursor::new(vec![0x01, 0x02]);
        let handle_id =
            handle_manager.create_write_handle(Arc::new(Mutex::new(Box::pin(write_stream))));
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

        assert!(handle_manager.get_write_handle("missing_handle").is_none());
    }

    #[test]
    fn test_generate_handle_id_creates_uuid() {
        let handle = generate_handle_id();

        assert_eq!(true, handle.len() > 0);
    }
}
