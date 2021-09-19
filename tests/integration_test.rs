#[macro_use]
extern crate lazy_static;

use std::{env, process::Stdio, str::FromStr};

use async_once::AsyncOnce;
use dray::{config::DrayConfig, error::Error, DraySshServer};
use log::LevelFilter;

use rusoto_core::{ByteStream, Region};
use rusoto_s3::{
    CreateBucketRequest, DeleteObjectRequest, ListObjectsV2Request, PutObjectRequest, S3Client, S3,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    process::Command,
    spawn,
    time::{sleep, Duration},
};

struct TestClient {
    s3_client: S3Client,
    bucket: String,
}

lazy_static! {
    static ref TEST_CLIENT: AsyncOnce<TestClient> = AsyncOnce::new(async {
        env_logger::Builder::new()
            .filter_level(LevelFilter::Info)
            .init();

        let dray_config = get_config();

        let s3_client = create_s3_client(&dray_config).await;

        let dray_server = DraySshServer::new(DrayConfig::new().unwrap());

        dray_server.health_check().await.unwrap();

        spawn(dray_server.run_server());
        wait_for_server_listening(&dray_config).await;

        let test_client = TestClient {
            s3_client,
            bucket: dray_config.s3.bucket,
        };

        put_object(
            &test_client,
            ".ssh/test/authorized_keys",
            include_bytes!("../.ssh/id_ed25519.pub").to_vec(),
        )
        .await;

        test_client
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

    let objects_to_delete = s3_client
        .list_objects_v2(ListObjectsV2Request {
            bucket: dray_config.s3.bucket.clone(),
            max_keys: Some(i64::MAX),
            ..Default::default()
        })
        .await
        .unwrap()
        .contents;

    if let Some(objects_to_delete) = objects_to_delete {
        for object in objects_to_delete {
            s3_client
                .delete_object(DeleteObjectRequest {
                    bucket: dray_config.s3.bucket.clone(),
                    key: object.key.unwrap(),
                    ..Default::default()
                })
                .await
                .unwrap();
        }
    }

    s3_client
}

async fn put_object(test_client: &TestClient, key: &str, data: Vec<u8>) {
    test_client
        .s3_client
        .put_object(PutObjectRequest {
            bucket: test_client.bucket.clone(),
            key: key.to_string(),
            body: Some(ByteStream::from(data)),
            ..Default::default()
        })
        .await
        .unwrap();
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

    put_object(test_client, "home/test/dir1/file1", "1".as_bytes().to_vec()).await;
    put_object(test_client, "home/test/file2", "2".as_bytes().to_vec()).await;

    let sftp_output = execute_sftp_command("ls").await.unwrap();

    assert!(sftp_output.contains("file2"));
    assert!(!sftp_output.contains("file1"));
}

#[tokio::test]
async fn test_list_directory_with_permission_error() {
    // let test_client = TEST_CLIENT.get().await;

    // put_object(test_client, "home/other/file1", "1".as_bytes().to_vec()).await;

    // let sftp_output = execute_sftp_command("ls /home/other").await.unwrap();

    // assert!(sftp_output.contains("file2"));
    // assert!(!sftp_output.contains("file1"));
}

#[test]
fn test_upload_directory() {}
