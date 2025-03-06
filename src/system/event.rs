use super::command::Command;
use crate::{
    components::{confirm_modal::ConfirmKind, input::input_modal::InputKind},
    connection::Connection,
};
use mongodb::{
    bson::{Bson, Document},
    results::{CollectionSpecification, DatabaseSpecification},
    Client as MongoClient,
};

#[derive(Debug, Clone, strum_macros::Display)]
pub enum Event {
    /// Emitted every event loop iteration to give components (eg. client)
    /// an opportunity to check for and process async process results
    Tick,

    ListSelectionChanged,

    StatusMessageCleared,

    ConnectionSelected(Connection),
    NewConnectionStarted,              // message
    EditConnectionStarted(Connection), // message
    ConnectionCreated(Connection),     // message
    ConnectionEdited(Connection),      // message
    ConnectionDeleted,

    ClientCreated(MongoClient),

    DatabasesUpdated(Vec<DatabaseSpecification>),
    DatabaseHighlighted(DatabaseSpecification),
    DatabaseSelected(DatabaseSpecification),
    DatabaseDropped(DatabaseSpecification), // message

    CollectionsUpdated(Vec<CollectionSpecification>),
    CollectionHighlighted(CollectionSpecification),
    CollectionSelected(CollectionSpecification),
    CollectionDropped(CollectionSpecification), // message

    /// carries a flag that indicates whether the dropped
    /// collection was the currently-selected one
    CollectionDropConfirmed(bool),
    CollectionCreationConfirmed,
    DatabaseDropConfirmed(bool),
    DatabaseCreationConfirmed,

    DocumentsUpdated {
        docs: Vec<Bson>,
        reset_state: bool,
    },
    CountUpdated(u64),
    DocumentPageChanged(usize),
    DocFilterUpdated(Document),
    DataSentToClipboard,
    ErrorOccurred(String),

    DocumentEdited(Document), // message
    UpdateConfirmed,
    DocumentCreated(Document), // message
    InsertConfirmed,
    DocumentDeleted(Document), // message
    DocDeleteConfirmed,
    RefreshRequested, // message

    // TODO: sort these out better
    FocusedForward,  // message
    FocusedBackward, // message
    FocusedChanged,

    RawModeEntered, // message
    RawModeExited,  // message
    InputKeyPressed,

    ReturnedFromAltScreen,
    ScreenResized,

    InputRequested(InputKind),
    InputConfirmed(InputKind, String),
    InputCanceled,

    ConfirmationRequested(ConfirmKind), // message
    // TODO: Better names
    ConfirmationYes(Command),
    ConfirmationNo,

    AppFocusGained,
    AppFocusLost,

    TabCreated,
    TabChanged,
    TabClosed,
}
