use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub skip_entropy_checks: Vec<String>,
    #[serde(default = "default_threshold")]
    pub threshold: f32,
    #[serde(default)]
    pub rules: Vec<CustomRule>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CustomRule {
    pub name: String,
    pub regex: String,
}

fn default_threshold() -> f32 {
    3.8
}

impl Default for Config {
    fn default() -> Self {
        Self {
            exclude: vec!["*.lock".to_string(), "package-lock.json".to_string(), "yarn.lock".to_string()],
            skip_entropy_checks: vec!["*.min.js".to_string(), "*.svg".to_string()],
            threshold: 3.8,
            rules: vec![],
        }
    }
}

pub fn load_config() -> Result<Config> {
    let config_path = Path::new("ward.toml");
    if config_path.exists() {
        let content = fs::read_to_string(config_path).context("Failed to read ward.toml")?;
        let config: Config = toml::from_str(&content).context("Failed to parse ward.toml")?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}
