use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub aws: AwsConfig,
}

#[derive(Debug, Deserialize)]
pub struct AwsConfig {
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub bucket: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        // First try to load from config.json in the current directory
        let config_path = Path::new("config.json");
        if config_path.exists() {
            let mut file = File::open(config_path).context("Failed to open config.json")?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .context("Failed to read config.json")?;
            let config: Config =
                serde_json::from_str(&contents).context("Failed to parse config.json")?;
            return Ok(config);
        }

        // If config.json doesn't exist, use default values
        Ok(Config {
            aws: AwsConfig {
                access_key: std::env::var("AWS_ACCESS_KEY_ID")
                    .unwrap_or_else(|_| "YOUR_ACCESS_KEY".to_string()),
                secret_key: std::env::var("AWS_SECRET_ACCESS_KEY")
                    .unwrap_or_else(|_| "YOUR_SECRET_KEY".to_string()),
                region: std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
                bucket: std::env::var("AWS_S3_BUCKET")
                    .unwrap_or_else(|_| "repo-analyzer".to_string()),
            },
        })
    }
}
