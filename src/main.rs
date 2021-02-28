use dray::run_server;
use log::info;
use std::time::Duration;
use tokio;
use tokio::runtime::Runtime;
use tokio::signal;

fn main() {
    env_logger::init();

    info!("Starting Dray");

    let runtime = Runtime::new().unwrap();
    runtime.spawn(run_server());
    runtime.block_on(signal::ctrl_c()).unwrap();

    info!("Received SIGINT - Shutting Down Dray");

    runtime.shutdown_timeout(Duration::from_secs(10))
}
