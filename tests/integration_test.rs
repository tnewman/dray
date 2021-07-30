use dray::{
    config::DrayConfig, storage::s3::S3StorageFactory, storage::StorageFactory, DraySshServer,
};
use log::LevelFilter;
use std::{env, thread::sleep, time::Duration};
use tokio::{net::TcpStream, runtime::Runtime};

fn setup() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    env::set_var("DRAY_HOST", "localhost:2222");
    env::set_var("DRAY_SSH_KEY_PATHS", ".ssh/id_ed25519");
    env::set_var("DRAY_S3_BUCKET", "integration-test");
    env::set_var("DRAY_S3_ENDPOINT_NAME", "http://localhost:9000");
    env::set_var("DRAY_S3_ENDPOINT_REGION", "custom");
    env::set_var("AWS_ACCESS_KEY_ID", "miniouser");
    env::set_var("AWS_SECRET_ACCESS_KEY", "miniopass");

    let dray_config = DrayConfig::new().unwrap();

    let s3_storage_factory = S3StorageFactory::new(&dray_config.s3);
    let s3_storage = s3_storage_factory.create_storage();

    let dray_server = DraySshServer::new(DrayConfig::new().unwrap());

    let runtime = Runtime::new().unwrap();

    runtime.block_on(s3_storage.init()).unwrap();
    runtime.block_on(dray_server.health_check()).unwrap();
    runtime.spawn(dray_server.run_server());

    // Give the server time to bind to it's port
    sleep(Duration::from_millis(100));

    match runtime.block_on(TcpStream::connect(dray_config.host)) {
        Ok(_) => (),
        Err(_) => panic!("Could not connect to Dray server. Check the log for startup errors."),
    }
}

#[test]
fn test_list_directory() {
    setup();
}
