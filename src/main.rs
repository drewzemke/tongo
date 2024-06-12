use anyhow::Result;
use app::App;
use clap::Parser;
use mongodb::{options::ClientOptions, Client};
use tui::{restore_terminal, setup_terminal};

mod app;
mod tree;
mod tui;

/// A TUI for viewing mongo databases.
#[derive(Parser)]
#[command(author)]
pub struct Args {
    /// The connection string for the mongo server
    #[arg(long, short)]
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client_options = ClientOptions::parse(args.url).await?;
    let client = Client::with_options(client_options)?;

    let mut terminal = setup_terminal()?;
    let mut app = App::new(client);
    let res = app.run(&mut terminal);

    restore_terminal(terminal)?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
