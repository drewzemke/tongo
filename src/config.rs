use anyhow::Result;
use key_map::KeyMap;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, rc::Rc};

pub mod key_map;

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

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub key_map: Rc<KeyMap>,
}

impl TryFrom<RawConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(config: RawConfig) -> Result<Self, Self::Error> {
        let key_map = Rc::new(config.keys.try_into()?);

        Ok(Self { key_map })
    }
}
