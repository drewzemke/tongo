use anyhow::{Context, Result};
use color_map::{ColorMap, RawColorMap};
use key_map::KeyMap;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, rc::Rc};

use crate::utils::storage::{CONFIG_FILE_NAME, THEME_FILE_NAME};

pub mod color_map;
pub mod key_map;

type RawKeyMap = HashMap<String, String>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RawConfig {
    #[serde(default)]
    pub keys: RawKeyMap,

    #[serde(default)]
    pub theme: Option<RawColorMap>,
}

impl TryFrom<(Option<String>, Option<String>)> for RawConfig {
    type Error = anyhow::Error;

    fn try_from((config_file, theme_file): (Option<String>, Option<String>)) -> Result<Self> {
        let config_file = config_file.unwrap_or_default();
        let mut config: Self = toml::from_str(&config_file)
            .context(format!("Could not parse `{CONFIG_FILE_NAME}`"))?;

        // if no theme was provided in the config file, try to parse the theme file
        if config.theme.is_none() {
            if let Some(theme_file) = theme_file {
                let theme = toml::from_str(&theme_file)
                    .context(format!("Could not parse `{THEME_FILE_NAME}`"))?;

                config.theme = theme;
            }
        }

        Ok(config)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub key_map: Rc<KeyMap>,
    pub color_map: Rc<ColorMap>,
}

impl TryFrom<RawConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(config: RawConfig) -> Result<Self, Self::Error> {
        let key_map = Rc::new(config.keys.try_into()?);
        let color_map = if let Some(raw_color_map) = config.theme {
            Rc::new(raw_color_map.try_into()?)
        } else {
            Rc::new(ColorMap::default())
        };

        Ok(Self { key_map, color_map })
    }
}
