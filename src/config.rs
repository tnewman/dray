use anyhow::Result;
use envy;
use serde::Deserialize;

use crate::storage::s3::S3Config;

#[derive(Deserialize, Debug)]
pub struct DrayConfig {
    #[serde(flatten)]
    pub s3: S3Config,
}

impl DrayConfig {
    pub fn new() -> Result<DrayConfig> {
        let config = envy::prefixed("DRAY_").from_env::<DrayConfig>().unwrap();
        Ok(config)
    }
}

#[cfg(test)]
mod test {}
