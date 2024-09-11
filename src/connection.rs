use crate::utils::files::FileManager;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Connection {
    pub name: String,
    pub connection_str: String,
}

const STORAGE_FILE_NAME: &str = "connections.json";

impl Connection {
    pub const fn new(name: String, connection_str: String) -> Self {
        Self {
            name,
            connection_str,
        }
    }

    // QUESTION: should these be here?
    pub fn read_from_storage() -> Result<Vec<Self>> {
        let file = FileManager::init()?.read_data(STORAGE_FILE_NAME.into())?;
        serde_json::from_str(&file).context("Error while parsing `connection.json`")
    }

    pub fn write_to_storage(connections: &[Self]) -> Result<()> {
        FileManager::init()?.write_data(
            STORAGE_FILE_NAME.into(),
            &serde_json::to_string_pretty(connections)?,
        )
    }
}
