use mongodb::{
    bson::{Bson, Document},
    results::{CollectionSpecification, DatabaseSpecification},
    Client as MongoClient,
};

use crate::connection::Connection;

#[derive(Debug, Clone)]
pub enum Event {
    Tick,

    ListSelectionChanged,

    ConnectionSelected(Connection),
    NewConnectionStarted,
    ConnectionCreated(Connection),
    ConnectionDeleted,

    ClientCreated(MongoClient),

    DatabasesUpdated(Vec<DatabaseSpecification>),
    DatabaseHighlighted(DatabaseSpecification),
    DatabaseSelected,

    CollectionsUpdated(Vec<CollectionSpecification>),
    CollectionHighlighted(CollectionSpecification),
    CollectionSelected,

    DocumentsUpdated { docs: Vec<Bson>, reset_state: bool },
    CountUpdated(u64),
    DocumentPageChanged(usize),
    DocFilterUpdated(Document),
    ErrorOccurred(String),

    DocumentEdited(Document),
    UpdateConfirmed,
    DocumentCreated(Document),
    InsertConfirmed,
    DocumentDeleted(Document),
    DeleteConfirmed,

    // TODO: sort these out better
    FocusedForward,
    FocusedBackward,
    FocusedChanged,

    RawModeEntered,
    RawModeExited,
    InputKeyPressed,

    ReturnedFromAltScreen,
}
