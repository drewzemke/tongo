use serde::{Deserialize, Serialize};
use std::{
    cell::{Ref, RefCell, RefMut},
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
    pub fn new(name: String, connection_str: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            connection_str,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionManager {
    connections: Rc<RefCell<Vec<Connection>>>,
    // TODO: add file_manager
}

impl ConnectionManager {
    pub fn new(connections: Vec<Connection>) -> Self {
        Self {
            connections: Rc::new(RefCell::new(connections)),
        }
    }

    pub fn connections(&self) -> Ref<Vec<Connection>> {
        self.connections.borrow()
    }

    pub fn connections_mut(&mut self) -> RefMut<Vec<Connection>> {
        self.connections.borrow_mut()
    }
}
