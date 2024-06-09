use anyhow::Result;
use app::App;
use clap::Parser;
use futures::TryStreamExt;
use mongodb::{bson::Bson, options::ClientOptions, Client};
use tui::{restore_terminal, setup_terminal};
use tui_tree_widget::TreeItem;

mod app;
mod tree;
mod tui;

#[derive(Parser)]
#[command(author)]
pub struct Args {
    /// The connection string for the mongo server
    #[arg(long, short)]
    url: String,

    /// The name of the database to connect to
    #[arg(long, short)]
    database: String,

    /// The name of the collection to load
    #[arg(long, short)]
    collection: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client_options = ClientOptions::parse(args.url).await?;
    let client = Client::with_options(client_options)?;

    let db = client.database(&args.database);

    let items: Vec<TreeItem<String>> = db
        .collection::<Bson>(&args.collection)
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
