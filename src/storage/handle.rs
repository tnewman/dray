use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct HandleManager<ReadHandle: Send + Sync, WriteHandle: Send + Sync, DirHandle: Send + Sync>
{
    read_handles: RwLock<HashMap<String, Arc<ReadHandle>>>,
    write_handles: RwLock<HashMap<String, Arc<WriteHandle>>>,
    dir_handles: RwLock<HashMap<String, Arc<DirHandle>>>,
}

impl<ReadHandle: Send + Sync, WriteHandle: Send + Sync, DirHandle: Send + Sync>
    HandleManager<ReadHandle, WriteHandle, DirHandle>
{
    pub fn new() -> HandleManager<ReadHandle, WriteHandle, DirHandle> {
        HandleManager {
            read_handles: RwLock::new(HashMap::new()),
            write_handles: RwLock::new(HashMap::new()),
            dir_handles: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_dir_handle(&mut self, dir_handle: DirHandle) -> String {
        let handle_id = generate_handle_id();

        self.dir_handles
            .write()
            .await
            .insert(handle_id.clone(), Arc::from(dir_handle));

        handle_id
    }

    pub async fn create_read_handle(&mut self, read_handle: ReadHandle) -> String {
        let handle_id = generate_handle_id();

        self.read_handles
            .write()
            .await
            .insert(handle_id.clone(), Arc::from(read_handle));

        handle_id
    }

    pub async fn create_write_handle(&mut self, write_handle: WriteHandle) -> String {
        let handle_id = generate_handle_id();

        self.write_handles
            .write()
            .await
            .insert(handle_id.clone(), Arc::from(write_handle));

        handle_id
    }

    pub async fn get_dir_handle(&mut self, handle_id: &str) -> Option<Arc<DirHandle>> {
        self.dir_handles
            .read()
            .await
            .get(handle_id)
            .map(|dir_handle| dir_handle.clone())
    }

    pub async fn get_read_handle(&mut self, handle_id: &str) -> Option<Arc<ReadHandle>> {
        self.read_handles
            .read()
            .await
            .get(handle_id)
            .map(|read_handle| read_handle.clone())
    }

    pub async fn get_write_handle(&mut self, handle_id: &str) -> Option<Arc<WriteHandle>> {
        self.write_handles
            .write()
            .await
            .get(handle_id)
            .map(|write_handle| write_handle.clone())
    }

    pub async fn remove_handle(&mut self, handle: &str) {
        self.dir_handles.write().await.remove(handle);
        self.read_handles.write().await.remove(handle);
        self.write_handles.write().await.remove(handle);
    }
}

pub trait Handle {
    fn get_handle_id(&self) -> &str;

    fn get_handle_id_string(&self) -> String {
        self.get_handle_id().to_string()
    }
}

pub struct DirHandle {
    prefix: String,
    continuation_token: Option<String>,
    eof: bool,
}

impl DirHandle {
    pub fn new(prefix: String, continuation_token: Option<String>, eof: bool) -> DirHandle {
        DirHandle {
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

fn generate_handle_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_handle_manager_dir_handle_create_get() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager.create_dir_handle(String::from("dir")).await;

        assert!(handle_manager.get_dir_handle(&handle_id).await.is_some())
    }

    #[tokio::test]
    async fn test_handle_manager_dir_handle_delete() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager.create_dir_handle(String::from("dir")).await;
        assert!(handle_manager.get_dir_handle(&handle_id).await.is_some());

        handle_manager.remove_handle(&handle_id).await;
        assert!(handle_manager.get_dir_handle(&handle_id).await.is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_get_missing_dir_handle() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        assert!(handle_manager
            .get_dir_handle("missing_handle")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_read_handle_create_get() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager
            .create_read_handle(String::from("read"))
            .await;

        let handle = handle_manager.get_read_handle(&handle_id).await.unwrap();

        assert_eq!(&"read", &*handle);
    }

    #[tokio::test]
    async fn test_handle_manager_read_handle_delete() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager
            .create_read_handle(String::from("read"))
            .await;
        assert!(handle_manager.get_read_handle(&handle_id).await.is_some());

        handle_manager.remove_handle(&handle_id).await;
        assert!(handle_manager.get_read_handle(&handle_id).await.is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_get_missing_read_handle() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        assert!(handle_manager
            .get_read_handle("missing_handle")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_write_handle_create_get() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager
            .create_write_handle(String::from("write"))
            .await;

        let handle = handle_manager.get_write_handle(&handle_id).await.unwrap();

        assert_eq!("write", &*handle)
    }

    #[tokio::test]
    async fn test_handle_manager_write_handle_delete() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager
            .create_write_handle(String::from("write"))
            .await;
        assert!(handle_manager.get_write_handle(&handle_id).await.is_some());

        handle_manager.remove_handle(&handle_id).await;
        assert!(handle_manager.get_write_handle(&handle_id).await.is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_get_missing_write_handle() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        assert!(handle_manager
            .get_write_handle("missing_handle")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_remove_missing_handle() {
        let mut handle_manager: HandleManager<String, String, String> = HandleManager::new();

        handle_manager.remove_handle("missing_handle").await;

        assert!(handle_manager
            .get_write_handle("missing_handle")
            .await
            .is_none());
    }

    #[test]
    fn test_generate_handle_id_creates_uuid() {
        let handle = generate_handle_id();

        assert_eq!(true, handle.len() > 0);
    }
}
