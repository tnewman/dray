use anyhow::Result;
use crate::storage::s3::S3Config;

pub struct DrayConfig {
    pub s3: S3Config,
}

impl DrayConfig {
    pub fn new() -> Result<DrayConfig> {
        Ok(DrayConfig {
            s3: S3Config {
                endpoint: Some("localhost:9000".to_owned()),
                bucket: "bucket".to_owned()
            }
        })
    }
}

#[cfg(test)]
mod test {

}
