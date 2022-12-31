use std::{convert::TryFrom, mem};

use bytes::{BufMut, Bytes};
use log::info;
use russh::ChannelStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{error::Error, protocol::request::Request, sftp_session::SftpSession};

pub struct SftpStream {
    sftp_session: SftpSession,
}

impl SftpStream {
    pub fn new(sftp_session: SftpSession) -> SftpStream {
        SftpStream { sftp_session }
    }

    pub async fn process_stream(&self, mut stream: ChannelStream) -> Result<(), Error> {
        loop {
            match self.process_request(&mut stream).await {
                Ok(_) => {}
                Err(error) => match error {
                    Error::EndOfFile => break Ok(()),
                    _ => break Err(error),
                },
            }
        }
    }

    async fn process_request(&self, stream: &mut ChannelStream) -> Result<(), Error> {
        let request_data_size = stream.read_u32().await?;
        let request_size = request_data_size as usize + mem::size_of::<u32>();

        let mut request_buffer: Vec<u8> = Vec::with_capacity(request_size);
        request_buffer.put_u32(request_data_size);

        let mut request_data_buffer = vec![0; request_data_size as usize];
        stream.read_exact(&mut request_data_buffer).await?;
        request_buffer.put_slice(&request_data_buffer);

        let request = Request::try_from(&mut Bytes::from(request_buffer));

        let response = match request {
            Ok(request) => self.sftp_session.handle_request(request).await,
            Err(_) => {
                let response = SftpSession::build_invalid_request_message_response();
                info!("Sending error response: {:?}", response);
                response
            }
        };

        let mut response_bytes = Bytes::from(&response);
        stream.write_all_buf(&mut response_bytes).await?;

        Ok(())
    }
}
