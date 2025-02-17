use super::{
    input::{conn_name_input::ConnNameInput, conn_str_input::ConnStrInput},
    list::connections::{Connections, PersistedConnections},
    Component, ComponentCommand,
};
use crate::{
    app::AppFocus,
    connection::Connection,
    sessions::PersistedComponent,
    system::{command::CommandGroup, event::Event},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Clear},
};
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnScreenFocus {
    #[default]
    ConnList,
    NameInput,
    StringInput,
}

#[derive(Debug)]
pub struct ConnectionScreen {
    app_focus: Rc<RefCell<AppFocus>>,
    conn_list: Connections,
    conn_name_input: ConnNameInput,
    conn_str_input: ConnStrInput,
}

impl Default for ConnectionScreen {
    fn default() -> Self {
        let app_focus = Rc::new(RefCell::new(AppFocus::ConnScreen(
            ConnScreenFocus::ConnList,
        )));
        let cursor_pos = Rc::new(Cell::new((0, 0)));

        let conn_list = Connections::new(app_focus.clone(), vec![]);
        let conn_name_input = ConnNameInput::new(app_focus.clone(), cursor_pos.clone());
        let conn_str_input = ConnStrInput::new(app_focus.clone(), cursor_pos);

        Self {
            app_focus,
            conn_list,
            conn_name_input,
            conn_str_input,
        }
    }
}

impl ConnectionScreen {
    pub fn new(
        connection_list: Connections,
        app_focus: Rc<RefCell<AppFocus>>,
        cursor_pos: Rc<Cell<(u16, u16)>>,
    ) -> Self {
        let conn_name_input = ConnNameInput::new(app_focus.clone(), cursor_pos.clone());
        let conn_str_input = ConnStrInput::new(app_focus.clone(), cursor_pos);

        Self {
            app_focus,
            conn_list: connection_list,
            conn_name_input,
            conn_str_input,
        }
    }

    /// Narrows the shared `AppFocus` variable into the focus enum for this componenent
    fn internal_focus(&self) -> Option<ConnScreenFocus> {
        match &*self.app_focus.borrow() {
            AppFocus::ConnScreen(focus) => Some(focus.clone()),
            _ => None,
        }
    }
}

impl Component for ConnectionScreen {
    fn commands(&self) -> Vec<CommandGroup> {
        match self.internal_focus() {
            Some(ConnScreenFocus::ConnList) => self.conn_list.commands(),
            Some(ConnScreenFocus::NameInput) => self.conn_name_input.commands(),
            Some(ConnScreenFocus::StringInput) => self.conn_str_input.commands(),
            None => vec![],
        }
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        match self.internal_focus() {
            Some(ConnScreenFocus::ConnList) => self.conn_list.handle_command(command),
            Some(ConnScreenFocus::NameInput) => self.conn_name_input.handle_command(command),
            Some(ConnScreenFocus::StringInput) => self.conn_str_input.handle_command(command),
            None => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        match event {
            Event::NewConnectionStarted => {
                self.conn_name_input.focus();
                self.conn_name_input.start_editing();
                out.push(Event::RawModeEntered);
            }
            Event::FocusedForward => match self.internal_focus() {
                Some(ConnScreenFocus::NameInput) => {
                    self.conn_name_input.stop_editing();
                    self.conn_str_input.focus();
                    self.conn_str_input.start_editing();
                }
                Some(ConnScreenFocus::StringInput) => {
                    let conn = Connection::new(
                        self.conn_name_input.value().to_string(),
                        self.conn_str_input.value().to_string(),
                    );
                    out.push(Event::ConnectionCreated(conn));
                }
                Some(ConnScreenFocus::ConnList) | None => {}
            },
            Event::FocusedBackward => match self.internal_focus() {
                Some(ConnScreenFocus::NameInput) => {
                    self.conn_list.focus();
                    self.conn_name_input.stop_editing();
                }
                Some(ConnScreenFocus::StringInput) => {
                    self.conn_name_input.focus();
                    self.conn_name_input.start_editing();
                    self.conn_str_input.stop_editing();
                }
                Some(ConnScreenFocus::ConnList) | None => {}
            },
            Event::ConnectionCreated(conn) => {
                self.conn_list.items.push(conn.clone());
                Connection::write_to_storage(&self.conn_list.items).unwrap_or_else(|_| {
                    out.push(Event::ErrorOccurred(
                        "Could not save updated connections.".to_string(),
                    ));
                });
            }
            _ => {}
        }
        out.append(&mut self.conn_list.handle_event(event));
        out.append(&mut self.conn_name_input.handle_event(event));
        out.append(&mut self.conn_str_input.handle_event(event));
        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.conn_list.render(frame, area);

        // render new connection inputs in an overlay
        if self.internal_focus() == Some(ConnScreenFocus::NameInput)
            || self.internal_focus() == Some(ConnScreenFocus::StringInput)
        {
            let overlay_layout = Layout::default()
                .constraints([Constraint::Fill(1)])
                .horizontal_margin(3)
                .vertical_margin(2)
                .split(area);
            let overlay = overlay_layout[0];

            let inputs_layout = Layout::default()
                .constraints(vec![
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ])
                .horizontal_margin(2)
                .split(overlay);

            // render container for the inputs
            frame.render_widget(Clear, overlay);
            let block = Block::bordered()
                .title("New Connection")
                .style(Style::default().green());
            frame.render_widget(block, overlay);

            self.conn_name_input.render(frame, inputs_layout[1]);
            self.conn_str_input.render(frame, inputs_layout[3]);
        }
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::ConnScreen(ConnScreenFocus::default());
    }

    fn is_focused(&self) -> bool {
        matches!(*self.app_focus.borrow(), AppFocus::ConnScreen(..))
    }
}

#[derive(Serialize, Deserialize)]
pub struct PersistedConnectionScreen {
    pub conn_list: PersistedConnections,
}

impl PersistedComponent for ConnectionScreen {
    type StorageType = PersistedConnectionScreen;

    fn persist(&self) -> Self::StorageType {
        PersistedConnectionScreen {
            conn_list: self.conn_list.persist(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) -> Vec<Event> {
        self.conn_list.hydrate(storage.conn_list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{system::command::Command, testing::ComponentTestHarness};

    #[test]
    fn create_new_conn() {
        let mut test = ComponentTestHarness::new(ConnectionScreen::default());
        test.given_command(Command::CreateNew);

        // name of connection
        test.given_string("local");
        test.given_command(Command::Confirm);

        // connection string url
        test.given_string("url");
        test.given_command(Command::Confirm);

        test.expect_event(|e| matches!(e, Event::ConnectionCreated(..)));
    }
}
