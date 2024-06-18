#![allow(clippy::struct_field_names)]
use crate::tree::top_level_document;
use futures::TryStreamExt;
use mongodb::{
    bson::Bson,
    options::FindOptions,
    results::{CollectionSpecification, DatabaseSpecification},
    Client, Database,
};
use ratatui::widgets::ListState;
use std::sync::mpsc::{self, Receiver, Sender};
use tui_tree_widget::{TreeItem, TreeState};

const PAGE_SIZE: usize = 5;
const SEND_ERR_MSG: &str = "Error occurred while processing server response.";

pub enum MongoResponse {
    Query(Vec<Bson>),
    Databases(Vec<DatabaseSpecification>),
    Collections(Vec<CollectionSpecification>),
    Count(u64),
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    ChoosingDatabase,
    ChoosingCollection,
    MainView,
}

pub struct State<'a> {
    pub mode: Mode,
    client: Client,

    pub collection_name: Option<String>,
    pub db: Option<Database>,
    pub page: usize,
    pub count: u64,

    query_send: Sender<MongoResponse>,
    pub query_recv: Receiver<MongoResponse>,

    // main view
    pub main_view_state: TreeState<String>,
    pub main_view_items: Vec<TreeItem<'a, String>>,

    // database list
    pub db_names: Vec<String>,
    pub db_list_state: ListState,

    // collection list
    pub coll_names: Vec<String>,
    pub coll_list_state: ListState,

    pub new_data: bool,
}

impl<'a> State<'a> {
    pub fn new(client: Client) -> Self {
        let (query_send, query_recv) = mpsc::channel::<MongoResponse>();
        Self {
            client,
            mode: Mode::ChoosingDatabase,
            count: 0,
            collection_name: None,
            db: None,
            page: 0,
            query_send,
            query_recv,

            main_view_state: TreeState::default(),
            main_view_items: vec![],

            db_names: vec![],
            db_list_state: ListState::default(),

            coll_names: vec![],
            coll_list_state: ListState::default(),

            new_data: false,
        }
    }

    pub fn exec_get_dbs(&self) {
        let sender = self.query_send.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            let response: MongoResponse = client.list_databases(None, None).await.map_or_else(
                |err| MongoResponse::Error(err.to_string()),
                MongoResponse::Databases,
            );

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    pub fn exec_get_collections(&self) {
        // there should be a db and collection
        let Some(db_name) = self
            .db_list_state
            .selected()
            .and_then(|i| self.db_names.get(i))
        else {
            return;
        };

        let db = self.client.database(db_name);
        let sender = self.query_send.clone();

        tokio::spawn(async move {
            let resonse = match db.list_collections(None, None).await {
                Ok(cursor) => cursor.try_collect::<Vec<_>>().await.map_or_else(
                    |err| MongoResponse::Error(err.to_string()),
                    MongoResponse::Collections,
                ),
                Err(err) => MongoResponse::Error(err.to_string()),
            };

            sender.send(resonse).expect(SEND_ERR_MSG);
        });
    }

    pub fn exec_query(&self) {
        // there should be a db and collection
        let Some(db_name) = self
            .db_list_state
            .selected()
            .and_then(|i| self.db_names.get(i))
        else {
            return;
        };
        let Some(coll_name) = self
            .coll_list_state
            .selected()
            .and_then(|i| self.coll_names.get(i))
        else {
            return;
        };

        let db = self.client.database(db_name);
        let collection_name = coll_name.clone();
        let sender = self.query_send.clone();

        let skip = self.page * PAGE_SIZE;
        let mut options = FindOptions::default();
        options.skip = Some(skip as u64);
        options.limit = Some(PAGE_SIZE as i64);

        tokio::spawn(async move {
            let cursor = db
                .collection::<Bson>(&collection_name)
                .find(None, options)
                .await;
            let response = match cursor {
                Ok(cursor) => cursor.try_collect::<Vec<_>>().await.map_or_else(
                    |err| MongoResponse::Error(err.to_string()),
                    MongoResponse::Query,
                ),
                Err(err) => MongoResponse::Error(err.to_string()),
            };

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    pub fn exec_count(&self) {
        let Some(db_name) = self
            .db_list_state
            .selected()
            .and_then(|i| self.db_names.get(i))
        else {
            return;
        };
        let Some(coll_name) = self
            .coll_list_state
            .selected()
            .and_then(|i| self.coll_names.get(i))
        else {
            return;
        };

        let db = self.client.database(db_name);
        let collection_name = coll_name.clone();
        let sender = self.query_send.clone();

        tokio::spawn(async move {
            let response = db
                .collection::<Bson>(&collection_name)
                .count_documents(None, None)
                .await
                .map_or_else(
                    |err| MongoResponse::Error(err.to_string()),
                    MongoResponse::Count,
                );

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    pub fn update_content(&mut self, response: MongoResponse) {
        match response {
            MongoResponse::Query(content) => {
                let items: Vec<TreeItem<String>> = content
                    .iter()
                    .filter_map(|bson| bson.as_document().map(top_level_document))
                    .collect();

                // initial state has all top-level documents expanded
                let mut state = TreeState::default();
                for item in &items {
                    state.open(vec![item.identifier().clone()]);
                }

                self.main_view_items = items;
                self.main_view_state = state;
            }
            MongoResponse::Count(count) => self.count = count,
            MongoResponse::Databases(dbs) => {
                self.db_names = dbs.iter().map(|db| db.name.clone()).collect();
            }
            MongoResponse::Collections(colls) => {
                self.coll_names = colls.iter().map(|coll| coll.name.clone()).collect();
                self.coll_list_state.select(None);
            }
            MongoResponse::Error(_) => todo!("Need to implement better handling."),
        };
        self.new_data = true;
    }
}
