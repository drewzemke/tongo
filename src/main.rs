use anyhow::Result;
use app::App;
use clap::Parser;
use connection::Connection;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;

mod app;
mod connection;
mod files;
mod screens;
mod state;
mod tree;
mod widgets;

/// A TUI for viewing mongo databases.
#[derive(Parser)]
#[command(author)]
pub struct Args {
    /// The connection string for the mongo server
    #[arg(long, short)]
    url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let connection = args
        .url
        .map(|url| Connection::new("Unnamed Connection".to_string(), url));

    let mut terminal = setup_terminal()?;
    let mut app = App::new(connection);
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
