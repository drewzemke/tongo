use crate::{
    command::{Command, CommandGroup},
    event::Event,
};
use crossterm::event::Event as CrosstermEvent;
use ratatui::{layout::Rect, Frame};

pub mod input;
pub mod list;

// FIXME: crappy name
pub enum ComponentCommand<'a> {
    Command(Command),
    RawEvent(&'a CrosstermEvent),
}

/// Enables multiple blanket impls of the `Component` trait
pub trait ComponentType {}
pub struct ListType;
pub struct InputType;
pub struct UniqueType;
impl ComponentType for ListType {}
impl ComponentType for InputType {}
impl ComponentType for UniqueType {}

pub trait Component<T: ComponentType> {
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
