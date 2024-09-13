use crate::{
    components::{Component, ComponentCommand},
    sessions::PersistedComponent,
    system::{command::CommandGroup, event::Event},
};
use anyhow::Result;
use futures::{Future, TryStreamExt};
use mongodb::{
    bson::{doc, Bson, Document},
    options::{ClientOptions, FindOptions},
    results::{CollectionSpecification, DatabaseSpecification},
    Client as MongoClient, Collection, Database,
};
use ratatui::prelude::{Frame, Rect};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
};

// TODO: make configurable
pub const PAGE_SIZE: usize = 5;

#[derive(Debug)]
pub struct Client {
    #[expect(clippy::struct_field_names)]
    mongo_client: Option<MongoClient>,

    db: Option<DatabaseSpecification>,
    coll: Option<CollectionSpecification>,

    filter: Document,
    page: Rc<RefCell<usize>>,

    response_send: Sender<Event>,
    response_recv: Receiver<Event>,
}

impl Default for Client {
    fn default() -> Self {
        let (response_send, response_recv) = mpsc::channel::<Event>();
        Self {
            mongo_client: None,
            db: None,
            coll: None,

            filter: Document::default(),
            page: Rc::new(RefCell::new(0)),

            response_send,
            response_recv,
        }
    }
}

impl Client {
    pub fn new(doc_page: Rc<RefCell<usize>>) -> Self {
        Self {
            page: doc_page,
            ..Default::default()
        }
    }

    /// Executes an asynchronous operation and sends the result through a channel.
    ///
    /// # Arguments
    ///
    /// * `op` - A Future that resolves to a `Result<Event>`. It represents the operation to be executed.
    fn exec<F>(&self, op: F)
    where
        F: Future<Output = Result<Event>> + Send + 'static,
    {
        let sender = self.response_send.clone();

        tokio::spawn(async move {
            let result = match op.await {
                Ok(event) => event,
                Err(err) => Event::ErrorOccurred(err.to_string()),
            };

            sender
                .send(result)
                .expect("Error occurred while processing server response.");
        });
    }

    pub fn set_conn_str(&self, url: String) {
        self.exec(async move {
            let options = ClientOptions::parse(url).await?;
            let client = MongoClient::with_options(options)?;
            Ok(Event::ClientCreated(client))
        });
    }

    fn get_database(&self) -> Option<Database> {
        let client = self.mongo_client.as_ref()?;
        let db_spec = self.db.as_ref()?;
        Some(client.database(&db_spec.name))
    }

    fn get_collection<T>(&self) -> Option<Collection<T>> {
        let db = self.get_database()?;
        let coll = self.coll.as_ref()?;
        Some(db.collection::<T>(&coll.name))
    }

    fn query_dbs(&self) -> Option<()> {
        let client = self.mongo_client.clone()?;

        self.exec(async move {
            let dbs = client.list_databases(None, None).await?;
            Ok(Event::DatabasesUpdated(dbs))
        });

        Some(())
    }

    fn query_collections(&self) -> Option<()> {
        let db = self.get_database()?;

        self.exec(async move {
            let cursor = db.list_collections(None, None).await?;
            let colls = cursor.try_collect::<Vec<_>>().await?;
            Ok(Event::CollectionsUpdated(colls))
        });

        Some(())
    }

    fn query(&self, reset_state: bool) -> Option<()> {
        let coll = self.get_collection::<Bson>()?;
        let filter = Some(self.filter.clone()); // self.filter_editor.filter.clone();
        let skip = *self.page.borrow() * PAGE_SIZE;

        #[expect(clippy::cast_possible_wrap)]
        let options = FindOptions::builder()
            .skip(skip as u64)
            .limit(PAGE_SIZE as i64)
            .build();

        self.exec(async move {
            let cursor = coll.find(filter, options).await?;
            let docs = cursor.try_collect::<Vec<_>>().await?;
            Ok(Event::DocumentsUpdated { docs, reset_state })
        });

        Some(())
    }

    fn count(&self) -> Option<()> {
        let coll = self.get_collection::<Bson>()?;
        let filter = Some(self.filter.clone());

        self.exec(async move {
            let count = coll.count_documents(filter, None).await?;
            Ok(Event::CountUpdated(count))
        });

        Some(())
    }

    fn insert_doc(&self, doc: Document) -> Option<()> {
        let coll = self.get_collection::<Document>()?;

        self.exec(async move {
            coll.insert_one(doc, None).await?;
            Ok(Event::InsertConfirmed)
        });

        Some(())
    }

    fn update_doc(&self, filter: Document, update: Document) -> Option<()> {
        let coll = self.get_collection::<Bson>()?;
        let update = doc! { "$set": update };

        self.exec(async move {
            coll.update_one(filter, update, None).await?;
            Ok(Event::UpdateConfirmed)
        });

        Some(())
    }

    fn delete_doc(&self, filter: Document) -> Option<()> {
        let coll = self.get_collection::<Document>()?;

        self.exec(async move {
            coll.delete_one(filter, None).await?;
            Ok(Event::DeleteConfirmed)
        });

        Some(())
    }
}

impl Component for Client {
    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        // check for completed async operations
        let mut out = vec![];
        while let Ok(content) = self.response_recv.try_recv() {
            out.push(content);
        }

        // handle the event as normal
        match event {
            Event::ConnectionCreated(conn) | Event::ConnectionSelected(conn) => {
                self.set_conn_str(conn.connection_str.clone());
            }
            Event::ClientCreated(client) => {
                self.mongo_client = Some(client.clone());

                // TODO: should we query everything? if we're missing conn/db/coll
                // then it just won't run, and if we just hydrated data we want to
                // query as much as is relevant
                self.query_dbs();
                self.query_collections();
                self.query(true);
                self.count();
            }
            Event::DatabaseHighlighted(db) => {
                self.db = Some(db.clone());
                self.query_collections();
            }
            Event::CollectionHighlighted(coll) => {
                self.coll = Some(coll.clone());
            }
            Event::CollectionSelected(coll) => {
                self.coll = Some(coll.clone());
                *self.page.borrow_mut() = 0;
                self.query(true);
                self.count();
            }
            Event::DocumentPageChanged => {
                self.query(true);
            }
            Event::DocFilterUpdated(doc) => {
                self.filter.clone_from(doc);
                *self.page.borrow_mut() = 0;
                self.query(true);
                self.count();
            }
            Event::DocumentEdited(doc) => {
                if let Some(id) = doc.get("_id") {
                    self.update_doc(doc! { "_id": id }, doc.clone());
                } else {
                    out.push(Event::ErrorOccurred(
                        "Document does not have an `_id` field.".to_string(),
                    ));
                }
            }
            Event::UpdateConfirmed => {
                self.query(false);
            }
            Event::DocumentCreated(doc) => {
                self.insert_doc(doc.clone());
            }
            Event::DocumentDeleted(doc) => {
                if let Some(id) = doc.get("_id") {
                    self.delete_doc(doc! { "_id": id });
                } else {
                    out.push(Event::ErrorOccurred(
                        "Document does not have an `_id` field.".to_string(),
                    ));
                }
            }
            Event::RefreshRequested | Event::InsertConfirmed | Event::DeleteConfirmed => {
                self.count();
                self.query(false);
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
    fn commands(&self) -> Vec<CommandGroup> {
        vec![]
    }

    /// Not used
    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}
}

#[derive(Serialize, Deserialize)]
pub struct PersistedClient {
    db: Option<DatabaseSpecification>,
    coll: Option<CollectionSpecification>,
    page: usize,
}

impl PersistedComponent for Client {
    type StorageType = PersistedClient;

    fn persist(&self) -> Self::StorageType {
        PersistedClient {
            db: self.db.clone(),
            coll: self.coll.clone(),
            page: *self.page.borrow(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.db = storage.db;
        self.coll = storage.coll;
        *self.page.borrow_mut() = storage.page;
    }
}
