use crate::{
    components::{
        confirm_modal::ConfirmKind, input::input_modal::InputKind, primary_screen::PrimScrFocus,
    },
    model::{collection::Collection, connection::Connection, database::Database},
};
use mongodb::bson::Document;

use super::command::Command;

#[derive(Debug, Clone, strum_macros::Display)]
pub enum AppAction {
    /// Tells `App` to stop showing the help modal.
    CloseHelpModal,

    /// Tells `App` to pass a command through the system.
    DoCommand(Command),

    /// Tells `App` to start recording user input as raw keystrokes instead of
    /// mapping to commands.
    EnterRawMode,

    /// Tells `App` to sop recording user input as raw keystrokes and resume
    /// mapping input keys to commands.
    ExitRawMode,
}

#[derive(Debug, Clone, strum_macros::Display)]
pub enum TabAction {
    /// Tells the currently-visible `Tab` to show a modal asking the user to
    /// confirm an action of a given kind.
    RequestConfirmation(ConfirmKind),

    /// Tells the currently-visible `Tab` to show a modal prompting the user for
    /// input for a given purpose.
    RequestInput(InputKind),
}

#[derive(Debug, Clone, strum_macros::Display)]
pub enum ClientAction {
    /// Tells `Client` to connect to a given Mongo instance.
    Connect(Connection),

    /// Tells `Client` to delete the given document from the currently-selected
    /// collection.
    DeleteDoc(Document),

    /// Tells `Client` to drop the given collection from the currently-selected
    /// database.
    DropCollection(Collection),

    /// Tells `Client` to drop the given database.
    DropDatabase(Database),

    /// Tells `Client` to insert the given document into the currently-selected
    /// collection.
    InsertDoc(Document),

    /// Tells `Client` to refresh the current queries (document and count).
    RefreshQueries,

    /// Tells `Client` to update the given document in the currently-selected
    /// collection.
    UpdateDoc(Document),
}

#[derive(Debug, Clone, strum_macros::Display)]
pub enum ConnScreenAction {
    /// Tells `ConnectionScreen` to stop editing a connection without saving it.
    CancelEditingConn,

    /// Tells `ConnectionScreen` to stop editing a connection and save it.
    FinishEditingConn,

    /// Tells `ConnectionScreen` to focus the connection name input field.
    FocusConnNameInput,

    /// Tells `ConnectionScreen` to focus the connection string input field.
    FocusConnStrInput,

    /// Tells `ConnectionScreen` to start editing the currently-selected
    /// connection.
    StartEditingConn(Connection),

    /// Tells `ConnectionScreen` to start editing a new connection.
    StartNewConn,
}

#[derive(Debug, Clone, strum_macros::Display)]
pub enum PrimScreenAction {
    /// Tells `PrimaryScreenAction` to focus a specific component
    SetFocus(PrimScrFocus),
}

#[derive(Debug, Clone, strum_macros::Display)]
enum Action {
    App(AppAction),
    Client(ClientAction),
    ConnScreen(ConnScreenAction),
    PrimScreen(PrimScreenAction),
    Tab(TabAction),
}

#[derive(Debug, Clone)]
pub struct Message(Action);

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Action::App(app_action) => write!(f, "{app_action}"),
            Action::Tab(tab_action) => write!(f, "{tab_action}"),
            Action::Client(client_action) => write!(f, "{client_action}"),
            Action::ConnScreen(conn_screen_action) => write!(f, "{conn_screen_action}"),
            Action::PrimScreen(prim_screen_action) => write!(f, "{prim_screen_action}"),
        }
    }
}

impl Message {
    pub const fn to_app(action: AppAction) -> Self {
        Self(Action::App(action))
    }

    pub const fn to_tab(action: TabAction) -> Self {
        Self(Action::Tab(action))
    }

    pub const fn to_client(action: ClientAction) -> Self {
        Self(Action::Client(action))
    }

    pub const fn to_conn_scr(action: ConnScreenAction) -> Self {
        Self(Action::ConnScreen(action))
    }

    pub const fn to_prim_scr(action: PrimScreenAction) -> Self {
        Self(Action::PrimScreen(action))
    }

    pub const fn read_as_app(&self) -> Option<&AppAction> {
        if let Action::App(action) = &self.0 {
            Some(action)
        } else {
            None
        }
    }

    pub const fn read_as_tab(&self) -> Option<&TabAction> {
        if let Action::Tab(action) = &self.0 {
            Some(action)
        } else {
            None
        }
    }

    pub const fn read_as_client(&self) -> Option<&ClientAction> {
        if let Action::Client(action) = &self.0 {
            Some(action)
        } else {
            None
        }
    }

    pub const fn read_as_conn_scr(&self) -> Option<&ConnScreenAction> {
        if let Action::ConnScreen(action) = &self.0 {
            Some(action)
        } else {
            None
        }
    }

    pub const fn read_as_prim_scr(&self) -> Option<&PrimScreenAction> {
        if let Action::PrimScreen(action) = &self.0 {
            Some(action)
        } else {
            None
        }
    }
}
