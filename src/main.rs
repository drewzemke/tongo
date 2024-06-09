use anyhow::Result;
use app::App;
use futures::TryStreamExt;
use mongodb::{bson::Bson, options::ClientOptions, Client};
use tui::{restore_terminal, setup_terminal};
use tui_tree_widget::TreeItem;

const DB_NAME: &str = "deeb";
const COLLECTION_NAME: &str = "stuff";

mod app;
mod tree;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;

    let db = client.database(DB_NAME);

    let items: Vec<TreeItem<String>> = db
        .collection::<Bson>(COLLECTION_NAME)
        .find(None, None)
        .await?
        .try_collect::<Vec<Bson>>()
        .await?
        .iter()
        .map(|x| tree::top_level_document(x.as_document().unwrap()))
        .collect();

    let mut terminal = setup_terminal()?;

    // App
    let mut app = App::new(items);
    let res = app.run(&mut terminal);

    restore_terminal(terminal)?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
