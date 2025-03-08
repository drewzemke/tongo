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

    ConnectionCreated(Connection),
    ConnectionSelected(Connection),
    NewConnectionStarted,              // message for conn scr
    EditConnectionStarted(Connection), // message for conn scr
    ConnectionEdited(Connection),      // message for conn scr
    ConnectionDeleted,

    ClientCreated(MongoClient),

    DatabasesUpdated(Vec<DatabaseSpecification>),
    DatabaseHighlighted(DatabaseSpecification),
    DatabaseSelected(DatabaseSpecification),

    CollectionsUpdated(Vec<CollectionSpecification>),
    CollectionHighlighted(CollectionSpecification),
    CollectionSelected(CollectionSpecification),

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

    DocUpdateConfirmed,
    DocInsertConfirmed,
    DocDeleteConfirmed,

    // TODO: sort these out better
    FocusedForward,  // message for ... it depends!
    FocusedBackward, // message for ... it depends!
    FocusedChanged,

    InputKeyPressed,

    ReturnedFromAltScreen,
    ScreenResized,

    InputRequested(InputKind),
    InputConfirmed(InputKind, String),
    InputCanceled,

    ConfirmationRequested(ConfirmKind), // message for tab
    // TODO: Better names
    ConfirmationYes(Command),
    ConfirmationNo,

    AppFocusGained,
    AppFocusLost,

    TabCreated,
    TabChanged,
    TabClosed,
}
