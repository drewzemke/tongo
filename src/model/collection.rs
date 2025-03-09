use mongodb::results::CollectionSpecification;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Collection {
    pub name: String,
}

impl Collection {
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self { name }
    }
}

impl From<CollectionSpecification> for Collection {
    fn from(value: CollectionSpecification) -> Self {
        Self { name: value.name }
    }
}
