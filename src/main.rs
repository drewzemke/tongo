use anyhow::Result;
use app::App;
use clap::Parser;
use connection::Connection;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;

mod app;
mod connection;
mod files;
mod key_hint;
mod screens;
mod state;
mod tree;
mod widgets;

/// A TUI for viewing mongo databases.
#[derive(Parser)]
#[command(author)]
pub struct Args {
    #[clap(flatten)]
    auto_connect: Option<AutoConnectArgs>,
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
