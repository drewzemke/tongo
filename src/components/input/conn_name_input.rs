use super::{DefaultFormatter, InnerInput};
use crate::{
    components::{connection_screen::ConnScrFocus, tab::TabFocus, Component},
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        message::{AppAction, ConnScreenAction, Message},
        Signal,
    },
};
use ratatui::prelude::{Frame, Rect};
use std::{cell::Cell, rc::Rc};

#[derive(Debug, Default, Clone)]
pub struct ConnNameInput {
    focus: Rc<Cell<TabFocus>>,
    input: InnerInput<DefaultFormatter>,
}

impl ConnNameInput {
    pub fn new(focus: Rc<Cell<TabFocus>>, cursor_pos: Rc<Cell<(u16, u16)>>) -> Self {
        let input = InnerInput::new("Connection Name", cursor_pos, DefaultFormatter::default());
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

impl Component for ConnNameInput {
    fn is_focused(&self) -> bool {
        self.focus.get() == TabFocus::ConnScr(ConnScrFocus::NameIn)
    }

    fn focus(&self) {
        self.focus.set(TabFocus::ConnScr(ConnScrFocus::NameIn));
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::Confirm], "next field")
                .in_cat(CommandCategory::StatusBarOnly),
            CommandGroup::new(vec![Command::Back], "back").in_cat(CommandCategory::StatusBarOnly),
        ]
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        if !self.input.is_editing() {
            return vec![];
        }

        match command {
            Command::Confirm => {
                vec![Message::to_conn_scr(ConnScreenAction::FocusConnStrInput).into()]
            }
            Command::Back => {
                vec![
                    Message::to_conn_scr(ConnScreenAction::CancelEditingConn).into(),
                    Message::to_app(AppAction::ExitRawMode).into(),
                ]
            }
            _ => vec![],
        }
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event) -> Vec<Signal> {
        self.input.handle_raw_event(event)
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        match event {
            Event::ConnectionCreated(..) => self.input.set_value(""),
            Event::EditConnectionStarted(conn) => self.input.set_value(&conn.name),
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
    use crate::{model::connection::Connection, testing::ComponentTestHarness};

    #[test]
    fn reset_input_after_creating_connection() {
        let mut test = ComponentTestHarness::new(ConnNameInput::default());

        test.component_mut().start_editing();
        test.given_string("text!");

        // finish edit event
        test.given_event(Event::ConnectionCreated(Connection::default()));

        assert_eq!(test.component().value(), "");
    }

    #[test]
    fn populate_with_connection_on_edit() {
        let mut test = ComponentTestHarness::new(ConnNameInput::default());

        let connection = Connection::new("name".to_string(), "url".to_string());

        test.given_event(Event::EditConnectionStarted(connection));

        assert_eq!(test.component().value(), "name");
    }
}
