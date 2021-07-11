use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

pub struct HandleManager<ReadHandle, WriteHandle, DirHandle> {
    read_handles: RwLock<HashMap<String, Arc<Mutex<ReadHandle>>>>,
    write_handles: RwLock<HashMap<String, Arc<Mutex<WriteHandle>>>>,
    dir_handles: RwLock<HashMap<String, Arc<Mutex<DirHandle>>>>,
}

impl<ReadHandle, WriteHandle, DirHandle> HandleManager<ReadHandle, WriteHandle, DirHandle> {
    pub fn new() -> HandleManager<ReadHandle, WriteHandle, DirHandle> {
        HandleManager {
            read_handles: RwLock::new(HashMap::new()),
            write_handles: RwLock::new(HashMap::new()),
            dir_handles: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_dir_handle(&self, dir_handle: DirHandle) -> String {
        let handle_id = generate_handle_id();

        self.dir_handles
            .write()
            .await
            .insert(handle_id.clone(), Arc::from(Mutex::from(dir_handle)));

        handle_id
    }

    pub async fn create_read_handle(&self, read_handle: ReadHandle) -> String {
        let handle_id = generate_handle_id();

        self.read_handles
            .write()
            .await
            .insert(handle_id.clone(), Arc::from(Mutex::from(read_handle)));

        handle_id
    }

    pub async fn create_write_handle(&self, write_handle: WriteHandle) -> String {
        let handle_id = generate_handle_id();

        self.write_handles
            .write()
            .await
            .insert(handle_id.clone(), Arc::from(Mutex::from(write_handle)));

        handle_id
    }

    pub async fn get_dir_handle(&self, handle_id: &str) -> Option<Arc<Mutex<DirHandle>>> {
        self.dir_handles.read().await.get(handle_id).cloned()
    }

    pub async fn get_read_handle(&self, handle_id: &str) -> Option<Arc<Mutex<ReadHandle>>> {
        self.read_handles.read().await.get(handle_id).cloned()
    }

    pub async fn get_write_handle(&self, handle_id: &str) -> Option<Arc<Mutex<WriteHandle>>> {
        self.write_handles.write().await.get(handle_id).cloned()
    }

    pub async fn remove_handle(&self, handle: &str) {
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

fn generate_handle_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_handle_manager_dir_handle_create_get() {
        let handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager.create_dir_handle(String::from("dir")).await;

        assert!(handle_manager.get_dir_handle(&handle_id).await.is_some())
    }

    #[tokio::test]
    async fn test_handle_manager_dir_handle_delete() {
        let handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager.create_dir_handle(String::from("dir")).await;
        assert!(handle_manager.get_dir_handle(&handle_id).await.is_some());

        handle_manager.remove_handle(&handle_id).await;
        assert!(handle_manager.get_dir_handle(&handle_id).await.is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_get_missing_dir_handle() {
        let handle_manager: HandleManager<String, String, String> = HandleManager::new();

        assert!(handle_manager
            .get_dir_handle("missing_handle")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_read_handle_create_get() {
        let handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager
            .create_read_handle(String::from("read"))
            .await;

        let handle = handle_manager.get_read_handle(&handle_id).await.unwrap();
        let handle = handle.lock().await;

        assert_eq!("read", *handle);
    }

    #[tokio::test]
    async fn test_handle_manager_read_handle_delete() {
        let handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager
            .create_read_handle(String::from("read"))
            .await;
        assert!(handle_manager.get_read_handle(&handle_id).await.is_some());

        handle_manager.remove_handle(&handle_id).await;
        assert!(handle_manager.get_read_handle(&handle_id).await.is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_get_missing_read_handle() {
        let handle_manager: HandleManager<String, String, String> = HandleManager::new();

        assert!(handle_manager
            .get_read_handle("missing_handle")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_write_handle_create_get() {
        let handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager
            .create_write_handle(String::from("write"))
            .await;

        let handle = handle_manager.get_write_handle(&handle_id).await.unwrap();
        let handle = handle.lock().await;

        assert_eq!("write", &*handle)
    }

    #[tokio::test]
    async fn test_handle_manager_write_handle_delete() {
        let handle_manager: HandleManager<String, String, String> = HandleManager::new();

        let handle_id = handle_manager
            .create_write_handle(String::from("write"))
            .await;
        assert!(handle_manager.get_write_handle(&handle_id).await.is_some());

        handle_manager.remove_handle(&handle_id).await;
        assert!(handle_manager.get_write_handle(&handle_id).await.is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_get_missing_write_handle() {
        let handle_manager: HandleManager<String, String, String> = HandleManager::new();

        assert!(handle_manager
            .get_write_handle("missing_handle")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn test_handle_manager_remove_missing_handle() {
        let handle_manager: HandleManager<String, String, String> = HandleManager::new();

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
