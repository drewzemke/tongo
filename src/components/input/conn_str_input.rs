use super::{DefaultFormatter, InnerInput};
use crate::{
    components::{
        connection_screen::ConnScrFocus,
        tab::{CloneWithFocus, TabFocus},
        Component,
    },
    config::Config,
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        message::{ConnScreenAction, Message},
        signal::SignalQueue,
    },
};
use ratatui::prelude::{Frame, Rect};
use std::{cell::Cell, rc::Rc};

#[derive(Debug, Default, Clone)]
pub struct ConnStrInput {
    focus: Rc<Cell<TabFocus>>,
    input: InnerInput<DefaultFormatter>,
}

impl CloneWithFocus for ConnStrInput {
    fn clone_with_focus(&self, focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            focus,
            ..self.clone()
        }
    }
}

impl ConnStrInput {
    pub fn new(
        focus: Rc<Cell<TabFocus>>,
        cursor_pos: Rc<Cell<(u16, u16)>>,
        config: Config,
    ) -> Self {
        let input = InnerInput::new(
            "Connection String",
            cursor_pos,
            config,
            DefaultFormatter::default(),
        );
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
        self.focus.get() == TabFocus::ConnScr(ConnScrFocus::StringIn)
    }

    fn focus(&self) {
        self.focus.set(TabFocus::ConnScr(ConnScrFocus::StringIn));
    }

    fn commands(&self) -> Vec<crate::system::command::CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::Confirm], "confirm")
                .in_cat(CommandCategory::StatusBarOnly),
            CommandGroup::new(vec![Command::Back], "previous field")
                .in_cat(CommandCategory::StatusBarOnly),
        ]
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event, queue: &mut SignalQueue) {
        self.input.handle_raw_event(event, queue);
    }

    fn handle_command(&mut self, command: &Command, queue: &mut SignalQueue) {
        if !self.input.is_editing() {
            return;
        }

        match command {
            Command::Confirm => {
                queue.push(Message::to_conn_scr(ConnScreenAction::FinishEditingConn));
            }
            Command::Back => {
                queue.push(Message::to_conn_scr(ConnScreenAction::FocusConnNameInput));
            }
            _ => {},
        }
    }

    fn handle_event(&mut self, event: &Event, _queue: &mut SignalQueue) {
        match event {
            Event::ConnectionCreated(..) => self.input.set_value(""),
            Event::EditConnectionStarted(conn) => self.input.set_value(&conn.connection_str),
            _ => {}
        }
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
