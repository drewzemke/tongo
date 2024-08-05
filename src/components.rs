use crate::{
    command::{Command, CommandGroup},
    event::Event,
};
use crossterm::event::Event as CrosstermEvent;
use ratatui::{layout::Rect, Frame};

pub mod connection_list;
pub mod generic;

// FIXME: crappy name
#[allow(clippy::module_name_repetitions)]
pub enum ComponentCommand<'a> {
    Command(Command),
    RawEvent(&'a CrosstermEvent),
}

pub trait Component {
    fn commands(&self) -> Vec<CommandGroup> {
        vec![]
    }

    fn handle_command(&mut self, _command: ComponentCommand) -> Vec<Event> {
        vec![]
    }

    fn handle_event(&mut self, _event: Event) -> bool {
        false
    }

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}
}
