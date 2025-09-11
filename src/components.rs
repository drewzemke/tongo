use crate::system::{
    command::{Command, CommandGroup},
    event::Event,
    message::Message,
    signal::SignalQueue,
};
use crossterm::event::Event as CrosstermEvent;
use ratatui::{layout::Rect, Frame};

pub mod confirm_modal;
pub mod connection_screen;
pub mod documents;
pub mod help_modal;
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

    fn handle_command(&mut self, _command: &Command, _queue: &mut SignalQueue) {}

    fn handle_raw_event(&mut self, _event: &CrosstermEvent, _queue: &mut SignalQueue) {}

    fn handle_event(&mut self, _event: &Event, _queue: &mut SignalQueue) {}

    fn handle_message(&mut self, _message: &Message, _queue: &mut SignalQueue) {}

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}

    fn focus(&self) {}

    fn is_focused(&self) -> bool {
        false
    }
}
