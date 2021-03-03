use dray::run_server;
use log::{info, LevelFilter};
use std::time::Duration;
use tokio;
use tokio::runtime::Runtime;
use tokio::signal;

fn main() {
    std::env::set_var("AWS_ACCESS_KEY_ID", "miniouser");
    std::env::set_var("AWS_SECRET_ACCESS_KEY", "miniopass");

    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    info!("Starting Dray");

    let runtime = Runtime::new().unwrap();
    runtime.spawn(run_server());
    runtime.block_on(signal::ctrl_c()).unwrap();

    info!("Received SIGINT - Shutting Down Dray");

    runtime.shutdown_timeout(Duration::from_secs(10))
}
