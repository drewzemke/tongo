use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RawConfig {
    #[serde(default)]
    pub keys: HashMap<String, String>,
}

impl TryFrom<&str> for RawConfig {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let config = toml::from_str(value)?;
        Ok(config)
    }
}
