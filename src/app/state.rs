#![allow(clippy::struct_field_names)]
use crate::tree::top_level_document;
use futures::TryStreamExt;
use mongodb::{bson::Bson, options::FindOptions, Client, Database};
use ratatui::widgets::ListState;
use std::sync::mpsc::{self, Receiver, Sender};
use tui_tree_widget::{TreeItem, TreeState};

const PAGE_SIZE: usize = 5;

pub enum MongoResponse {
    Query(Vec<Bson>),
    DbNames(Vec<String>),
    CollectionNames(Vec<String>),
    Count(u64),
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
            let result: Vec<String> = client
                .list_databases(None, None)
                .await
                .unwrap()
                .iter()
                .map(|db| db.name.clone())
                .collect();

            // FIXME: Need a way (maybe another channel) to communicate to the UI
            // that the sync failed
            sender
                .send(MongoResponse::DbNames(result))
                .expect("Error occurred while processing server response.");
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
            let result: Vec<String> = db
                .list_collections(None, None)
                .await
                .unwrap()
                .try_collect::<Vec<_>>()
                .await
                .unwrap()
                .iter()
                .map(|coll| coll.name.clone())
                .collect();

            // FIXME: Need a way (maybe another channel) to communicate to the UI
            // that the sync failed
            sender
                .send(MongoResponse::CollectionNames(result))
                .expect("Error occurred while processing server response.");
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
            let result: Vec<Bson> = db
                .collection::<Bson>(&collection_name)
                .find(None, options)
                .await
                .unwrap()
                .try_collect::<Vec<Bson>>()
                .await
                .unwrap();

            // FIXME: Need a way (maybe another channel) to communicate to the UI
            // that the sync failed
            sender
                .send(MongoResponse::Query(result))
                .expect("Error occurred while processing server response.");
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
            let count = db
                .collection::<Bson>(&collection_name)
                .count_documents(None, None)
                .await
                .unwrap();

            // FIXME: Need a way (maybe another channel) to communicate to the UI
            // that the sync failed
            sender
                .send(MongoResponse::Count(count))
                .expect("Error occurred while processing server response.");
        });
    }

    pub fn update_content(&mut self, response: MongoResponse) {
        match response {
            MongoResponse::Query(content) => {
                let items: Vec<TreeItem<String>> = content
                    .iter()
                    .map(|x| top_level_document(x.as_document().unwrap()))
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
            MongoResponse::DbNames(db_names) => self.db_names = db_names,
            MongoResponse::CollectionNames(coll_names) => {
                self.coll_names = coll_names;
                self.coll_list_state.select(None);
            }
        };
        self.new_data = true;
    }
}
