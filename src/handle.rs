use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite};
use uuid::Uuid;

pub struct HandleManager {

}

impl HandleManager {

    pub fn new() -> HandleManager {
        HandleManager {
            
        }
    }

    pub fn add_read_handle(read_handle: ReadHandle) {

    }

    pub fn get_dir_handle(handle_id: &str) -> Option<&mut ReadHandle> {
        Option::None
    }

    pub fn add_write_handle(write_handle: WriteHandle) {

    }

    pub fn add_dir_handle(dir_handle: DirHandle) {

    }

    pub fn remove_handle(handle: &str) {

    }
}

trait Handle {
    fn get_handle_id(&self) -> &str;
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

fn generate_handle_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod test {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

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
