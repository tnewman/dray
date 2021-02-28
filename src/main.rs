use dray::run_server;
use tokio;
use tokio::signal;

#[tokio::main]
async fn main() {
    let ctrl_c = signal::ctrl_c();
    let server = run_server();

    tokio::select! {
        _ = ctrl_c => {}
        _ = server => {}
    }
}
