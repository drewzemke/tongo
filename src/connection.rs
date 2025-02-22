use serde::{Deserialize, Serialize};
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
