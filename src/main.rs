use dotenv::dotenv;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::signal;
use tracing::info;
use dray::{config::DrayConfig, observability::init_observability, ssh_server::DraySshServer};

fn main() {
    dotenv().ok();

    let runtime = Runtime::new().expect("Tokio Runtime should initialize");

    let _observability = init_observability();

    info!("Starting Dray");

    let dray_config = DrayConfig::new().unwrap();
    let dray_server = runtime.block_on(DraySshServer::new(dray_config));

    runtime.block_on(dray_server.health_check()).unwrap();
    runtime.spawn(dray_server.run_server());

    runtime.block_on(signal::ctrl_c()).unwrap();

    info!("Received SIGINT - Shutting Down Dray");

    runtime.shutdown_timeout(Duration::from_secs(10))
}
