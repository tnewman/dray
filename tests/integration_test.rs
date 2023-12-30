use std::{env, net::TcpListener, process::Stdio};

use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::primitives::ByteStream;
use dray::{
    config::{DrayConfig, S3Config},
    error::Error,
    ssh_server::DraySshServer,
};

use once_cell::sync::Lazy;
use rand::Rng;
use tempfile::NamedTempFile;
use testcontainers_modules::{
    minio::MinIO,
    testcontainers::{self, Container},
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    process::Command,
    spawn,
    task::spawn_blocking,
    time::{sleep, Duration},
};

struct TestClient {
    host: String,
    s3_client: aws_sdk_s3::Client,
    bucket: String,
}

static DOCKER_CLI: Lazy<testcontainers::clients::Cli> =
    Lazy::new(testcontainers::clients::Cli::default);

static MINIO: Lazy<Container<'_, MinIO>> =
    Lazy::new(|| DOCKER_CLI.run(testcontainers_modules::minio::MinIO::default()));

async fn setup() -> TestClient {
    let dray_config = get_config().await;

    let s3_client = create_s3_client(&dray_config).await;

    let dray_server = DraySshServer::new(dray_config.clone()).await;

    dray_server.health_check().await.unwrap();

    spawn(dray_server.run_server());
    wait_for_server_listening(&dray_config).await;

    let test_client = TestClient {
        host: dray_config.host,
        s3_client,
        bucket: dray_config.s3.bucket,
    };

    put_object(
        &test_client,
        ".ssh/test/authorized_keys",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/.ssh/id_ed25519.pub")).to_vec(),
    )
    .await;

    test_client
}

async fn get_config() -> DrayConfig {
    let port = get_free_port()
        .await
        .expect("Could not find a free port to bind!");

    env::set_var("AWS_ACCESS_KEY_ID", "minioadmin");
    env::set_var("AWS_SECRET_ACCESS_KEY", "minioadmin");

    let mut rng = rand::thread_rng();

    // Since the tests are multi-threaded, do not use environment variables for configuration
    // that may vary between tests, such as the port binding.
    DrayConfig {
        host: format!("127.0.0.1:{}", port),
        ssh_key_paths: ".ssh/id_ed25519".to_string(),
        s3: S3Config {
            endpoint_name: Some(format!(
                "http://localhost:{}",
                MINIO.get_host_port_ipv4(9000)
            )),
            endpoint_region: "custom".to_string(),
            bucket: format!("integration-test-{}", rng.gen::<u32>()),
        },
    }
}

async fn get_free_port() -> Option<u16> {
    spawn_blocking(|| {
        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            let port = rng.gen_range(49152..65535);

            match TcpListener::bind(format!("127.0.0.1:{}", port)) {
                Ok(_) => return Some(port),
                Err(_) => continue,
            }
        }

        None
    })
    .await
    .unwrap()
}

async fn create_s3_client(dray_config: &DrayConfig) -> aws_sdk_s3::Client {
    let mut config_loader = aws_config::defaults(BehaviorVersion::latest());

    if let Some(endpoint_name) = &dray_config.s3.endpoint_name {
        config_loader = config_loader.endpoint_url(endpoint_name);
    };

    let config = config_loader.load().await;

    let s3_client_builder = aws_sdk_s3::config::Builder::new();

    if config.endpoint_url().is_some() {
        s3_client_builder.force_path_style(true);
    }

    let mut s3_config = aws_sdk_s3::config::Builder::from(&config);

    if config.endpoint_url().is_some() {
        s3_config = s3_config.force_path_style(true);
    }

    s3_config = s3_config.region(Region::new(dray_config.s3.endpoint_region.clone()));

    let s3_client = aws_sdk_s3::Client::from_conf(s3_config.build());

    let head_bucket_result = s3_client
        .head_bucket()
        .bucket(&dray_config.s3.bucket)
        .send()
        .await;

    if head_bucket_result.is_err() {
        let create_bucket_result = s3_client
            .create_bucket()
            .bucket(&dray_config.s3.bucket)
            .send()
            .await;

        match create_bucket_result {
            Ok(_) => Ok(()),
            Err(create_bucket_error) => match create_bucket_error.to_string().contains("succeeded")
            {
                true => Ok(()),
                false => Err(create_bucket_error),
            },
        }
        .unwrap();
    }

    let objects_to_delete = s3_client
        .list_objects_v2()
        .bucket(&dray_config.s3.bucket)
        .max_keys(i32::MAX)
        .send()
        .await
        .unwrap()
        .contents;

    if let Some(objects_to_delete) = objects_to_delete {
        for object in objects_to_delete {
            s3_client
                .delete_object()
                .bucket(&dray_config.s3.bucket)
                .key(object.key.unwrap())
                .send()
                .await
                .unwrap();
        }
    }

    s3_client
}

async fn put_object(test_client: &TestClient, key: &str, data: Vec<u8>) {
    test_client
        .s3_client
        .put_object()
        .bucket(&test_client.bucket)
        .key(key)
        .body(ByteStream::from(data))
        .send()
        .await
        .unwrap();

    // S3 is eventually consistent. Allow time for changes to be visible.
    sleep(Duration::from_millis(100)).await;
}

async fn get_object(test_client: &TestClient, key: &str) -> Vec<u8> {
    let get_object_result = test_client
        .s3_client
        .get_object()
        .bucket(&test_client.bucket)
        .key(key)
        .send()
        .await
        .unwrap();

    let mut object_data: Vec<u8> = vec![];
    get_object_result
        .body
        .into_async_read()
        .read_to_end(&mut object_data)
        .await
        .unwrap();

    object_data
}

async fn execute_sftp_command(test_client: &TestClient, command: &str) -> Result<String, Error> {
    let host_pieces: Vec<&str> = test_client.host.split(':').collect();
    let host_name = host_pieces[0];
    let port = host_pieces[1];

    let mut child = Command::new("sftp")
        .arg("-b-")
        .arg(format!("-i{}/.ssh/id_ed25519", env!("CARGO_MANIFEST_DIR")))
        .arg("-o StrictHostKeyChecking=no")
        .arg(format!("-P{}", port))
        .arg(format!("test@{}", host_name))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(command.as_bytes()).await.unwrap();
    child_stdin.write_all(b"\nquit\n").await.unwrap();

    let output = child.wait_with_output().await.unwrap();

    let result = match output.status.success() {
        true => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
        false => Err(Error::Failure(
            String::from_utf8_lossy(&output.stderr).to_string(),
        )),
    };

    // S3 is eventually consistent. Allow time for changes to be visible.
    sleep(Duration::from_millis(100)).await;

    result
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
    let test_client = setup().await;

    put_object(
        &test_client,
        "home/test/dir1/file1",
        "1".as_bytes().to_vec(),
    )
    .await;
    put_object(&test_client, "home/test/file2", "2".as_bytes().to_vec()).await;

    let sftp_output = execute_sftp_command(&test_client, "ls").await.unwrap();

    assert!(sftp_output.contains("file2"));
    assert!(!sftp_output.contains("file1"));
}

#[tokio::test]
#[should_panic(expected = "Can't ls")]
async fn test_list_directory_with_permission_error() {
    let test_client = setup().await;

    put_object(&test_client, "home/other/file1", "1".as_bytes().to_vec()).await;

    let sftp_output = execute_sftp_command(&test_client, "ls /home/other")
        .await
        .unwrap();

    assert!(sftp_output.contains("file2"));
    assert!(!sftp_output.contains("file1"));
}

#[tokio::test]
async fn test_read_file() {
    let test_client = setup().await;

    put_object(
        &test_client,
        "home/test/read-test.txt",
        b"Test read data!".to_vec(),
    )
    .await;

    let temp_file = NamedTempFile::new().unwrap().into_temp_path();

    execute_sftp_command(
        &test_client,
        &format!(
            "GET /home/test/read-test.txt {}",
            temp_file.to_string_lossy()
        ),
    )
    .await
    .unwrap();

    let file_data = fs::read_to_string(temp_file).await.unwrap();

    assert_eq!("Test read data!", &file_data);
}

#[tokio::test]
#[should_panic]
async fn test_read_file_with_permission_error() {
    let test_client = setup().await;

    execute_sftp_command(&test_client, "GET /home/other/test.txt /tmp/test.txt")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_write_file() {
    let test_client = setup().await;

    let temp_file = NamedTempFile::new().unwrap().into_temp_path();

    fs::write(&temp_file, b"Test write data!").await.unwrap();

    execute_sftp_command(
        &test_client,
        &format!(
            "PUT {} /home/test/write-test.txt",
            temp_file.to_string_lossy()
        ),
    )
    .await
    .unwrap();

    let file_data = get_object(&test_client, "home/test/write-test.txt").await;

    assert_eq!(b"Test write data!", file_data.as_slice());
}

#[tokio::test]
#[should_panic(expected = "Permission denied")]
async fn test_write_file_with_permission_error() {
    let test_client = setup().await;

    let temp_file = NamedTempFile::new().unwrap().into_temp_path();

    execute_sftp_command(
        &test_client,
        &format!(
            "PUT {} /home/other/write-test.txt",
            temp_file.to_string_lossy()
        ),
    )
    .await
    .unwrap();
}

#[tokio::test]
#[should_panic(expected = "The specified key does not exist.")]
async fn test_remove_file() {
    let test_client = setup().await;

    put_object(
        &test_client,
        "home/test/test.txt",
        b"Test read data!".to_vec(),
    )
    .await;

    execute_sftp_command(&test_client, "RM /home/test/test.txt")
        .await
        .unwrap();

    get_object(&test_client, "home/test/test.txt").await;
}

#[tokio::test]
#[should_panic(expected = "Permission denied")]
async fn test_remove_file_with_permission_error() {
    let test_client = setup().await;

    execute_sftp_command(&test_client, "RM /home/other/test.txt")
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "The specified key does not exist.")]
async fn test_remove_directory() {
    let test_client = setup().await;

    put_object(
        &test_client,
        "home/test/rmdir/test1.txt",
        b"Test data!".to_vec(),
    )
    .await;

    put_object(
        &test_client,
        "home/test/rmdir/test2.txt",
        b"Test data!".to_vec(),
    )
    .await;

    execute_sftp_command(&test_client, "RMDIR /home/test/rmdir")
        .await
        .unwrap();

    get_object(&test_client, "home/test/rmdir/test1.txt").await;
}

#[tokio::test]
#[should_panic(expected = "Permission denied")]
async fn test_remove_directory_with_permission_error() {
    let test_client = setup().await;

    execute_sftp_command(&test_client, "RMDIR /home/other")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_create_directory() {
    let test_client = setup().await;

    execute_sftp_command(&test_client, "MKDIR /home/test/mkdir/new")
        .await
        .unwrap();

    execute_sftp_command(&test_client, "LS /home/test/mkdir/new")
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "Permission denied")]
async fn test_create_directory_with_permission_error() {
    let test_client = setup().await;

    execute_sftp_command(&test_client, "MKDIR /home/other/mkdir/new")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_rename_file() {
    let test_client = setup().await;

    put_object(
        &test_client,
        "home/test/rename/test1.txt",
        b"Test data!".to_vec(),
    )
    .await;

    execute_sftp_command(
        &test_client,
        "RENAME /home/test/rename/test1.txt /home/test/rename/test2.txt",
    )
    .await
    .unwrap();

    get_object(&test_client, "home/test/rename/test2.txt").await;
}

#[tokio::test]
async fn test_rename_folder() {
    let test_client = setup().await;

    put_object(
        &test_client,
        "home/test/rename_folder/test1.txt",
        b"Test data!".to_vec(),
    )
    .await;

    execute_sftp_command(
        &test_client,
        "RENAME /home/test/rename_folder /home/test/renamed_folder",
    )
    .await
    .unwrap();

    get_object(&test_client, "home/test/renamed_folder/test1.txt").await;
}

#[tokio::test]
#[should_panic(expected = "Permission denied")]
async fn test_rename_file_with_permission_error_source() {
    let test_client = setup().await;

    execute_sftp_command(
        &test_client,
        "RENAME /home/other/rename/test1.txt /home/test/rename/test2.txt",
    )
    .await
    .unwrap();
}

#[tokio::test]
#[should_panic(expected = "Permission denied")]
async fn test_rename_file_with_permission_error_destination() {
    let test_client = setup().await;

    execute_sftp_command(
        &test_client,
        "RENAME /home/test/rename/test1.txt /home/other/rename/test2.txt",
    )
    .await
    .unwrap();
}

#[tokio::test]
#[should_panic(expected = "Operation unsupported")]
async fn test_symlink() {
    let test_client = setup().await;

    execute_sftp_command(
        &test_client,
        "LN -s /home/test/link/source /home/test/link/destination",
    )
    .await
    .unwrap();
}
