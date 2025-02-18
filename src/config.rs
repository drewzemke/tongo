use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

use crate::utils::files::{get_app_config_path, FileManager};

const STORAGE_FILE_NAME: &str = "config.toml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub keys: HashMap<String, String>,
}

impl Config {
    // FIXME: bad separation of concerns between this and `file.rs`
    pub fn read_from_storage() -> Result<Self> {
        let config_path = Path::new(&get_app_config_path()?).join(STORAGE_FILE_NAME);

        if !config_path.exists() {
            fs::write(&config_path, include_str!("../assets/default-config.toml"))?;
        }

        let file = FileManager::init()?.read_config(STORAGE_FILE_NAME.into())?;
        Self::read_from_string(&file)
    }

    pub fn read_from_string(str: &str) -> Result<Self> {
        toml::from_str(str).context("Error while parsing `config.toml`")
    }
}
