use super::{
    input::{conn_name_input::ConnNameInput, conn_str_input::ConnStrInput},
    list::connections::{Connections, PersistedConnections},
    tab::TabFocus,
    Component, ComponentCommand,
};
use crate::{
    connection::{Connection, ConnectionManager},
    persistence::PersistedComponent,
    system::{
        command::CommandGroup,
        event::Event,
        message::{AppAction, ClientAction, ConnScreenAction, Message},
        Signal,
    },
    utils::storage::FileStorage,
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
pub enum ConnScrFocus {
    #[default]
    ConnList,
    NameIn,
    StringIn,
}

#[derive(Debug, Clone)]
pub struct ConnectionScreen {
    focus: Rc<RefCell<TabFocus>>,
    pub connection_manager: ConnectionManager,
    conn_list: Connections,
    conn_name_input: ConnNameInput,
    conn_str_input: ConnStrInput,
    editing_conn: Option<Connection>,
}

impl Default for ConnectionScreen {
    fn default() -> Self {
        let focus = Rc::new(RefCell::new(TabFocus::ConnScr(ConnScrFocus::ConnList)));
        let cursor_pos = Rc::new(Cell::new((0, 0)));

        let storage = Rc::new(FileStorage::default());

        let connection_manager = ConnectionManager::new(vec![], storage.clone());
        let conn_list = Connections::new(focus.clone(), connection_manager.clone());
        let conn_name_input = ConnNameInput::new(focus.clone(), cursor_pos.clone());
        let conn_str_input = ConnStrInput::new(focus.clone(), cursor_pos);

        Self {
            focus,
            connection_manager,
            conn_list,
            conn_name_input,
            conn_str_input,
            editing_conn: None,
        }
    }
}

impl ConnectionScreen {
    pub fn new(
        connection_list: Connections,
        app_focus: Rc<RefCell<TabFocus>>,
        cursor_pos: Rc<Cell<(u16, u16)>>,
        connection_manager: ConnectionManager,
    ) -> Self {
        let conn_name_input = ConnNameInput::new(app_focus.clone(), cursor_pos.clone());
        let conn_str_input = ConnStrInput::new(app_focus.clone(), cursor_pos);

        Self {
            focus: app_focus,
            conn_list: connection_list,
            conn_name_input,
            conn_str_input,
            connection_manager,
            editing_conn: None,
        }
    }

    /// Narrows the shared `AppFocus` variable into the focus enum for this componenent
    fn internal_focus(&self) -> Option<ConnScrFocus> {
        match &*self.focus.borrow() {
            TabFocus::ConnScr(focus) => Some(focus.clone()),
            _ => None,
        }
    }
}

impl Component for ConnectionScreen {
    fn commands(&self) -> Vec<CommandGroup> {
        match self.internal_focus() {
            Some(ConnScrFocus::ConnList) => self.conn_list.commands(),
            Some(ConnScrFocus::NameIn) => self.conn_name_input.commands(),
            Some(ConnScrFocus::StringIn) => self.conn_str_input.commands(),
            None => vec![],
        }
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Signal> {
        match self.internal_focus() {
            Some(ConnScrFocus::ConnList) => self.conn_list.handle_command(command),
            Some(ConnScrFocus::NameIn) => self.conn_name_input.handle_command(command),
            Some(ConnScrFocus::StringIn) => self.conn_str_input.handle_command(command),
            None => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        let mut out = vec![];
        match event {
            Event::ConnectionCreated(conn) => {
                self.connection_manager
                    .add_connection(conn.clone())
                    .unwrap_or_else(|_| {
                        out.push(
                            Event::ErrorOccurred("Could not save updated connections.".to_string())
                                .into(),
                        );
                    });
            }
            _ => {}
        }
        out.append(&mut self.conn_list.handle_event(event));
        out.append(&mut self.conn_name_input.handle_event(event));
        out.append(&mut self.conn_str_input.handle_event(event));
        out
    }

    fn handle_message(&mut self, message: &Message) -> Vec<Signal> {
        let mut out = vec![];

        match message.read_as_conn_scr() {
            Some(ConnScreenAction::StartNewConn) => {
                self.conn_name_input.focus();
                self.conn_name_input.start_editing();
                out.push(Message::to_app(AppAction::EnterRawMode).into());
            }
            Some(ConnScreenAction::StartEditingConn(conn)) => {
                self.conn_name_input.focus();
                self.conn_name_input.start_editing();
                self.editing_conn = Some(conn.clone());
                out.push(Message::to_app(AppAction::EnterRawMode).into());
                out.push(Event::EditConnectionStarted(conn.clone()).into());
            }
            Some(ConnScreenAction::FocusConnNameInput) => {
                self.conn_name_input.focus();
                self.conn_name_input.start_editing();
                self.conn_str_input.stop_editing();
            }
            Some(ConnScreenAction::FocusConnStrInput) => {
                self.conn_name_input.stop_editing();
                self.conn_str_input.focus();
                self.conn_str_input.start_editing();
            }
            Some(ConnScreenAction::FinishEditingConn) => {
                if let Some(mut editing_conn) = self.editing_conn.take() {
                    editing_conn.name = self.conn_name_input.value().to_string();
                    editing_conn.connection_str = self.conn_str_input.value().to_string();

                    // update the connection in storage
                    self.connection_manager
                        .update_connection(&editing_conn)
                        .unwrap_or_else(|_| {
                            out.push(
                                Event::ErrorOccurred(
                                    "Could not save updated connections.".to_string(),
                                )
                                .into(),
                            );
                        });

                    self.conn_list.focus();
                    out.push(Event::ConnectionUpdated(editing_conn).into());
                } else {
                    let conn = Connection::new(
                        self.conn_name_input.value().to_string(),
                        self.conn_str_input.value().to_string(),
                    );
                    out.push(Event::ConnectionCreated(conn.clone()).into());
                    out.push(Message::to_client(ClientAction::Connect(conn)).into());
                };
                out.push(Message::to_app(AppAction::ExitRawMode).into());
            }
            Some(ConnScreenAction::CancelEditingConn) => {
                self.conn_name_input.stop_editing();
                self.conn_list.focus();
                out.push(Message::to_app(AppAction::ExitRawMode).into());
            }
            None => {}
        }

        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.conn_list.render(frame, area);

        // render new connection inputs in an overlay
        if self.internal_focus() == Some(ConnScrFocus::NameIn)
            || self.internal_focus() == Some(ConnScrFocus::StringIn)
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
        *self.focus.borrow_mut() = TabFocus::ConnScr(ConnScrFocus::default());
    }

    fn is_focused(&self) -> bool {
        matches!(*self.focus.borrow(), TabFocus::ConnScr(..))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.conn_list.hydrate(storage.conn_list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        system::command::Command,
        testing::{mock_storage::MockStorage, ComponentTestHarness},
    };

    impl ConnectionScreen {
        fn new_mock(connections: Vec<Connection>) -> Self {
            let storage = Rc::new(MockStorage::default());
            let connection_manager = ConnectionManager::new(connections, storage);
            ConnectionScreen {
                conn_list: Connections {
                    connection_manager: connection_manager.clone(),
                    ..Default::default()
                },
                connection_manager,
                ..Default::default()
            }
        }
    }

    #[test]
    fn create_new_conn() {
        let mut test = ComponentTestHarness::new(ConnectionScreen::new_mock(vec![]));
        test.given_command(Command::CreateNew);

        // name of connection
        test.given_string("local");
        test.given_command(Command::Confirm);

        // connection string url
        test.given_string("url");
        test.given_command(Command::Confirm);

        test.expect_event(|e| matches!(e, Event::ConnectionCreated(c) if c.name == "local"));
        test.expect_message(|m| {
            matches!(
                m.read_as_client(),
                Some(ClientAction::Connect(c)) if c.name == "local"
            )
        });
    }

    #[test]
    fn edit_connection() {
        let connection = Connection::new("conn".to_string(), "url".to_string());
        let mut test =
            ComponentTestHarness::new(ConnectionScreen::new_mock(vec![connection.clone()]));

        // start editing
        test.given_command(Command::NavDown);
        test.given_command(Command::Edit);
        test.expect_event(|e| matches!(e, Event::EditConnectionStarted(c) if c.name == "conn"));

        // focus should be on name
        // move to url field
        test.given_command(Command::Confirm);

        // connection string url
        test.given_key("backspace");
        test.given_key("backspace");
        test.given_key("backspace");
        test.given_key("backspace");
        test.given_string("new_url");
        test.given_command(Command::Confirm);

        test.expect_event(
            |e| matches!(e, Event::ConnectionUpdated(c) if c.connection_str == "new_url" && c.id() == connection.id()),
        );
    }
}
