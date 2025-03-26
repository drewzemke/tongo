use crate::{
    components::{input::input_modal::InputKind, Component},
    config::Config,
    model::{collection::Collection, database::Database},
    persistence::PersistedComponent,
    system::{
        event::Event,
        message::{ClientAction, Message},
        Signal,
    },
};
use futures::{Future, TryStreamExt};
use mongodb::{
    bson::{doc, Bson, Document},
    error::Error as MongoError,
    options::{ClientOptions, FindOptions},
    Client as MongoClient, Collection as MongoCollection, Database as MongoDatabase,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::mpsc::{self, Receiver, Sender},
};

/// The types of async queries that `Client` can do.
#[derive(Debug, Hash, Eq, PartialEq)]
enum Operation {
    Query(bool),
    QueryCollections,
    QueryDatabases,
    Count,
    CreateCollection(String),
    DropCollection(String),
    CreateDatabase(String),
    DropDatabase(String),
}

#[derive(Debug)]
pub struct Client {
    #[expect(clippy::struct_field_names)]
    mongo_client: Option<MongoClient>,

    db: Option<Database>,
    coll: Option<Collection>,

    filter: Document,
    page: usize,

    response_send: Sender<Event>,
    response_recv: Receiver<Event>,

    config: Config,

    /// Used to queue operations and avoid duplicate async calls.
    queued_ops: HashSet<Operation>,
}

impl Default for Client {
    fn default() -> Self {
        let (response_send, response_recv) = mpsc::channel::<Event>();
        Self {
            mongo_client: None,
            db: None,
            coll: None,
            filter: Document::default(),
            page: 0,
            response_send,
            response_recv,
            config: Config::default(),
            queued_ops: HashSet::default(),
        }
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        let (response_send, response_recv) = mpsc::channel::<Event>();
        Self {
            mongo_client: self.mongo_client.clone(),
            db: self.db.clone(),
            coll: self.coll.clone(),
            filter: self.filter.clone(),
            page: self.page,
            response_send,
            response_recv,
            config: self.config.clone(),
            queued_ops: HashSet::default(),
        }
    }
}

impl Client {
    pub fn new(config: Config) -> Self {
        Self {
            config,
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
        F: Future<Output = Result<Event, MongoError>> + Send + 'static,
    {
        let sender = self.response_send.clone();

        tokio::spawn(async move {
            let result = match op.await {
                Ok(event) => event,
                Err(err) => Event::ErrorOccurred(err.into()),
            };

            sender
                .send(result)
                .expect("Error occurred while processing server response.");
        });
    }

    pub fn connect(&self, url: String) {
        self.exec(async move {
            let options = ClientOptions::parse(url).await?;
            let client = MongoClient::with_options(options)?;
            Ok(Event::ClientCreated(client))
        });
    }

    fn get_database(&self) -> Option<MongoDatabase> {
        let client = self.mongo_client.as_ref()?;
        let db_spec = self.db.as_ref()?;
        Some(client.database(&db_spec.name))
    }

    fn get_collection<T>(&self) -> Option<MongoCollection<T>>
    where
        T: Send + Sync,
    {
        let db = self.get_database()?;
        let coll = self.coll.as_ref()?;
        Some(db.collection::<T>(&coll.name))
    }

    fn query_dbs(&self) -> Option<()> {
        let client = self.mongo_client.clone()?;

        self.exec(async move {
            let dbs = client.list_databases().await?;
            Ok(Event::DatabasesUpdated(
                dbs.into_iter().map(Database::from).collect(),
            ))
        });

        Some(())
    }

    fn query_collections(&self) -> Option<()> {
        let db = self.get_database()?;

        self.exec(async move {
            let cursor = db.list_collections().await?;
            let colls = cursor.try_collect::<Vec<_>>().await?;
            Ok(Event::CollectionsUpdated(
                colls.into_iter().map(Collection::from).collect(),
            ))
        });

        Some(())
    }

    fn query(&self, reset_state: bool) -> Option<()> {
        let coll = self.get_collection::<Bson>()?;
        let filter = self.filter.clone();
        let page_size = self.config.page_size;
        let skip = self.page * page_size;

        #[expect(clippy::cast_possible_wrap)]
        let options = FindOptions::builder()
            .skip(skip as u64)
            .limit(page_size as i64)
            .build();

        self.exec(async move {
            let cursor = coll.find(filter).with_options(options).await?;
            let docs = cursor.try_collect::<Vec<_>>().await?;
            Ok(Event::DocumentsUpdated { docs, reset_state })
        });

        Some(())
    }

    fn count(&self) -> Option<()> {
        let coll = self.get_collection::<Bson>()?;
        let filter = self.filter.clone();

        self.exec(async move {
            let count = coll.count_documents(filter).await?;
            Ok(Event::CountUpdated(count))
        });

        Some(())
    }

    fn insert_doc(&self, doc: Document) -> Option<()> {
        let coll = self.get_collection::<Document>()?;

        self.exec(async move {
            coll.insert_one(doc).await?;
            Ok(Event::DocInsertComplete)
        });

        Some(())
    }

    fn update_doc(&self, filter: Document, update: Document) -> Option<()> {
        let coll = self.get_collection::<Bson>()?;
        let update = doc! { "$set": update };

        self.exec(async move {
            coll.update_one(filter, update).await?;
            Ok(Event::DocUpdateComplete)
        });

        Some(())
    }

    fn delete_doc(&self, filter: Document) -> Option<()> {
        let coll = self.get_collection::<Document>()?;

        self.exec(async move {
            coll.delete_one(filter).await?;
            Ok(Event::DocDeleteComplete)
        });

        Some(())
    }

    fn drop_coll(&self, coll_name: &str) -> Option<()> {
        let db = self.get_database()?;
        let coll = db.collection::<Document>(coll_name);
        let dropping_selected_coll = self
            .coll
            .as_ref()
            .is_some_and(|coll| coll.name == *coll_name);

        self.exec(async move {
            coll.drop().await?;
            Ok(Event::CollectionDropConfirmed(dropping_selected_coll))
        });

        Some(())
    }

    fn create_coll(&self, coll_name: String) -> Option<()> {
        let db = self.get_database()?;

        self.exec(async move {
            db.create_collection(coll_name).await?;
            Ok(Event::CollectionCreationConfirmed)
        });

        Some(())
    }

    fn drop_db(&self, db_name: &str) -> Option<()> {
        let db = self.get_database()?;
        let dropping_selected_db = self.db.as_ref().is_some_and(|db| db.name == *db_name);

        self.exec(async move {
            db.drop().await?;
            Ok(Event::DatabaseDropConfirmed(dropping_selected_db))
        });

        Some(())
    }

    fn create_db(&self, db_name: &str) -> Option<()> {
        let client = self.mongo_client.clone()?;
        let db = client.database(db_name);

        self.exec(async move {
            // HACK: the only way to create a db on the server is to create a collection
            // (or do something else that creates data). We could fix this by, instead
            // of doing this operation, just add a dummy list item for the database
            // and then it can chill there until a collection is added
            db.create_collection("coll").await?;
            Ok(Event::DatabaseCreationConfirmed)
        });

        Some(())
    }

    fn queue(&mut self, op: Operation) {
        self.queued_ops.insert(op);
    }

    pub fn exec_queued_ops(&mut self) {
        for op in &self.queued_ops {
            let _ = match op {
                Operation::Query(reset_state) => self.query(*reset_state),
                Operation::QueryCollections => self.query_collections(),
                Operation::QueryDatabases => self.query_dbs(),
                Operation::Count => self.count(),
                Operation::CreateCollection(coll_name) => self.create_coll(coll_name.clone()),
                Operation::DropCollection(coll_name) => self.drop_coll(coll_name),
                Operation::CreateDatabase(db_name) => self.create_db(db_name),
                Operation::DropDatabase(db_name) => self.drop_db(db_name),
            };
        }
        self.queued_ops = HashSet::default();
    }
}

impl Component for Client {
    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        // check for completed async operations
        let mut out = vec![];
        while let Ok(content) = self.response_recv.try_recv() {
            out.push(content);
        }

        // handle the event as normal
        match event {
            Event::ConnectionSelected(conn) => {
                self.connect(conn.connection_str.clone());
            }
            Event::ClientCreated(client) => {
                self.mongo_client = Some(client.clone());

                // TODO: should we query everything? if we're missing conn/db/coll
                // then it just won't run, and if we just hydrated data we want to
                // query as much as is relevant
                self.queue(Operation::Query(true));
                self.queue(Operation::QueryDatabases);
                self.queue(Operation::QueryCollections);
                self.queue(Operation::Count);
            }
            Event::DatabaseHighlighted(db) => {
                self.db = Some(db.clone());
                self.queue(Operation::QueryCollections);
            }
            Event::CollectionHighlighted(coll) => {
                self.coll = Some(coll.clone());
            }
            Event::CollectionSelected(coll) => {
                self.coll = Some(coll.clone());
                self.queue(Operation::Query(true));
                self.queue(Operation::Count);
            }
            Event::DocumentPageChanged(page) => {
                self.page = *page;
                self.queue(Operation::Query(true));
            }
            Event::DocFilterUpdated(doc) => {
                self.filter.clone_from(doc);
                self.queue(Operation::Query(true));
                self.queue(Operation::Count);
            }
            Event::DocUpdateComplete => self.queue(Operation::Query(false)),
            Event::DocInsertComplete | Event::DocDeleteComplete => {
                self.queue(Operation::Count);
                self.queue(Operation::Query(false));
            }
            Event::CollectionDropConfirmed(dropped_selected) => {
                if *dropped_selected {
                    self.coll = None;
                }
                self.queue(Operation::QueryCollections);
            }
            Event::CollectionCreationConfirmed => {
                self.queue(Operation::QueryCollections);
            }
            Event::InputConfirmed(InputKind::NewCollectionName, coll_name) => {
                self.queue(Operation::CreateCollection(coll_name.to_string()));
            }
            Event::DatabaseDropConfirmed(dropped_selected) => {
                if *dropped_selected {
                    self.db = None;
                    self.coll = None;
                }
                self.queue(Operation::QueryDatabases);
            }
            Event::DatabaseCreationConfirmed => {
                self.queue(Operation::QueryDatabases);
            }
            Event::InputConfirmed(InputKind::NewDatabaseName, coll_name) => {
                self.queue(Operation::CreateDatabase(coll_name.to_string()));
            }
            _ => (),
        }

        out.into_iter().map(Signal::from).collect()
    }

    fn handle_message(&mut self, message: &Message) -> Vec<Signal> {
        let mut out = vec![];

        match message.read_as_client() {
            Some(ClientAction::Connect(conn)) => self.connect(conn.connection_str.clone()),
            Some(ClientAction::DropDatabase(db)) => {
                self.queue(Operation::DropDatabase(db.name.clone()));
            }
            Some(ClientAction::DropCollection(db)) => {
                self.queue(Operation::DropCollection(db.name.clone()));
            }
            Some(ClientAction::UpdateDoc(doc)) => {
                if let Some(id) = doc.get("_id") {
                    self.update_doc(doc! { "_id": id }, doc.clone());
                } else {
                    out.push(Event::ErrorOccurred(
                        "Document does not have an `_id` field.".into(),
                    ));
                }
            }
            Some(ClientAction::InsertDoc(doc)) => {
                self.insert_doc(doc.clone());
            }
            Some(ClientAction::DeleteDoc(doc)) => {
                if let Some(id) = doc.get("_id") {
                    self.delete_doc(doc! { "_id": id });
                } else {
                    out.push(Event::ErrorOccurred(
                        "Document does not have an `_id` field.".into(),
                    ));
                }
            }
            Some(ClientAction::RefreshQueries) => {
                self.queue(Operation::Count);
                self.queue(Operation::Query(false));
            }
            None => {}
        }

        out.into_iter().map(Signal::from).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedClient {
    db: Option<Database>,
    coll: Option<Collection>,
    filter: Document,
    page: usize,
}

impl PersistedComponent for Client {
    type StorageType = PersistedClient;

    fn persist(&self) -> Self::StorageType {
        PersistedClient {
            db: self.db.clone(),
            coll: self.coll.clone(),
            filter: self.filter.clone(),
            page: self.page,
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.db = storage.db;
        self.coll = storage.coll;
        self.filter = storage.filter;
        self.page = storage.page;
    }
}
