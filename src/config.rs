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
    4.5
}

impl Default for Config {
    fn default() -> Self {
        Self {
            exclude: vec!["*.lock".to_string(), "package-lock.json".to_string(), "yarn.lock".to_string()],
            skip_entropy_checks: vec!["*.min.js".to_string(), "*.svg".to_string()],
            threshold: 4.5,
            rules: vec![],
        }
    }
}

pub fn load_config() -> Result<Config> {
    let mut config = if Path::new("ward.toml").exists() {
        let content = fs::read_to_string("ward.toml").context("Failed to read ward.toml")?;
        toml::from_str(&content).context("Failed to parse ward.toml")?
    } else {
        Config::default()
    };

    // Load .wardignore if exists
    let ignore_path = Path::new(".wardignore");
    if ignore_path.exists() {
        let content = fs::read_to_string(ignore_path).context("Failed to read .wardignore")?;
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                config.exclude.push(line.to_string());
            }
        }
    }

    Ok(config)
}
