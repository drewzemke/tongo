use crate::utils::storage::{FileStorage, Storage};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Connection {
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    pub name: String,
    pub connection_str: String,
}

impl Connection {
    // TODO: change arg types to AsRef<String> and clone here
    #[must_use]
    pub fn new(name: String, connection_str: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            connection_str,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &Uuid {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionManager {
    connections: Rc<RefCell<Vec<Connection>>>,
    storage: Rc<dyn Storage>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self {
            connections: Rc::default(),
            storage: Rc::new(FileStorage::default()),
        }
    }
}

impl ConnectionManager {
    pub fn new(connections: Vec<Connection>, storage: Rc<dyn Storage>) -> Self {
        Self {
            connections: Rc::new(RefCell::new(connections)),
            storage,
        }
    }

    #[must_use]
    pub fn connections(&self) -> Ref<Vec<Connection>> {
        self.connections.borrow()
    }

    pub fn set_connections(&mut self, connections: Vec<Connection>) {
        *self.connections.borrow_mut() = connections;
    }

    /// # Errors
    /// If something goes wrong while writing to the filesystem.
    pub fn add_connection(&mut self, connection: Connection) -> Result<()> {
        let mut connections = self.connections.borrow_mut();
        connections.push(connection);

        self.storage.write_connections(&connections)
    }

    /// # Errors
    /// If something goes wrong while writing to the filesystem.
    pub fn update_connection(&mut self, connection: &Connection) -> Result<()> {
        let mut connections = self.connections.borrow_mut();
        let edited_conn = connections.iter_mut().find(|c| c.id() == connection.id());

        if let Some(edited_conn) = edited_conn {
            *edited_conn = connection.clone();
            self.storage.write_connections(&connections)?;
        }

        Ok(())
    }

    /// # Errors
    /// If something goes wrong while writing to the filesystem.
    pub fn delete_connection(&mut self, index: usize) -> Result<()> {
        let mut connections = self.connections.borrow_mut();
        connections.remove(index);
        self.storage.write_connections(&connections)
    }
}
