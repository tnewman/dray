use dotenv::dotenv;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::signal;
use tracing::info;

use dray::{config::DrayConfig, ssh_server::DraySshServer};

fn main() {
    dotenv().ok();

    info!("Starting Dray");

    let runtime = Runtime::new().unwrap();

    let dray_config = DrayConfig::new().unwrap();
    let dray_server = runtime.block_on(DraySshServer::new(dray_config));

    runtime.block_on(dray_server.health_check()).unwrap();
    runtime.spawn(dray_server.run_server());

    runtime.block_on(signal::ctrl_c()).unwrap();

    info!("Received SIGINT - Shutting Down Dray");

    runtime.shutdown_timeout(Duration::from_secs(10))
}
