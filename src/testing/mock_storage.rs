use crate::{
    app::PersistedApp, config::RawConfig, model::connection::Connection, utils::storage::Storage,
};
use anyhow::{anyhow, Result};

#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockStorage {
    pub connections: Vec<Connection>,
    pub persisted_app: Option<PersistedApp>,
    pub config: RawConfig,
}

#[cfg(test)]
impl Storage for MockStorage {
    fn read_connections(&self) -> Result<Vec<Connection>> {
        Ok(self.connections.clone())
    }

    fn write_connections(&self, _connections: &[Connection]) -> Result<()> {
        Ok(())
    }

    fn write_last_session(&self, _persisted_app: &PersistedApp) -> Result<()> {
        Ok(())
    }

    fn read_last_session(&self) -> Result<PersistedApp> {
        self.persisted_app
            .clone()
            .ok_or_else(|| anyhow!("No persisted app in mock"))
    }

    fn read_config(&self) -> Result<RawConfig> {
        Ok(self.config.clone())
    }
}
