use std::{
    cell::{Cell, Ref},
    rc::Rc,
};

use super::InnerList;
use crate::{
    components::{
        confirm_modal::ConfirmKind,
        connection_screen::ConnScrFocus,
        list::InnerListItem,
        tab::{CloneWithFocus, TabFocus},
        Component,
    },
    config::{color_map::ColorKey, Config},
    model::connection::{Connection, ConnectionManager},
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        message::{ClientAction, ConnScreenAction, Message, TabAction},
        signal::SignalQueue,
    },
};
use crossterm::event::KeyCode;
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, ListItem, Paragraph},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
pub struct Connections {
    pub connection_manager: ConnectionManager,
    pub list: InnerList<Connection>,
    pub config: Config,
}

impl InnerListItem for Connection {
    fn to_list_item(&self, config: &Config) -> ListItem<'_> {
        let masked_conn_str = Connections::mask_password(&self.connection_str);

        let text = Text::from(vec![
            Line::from(self.name.clone()).fg(config.color_map.get(&ColorKey::FgPrimary)),
            Line::from(format!(" {masked_conn_str}"))
                .fg(config.color_map.get(&ColorKey::FgSecondary)),
        ]);
        ListItem::new(text)
    }

    fn to_search_string(&self) -> String {
        format!("{} {}", self.name, self.connection_str)
    }
}

impl CloneWithFocus for Connections {
    fn clone_with_focus(&self, focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            list: self.list.clone_with_focus(focus),
            ..self.clone()
        }
    }
}

impl Connections {
    pub fn new(
        focus: Rc<Cell<TabFocus>>,
        config: Config,
        connection_manager: ConnectionManager,
    ) -> Self {
        let mut comp = Self {
            connection_manager,
            list: InnerList::new(
                "Connections",
                config.clone(),
                focus,
                TabFocus::ConnScr(ConnScrFocus::ConnList),
            ),
            config,
        };
        comp.sync_connections();
        comp
    }

    fn get_selected_conn(&self) -> Option<Ref<'_, Connection>> {
        let index = self.list.state.selected()?;
        Ref::filter_map(self.connection_manager.connections(), |v| v.get(index)).ok()
    }

    pub fn select_conn(&mut self, conn: &Connection) {
        let index = self
            .connection_manager
            .connections()
            .iter()
            .position(|c| c == conn);
        self.list.state.select(index);
    }

    fn mask_password(conn_str: &str) -> String {
        let Some((before_slashes, after_slashes)) = conn_str.split_once("//") else {
            return String::from(conn_str);
        };
        let Some((user_and_pw, after_at)) = after_slashes.split_once('@') else {
            return String::from(conn_str);
        };
        let Some((user, _)) = user_and_pw.split_once(':') else {
            return String::from(conn_str);
        };
        format!("{before_slashes}//{user}:******@{after_at}")
    }

    fn set_connections(&mut self, connections: Vec<Connection>) {
        self.connection_manager.set_connections(connections);
        self.sync_connections();
    }

    fn sync_connections(&mut self) {
        let conns = self.connection_manager.connections();
        self.list.set_items(&conns);
    }
}

impl Component for Connections {
    fn is_focused(&self) -> bool {
        self.list.is_focused()
    }

    fn focus(&self) {
        self.list.focus();
    }

    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = self.list.commands();
        out.append(&mut vec![
            CommandGroup::new(vec![Command::Confirm], "connect")
                .in_cat(CommandCategory::CollActions),
            CommandGroup::new(vec![Command::CreateNew], "new connection")
                .in_cat(CommandCategory::CollActions),
            CommandGroup::new(vec![Command::Edit], "edit connection")
                .in_cat(CommandCategory::CollActions),
            CommandGroup::new(vec![Command::Delete], "delete connection")
                .in_cat(CommandCategory::CollActions),
        ]);
        out
    }

    fn handle_command(&mut self, command: &Command, queue: &mut SignalQueue) {
        let was_capturing = self.is_capturing_input();

        self.list.handle_command(command, queue);

        if was_capturing {
            return;
        }

        match command {
            Command::Confirm => {
                if let Some(conn) = self.get_selected_conn() {
                    queue.push(Event::ConnectionSelected(conn.clone()));
                    queue.push(Message::to_client(ClientAction::Connect(conn.clone())));
                }
            }
            Command::CreateNew => {
                queue.push(Message::to_conn_scr(ConnScreenAction::StartNewConn));
            }
            Command::Edit => {
                if let Some(conn) = self.get_selected_conn() {
                    queue.push(Message::to_conn_scr(ConnScreenAction::StartEditingConn(
                        conn.clone(),
                    )));
                }
            }
            Command::Delete => {
                queue.push(Message::to_tab(TabAction::RequestConfirmation(
                    ConfirmKind::DeleteConnection,
                )));
            }
            _ => {}
        }
    }

    fn handle_event(&mut self, event: &Event, queue: &mut SignalQueue) {
        match event {
            Event::ConfirmYes(Command::Delete) if self.is_focused() => {
                // only process the confirmation if this component is focused

                let Some(id) = self.get_selected_conn().map(|c| *c.id()) else {
                    return;
                };

                if matches!(self.connection_manager.delete_connection(id), Ok(())) {
                    queue.push(Event::ConnectionDeleted);
                } else {
                    queue.push(Event::ErrorOccurred(
                        "An error occurred while saving connection preferences".into(),
                    ));
                }
            }

            Event::ConnectionCreated(_)
            | Event::ConnectionUpdated(_)
            | Event::ConnectionDeleted
            | Event::TabChanged => self.sync_connections(),

            _ => {}
        }
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event, queue: &mut SignalQueue) {
        self.list.handle_raw_event(event, queue);
    }

    fn is_capturing_input(&self) -> bool {
        self.list.is_capturing_input()
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.connection_manager.connections().is_empty() {
            let (border_color, bg_color) = if self.is_focused() {
                (
                    self.config.color_map.get(&ColorKey::PanelActiveBorder),
                    self.config.color_map.get(&ColorKey::PanelActiveBg),
                )
            } else {
                (
                    self.config.color_map.get(&ColorKey::PanelInactiveBorder),
                    self.config.color_map.get(&ColorKey::PanelInactiveBg),
                )
            };
            let block = Block::bordered()
                .title(" Connections ")
                .bg(bg_color)
                .border_style(Style::default().fg(border_color));
            frame.render_widget(block, area);

            let layout = Layout::vertical([2]).flex(Flex::Center).split(area);
            let create_new_key = self
                .config
                .key_map
                .key_for_command(Command::CreateNew)
                .unwrap_or_else(|| KeyCode::Char('?').into());

            let line1 = Line::from(
                "You don't have any connections."
                    .fg(self.config.color_map.get(&ColorKey::FgSecondary)),
            )
            .centered();
            let line2 = Line::from(vec![
                "Press ".fg(self.config.color_map.get(&ColorKey::FgSecondary)),
                create_new_key
                    .to_string()
                    .fg(self.config.color_map.get(&ColorKey::FgPrimary)),
                " to create one!".fg(self.config.color_map.get(&ColorKey::FgSecondary)),
            ])
            .centered();

            frame.render_widget(Paragraph::new(vec![line1, line2]), layout[0]);
        } else {
            self.list.render(frame, area);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedConnections {
    pub selected_conn: Option<Connection>,
    connections: Vec<Connection>,
}

impl PersistedComponent for Connections {
    type StorageType = PersistedConnections;

    fn persist(&self) -> Self::StorageType {
        PersistedConnections {
            selected_conn: self.get_selected_conn().map(|conn| conn.clone()),
            connections: self.connection_manager.connections().clone(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.set_connections(storage.connections);

        if let Some(conn) = storage.selected_conn {
            self.select_conn(&conn);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{mock_storage::MockStorage, ComponentTestHarness};

    #[test]
    fn test_mask_password() {
        let conn_str = "mongodb://user:D1fficultP%40ssw0rd@cluster0.example.mongodb.net/";
        let masked_str = Connections::mask_password(conn_str);
        let expected = "mongodb://user:******@cluster0.example.mongodb.net/";

        assert_eq!(masked_str, expected);
    }

    #[test]
    fn test_mask_password_with_srv() {
        let conn_str = "mongodb+srv://user:D1fficultP%40ssw0rd@cluster0.example.mongodb.net/";
        let masked_str = Connections::mask_password(conn_str);
        let expected = "mongodb+srv://user:******@cluster0.example.mongodb.net/";

        assert_eq!(masked_str, expected);
    }

    #[test]
    fn test_mask_password_no_passwd() {
        let conn_str = "mongodb://cluster0.example.mongodb.net/";
        let masked_str = Connections::mask_password(conn_str);
        let expected = "mongodb://cluster0.example.mongodb.net/";

        assert_eq!(masked_str, expected);
    }

    #[test]
    fn test_changing_tabs_syncs_connections() {
        let storage = Rc::new(MockStorage::default());
        let mut manager = ConnectionManager::new(vec![], storage);
        let component = Connections::new(
            Rc::new(Cell::new(TabFocus::default())),
            Config::default(),
            manager.clone(),
        );
        let mut test = ComponentTestHarness::new(component);

        // simulate another tab adding a connection to the shared manager
        let conn = Connection::new("local".into(), "mongodb://localhost:27017".into());
        manager.add_connection(conn.clone()).unwrap();
        assert_eq!(test.component().list.all_items(), []);

        test.given_event(Event::TabChanged);
        assert_eq!(test.component().list.all_items(), &[conn]);
    }

    #[test]
    fn test_select_connection() {
        let conn1 = Connection::new("conn1".into(), "mongodb://1".into());
        let conn2 = Connection::new("conn2".into(), "mongodb://2".into());
        let conn3 = Connection::new("conn3".into(), "mongodb://3".into());

        let storage = Rc::new(MockStorage::default());
        let component = Connections::new(
            Rc::new(Cell::new(TabFocus::default())),
            Config::default(),
            ConnectionManager::new(vec![conn1, conn2, conn3], storage),
        );

        let mut test = ComponentTestHarness::new(component);

        // select the second connection
        test.given_command(Command::NavDown); // highlights first
        test.given_command(Command::NavDown); // highlights second
        test.given_command(Command::Confirm);

        test.expect_event(|e| matches!(e, Event::ConnectionSelected(c) if c.name == "conn2"));
    }

    #[test]
    fn test_edit_connection() {
        let conn1 = Connection::new("conn1".into(), "mongodb://1".into());
        let conn2 = Connection::new("conn2".into(), "mongodb://2".into());
        let conn3 = Connection::new("conn3".into(), "mongodb://3".into());

        let storage = Rc::new(MockStorage::default());
        let component = Connections::new(
            Rc::new(Cell::new(TabFocus::default())),
            Config::default(),
            ConnectionManager::new(vec![conn1, conn2, conn3], storage),
        );

        let mut test = ComponentTestHarness::new(component);

        // edit the second connection
        test.given_command(Command::NavDown); // highlights first
        test.given_command(Command::NavDown); // highlights second
        test.given_command(Command::Edit);

        test.expect_message(|m| {
            matches!(
                m.read_as_conn_scr(),
                Some(ConnScreenAction::StartEditingConn(c)) if c.name == "conn2"
            )
        });
    }

    #[test]
    fn test_delete_connection() {
        let conn1 = Connection::new("conn1".into(), "mongodb://1".into());
        let conn2 = Connection::new("conn2".into(), "mongodb://2".into());
        let conn3 = Connection::new("conn3".into(), "mongodb://3".into());

        let storage = Rc::new(MockStorage::default());
        let connection_manager = ConnectionManager::new(vec![conn1, conn2, conn3], storage);
        let component = Connections::new(
            Rc::new(Cell::new(TabFocus::default())),
            Config::default(),
            connection_manager.clone(),
        );

        let mut test = ComponentTestHarness::new(component);

        // delete the second connection
        test.given_command(Command::NavDown); // highlights first
        test.given_command(Command::NavDown); // highlights second
        test.given_command(Command::Delete); // shows modal
        test.given_event(Event::ConfirmYes(Command::Delete));

        test.expect_event(|e| matches!(e, Event::ConnectionDeleted));
        assert_eq!(
            connection_manager
                .connections()
                .iter()
                .map(|c| c.name.clone())
                .collect::<Vec<_>>(),
            vec!["conn1", "conn3"]
        );
    }
}
