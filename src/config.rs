use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config {
    pub keys: HashMap<String, String>,
}

impl Config {
    pub fn read_from_string(str: &str) -> Result<Self> {
        toml::from_str(str).context("Error while parsing `config.toml`")
    }
}
