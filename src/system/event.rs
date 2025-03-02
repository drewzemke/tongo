use mongodb::{
    bson::{Bson, Document},
    results::{CollectionSpecification, DatabaseSpecification},
    Client as MongoClient,
};

use crate::connection::Connection;

use super::command::Command;

#[derive(Debug, Clone, strum_macros::Display)]
pub enum Event {
    Tick,

    ListSelectionChanged,

    StatusMessageCleared,

    ConnectionSelected(Connection),
    NewConnectionStarted,
    EditConnectionStarted(Connection),
    ConnectionCreated(Connection),
    ConnectionEdited(Connection),
    ConnectionDeleted,

    ClientCreated(MongoClient),

    DatabasesUpdated(Vec<DatabaseSpecification>),
    DatabaseHighlighted(DatabaseSpecification),
    DatabaseSelected,

    CollectionsUpdated(Vec<CollectionSpecification>),
    CollectionHighlighted(CollectionSpecification),
    CollectionSelected(CollectionSpecification),

    DocumentsUpdated { docs: Vec<Bson>, reset_state: bool },
    CountUpdated(u64),
    DocumentPageChanged(usize),
    DocFilterUpdated(Document),
    DataSentToClipboard,
    ErrorOccurred(String),

    DocumentEdited(Document),
    UpdateConfirmed,
    DocumentCreated(Document),
    InsertConfirmed,
    DocumentDeleted(Document),
    DeleteConfirmed,
    RefreshRequested,

    // TODO: sort these out better
    FocusedForward,
    FocusedBackward,
    FocusedChanged,

    RawModeEntered,
    RawModeExited,
    InputKeyPressed,

    ReturnedFromAltScreen,
    ScreenResized,

    ConfirmationRequested(Command),
    // TODO: Better names
    ConfirmationYes(Command),
    ConfirmationNo,

    AppFocusGained,
    AppFocusLost,

    TabCreated,
    TabChanged,
}
