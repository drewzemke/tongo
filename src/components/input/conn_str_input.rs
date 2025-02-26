use ratatui::prelude::{Frame, Rect};

use super::{DefaultFormatter, InnerInput};
use crate::{
    components::{connection_screen::ConnScrFocus, tab::TabFocus, Component, ComponentCommand},
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Debug, Default)]
pub struct ConnStrInput {
    focus: Rc<RefCell<TabFocus>>,
    input: InnerInput<DefaultFormatter>,
}

impl ConnStrInput {
    pub fn new(focus: Rc<RefCell<TabFocus>>, cursor_pos: Rc<Cell<(u16, u16)>>) -> Self {
        let input = InnerInput::new("Connection String", cursor_pos, DefaultFormatter::default());
        Self { focus, input }
    }

    pub fn value(&self) -> &str {
        self.input.value()
    }

    pub const fn start_editing(&mut self) {
        self.input.start_editing();
    }

    pub const fn stop_editing(&mut self) {
        self.input.stop_editing();
    }
}

impl Component for ConnStrInput {
    fn is_focused(&self) -> bool {
        *self.focus.borrow() == TabFocus::ConnScr(ConnScrFocus::StringIn)
    }

    fn focus(&self) {
        *self.focus.borrow_mut() = TabFocus::ConnScr(ConnScrFocus::StringIn);
    }

    fn commands(&self) -> Vec<crate::system::command::CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::Confirm], "confirm"),
            CommandGroup::new(vec![Command::Back], "prev field"),
        ]
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        match command {
            ComponentCommand::RawEvent(event) => self.input.handle_raw_event(event),
            ComponentCommand::Command(command) => {
                if self.input.is_editing() {
                    match command {
                        Command::Confirm => vec![Event::FocusedForward, Event::RawModeExited],
                        Command::Back => vec![Event::FocusedBackward],
                        _ => vec![],
                    }

                    // see confirm and back events in previous version
                } else {
                    vec![]
                }
            }
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        match event {
            Event::ConnectionCreated(..) => self.input.set_value(""),
            Event::EditConnectionStarted(conn) => self.input.set_value(&conn.connection_str),
            _ => {}
        }
        vec![]
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.input.render(frame, area, self.is_focused());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connection::Connection, testing::ComponentTestHarness};

    #[test]
    fn reset_input_after_creating_connection() {
        let mut test = ComponentTestHarness::new(ConnStrInput::default());

        test.component_mut().start_editing();
        test.given_string("text!");

        // finish edit event
        test.given_event(Event::ConnectionCreated(Connection::default()));

        assert_eq!(test.component().value(), "");
    }

    #[test]
    fn populate_with_connection_on_edit() {
        let mut test = ComponentTestHarness::new(ConnStrInput::default());

        let connection = Connection::new("name".to_string(), "url".to_string());

        test.given_event(Event::EditConnectionStarted(connection));

        assert_eq!(test.component().value(), "url");
    }
}
