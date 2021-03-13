pub mod config;
mod error;
mod protocol;
mod ssh_keys;
mod storage;
mod try_buf;

use crate::config::DrayConfig;
use anyhow::Error;
use futures::{
    future::{ready, Ready},
    Future,
};
use log::{error, info};
use std::{pin::Pin, sync::Arc};
use storage::{s3::S3ObjectStorage, ObjectStorage};
use thrussh::server::{run, Auth, Config, Handler, Server, Session};
use thrussh_keys::{
    key::{self, KeyPair},
    PublicKeyBase64,
};

#[derive(Clone)]
pub struct DraySshServer {
    dray_config: Arc<DrayConfig>,
    object_storage: Arc<dyn ObjectStorage>,
}

impl DraySshServer {
    pub fn new(dray_config: DrayConfig) -> DraySshServer {
        let s3_object_storage = S3ObjectStorage::new(&dray_config.s3);

        DraySshServer {
            dray_config: Arc::from(dray_config),
            object_storage: Arc::from(s3_object_storage),
        }
    }

    pub async fn health_check(&self) -> Result<(), Error> {
        self.object_storage.health_check().await
    }

    pub async fn run_server(self) -> Result<(), Error> {
        let ssh_config = Config {
            keys: vec![KeyPair::generate_ed25519().unwrap()],
            ..Default::default()
        };

        let ssh_config = Arc::new(ssh_config);

        run(ssh_config, &self.dray_config.host.clone(), self)
            .await
            .map_err(Error::from)
    }

    async fn auth_publickey(
        self,
        user: String,
        public_key: key::PublicKey,
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
                Ok((self, Auth::Accept))
            }
            false => {
                info!("Rejected public key authentication attempt from {}", user);
                Ok((self, Auth::Reject))
            }
        }
    }
}

impl Server for DraySshServer {
    type Handler = Self;

    fn new(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        self.clone()
    }
}

impl Handler for DraySshServer {
    type Error = Error;

    #[allow(clippy::type_complexity)]
    type FutureAuth =
        Pin<Box<dyn Future<Output = Result<(DraySshServer, Auth), Self::Error>> + Send>>;

    type FutureBool = Ready<Result<(Self, Session, bool), anyhow::Error>>;

    type FutureUnit = Ready<Result<(Self, Session), anyhow::Error>>;

    fn auth_publickey(self, user: &str, public_key: &key::PublicKey) -> Self::FutureAuth {
        let public_key = key::parse_public_key(&public_key.public_key_bytes()).unwrap();
        Box::pin(self.auth_publickey(user.to_owned(), public_key))
    }

    fn finished_bool(self, b: bool, session: Session) -> Self::FutureBool {
        ready(Ok((self, session, b)))
    }

    fn finished(self, session: Session) -> Self::FutureUnit {
        ready(Ok((self, session)))
    }

    fn finished_auth(self, auth: Auth) -> Self::FutureAuth {
        Box::pin(ready(Ok((self, auth))))
    }
}
