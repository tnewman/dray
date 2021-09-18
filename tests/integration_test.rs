#[macro_use]
extern crate lazy_static;

use std::{env, process::Stdio, str::FromStr};

use async_once::AsyncOnce;
use dray::{config::DrayConfig, error::Error, DraySshServer};
use log::LevelFilter;

use rusoto_core::{ByteStream, Region};
use rusoto_s3::{
    CreateBucketRequest, DeleteBucketRequest, PutObjectRequest, S3Client, StreamingBody, S3,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    process::Command,
    spawn,
    time::{sleep, Duration},
};

struct TestClient {
    bucket: String,
    s3_client: S3Client,
}

lazy_static! {
    static ref TEST_CLIENT: AsyncOnce<TestClient> = AsyncOnce::new(async {
        env_logger::Builder::new()
            .filter_level(LevelFilter::Info)
            .init();

        let dray_config = get_config();

        let s3_client = create_s3_client(&dray_config).await;

        s3_client
            .put_object(PutObjectRequest {
                bucket: dray_config.s3.bucket.clone(),
                key: ".ssh/test/authorized_keys".to_string(),
                body: Some(ByteStream::from(
                    include_bytes!("../.ssh/id_ed25519.pub").to_vec(),
                )),
                ..Default::default()
            })
            .await
            .unwrap();

        let dray_server = DraySshServer::new(DrayConfig::new().unwrap());

        dray_server.health_check().await.unwrap();

        spawn(dray_server.run_server());
        wait_for_server_listening(&dray_config).await;

        TestClient {
            bucket: dray_config.s3.bucket,
            s3_client,
        }
    });
}

fn get_config() -> DrayConfig {
    env::set_var("DRAY_HOST", "127.0.0.1:2222");
    env::set_var("DRAY_SSH_KEY_PATHS", ".ssh/id_ed25519");
    env::set_var("DRAY_S3_BUCKET", "integration-test");
    env::set_var("DRAY_S3_ENDPOINT_NAME", "http://localhost:9000");
    env::set_var("DRAY_S3_ENDPOINT_REGION", "custom");
    env::set_var("AWS_ACCESS_KEY_ID", "miniouser");
    env::set_var("AWS_SECRET_ACCESS_KEY", "miniopass");

    DrayConfig::new().unwrap()
}

async fn create_s3_client(dray_config: &DrayConfig) -> S3Client {
    let region = match &dray_config.s3.endpoint_name {
        Some(endpoint_name) => Region::Custom {
            endpoint: endpoint_name.clone(),
            name: dray_config.s3.endpoint_region.clone(),
        },
        None => Region::from_str(&dray_config.s3.endpoint_region).unwrap(),
    };

    let s3_client = S3Client::new(region);

    let create_bucket_result = s3_client
        .create_bucket(CreateBucketRequest {
            bucket: dray_config.s3.bucket.clone(),
            ..Default::default()
        })
        .await;

    match create_bucket_result {
        Ok(_) => Ok(()),
        Err(create_bucket_error) => match create_bucket_error.to_string().contains("succeeded") {
            true => Ok(()),
            false => Err(create_bucket_error),
        },
    }
    .unwrap();

    s3_client
}

async fn execute_sftp_command(command: &str) -> Result<String, Error> {
    let mut child = Command::new("sftp")
        .arg("-b-")
        .arg(format!("-i{}/.ssh/id_ed25519", env!("CARGO_MANIFEST_DIR")))
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
    let test_client = TEST_CLIENT.get().await;

    execute_sftp_command("ls").await.unwrap();
}

#[test]
fn test_upload_directory() {}
