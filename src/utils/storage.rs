use anyhow::{anyhow, Context, Result};
use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use crate::{app::PersistedApp, config::Config, connection::Connection};

const APP_DIR_NAME: &str = "tongo";
const CONNECTIONS_FILE_NAME: &str = "connections.json";
const LAST_SESSION_FILE_NAME: &str = "last-session.json";
const CONFIG_FILE_NAME: &str = "config.toml";

// NOTE: stole this from `gitui`
pub fn get_app_config_path() -> Result<PathBuf> {
    let mut path = if cfg!(target_os = "macos") {
        dirs::home_dir().map(|h| h.join(".config"))
    } else {
        dirs::config_dir()
    }
    .ok_or_else(|| anyhow!("failed to find os config dir."))?;

    path.push(APP_DIR_NAME);
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn get_app_data_path() -> Result<PathBuf> {
    let mut path = if cfg!(target_os = "macos") {
        dirs::home_dir().map(|h| h.join(".local").join("share"))
    } else {
        dirs::data_local_dir()
    }
    .ok_or_else(|| anyhow!("failed to find os local data dir."))?;

    path.push(APP_DIR_NAME);
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub trait Storage: Debug {
    fn read_connections(&self) -> Result<Vec<Connection>>;
    fn write_connections(&self, connections: &[Connection]) -> Result<()>;
    fn write_last_session(&self, persisted_app: &PersistedApp) -> Result<()>;
    fn read_last_session(&self) -> Result<PersistedApp>;
    fn read_config(&self) -> Result<Config>;
}

#[derive(Debug, Clone, Default)]
pub struct FileStorage {
    data_dir: PathBuf,
    config_dir: PathBuf,
}

impl Storage for FileStorage {
    fn read_connections(&self) -> Result<Vec<Connection>> {
        let file = self.read_from_data_dir(CONNECTIONS_FILE_NAME.into())?;
        serde_json::from_str(&file).context("Error while parsing `connection.json`")
    }

    fn write_connections(&self, connections: &[Connection]) -> Result<()> {
        self.write_to_data_dir(
            CONNECTIONS_FILE_NAME.into(),
            &serde_json::to_string_pretty(connections)?,
        )
    }

    fn write_last_session(&self, persisted_app: &PersistedApp) -> Result<()> {
        let json = serde_json::to_string_pretty(persisted_app)?;
        self.write_to_data_dir(LAST_SESSION_FILE_NAME.into(), &json)?;
        Ok(())
    }

    fn read_last_session(&self) -> Result<PersistedApp> {
        let file = self
            .read_from_data_dir(LAST_SESSION_FILE_NAME.into())
            .context("TODO: better error handling")?;

        let session =
            serde_json::from_str::<PersistedApp>(&file).context("TODO: better error handling")?;

        Ok(session)
    }

    fn read_config(&self) -> Result<Config> {
        let config_path = Path::new(&get_app_config_path()?).join(CONFIG_FILE_NAME);

        if !config_path.exists() {
            fs::write(
                &config_path,
                include_str!("../../assets/default-config.toml"),
            )?;
        }

        let file = self.read_from_config_dir(CONFIG_FILE_NAME.into())?;
        Config::read_from_string(&file)
    }
}

impl FileStorage {
    pub fn init() -> Result<Self> {
        Ok(Self {
            data_dir: get_app_data_path()?,
            config_dir: get_app_config_path()?,
        })
    }

    fn read_from_config_dir(&self, path_from_config_dir: PathBuf) -> Result<String> {
        let file_path = Path::new(&self.config_dir).join(path_from_config_dir);
        let file = fs::read_to_string(file_path)?;
        Ok(file)
    }

    fn read_from_data_dir(&self, path_from_data_dir: PathBuf) -> Result<String> {
        let file_path = Path::new(&self.data_dir).join(path_from_data_dir);
        let file = fs::read_to_string(file_path)?;
        Ok(file)
    }

    fn write_to_data_dir(&self, path_from_data_dir: PathBuf, data: &str) -> Result<()> {
        let file_path = Path::new(&self.data_dir).join(path_from_data_dir);
        fs::write(file_path, data)?;
        Ok(())
    }
}
