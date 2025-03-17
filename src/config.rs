use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_url: Option<String>,
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
            api_key: std::env::var("REPO_ANALYZER_API_KEY").ok(),
            api_url: std::env::var("REPO_ANALYZER_API_URL").ok(),
        })
    }
}
