use dotenv::dotenv;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::signal;
use tracing::{info, Level};

use dray::{config::DrayConfig, ssh_server::DraySshServer};

use opentelemetry::global;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    dotenv().ok();
    init_tracer();

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

fn init_tracer() {
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("dray")
        .install_simple()
        .unwrap();

    // Create a tracing layer with the configured tracer
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env()
        .unwrap();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(opentelemetry)
        .with(fmt::Layer::default())
        .try_init()
        .unwrap();
}
