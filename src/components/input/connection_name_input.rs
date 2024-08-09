use super::{ComponentCommand, InputComponent};
use crate::{
    command::{Command, CommandGroup},
    event::Event,
};
use tui_input::Input;

#[derive(Debug, Default)]
pub struct ConnectionNameInput {
    pub input: Input,
    pub editing: bool,
    pub cursor_pos: (u16, u16),
}

impl InputComponent for ConnectionNameInput {
    fn title() -> &'static str {
        "Name"
    }

    // TODO: make this dynamic
    fn is_focused(&self) -> bool {
        true
    }

    fn is_editing(&self) -> bool {
        self.editing
    }

    fn input(&mut self) -> &mut Input {
        &mut self.input
    }

    fn cursor_pos(&mut self) -> &mut (u16, u16) {
        &mut self.cursor_pos
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::Confirm], "enter", "next field"),
            CommandGroup::new(vec![Command::Back], "esc", "back"),
        ]
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<crate::event::Event> {
        let ComponentCommand::Command(command) = command else {
            return vec![];
        };
        match command {
            Command::Confirm => vec![Event::FocusedForward],
            Command::Back => vec![Event::FocusedBackward, Event::RawModeExited],
            _ => vec![],
        }
    }
}
