use mongodb::{
    bson::Document,
    results::{CollectionSpecification, DatabaseSpecification},
};

use crate::connection::Connection;

#[derive(Debug, Clone, strum_macros::Display)]
pub enum AppAction {
    EnterRawMode,
    ExitRawMode,
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
enum Action {
    AppAction(AppAction),
    ClientAction(ClientAction),
}

#[derive(Debug, Clone)]
pub struct Message(Action);

impl Message {
    pub fn to_app(action: AppAction) -> Self {
        Self(Action::AppAction(action))
    }

    pub fn to_client(action: ClientAction) -> Self {
        Self(Action::ClientAction(action))
    }

    pub fn read_as_app(&self) -> Option<&AppAction> {
        if let Action::AppAction(action) = &self.0 {
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
}
