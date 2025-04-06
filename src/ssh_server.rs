use crate::config::DrayConfig;
use crate::error::Error;
use crate::sftp_session::SftpSession;
use crate::sftp_stream::SftpStream;
use crate::storage::{s3::S3StorageFactory, Storage, StorageFactory};
use russh::keys::PublicKey;
use russh::SshId;
use russh::{
    server::{Auth, Config, Handler, Msg, Server, Session},
    Channel, ChannelId,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info};

pub struct DraySshServer {
    dray_config: Arc<DrayConfig>,
    object_storage_factory: Arc<dyn StorageFactory>,
    object_storage: Arc<dyn Storage>,
    channels: Arc<Mutex<HashMap<ChannelId, Channel<Msg>>>>,
    user: RwLock<Option<String>>,
}

impl DraySshServer {
    pub async fn new(dray_config: DrayConfig) -> DraySshServer {
        let object_storage_factory = Arc::from(S3StorageFactory::new(&dray_config.s3).await);
        let object_storage = object_storage_factory.create_storage();

        DraySshServer {
            dray_config: Arc::from(dray_config),
            object_storage_factory,
            object_storage,
            channels: Arc::from(Mutex::from(HashMap::new())),
            user: RwLock::from(Option::None),
        }
    }

    pub async fn health_check(&self) -> Result<(), Error> {
        self.object_storage.health_check().await?;
        Ok(())
    }

    pub async fn run_server(mut self) -> Result<(), Error> {
        let ssh_config = Config {
            server_id: SshId::Standard(format!(
                "SSH-2.0-{}_{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )),
            keys: self.dray_config.get_ssh_keys()?,
            window_size: 16777216,
            maximum_packet_size: 32768,
            ..Default::default()
        };

        let ssh_config = Arc::new(ssh_config);
        let addr = &self.dray_config.get_host_socket_addr()?;

        info!("Binding to Host {}", self.dray_config.host);

        self.run_on_address(ssh_config, addr)
            .await
            .map_err(|error| Error::Failure(error.to_string()))
    }
}

impl Server for DraySshServer {
    type Handler = Self;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        DraySshServer {
            dray_config: self.dray_config.clone(),
            object_storage_factory: self.object_storage_factory.clone(),
            object_storage: self.object_storage_factory.create_storage(),
            channels: Arc::from(Mutex::from(HashMap::new())),
            user: RwLock::from(None),
        }
    }
}

impl Handler for DraySshServer {
    type Error = Error;

    async fn auth_publickey(
        &mut self,
        user: &str,
        public_key: &PublicKey,
    ) -> Result<Auth, Self::Error> {
        let authorized_keys = match self.object_storage.get_authorized_keys(user).await {
            Ok(authorized_keys) => authorized_keys,
            Err(error) => {
                error!(
                    "Error during public key authentication for {}: {}",
                    user, error
                );
                return Err(error);
            }
        };

        match authorized_keys.contains(public_key) {
            true => {
                info!(
                    "Successfully authenticated {} with public key authentication",
                    user
                );

                {
                    let mut self_user = self.user.write().await;
                    *self_user = Some(user.to_string());
                }

                Ok(Auth::Accept)
            }
            false => {
                info!("Rejected public key authentication attempt from {}", user);
                Ok(Auth::Reject {
                    proceed_with_methods: Option::None,
                    partial_success: false,
                })
            }
        }
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        {
            let mut channels = self.channels.lock().await;
            channels.insert(channel.id(), channel);
        }

        Ok(true)
    }

    async fn channel_close(
        &mut self,
        channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        {
            let mut channels = self.channels.lock().await;
            channels.remove(&channel);
        }

        Ok(())
    }

    async fn subsystem_request(
        &mut self,
        channel_id: ChannelId,
        name: &str,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if name != "sftp" {
            error!("Failed to start unsupported subsystem {}", name);
            session.channel_failure(channel_id);
            return Ok(());
        }

        let user = {
            let user = self.user.read().await;
            user.clone()
        };

        let user = match user {
            Some(user) => user,
            None => {
                error!(
                    "Failed to start sftp subsystem because a user was not found on the channel"
                );
                session.channel_failure(channel_id);
                return Ok(());
            }
        };

        let channel = {
            let mut channels = self.channels.lock().await;
            channels.remove(&channel_id)
        };

        let channel = match channel {
            Some(channel) => channel,
            None => {
                error!(
                    "Failed to start sftp subsystem because the requested channel {} was not found",
                    channel_id
                );
                session.channel_failure(channel_id);
                return Ok(());
            }
        };

        session.channel_success(channel_id);

        let handle = session.handle();
        let sftp_session = SftpSession::new(self.object_storage.clone(), user);
        let sftp_stream = SftpStream::new(sftp_session);

        tokio::spawn(async move {
            info!("Sftp subsystem starting");

            let stream = channel.into_stream();

            match sftp_stream.process_stream(stream).await {
                Ok(_) => info!("Sftp subsystem finished"),
                Err(error) => error!("Sftp subsystem failed: {}", error),
            };

            debug!("Closing channel");

            match handle.close(channel_id).await {
                Ok(_) => debug!("Successfully closed channel"),
                Err(_) => error!("Failed to close channel"),
            };
        });

        Ok(())
    }
}
