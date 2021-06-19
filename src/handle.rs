use std::collections::HashMap;
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

    pub fn create_read_handle(&mut self, key: String, len: u64) -> String {
        let read_handle = ReadHandle::new(key, len);
        let handle_id = read_handle.get_handle_id_string();

        self.read_handles
            .insert(read_handle.get_handle_id().to_string(), read_handle);

        handle_id
    }

    pub fn create_write_handle(&mut self, multipart_upload_id: String) -> String {
        let write_handle = WriteHandle::new(multipart_upload_id);
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

pub struct ReadHandle {
    id: String,
    key: String,
    len: u64,
}

impl ReadHandle {
    pub fn new(key: String, len: u64) -> ReadHandle {
        ReadHandle {
            id: generate_handle_id(),
            key,
            len,
        }
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn len(&self) -> u64 {
        self.len
    }
}

impl Handle for ReadHandle {
    fn get_handle_id(&self) -> &str {
        &self.id
    }
}

pub struct WriteHandle {
    id: String,
    multipart_upload_id: String,
}

impl WriteHandle {
    pub fn new(multipart_upload_id: String) -> WriteHandle {
        WriteHandle {
            id: generate_handle_id(),
            multipart_upload_id,
        }
    }

    pub fn get_multipart_upload_id(&self) -> &str {
        &self.multipart_upload_id
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
    use super::*;

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

    #[test]
    fn test_handle_manager_read_handle_create_get() {
        let mut handle_manager = HandleManager::new();
        let handle_id = handle_manager.create_read_handle(String::from("key"), 1);

        let handle = handle_manager.get_read_handle(&handle_id).unwrap();

        assert_eq!(handle_id, handle.get_handle_id_string());
        assert_eq!("key", handle.get_key());
        assert_eq!(1, handle.len());
    }

    #[test]
    fn test_handle_manager_read_handle_delete() {
        let mut handle_manager = HandleManager::new();

        let handle_id = handle_manager.create_read_handle(String::from("key"), 1);
        assert!(handle_manager.get_read_handle(&handle_id).is_some());

        handle_manager.remove_handle(&handle_id);
        assert!(handle_manager.get_read_handle(&handle_id).is_none());
    }

    #[test]
    fn test_handle_manager_get_missing_read_handle() {
        let mut handle_manager = HandleManager::new();

        assert!(handle_manager.get_read_handle("missing_handle").is_none());
    }

    #[test]
    fn test_handle_manager_write_handle_create_get() {
        let mut handle_manager = HandleManager::new();

        let handle_id = handle_manager.create_write_handle(String::from("upload_id"));
        let handle = handle_manager.get_write_handle(&handle_id).unwrap();

        assert_eq!(handle_id, handle.get_handle_id_string());
        assert_eq!("upload_id", handle.get_multipart_upload_id());
    }

    #[test]
    fn test_handle_manager_write_handle_delete() {
        let mut handle_manager = HandleManager::new();

        let handle_id = handle_manager.create_write_handle(String::from("upload_id"));
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
