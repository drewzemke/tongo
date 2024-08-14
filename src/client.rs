use crate::{
    components::{Component, ComponentCommand, UniqueType},
    event::Event,
};
use futures::TryStreamExt;
use mongodb::{options::ClientOptions, results::DatabaseSpecification, Client as MongoClient};
use std::sync::mpsc::{self, Receiver, Sender};

const SEND_ERR_MSG: &str = "Error occurred while processing server response.";

#[derive(Debug)]
pub struct Client {
    client: Option<MongoClient>,

    response_send: Sender<Event>,
    response_recv: Receiver<Event>,
}

impl Default for Client {
    fn default() -> Self {
        let (response_send, response_recv) = mpsc::channel::<Event>();
        Self {
            client: None,
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

    fn exec_get_collections(&self, db: &DatabaseSpecification) {
        let client = match self.client {
            Some(ref client) => client.clone(),
            None => return,
        };

        let db = client.database(&db.name);
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
            Event::DatabaseSelected(db) => {
                self.exec_get_collections(db);
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
