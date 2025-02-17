use anyhow::{Context, Result};
use app::{App, PersistedApp};
use clap::Parser;
use connection::Connection;
use ratatui::{backend::CrosstermBackend, Terminal};
use sessions::PersistedComponent;
use std::{io::Stdout, path::PathBuf};
use utils::files::FileManager;

mod app;
mod client;
mod components;
mod connection;
mod key_map;
mod sessions;
mod system;
#[cfg(test)]
mod testing;
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
    init_tracing()?;

    tracing::info!(
        "Started {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );

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
            tracing::info!("Loading previous app state");
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

#[tracing::instrument(skip())]
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    tracing::debug!("Setting up terminal");

    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture,
        crossterm::event::EnableFocusChange
    )?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    Ok(terminal)
}

#[tracing::instrument(skip(terminal))]
fn restore_terminal(mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture,
        crossterm::event::DisableFocusChange
    )?;
    terminal.show_cursor()?;

    tracing::debug!("Terminal restored");

    Ok(())
}

/// Initializes the `tracing` system for logging.
fn init_tracing() -> Result<()> {
    let log_env = format!("{}_LOGLEVEL", env!("CARGO_PKG_NAME").to_uppercase());
    let log_filename = format!("{}.log", env!("CARGO_PKG_NAME"));

    let log_file_path = if let Some(dir) = dirs::data_local_dir() {
        dir.join(env!("CARGO_PKG_NAME")).join(log_filename)
    } else {
        PathBuf::from(".")
            .join(format!(".{}", env!("CARGO_PKG_NAME")))
            .join(log_filename)
    };

    let log_file = std::fs::File::create(log_file_path)?;

    // set up the logging level env var
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG")
            .or_else(|_| std::env::var(log_env))
            .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME"))),
    );

    let subscriber = tracing_subscriber::fmt()
        .with_line_number(true)
        // TODO: default to `info` if no env is set
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(log_file)
        .pretty()
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
