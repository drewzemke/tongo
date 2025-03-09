use super::command::Command;
use crate::{components::input::input_modal::InputKind, connection::Connection};
use mongodb::{
    bson::{Bson, Document},
    results::{CollectionSpecification, DatabaseSpecification},
    Client as MongoClient,
};

#[derive(Debug, Clone, strum_macros::Display)]
pub enum Event {
    /// Emitted every event loop iteration to give components (eg. client)
    /// an opportunity to check for and process async process results.
    Tick,

    /// Emitted when the selection of a list has changed.
    ListSelectionChanged,

    /// Emitted when a message that was being shown in the status bar has
    /// expired and the status bar has returned to its normal state.
    StatusMessageCleared,

    /// Emitted when a new collection has been created and added to storage.
    ConnectionCreated(Connection),

    /// Emitted when a collection has been selected by the user.
    ConnectionSelected(Connection),

    /// Emitted when the user starts editing an existing collection
    EditConnectionStarted(Connection),

    /// Emitted when an existing collection has been updated.
    ConnectionUpdated(Connection),

    /// Emitted when a collection has been removed from the user's list.
    ConnectionDeleted,

    /// Emitted when a new client has been created and has successfully
    /// connected to a Mongo instance.
    ClientCreated(MongoClient),

    /// Emitted when the list of databases for the currently-active collection
    /// in a tab has been updated.
    DatabasesUpdated(Vec<DatabaseSpecification>),

    /// Emitted when the user has changed the selection in the database list
    /// to a specific database (but not necessarily selected it).
    DatabaseHighlighted(DatabaseSpecification),

    /// Emitted when a database has been selected by the user.
    DatabaseSelected(DatabaseSpecification),

    /// Emitted when the list of collections for the currently-active database
    /// in a tab has been updated.
    CollectionsUpdated(Vec<CollectionSpecification>),

    /// Emitted when the user has changed the selection in the collection list
    /// to a specific collection (but not necessarily selected it).
    CollectionHighlighted(CollectionSpecification),

    /// Emitted when a collection has been selected by the user.
    CollectionSelected(CollectionSpecification),

    /// Emitted when a collection has been successfully dropped on the Mongo
    /// server. The attached boolean that indicates whether the dropped database
    /// was the currently-selected one.
    DatabaseDropConfirmed(bool),

    /// Emitted when a database has been successfully created on the Mongo
    /// server.
    DatabaseCreationConfirmed,

    /// Emitted when a collection has been successfully dropped on the Mongo
    /// server. The attached boolean that indicates whether the dropped
    /// collection was the currently-selected one.
    CollectionDropConfirmed(bool),

    /// Emitted when a collection has been successfully created on the Mongo
    /// server.
    CollectionCreationConfirmed,

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

    FocusedChanged,

    InputKeyPressed,

    ReturnedFromAltScreen,
    ScreenResized,

    InputRequested(InputKind),
    InputConfirmed(InputKind, String),
    InputCanceled,

    // TODO: Better names
    ConfirmationYes(Command),
    ConfirmationNo,

    AppFocusGained,
    AppFocusLost,

    TabCreated,
    TabChanged,
    TabClosed,
}
