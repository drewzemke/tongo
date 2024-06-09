pub use anyhow::Result;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
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

pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
