use super::command::Command;
use crate::{
    components::input::input_modal::InputKind,
    model::{collection::Collection, connection::Connection, database::Database},
};
use mongodb::{
    bson::{Bson, Document},
    Client as MongoClient,
};

#[derive(Debug, Clone, strum_macros::Display)]
pub enum Event {
    /// Emitted when the app (rather, the terminal window) gains focus.
    AppFocusGained,

    /// Emitted when the app (rather, the terminal window) loses focus.
    AppFocusLost,

    /// Emitted when a new client has been created and has successfully
    /// connected to a Mongo instance.
    ClientCreated(MongoClient),

    /// Emitted when a collection has been successfully created on the Mongo
    /// server.
    CollectionCreationConfirmed,

    /// Emitted when a collection has been successfully dropped on the Mongo
    /// server. The attached boolean that indicates whether the dropped
    /// collection was the currently-selected one.
    CollectionDropConfirmed(bool),

    /// Emitted when the user has changed the selection in the collection list
    /// to a specific collection (but not necessarily selected it).
    CollectionHighlighted(Collection),

    /// Emitted when a collection has been selected by the user.
    CollectionSelected(Collection),

    /// Emitted when the list of collections for the currently-active database
    /// in a tab has been updated.
    CollectionsUpdated(Vec<Collection>),

    /// Emitted when the user has canceled something in the confirm modal.
    ConfirmNo,

    /// Emitted when the user has confirmed something in
    /// the confirm modal. The included command represents the user intention
    /// that was being confirmed.
    ConfirmYes(Command),

    /// Emitted when a new collection has been created and added to storage.
    ConnectionCreated(Connection),

    /// Emitted when a collection has been removed from the user's list.
    ConnectionDeleted,

    /// Emitted when a collection has been selected by the user.
    ConnectionSelected(Connection),

    /// Emitted when an existing collection has been updated.
    ConnectionUpdated(Connection),

    /// Emitted when the document count has been updated.
    CountUpdated(u64),

    /// Emitted when the user has yanked something to the clipboard.
    DataSentToClipboard,

    /// Emitted when a database has been successfully created on the Mongo
    /// server.
    DatabaseCreationConfirmed,

    /// Emitted when a collection has been successfully dropped on the Mongo
    /// server. The attached boolean that indicates whether the dropped database
    /// was the currently-selected one.
    DatabaseDropConfirmed(bool),

    /// Emitted when the user has changed the selection in the database list to
    /// a specific database (but not necessarily selected it).
    DatabaseHighlighted(Database),

    /// Emitted when a database has been selected by the user.
    DatabaseSelected(Database),

    /// Emitted when the list of databases for the currently-active collection
    /// in a tab has been updated.
    DatabasesUpdated(Vec<Database>),

    /// Emitted when a document has been successfully deleted from the Mongo
    /// server.
    DocDeleteComplete,

    /// Emitted when the user has changed the document search filter.
    DocFilterUpdated(Document),

    /// Emitted when a document has been successfully inserted into a collection
    /// in the Mongo server.
    DocInsertComplete,

    /// Emitted when a document has been successfully updated in the Mongo
    /// server.
    DocUpdateComplete,

    /// Emitted when the user has changed which page of the docs view is
    /// visible.
    DocumentPageChanged(usize),

    // TODO: split this into two events? one to set the docs and one to reset the state
    /// Emitted when the set of Mongo documents have been updated.
    DocumentsUpdated { docs: Vec<Bson>, reset_state: bool },

    /// Emitted when the user starts editing an existing collection
    EditConnectionStarted(Connection),

    /// Emitted when an error occurs. The attached string should be a human-
    /// readable description of the error.
    ErrorOccurred(String),

    /// Emitted when the currently-focused panel has changed.
    FocusedChanged,

    /// Emitted when the user closes the input modal without confirming.
    InputCanceled,

    /// Emitted when the user has confirmed input in the input modal.
    InputConfirmed(InputKind, String),

    /// Emitted when a key has been presssed while the app is in raw mode.
    InputKeyPressed,

    /// Emitted when the selection of a list has changed.
    ListSelectionChanged,

    /// Emitted when the app has gone to and returned from an "alternate
    /// screen", such as opening the user's editor to edit a document.
    ReturnedFromAltScreen,

    /// Emitted when the terminal window changes size.
    ScreenResized,

    /// Emitted when a message that was being shown in the status bar has
    /// expired and the status bar has returned to its normal state.
    StatusMessageCleared,

    /// Emitted when the currently-visible tab has changed.
    TabChanged,

    /// Emitted when a tab has closed.
    TabClosed,

    /// Emitted when a new tab has been created.
    TabCreated,

    /// Emitted every event loop iteration to give components (eg. client) an
    /// opportunity to check for and process async process results.
    Tick,
}
