#![allow(clippy::cast_possible_wrap)]

use crate::{
    json_labeler::JsonLabeler,
    tree::top_level_document,
    widgets::{
        coll_list::CollectionListState, db_list::DatabaseListState,
        filter_input::FilterEditorState, main_view::MainViewState, status_bar::StatusBarState,
    },
};
use futures::TryStreamExt;
use mongodb::{
    bson::{Bson, Document},
    error::Error,
    options::{ClientOptions, FindOptions},
    results::{CollectionSpecification, DatabaseSpecification},
    Client,
};
use std::sync::mpsc::{self, Receiver, Sender};
use tui_tree_widget::TreeState;

const PAGE_SIZE: usize = 5;
const SEND_ERR_MSG: &str = "Error occurred while processing server response.";

pub enum MongoResponse {
    Query {
        docs: Vec<Bson>,
        reset_selection: bool,
        reset_page: bool,
    },
    Databases(Vec<DatabaseSpecification>),
    Collections(Vec<CollectionSpecification>),
    Count(u64),
    NewClient(Client),
    InsertConfirmed,
    UpdateConfirmed,
    DeleteConfirmed,
    Error(Error),
}

// FIXME: these three state enums are not independent,
//   so they shouldn't be indepedent types

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Connection,
    Primary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Navigating,
    EditingFilter,
    CreatingNewConnection,
    Exiting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetFocus {
    DatabaseList,
    CollectionList,
    ConnectionList,
    ConnectionStringEditor,
    ConnectionNameEditor,
    FilterEditor,
    MainView,
}

pub struct State<'a> {
    pub screen: Screen,
    pub focus: WidgetFocus,
    pub mode: Mode,

    // mongo stuff
    client: Option<Client>,
    response_send: Sender<MongoResponse>,
    pub response_recv: Receiver<MongoResponse>,

    // widget states
    pub main_view: MainViewState<'a>,
    pub db_list: DatabaseListState,
    pub coll_list: CollectionListState,
    // pub connection_list: ConnectionListState,
    // pub conn_str_editor: ConnStrEditorState,
    pub filter_editor: FilterEditorState,
    pub status_bar: StatusBarState,

    // should this be elsewhere?
    pub json_labeler: JsonLabeler,

    // TODO: replace with an event system?
    pub new_data: bool,
    pub clear_screen: bool,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> State<'a> {
    pub fn new() -> Self {
        let (response_send, response_recv) = mpsc::channel::<MongoResponse>();
        Self {
            screen: Screen::Primary,
            focus: WidgetFocus::DatabaseList,
            mode: Mode::Navigating,

            client: None,
            response_send,
            response_recv,

            main_view: MainViewState::default(),
            db_list: DatabaseListState::default(),
            coll_list: CollectionListState::default(),
            // conn_str_editor: ConnStrEditorState::default(),
            filter_editor: FilterEditorState::default(),
            status_bar: StatusBarState::default(),
            // connection_list: ConnectionListState::default(),
            json_labeler: JsonLabeler::new(),

            new_data: false,
            clear_screen: false,
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

    pub fn set_conn_str(&self, url: String) {
        let sender = self.response_send.clone();

        tokio::spawn(async move {
            let response = ClientOptions::parse(url)
                .await
                .and_then(Client::with_options)
                .map_or_else(MongoResponse::Error, MongoResponse::NewClient);

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
            let response: MongoResponse = client
                .list_databases(None, None)
                .await
                .map_or_else(MongoResponse::Error, MongoResponse::Databases);

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    pub fn exec_get_collections(&self) {
        let client = match self.client {
            Some(ref client) => client.clone(),
            None => return,
        };
        let Some(db_name) = self.selected_db_name() else {
            return;
        };

        let db = client.database(db_name);
        let sender = self.response_send.clone();

        tokio::spawn(async move {
            let resonse = match db.list_collections(None, None).await {
                Ok(cursor) => cursor
                    .try_collect::<Vec<_>>()
                    .await
                    .map_or_else(MongoResponse::Error, MongoResponse::Collections),
                Err(err) => MongoResponse::Error(err),
            };

            sender.send(resonse).expect(SEND_ERR_MSG);
        });
    }

    pub fn exec_query(&self, reset_selection: bool, reset_page: bool) {
        let client = match self.client {
            Some(ref client) => client.clone(),
            None => return,
        };
        let Some(db_name) = self.selected_db_name() else {
            return;
        };
        let Some(coll_name) = self.selected_coll_name() else {
            return;
        };

        let db = client.database(db_name);
        let collection_name = coll_name.clone();
        let sender = self.response_send.clone();

        let filter = self.filter_editor.filter.clone();
        let skip = if reset_page {
            0
        } else {
            self.main_view.page * PAGE_SIZE
        };
        let options = FindOptions::builder()
            .skip(skip as u64)
            .limit(PAGE_SIZE as i64)
            .build();

        tokio::spawn(async move {
            let cursor = db
                .collection::<Bson>(&collection_name)
                .find(filter, options)
                .await;
            let response =
                match cursor {
                    Ok(cursor) => cursor.try_collect::<Vec<_>>().await.map_or_else(
                        MongoResponse::Error,
                        |docs| MongoResponse::Query {
                            docs,
                            reset_selection,
                            reset_page,
                        },
                    ),
                    Err(err) => MongoResponse::Error(err),
                };

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    pub fn exec_count(&self) {
        let client = match self.client {
            Some(ref client) => client.clone(),
            None => return,
        };
        let Some(db_name) = self.selected_db_name() else {
            return;
        };
        let Some(coll_name) = self.selected_coll_name() else {
            return;
        };

        let db = client.database(db_name);
        let collection_name = coll_name.clone();
        let sender = self.response_send.clone();
        let filter = self.filter_editor.filter.clone();

        tokio::spawn(async move {
            let response = db
                .collection::<Bson>(&collection_name)
                .count_documents(filter, None)
                .await
                .map_or_else(MongoResponse::Error, MongoResponse::Count);

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    pub fn exec_insert_one(&self, doc: Document) {
        let client = match self.client {
            Some(ref client) => client.clone(),
            None => return,
        };
        let Some(db_name) = self.selected_db_name() else {
            return;
        };
        let Some(coll_name) = self.selected_coll_name() else {
            return;
        };

        let db = client.database(db_name);
        let collection_name = coll_name.clone();
        let sender = self.response_send.clone();

        tokio::spawn(async move {
            let response = db
                .collection::<Document>(&collection_name)
                .insert_one(doc, None)
                .await
                .map_or_else(MongoResponse::Error, |_| MongoResponse::InsertConfirmed);

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    pub fn exec_update_one(&self, filter: Document, update: Document) {
        let client = match self.client {
            Some(ref client) => client.clone(),
            None => return,
        };
        let Some(db_name) = self.selected_db_name() else {
            return;
        };
        let Some(coll_name) = self.selected_coll_name() else {
            return;
        };

        let db = client.database(db_name);
        let collection_name = coll_name.clone();
        let sender = self.response_send.clone();

        tokio::spawn(async move {
            let response = db
                .collection::<Bson>(&collection_name)
                .update_one(filter, update, None)
                .await
                .map_or_else(MongoResponse::Error, |_| MongoResponse::UpdateConfirmed);

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    pub fn exec_delete_one(&self, filter: Document) {
        let client = match self.client {
            Some(ref client) => client.clone(),
            None => return,
        };
        let Some(db_name) = self.selected_db_name() else {
            return;
        };
        let Some(coll_name) = self.selected_coll_name() else {
            return;
        };

        let db = client.database(db_name);
        let collection_name = coll_name.clone();
        let sender = self.response_send.clone();

        tokio::spawn(async move {
            let response = db
                .collection::<Bson>(&collection_name)
                .delete_one(filter, None)
                .await
                .map_or_else(MongoResponse::Error, |r| {
                    if r.deleted_count > 0 {
                        MongoResponse::DeleteConfirmed
                    } else {
                        // TODO: refactor Error enum type to accept/display custom errors.
                        MongoResponse::Error(Error::custom("No documents were deleted."))
                    }
                });

            sender.send(response).expect(SEND_ERR_MSG);
        });
    }

    pub fn update_content(&mut self, response: MongoResponse) {
        // Clear the status bar message if there is one.
        self.status_bar.message = None;

        match response {
            MongoResponse::Query {
                docs,
                reset_selection,
                reset_page,
            } => {
                let items: Vec<_> = docs
                    .iter()
                    .filter_map(|bson| bson.as_document().map(top_level_document))
                    .collect();

                if reset_selection {
                    // reset state to have all top-level documents expanded
                    let mut state = TreeState::default();
                    for item in &items {
                        state.open(vec![item.identifier().clone()]);
                    }
                    self.main_view.state = state;
                }

                if reset_page {
                    self.main_view.page = 0;
                }

                self.main_view.documents = docs;
                self.main_view.items = items;
            }
            MongoResponse::Count(count) => self.main_view.count = count,
            MongoResponse::Databases(dbs) => self.db_list.items = dbs,
            MongoResponse::Collections(colls) => {
                self.coll_list.items = colls;
                self.coll_list.state.select(None);
            }
            MongoResponse::NewClient(client) => {
                self.client = Some(client);
                self.exec_get_dbs();
            }
            MongoResponse::DeleteConfirmed
            | MongoResponse::UpdateConfirmed
            | MongoResponse::InsertConfirmed => {
                self.exec_query(false, false);
                self.exec_count();
            }
            MongoResponse::Error(error) => {
                self.status_bar.message = Some(error.kind.to_string());
            }
        };
        self.new_data = true;
    }
}
