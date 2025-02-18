use anyhow::{anyhow, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

const APP_DIR_NAME: &str = "tongo";

// NOTE: stole this from `gitui`
fn get_app_config_path() -> Result<PathBuf> {
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

pub struct FileManager {
    data_dir: PathBuf,
    config_dir: PathBuf,
}

impl FileManager {
    /// # Errors
    ///
    /// Returns an error if the local data directory cannot be found.
    pub fn init() -> Result<Self> {
        Ok(Self {
            data_dir: get_app_data_path()?,
            config_dir: get_app_config_path()?,
        })
    }

    /// # Errors
    ///
    /// Returns an error if the file does not exist, cannot be opened, or if
    /// an error occurs while reading.
    pub fn read_config(&self, path_from_config_dir: PathBuf) -> Result<String> {
        let file_path = Path::new(&self.config_dir).join(path_from_config_dir);
        let file = fs::read_to_string(file_path)?;
        Ok(file)
    }

    /// # Errors
    ///
    /// Returns an error if the file does not exist, cannot be opened, or if
    /// an error occurs while reading.
    pub fn read_data(&self, path_from_data_dir: PathBuf) -> Result<String> {
        let file_path = Path::new(&self.data_dir).join(path_from_data_dir);
        let file = fs::read_to_string(file_path)?;
        Ok(file)
    }

    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or if an error occurs while writing.
    pub fn write_data(&self, path_from_data_dir: PathBuf, data: &str) -> Result<()> {
        let file_path = Path::new(&self.data_dir).join(path_from_data_dir);
        fs::write(file_path, data)?;
        Ok(())
    }
}
