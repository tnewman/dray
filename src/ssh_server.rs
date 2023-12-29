use crate::config::DrayConfig;
use crate::error::Error;
use crate::sftp_session::SftpSession;
use crate::sftp_stream::SftpStream;
use crate::storage::{s3::S3StorageFactory, Storage, StorageFactory};
use async_trait::async_trait;
use log::{debug, error, info};
use russh::SshId;
use russh::{
    server::{run, Auth, Config, Handler, Msg, Server, Session},
    Channel, ChannelId,
};
use russh_keys::{
    key::{self, PublicKey},
    PublicKeyBase64,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

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

    pub async fn run_server(self) -> Result<(), Error> {
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

        info!("Binding to Host {}", self.dray_config.host);

        run(ssh_config, &self.dray_config.get_host_socket_addr()?, self)
            .await
            .map_err(|error| Error::Failure(error.to_string()))
    }

    async fn auth_publickey(
        self,
        user: String,
        public_key: PublicKey,
    ) -> Result<(DraySshServer, Auth), Error> {
        let authorized_keys = match self
            .object_storage
            .get_authorized_keys_fingerprints(&user)
            .await
        {
            Ok(authorized_keys) => authorized_keys,
            Err(error) => {
                error!(
                    "Error during public key authentication for {}: {}",
                    user, error
                );
                return Err(error);
            }
        };

        let public_key_fingerprint = public_key.fingerprint();

        match authorized_keys.contains(&public_key_fingerprint) {
            true => {
                info!(
                    "Successfully authenticated {} with public key authentication",
                    user
                );

                {
                    let mut self_user = self.user.write().await;
                    *self_user = Some(user);
                }

                Ok((self, Auth::Accept))
            }
            false => {
                info!("Rejected public key authentication attempt from {}", user);
                Ok((
                    self,
                    Auth::Reject {
                        proceed_with_methods: Option::None,
                    },
                ))
            }
        }
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

#[async_trait]
impl Handler for DraySshServer {
    type Error = Error;

    async fn auth_publickey(
        self,
        user: &str,
        public_key: &PublicKey,
    ) -> Result<(Self, Auth), Self::Error> {
        let public_key =
            key::parse_public_key(&public_key.public_key_bytes(), Option::None).unwrap();

        self.auth_publickey(user.to_owned(), public_key).await
    }

    async fn channel_open_session(
        self,
        channel: Channel<Msg>,
        session: Session,
    ) -> Result<(Self, bool, Session), Self::Error> {
        {
            let mut channels = self.channels.lock().await;
            channels.insert(channel.id(), channel);
        }

        Ok((self, true, session))
    }

    async fn channel_close(
        self,
        channel: ChannelId,
        session: Session,
    ) -> Result<(Self, Session), Self::Error> {
        {
            let mut channels = self.channels.lock().await;
            channels.remove(&channel);
        }

        Ok((self, session))
    }

    async fn subsystem_request(
        self,
        channel_id: ChannelId,
        name: &str,
        mut session: Session,
    ) -> Result<(Self, Session), Self::Error> {
        if name != "sftp" {
            error!("Failed to start unsupported subsystem {}", name);
            session.channel_failure(channel_id);
            return Ok((self, session));
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
                return Ok((self, session));
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
                return Ok((self, session));
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

        Ok((self, session))
    }
}
