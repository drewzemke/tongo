use crate::system::{
    command::{Command, CommandGroup},
    event::Event,
    message::Message,
    Signal,
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
pub mod tab;
pub mod tab_bar;

// FIXME: crappy name
// OR: remove this enum entirely, and add a function called `handle_crossterm_event`
// to this trait
// ... yeahhhh that's the play
pub enum ComponentCommand {
    Command(Command),
    RawEvent(CrosstermEvent),
}

pub trait Component {
    fn commands(&self) -> Vec<CommandGroup> {
        vec![]
    }

    fn handle_command(&mut self, _command: &ComponentCommand) -> Vec<Signal> {
        vec![]
    }

    fn handle_event(&mut self, _event: &Event) -> Vec<Signal> {
        vec![]
    }

    fn handle_message(&mut self, _message: &Message) -> Vec<Signal> {
        vec![]
    }

    // TODO: default impl
    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}

    // TODO: default impl
    fn focus(&self);

    // TODO: default impl
    fn is_focused(&self) -> bool;
}
