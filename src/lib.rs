pub mod error;
pub mod protocol;
pub mod storage;
pub mod try_buf;

use anyhow::Error;
use futures::future::{ready, Ready};
use std::{sync::Arc, time::Duration};
use thrussh::server::{run, Auth, Config, Handler, Server, Session};
use thrussh_keys::key::KeyPair;

#[tokio::main]
pub async fn run_server() {
    let config = Config {
        connection_timeout: Some(Duration::from_secs(3)),
        auth_rejection_time: Duration::from_secs(3),
        keys: vec![KeyPair::generate_ed25519().unwrap()],
        ..Default::default()
    };

    let config = Arc::new(config);

    let dray_ssh_server = DraySshServer::new();

    run(config, "0.0.0.0:2222", dray_ssh_server).await.unwrap()
}

#[derive(Clone)]
struct DraySshServer {}

impl DraySshServer {
    pub fn new() -> DraySshServer {
        DraySshServer {}
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
    type FutureAuth = Ready<Result<(Self, Auth), anyhow::Error>>;
    type FutureBool = Ready<Result<(Self, Session, bool), anyhow::Error>>;
    type FutureUnit = Ready<Result<(Self, Session), anyhow::Error>>;

    fn finished_auth(self, auth: Auth) -> Self::FutureAuth {
        ready(Ok((self, auth)))
    }

    fn finished_bool(self, b: bool, session: Session) -> Self::FutureBool {
        ready(Ok((self, session, b)))
    }

    fn finished(self, session: Session) -> Self::FutureUnit {
        ready(Ok((self, session)))
    }
}
