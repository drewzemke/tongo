use mongodb::{
    bson::Document,
    results::{CollectionSpecification, DatabaseSpecification},
};

use crate::connection::Connection;

#[derive(Debug, Clone, strum_macros::Display)]
pub enum Action {
    // app
    EnterRawMode,
    ExitRawMode,

    // client
    Connect(Connection),
    DropDatabase(DatabaseSpecification),
    DropCollection(CollectionSpecification),
    UpdateDoc(Document),
    InsertDoc(Document),
    DeleteDoc(Document),
    RefreshQueries,
}

#[derive(Debug, Clone, strum_macros::Display, PartialEq, Eq)]
pub enum Target {
    App,
    Client,
}

#[derive(Debug, Clone)]
pub struct Message(Action, Target);

impl Message {
    pub fn new(action: Action, target: Target) -> Self {
        Self(action, target)
    }

    pub fn action(&self) -> &Action {
        &self.0
    }

    pub fn target(&self) -> &Target {
        &self.1
    }
}
