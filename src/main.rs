use anyhow::{Context, Result};
use app::{App, PersistedApp};
use clap::Parser;
use connection::Connection;
use ratatui::{backend::CrosstermBackend, Terminal};
use sessions::PersistedComponent;
use std::io::Stdout;
use utils::files::FileManager;

mod app;
mod client;
mod components;
mod connection;
mod sessions;
mod system;
mod utils;

/// A TUI for viewing mongo databases.
#[derive(Parser)]
#[command(author)]
pub struct Args {
    #[clap(flatten)]
    auto_connect: Option<AutoConnectArgs>,

    /// Restore the most-recently-closed session
    #[arg(long, short)]
    last: bool,
}

#[derive(Debug, clap::Args)]
#[group(required = false, multiple = false)]
pub struct AutoConnectArgs {
    /// Automatically connect to a given connection string
    #[arg(long, short)]
    url: Option<String>,

    /// Automatically connect to a (previously stored) connection
    #[arg(long, short)]
    connection: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // load connections
    let store_connections = Connection::read_from_storage().unwrap_or_default();

    // connect to a connection based on command line argument (if applicable)
    let connection = args
        .auto_connect
        .and_then(|group| match (group.url, group.connection) {
            (Some(url), _) => Some(Connection::new("Unnamed Connection".to_string(), url)),
            (_, Some(conn_name)) => store_connections
                .iter()
                .find(|c| c.name.to_lowercase() == conn_name.to_lowercase())
                .cloned(),
            _ => unreachable!(),
        });

    let mut terminal = setup_terminal()?;
    let mut app = App::new(connection, store_connections);

    // load stored app state
    if args.last {
        let session = FileManager::init()
            .and_then(|fm| fm.read_data("last-session.json".into()))
            .context("TODO: better error handling")
            .and_then(|file| {
                serde_json::from_str::<PersistedApp>(&file).context("TODO: better error handling")
            });
        if let Ok(session) = session {
            app.hydrate(session);
        }
    }

    let res = app.run(&mut terminal);

    restore_terminal(terminal)?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
