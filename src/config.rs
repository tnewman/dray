use std::path::Path;

use serde::Deserialize;
use thrussh_keys::key;

use crate::error::Error;
pub use crate::storage::s3::S3Config;

#[derive(Deserialize, Debug)]
pub struct DrayConfig {
    pub host: String,

    ssh_key_paths: String,

    #[serde(flatten)]
    pub s3: S3Config,
}

impl DrayConfig {
    pub fn new() -> Result<DrayConfig, Error> {
        let dray_config = envy::prefixed("DRAY_").from_env::<DrayConfig>()?;
        Ok(dray_config)
    }

    pub fn get_ssh_keys(&self) -> Result<Vec<key::KeyPair>, Error> {
        let keys: Result<Vec<key::KeyPair>, _> = self
            .ssh_key_paths
            .split(',')
            .map(|key_path| key_path.trim())
            .map(|key_path| thrussh_keys::load_secret_key(Path::new(key_path), None))
            .collect();

        let keys = keys?;
        Ok(keys)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{env, fs::File, io::Write};

    #[test]
    fn test_get_ssh_keys_with_single_key() {
        let config = create_config(create_temp_key());

        assert_eq!(1, config.get_ssh_keys().unwrap().len())
    }

    #[test]
    fn test_get_ssh_keys_with_multiple_keys() {
        let temp_key = create_temp_key();
        let config = create_config(vec![temp_key.clone(), temp_key].join(","));

        assert_eq!(2, config.get_ssh_keys().unwrap().len())
    }

    #[test]
    #[should_panic]
    fn test_get_ssh_keys_with_invalid_key() {
        let config = create_config(String::from("invalid_key"));

        config.get_ssh_keys().unwrap();
    }

    fn create_config(key_paths: String) -> DrayConfig {
        DrayConfig {
            host: String::from(""),
            ssh_key_paths: key_paths,
            s3: S3Config {
                endpoint_name: None,
                endpoint_region: String::from("us-east-1"),
                bucket: String::from("bucket"),
            },
        }
    }

    fn create_temp_key() -> String {
        let temp_file = env::temp_dir().join("id_ed25519");

        let mut file = File::create(temp_file.clone()).unwrap();

        file.write_all(
            b"-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACACJda1/GrWii+6Uk5xeVCK0QIHVr42/ih0X9qI+im4LAAAAKDjBAHe4wQB
3gAAAAtzc2gtZWQyNTUxOQAAACACJda1/GrWii+6Uk5xeVCK0QIHVr42/ih0X9qI+im4LA
AAAEBduesfcFRw+XEu4McoUjygPMccUj6bi+q85Eu3859n3gIl1rX8ataKL7pSTnF5UIrR
AgdWvjb+KHRf2oj6KbgsAAAAGXRuZXdtYW5AdG9tLWxpbnV4LWRlc2t0b3ABAgME
-----END OPENSSH PRIVATE KEY-----",
        )
        .unwrap();

        file.sync_all().unwrap();

        temp_file.into_os_string().into_string().unwrap()
    }
}
