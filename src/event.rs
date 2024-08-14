use mongodb::{
    results::{CollectionSpecification, DatabaseSpecification},
    Client as MongoClient,
};

use crate::connection::Connection;

#[derive(Debug, Clone)]
pub enum Event {
    Tick,
    ListSelectionChanged,
    ConnectionSelected(Connection),
    ConnectionCreated(Connection),
    DatabaseSelected(DatabaseSpecification),
    DatabasesUpdated(Vec<DatabaseSpecification>),
    CollectionSelected(CollectionSpecification),
    CollectionsUpdated(Vec<CollectionSpecification>),
    ClientCreated(MongoClient),
    ConnectionDeleted,
    ErrorOccurred(String),
    NewConnectionStarted,
    FocusedForward,
    FocusedBackward,
    RawModeEntered,
    RawModeExited,
    InputKeyPressed,
}
