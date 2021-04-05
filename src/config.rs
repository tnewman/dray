use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use thrussh_keys::key;

pub use crate::storage::s3::S3Config;

#[derive(Deserialize, Debug)]
pub struct DrayConfig {
    pub host: String,

    pub ed25519_key_path: Option<String>,

    pub rsa_key_path: Option<String>,

    #[serde(flatten)]
    pub s3: S3Config,
}

impl DrayConfig {
    pub fn new() -> Result<DrayConfig> {
        let dray_config = envy::prefixed("DRAY_").from_env::<DrayConfig>()?;
        Ok(dray_config)
    }

    pub fn get_private_keys(&self) -> Result<Vec<key::KeyPair>> {
        let mut private_keys = vec![];

        if let Some(rsa_key_path) = &self.rsa_key_path {
            thrussh_keys::load_secret_key(Path::new(rsa_key_path), None)?;
        };

        if let Some(ed25519_key_path) = &self.ed25519_key_path {
            private_keys.push(thrussh_keys::load_secret_key(Path::new(ed25519_key_path), None)?);
        }

        Ok(private_keys)
    }
}

#[cfg(test)]
mod test {}
