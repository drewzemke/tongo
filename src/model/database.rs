use mongodb::results::DatabaseSpecification;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Database {
    pub name: String,
}

impl Database {
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self { name }
    }
}

impl From<DatabaseSpecification> for Database {
    fn from(value: DatabaseSpecification) -> Self {
        Self { name: value.name }
    }
}
