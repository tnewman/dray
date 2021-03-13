use anyhow::Result;
use envy;
use serde::Deserialize;

use crate::storage::s3::S3Config;

#[derive(Clone, Deserialize, Debug)]
pub struct DrayConfig {
    pub host: String,

    #[serde(flatten)]
    pub s3: S3Config,
}

impl DrayConfig {
    pub fn new() -> Result<DrayConfig> {
        let dray_config = envy::prefixed("DRAY_").from_env::<DrayConfig>()?;
        Ok(dray_config)
    }
}

#[cfg(test)]
mod test {}
