use crate::system::{
    command::{Command, CommandGroup},
    event::Event,
};
use crossterm::event::Event as CrosstermEvent;
use ratatui::{layout::Rect, Frame};

pub mod confirm_modal;
pub mod connection_screen;
pub mod documents;
pub mod input;
pub mod list;
pub mod primary_screen;
pub mod status_bar;

// FIXME: crappy name
pub enum ComponentCommand {
    Command(Command),
    RawEvent(CrosstermEvent),
}

// NOTE: this only used in tests for now
#[cfg(test)]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
#[cfg(test)]
impl ComponentCommand {
    pub const fn raw_from_key_code(key_code: KeyCode) -> Self {
        let event = CrosstermEvent::Key(KeyEvent::new(key_code, KeyModifiers::empty()));
        Self::RawEvent(event)
    }
}

pub trait Component {
    fn commands(&self) -> Vec<CommandGroup> {
        vec![]
    }

    fn handle_command(&mut self, _command: &ComponentCommand) -> Vec<Event> {
        vec![]
    }

    fn handle_event(&mut self, _event: &Event) -> Vec<Event> {
        vec![]
    }

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}

    fn focus(&self);

    fn is_focused(&self) -> bool;
}
