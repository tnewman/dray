#[macro_use]
extern crate lazy_static;

use std::{env, process::Stdio};

use async_once::AsyncOnce;
use dray::{
    config::DrayConfig, error::Error, storage::s3::S3StorageFactory, storage::StorageFactory,
    DraySshServer,
};
use log::LevelFilter;

use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    process::Command,
    spawn,
    time::{sleep, Duration},
};

struct TestClient {}

lazy_static! {
    static ref TEST_CLIENT: AsyncOnce<TestClient> = AsyncOnce::new(async {
        env_logger::Builder::new()
            .filter_level(LevelFilter::Info)
            .init();

        env::set_var("DRAY_HOST", "127.0.0.1:2222");
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

        s3_storage.init().await.unwrap();
        dray_server.health_check().await.unwrap();
        spawn(dray_server.run_server());

        wait_for_server_listening(&dray_config).await;

        TestClient {}
    });
}

async fn execute_sftp_command(command: &str) -> Result<String, Error> {
    let mut child = Command::new("sftp")
        .arg("-b-")
        .arg("-i/home/tnewman/repos/dray/.ssh/id_ed25519")
        .arg("-oStrictHostKeyChecking=no")
        .arg("-P2222")
        .arg("test@127.0.0.1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(command.as_bytes()).await.unwrap();
    child_stdin.write_all(b"\nquit\n").await.unwrap();

    let output = child.wait_with_output().await.unwrap();

    match output.status.success() {
        true => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
        false => Err(Error::Failure(
            String::from_utf8_lossy(&output.stderr).to_string(),
        )),
    }
}

async fn wait_for_server_listening(dray_config: &DrayConfig) {
    for count in 1..=1000 {
        match TcpStream::connect(&dray_config.host).await {
            Ok(_) => break,
            Err(_) => sleep(Duration::from_millis(10)).await,
        };

        if count == 1000 {
            panic!("Could not connect to Dray server. Check the log for startup errors.");
        }
    }
}

#[tokio::test]
async fn test_list_directory() {
    TEST_CLIENT.get().await;

    execute_sftp_command("ls").await.unwrap();
}

#[test]
fn test_upload_directory() {}
