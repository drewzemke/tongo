use anyhow::Result;
use clap::Parser;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io::Stdout, path::PathBuf, rc::Rc};

use tongo::{
    app::App,
    config::Config,
    model::connection::Connection,
    persistence::PersistedComponent,
    utils::storage::{get_app_data_path, FileStorage, Storage},
};

/// A TUI for viewing and interacting with `MongoDB` databases.
#[derive(Parser)]
#[command(author)]
pub struct Args {
    #[clap(flatten)]
    load_options: Option<LoadOptions>,
}

/// Detemines how (if at all) the app will automatically load a connection
/// on startup
#[derive(Debug, clap::Args, Clone)]
#[group(required = false, multiple = false)]
pub struct LoadOptions {
    /// Restore the most-recently-closed session
    #[arg(long, short)]
    last: bool,

    /// Automatically connect to a (previously stored) connection
    #[arg(long, short)]
    connection: Option<String>,

    /// Automatically connect to a given connection string
    #[arg(long, short)]
    url: Option<String>,
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

    let storage = FileStorage::init()?;

    // load config
    let config: Config = storage.read_config()?.try_into()?;

    // load connections
    let stored_connections = storage.read_connections().unwrap_or_default();

    // connect to a connection based on command line argument (if applicable)
    let connection =
        args.load_options
            .clone()
            .and_then(|group| match (group.url, group.connection) {
                (Some(url), _) => Some(Connection::new("Unnamed Connection".to_string(), url)),
                (_, Some(conn_name)) => stored_connections
                    .iter()
                    .find(|c| c.name.to_lowercase() == conn_name.to_lowercase())
                    .cloned(),
                _ => None,
            });

    let mut terminal = setup_terminal()?;
    let mut app = App::new(
        connection,
        stored_connections,
        config,
        Rc::new(storage.clone()),
    );

    // load stored app state

    if args
        .load_options
        .is_some_and(|loading_args| loading_args.last)
    {
        if let Ok(session) = storage.read_last_session() {
            tracing::info!("Loading previous app state");
            app.hydrate(session);
            tracing::info!("Done loading app");
        }
    }

    let res = app.run(&mut terminal);

    restore_terminal(terminal)?;

    if let Err(err) = res {
        println!("{err}");
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

    // FIXME: could probably consolidate this better with the stuff in `files` module
    let log_file_path = if let Ok(dir) = get_app_data_path() {
        dir.join(log_filename)
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
