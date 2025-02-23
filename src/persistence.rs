use serde::{Deserialize, Serialize};

pub trait PersistedComponent {
    /// The type used for serialization and deserialization of the component
    type StorageType: Serialize + for<'a> Deserialize<'a>;

    /// Converts the component into its serializable form
    fn persist(&self) -> Self::StorageType;

    /// Populates the component with data from its serialized form
    fn hydrate(&mut self, storage: Self::StorageType);
}
