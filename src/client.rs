#![allow(clippy::cast_possible_wrap)]

use crate::{
    components::{Component, ComponentCommand, UniqueType},
    event::Event,
};
use futures::TryStreamExt;
use mongodb::{
    bson::Bson,
    options::{ClientOptions, FindOptions},
    results::{CollectionSpecification, DatabaseSpecification},
    Client as MongoClient,
};
use std::sync::mpsc::{self, Receiver, Sender};

const SEND_ERR_MSG: &str = "Error occurred while processing server response.";

// TODO: make configurable
pub const PAGE_SIZE: usize = 5;

#[derive(Debug)]
pub struct Client {
    #[allow(clippy::struct_field_names)]
    client: Option<MongoClient>,

    db: Option<DatabaseSpecification>,
    coll: Option<CollectionSpecification>,

    response_send: Sender<Event>,
    response_recv: Receiver<Event>,
}

impl Default for Client {
    fn default() -> Self {
        let (response_send, response_recv) = mpsc::channel::<Event>();
        Self {
            client: None,
            db: None,
            coll: None,

            response_send,
            response_recv,
        }
    }
}

impl Client {
    pub fn set_conn_str(&self, url: String) {
        let sender = self.response_send.clone();

        tokio::spawn(async move {
            let response = ClientOptions::parse(url)
                .await
                .and_then(MongoClient::with_options)
                .map_or_else(
                    |err| Event::ErrorOccurred(err.to_string()),
                    Event::ClientCreated,
                );

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    pub fn exec_get_dbs(&self) {
        let client = match self.client {
            Some(ref client) => client.clone(),
            None => return,
        };
        let sender = self.response_send.clone();

        tokio::spawn(async move {
            let response = client.list_databases(None, None).await.map_or_else(
                |err| Event::ErrorOccurred(err.to_string()),
                Event::DatabasesUpdated,
            );

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    fn exec_get_collections(&self) {
        let db = match (&self.client, &self.db) {
            (Some(client), Some(db)) => client.database(&db.name),
            _ => return,
        };

        let sender = self.response_send.clone();

        tokio::spawn(async move {
            let resonse = match db.list_collections(None, None).await {
                Ok(cursor) => cursor.try_collect::<Vec<_>>().await.map_or_else(
                    |err| Event::ErrorOccurred(err.to_string()),
                    Event::CollectionsUpdated,
                ),
                Err(err) => Event::ErrorOccurred(err.to_string()),
            };

            sender.send(resonse).expect(SEND_ERR_MSG);
        });
    }

    // TODO: handle 'reset_selection', which is a thing the old `State` client did
    pub fn exec_query(&self, page: usize) {
        let coll = match (&self.client, &self.db, &self.coll) {
            (Some(client), Some(db), Some(coll)) => {
                client.database(&db.name).collection::<Bson>(&coll.name)
            }
            _ => return,
        };

        let sender = self.response_send.clone();

        let filter = None; // self.filter_editor.filter.clone();
        let skip = page * PAGE_SIZE;
        let options = FindOptions::builder()
            .skip(skip as u64)
            .limit(PAGE_SIZE as i64)
            .build();

        tokio::spawn(async move {
            let cursor = coll.find(filter, options).await;
            let response = match cursor {
                Ok(cursor) => cursor.try_collect::<Vec<_>>().await.map_or_else(
                    |err| Event::ErrorOccurred(err.to_string()),
                    Event::DocumentsUpdated,
                ),
                Err(err) => Event::ErrorOccurred(err.to_string()),
            };

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    pub fn exec_count(&self) {
        let coll = match (&self.client, &self.db, &self.coll) {
            (Some(client), Some(db), Some(coll)) => {
                client.database(&db.name).collection::<Bson>(&coll.name)
            }
            _ => return,
        };

        let sender = self.response_send.clone();

        let filter = None; // self.filter_editor.filter.clone();

        tokio::spawn(async move {
            let response = coll.count_documents(filter, None).await.map_or_else(
                |err| Event::ErrorOccurred(err.to_string()),
                Event::CountUpdated,
            );

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }
}

impl Component<UniqueType> for Client {
    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        // check for completed async operations
        let mut out = vec![];
        while let Ok(content) = self.response_recv.try_recv() {
            out.push(content);
        }

        // handle the event as normal
        match event {
            Event::ClientCreated(client) => {
                self.client = Some(client.clone());
                self.exec_get_dbs();
            }
            Event::DatabaseHighlighted(db) => {
                self.db = Some(db.clone());
                self.exec_get_collections();
            }
            Event::CollectionHighlighted(coll) => {
                self.coll = Some(coll.clone());
            }
            Event::CollectionSelected(..) => {
                self.exec_query(0);
                self.exec_count();
            }
            Event::DocumentPageChanged(page) => {
                self.exec_query(*page);
            }
            _ => (),
        }
        out
    }

    /// Not used
    fn handle_command(&mut self, _command: &ComponentCommand) -> Vec<Event> {
        vec![]
    }

    /// Not used
    fn focus(&self) {}

    /// Not used
    fn is_focused(&self) -> bool {
        false
    }

    /// Not used
    fn commands(&self) -> Vec<crate::command::CommandGroup> {
        vec![]
    }

    /// Not used
    fn render(&mut self, _frame: &mut ratatui::prelude::Frame, _area: ratatui::prelude::Rect) {}
}
