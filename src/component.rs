use ratatui::{layout::Rect, Frame};

use crate::command::{Command, CommandGroup};

pub trait Component {
    fn commands(&self) -> Vec<CommandGroup> {
        vec![]
    }

    // TODO: eventually should mutate internal component state
    // TODO: this should return a list of events
    fn handle_command(_command: Command) {}

    fn render(&self, _frame: &mut Frame, _area: Rect);
}
