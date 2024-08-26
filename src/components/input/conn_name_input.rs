use ratatui::prelude::{Frame, Rect};

use super::{DefaultFormatter, InnerInput};
use crate::{
    app::AppFocus,
    components::{connection_screen::ConnScreenFocus, Component, ComponentCommand, InputType},
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default)]
pub struct ConnNameInput {
    app_focus: Rc<RefCell<AppFocus>>,
    input: InnerInput<DefaultFormatter>,
}

impl ConnNameInput {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>, cursor_pos: Rc<RefCell<(u16, u16)>>) -> Self {
        let input = InnerInput::new("Connection Name", cursor_pos, DefaultFormatter::default());
        Self { app_focus, input }
    }

    pub fn value(&self) -> &str {
        self.input.state.value()
    }

    pub fn start_editing(&mut self) {
        self.input.start_editing();
    }

    pub fn stop_editing(&mut self) {
        self.input.stop_editing();
    }
}

impl Component<InputType> for ConnNameInput {
    fn is_focused(&self) -> bool {
        *self.app_focus.borrow() == AppFocus::ConnScreen(ConnScreenFocus::NameInput)
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::ConnScreen(ConnScreenFocus::NameInput);
    }

    fn commands(&self) -> Vec<crate::system::command::CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::Confirm], "enter", "next field"),
            CommandGroup::new(vec![Command::Back], "esc", "back"),
        ]
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        match command {
            ComponentCommand::RawEvent(event) => self.input.handle_raw_event(event),
            ComponentCommand::Command(command) => {
                if self.input.is_editing() {
                    match command {
                        Command::Confirm => vec![Event::FocusedForward],
                        Command::Back => vec![Event::FocusedBackward, Event::RawModeExited],
                        _ => vec![],
                    }

                    // see confirm and back events in previous version
                } else {
                    vec![]
                }
            }
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.input.render(frame, area, self.is_focused());
    }
}
