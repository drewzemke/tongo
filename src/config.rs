use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::files::FileManager;

const STORAGE_FILE_NAME: &str = "config.toml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub keys: HashMap<String, String>,
}

impl Config {
    pub fn read_from_storage() -> Result<Self> {
        let file = FileManager::init()?.read_config(STORAGE_FILE_NAME.into())?;
        toml::from_str(&file).context("Error while parsing `config.toml`")
    }
}
