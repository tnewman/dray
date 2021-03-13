pub mod config;
pub mod error;
pub mod protocol;
pub mod storage;
pub mod try_buf;

use crate::config::DrayConfig;
use anyhow::Error;
use futures::{
    future::{ready, Ready},
    Future,
};
use log::{error, info};
use std::{pin::Pin, sync::Arc, time::Duration};
use storage::{s3::S3ObjectStorage, ObjectStorage};
use thrussh::server::{run, Auth, Config, Handler, Server, Session};
use thrussh_keys::{
    key::{self, KeyPair},
    PublicKeyBase64,
};

pub async fn run_server() {
    let dray_config = DrayConfig::new().unwrap();

    let ssh_config = Config {
        connection_timeout: Some(Duration::from_secs(3)),
        auth_rejection_time: Duration::from_secs(3),
        keys: vec![KeyPair::generate_ed25519().unwrap()],
        ..Default::default()
    };

    let ssh_config = Arc::new(ssh_config);

    let dray_ssh_server = DraySshServer::new(&dray_config);

    run(ssh_config, "0.0.0.0:2222", dray_ssh_server)
        .await
        .unwrap()
}

#[derive(Clone)]
struct DraySshServer {
    s3_object_storage: S3ObjectStorage,
}

impl DraySshServer {
    pub fn new(dray_config: &DrayConfig) -> DraySshServer {
        DraySshServer {
            s3_object_storage: S3ObjectStorage::new(&dray_config.s3),
        }
    }

    async fn auth_publickey(
        self,
        user: String,
        public_key: key::PublicKey,
    ) -> Result<(DraySshServer, Auth), Error> {
        let authorized_keys = match self
            .s3_object_storage
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
