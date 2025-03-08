use mongodb::{
    bson::Document,
    results::{CollectionSpecification, DatabaseSpecification},
};

use crate::{components::confirm_modal::ConfirmKind, connection::Connection};

#[derive(Debug, Clone, strum_macros::Display)]
pub enum AppAction {
    EnterRawMode,
    ExitRawMode,
}

#[derive(Debug, Clone, strum_macros::Display)]
pub enum TabAction {
    RequestConfirmation(ConfirmKind),
}

#[derive(Debug, Clone, strum_macros::Display)]
pub enum ClientAction {
    Connect(Connection),
    DropDatabase(DatabaseSpecification),
    DropCollection(CollectionSpecification),
    UpdateDoc(Document),
    InsertDoc(Document),
    DeleteDoc(Document),
    RefreshQueries,
}

#[derive(Debug, Clone, strum_macros::Display)]
pub enum ConnScreenAction {
    StartNewConn,
    StartEditingConn(Connection),
    FocusConnStrInput,
    FocusConnNameInput,
    FinishEditingConn,
    CancelEditingConn,
}

#[derive(Debug, Clone, strum_macros::Display)]
enum Action {
    AppAction(AppAction),
    ClientAction(ClientAction),
    ConnScreenAction(ConnScreenAction),
    TabAction(TabAction),
}

#[derive(Debug, Clone)]
pub struct Message(Action);

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Action::AppAction(app_action) => write!(f, "{app_action}"),
            Action::TabAction(tab_action) => write!(f, "{tab_action}"),
            Action::ClientAction(client_action) => write!(f, "{client_action}"),
            Action::ConnScreenAction(conn_screen_action) => write!(f, "{conn_screen_action}"),
        }
    }
}

impl Message {
    pub fn to_app(action: AppAction) -> Self {
        Self(Action::AppAction(action))
    }

    pub fn to_tab(action: TabAction) -> Self {
        Self(Action::TabAction(action))
    }

    pub fn to_client(action: ClientAction) -> Self {
        Self(Action::ClientAction(action))
    }

    pub fn to_conn_scr(action: ConnScreenAction) -> Self {
        Self(Action::ConnScreenAction(action))
    }

    pub fn read_as_app(&self) -> Option<&AppAction> {
        if let Action::AppAction(action) = &self.0 {
            Some(action)
        } else {
            None
        }
    }

    pub fn read_as_tab(&self) -> Option<&TabAction> {
        if let Action::TabAction(action) = &self.0 {
            Some(action)
        } else {
            None
        }
    }

    pub fn read_as_client(&self) -> Option<&ClientAction> {
        if let Action::ClientAction(action) = &self.0 {
            Some(action)
        } else {
            None
        }
    }

    pub fn read_as_conn_scr(&self) -> Option<&ConnScreenAction> {
        if let Action::ConnScreenAction(action) = &self.0 {
            Some(action)
        } else {
            None
        }
    }
}
