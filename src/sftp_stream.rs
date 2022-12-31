use std::{convert::TryFrom, io::{ErrorKind, Cursor}, mem};

use bytes::{Bytes, BytesMut, Buf};
use russh::ChannelStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{error::Error, protocol::{request::Request, response::data}, sftp_session::SftpSession, try_buf::TryBuf};

pub struct SftpStream {
    sftp_session: SftpSession,
}

impl SftpStream {
    pub fn new(sftp_session: SftpSession) -> SftpStream {
        SftpStream { sftp_session }
    }

    pub async fn process_stream(&self, mut stream: ChannelStream) -> Result<(), Error> {
        let mut buffer = BytesMut::with_capacity(4096);

        loop {
            if let Some(mut frame) = parse_request_frame(&mut buffer) {
                let response = match Request::try_from(&mut frame) {
                    Ok(request) => self.sftp_session.handle_request(request).await,
                    Err(_) => SftpSession::build_invalid_request_message_response(),
                };

                let response_bytes = Bytes::from(&response);

                match stream.write_all(&response_bytes).await {
                    Ok(_) => {}
                    Err(error) => return Err(Error::Failure(error.to_string())),
                }
            }

            let bytes_read = match stream.read_buf(&mut buffer).await {
                Ok(bytes_read) => bytes_read,
                Err(error) => return Err(Error::Failure(error.to_string())),
            };

            if 0 == bytes_read {
                if buffer.is_empty() {
                    return Ok(());
                } else {
                    return Err(Error::Failure(ErrorKind::ConnectionReset.to_string()));
                }
            }
        }
    }
}

fn parse_request_frame(buffer: &mut BytesMut) -> Option<Bytes> {
    let length_field_size = mem::size_of::<u32>();

    let mut peeker = Cursor::new(&buffer[..]);

    let data_length = match peeker.try_get_u32() {
        Ok(data_length) => data_length,
        Err(_) => return None,
    };
    
    let frame_length = data_length + length_field_size as u32;

    match buffer.try_get_bytes(frame_length) {
        Ok(frame) => Some(frame),
        Err(_) => None,
    }
}
