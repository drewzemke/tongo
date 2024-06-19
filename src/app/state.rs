#![allow(clippy::struct_field_names)]

use super::{
    coll_list::CollectionListState, db_list::DatabaseListState, filter_input::FilterEditorState,
    main_view::MainViewState,
};
use crate::tree::top_level_document;
use futures::TryStreamExt;
use mongodb::{
    bson::Bson,
    options::FindOptions,
    results::{CollectionSpecification, DatabaseSpecification},
    Client,
};
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
    Navigating,
    EditingFilter,
    Exiting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetFocus {
    DatabaseList,
    CollectionList,
    FilterEditor,
    MainView,
}

pub struct State<'a> {
    pub focus: WidgetFocus,
    pub mode: Mode,

    client: Client,
    response_send: Sender<MongoResponse>,
    pub response_recv: Receiver<MongoResponse>,

    // widget states
    pub main_view: MainViewState<'a>,
    pub db_list: DatabaseListState,
    pub coll_list: CollectionListState,
    pub filter_editor: FilterEditorState,

    pub new_data: bool,
}

impl<'a> State<'a> {
    pub fn new(client: Client) -> Self {
        let (response_send, response_recv) = mpsc::channel::<MongoResponse>();
        Self {
            focus: WidgetFocus::DatabaseList,
            mode: Mode::Navigating,

            client,
            response_send,
            response_recv,

            main_view: MainViewState::default(),
            db_list: DatabaseListState::default(),
            coll_list: CollectionListState::default(),
            filter_editor: FilterEditorState::default(),

            new_data: false,
        }
    }

    fn selected_db_name(&self) -> Option<&String> {
        self.db_list
            .state
            .selected()
            .and_then(|i| self.db_list.items.get(i))
            .map(|db| &db.name)
    }

    fn selected_coll_name(&self) -> Option<&String> {
        self.coll_list
            .state
            .selected()
            .and_then(|i| self.coll_list.items.get(i))
            .map(|coll| &coll.name)
    }

    pub fn exec_get_dbs(&self) {
        let sender = self.response_send.clone();
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
        let Some(db_name) = self.selected_db_name() else {
            return;
        };

        let db = self.client.database(db_name);
        let sender = self.response_send.clone();

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
        let Some(db_name) = self.selected_db_name() else {
            return;
        };

        let Some(coll_name) = self.selected_coll_name() else {
            return;
        };

        let db = self.client.database(db_name);
        let collection_name = coll_name.clone();
        let sender = self.response_send.clone();

        let filter = self.filter_editor.filter.clone();
        let skip = self.main_view.page * PAGE_SIZE;
        let mut options = FindOptions::default();
        options.skip = Some(skip as u64);
        options.limit = Some(PAGE_SIZE as i64);

        tokio::spawn(async move {
            let cursor = db
                .collection::<Bson>(&collection_name)
                .find(filter, options)
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
        let Some(db_name) = self.selected_db_name() else {
            return;
        };

        let Some(coll_name) = self.selected_coll_name() else {
            return;
        };

        let db = self.client.database(db_name);
        let collection_name = coll_name.clone();
        let sender = self.response_send.clone();
        let filter = self.filter_editor.filter.clone();

        tokio::spawn(async move {
            let response = db
                .collection::<Bson>(&collection_name)
                .count_documents(filter, None)
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

                self.main_view.items = items;
                self.main_view.state = state;
            }
            MongoResponse::Count(count) => self.main_view.count = count,
            MongoResponse::Databases(dbs) => self.db_list.items = dbs,
            MongoResponse::Collections(colls) => {
                self.coll_list.items = colls;
                self.coll_list.state.select(None);
            }
            MongoResponse::Error(_) => todo!("Need to implement better handling."),
        };
        self.new_data = true;
    }
}
