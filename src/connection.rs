use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Connection {
    pub name: String,
    pub connection_str: String,
}

impl Connection {
    pub const fn new(name: String, connection_str: String) -> Self {
        Self {
            name,
            connection_str,
        }
    }
}
