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

pub trait Component {
    fn commands(&self) -> Vec<CommandGroup> {
        vec![]
    }

    fn handle_command(&mut self, _command: &Command) -> Vec<Signal> {
        vec![]
    }

    fn handle_raw_event(&mut self, _event: &CrosstermEvent) -> Vec<Signal> {
        vec![]
    }

    fn handle_event(&mut self, _event: &Event) -> Vec<Signal> {
        vec![]
    }

    fn handle_message(&mut self, _message: &Message) -> Vec<Signal> {
        vec![]
    }

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}

    fn focus(&self) {}

    fn is_focused(&self) -> bool {
        false
    }
}
