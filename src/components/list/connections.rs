use std::{
    cell::{Cell, Ref},
    rc::Rc,
};

use super::InnerList;
use crate::{
    components::{
        confirm_modal::ConfirmKind,
        connection_screen::ConnScrFocus,
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
        Signal,
    },
};
use crossterm::event::KeyCode;
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, ListItem, Paragraph},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Connections {
    pub focus: Rc<Cell<TabFocus>>,
    pub connection_manager: ConnectionManager,
    pub list: InnerList,
    pub config: Config,
}

impl Default for Connections {
    fn default() -> Self {
        Self {
            focus: Rc::new(Cell::new(TabFocus::default())),
            connection_manager: ConnectionManager::default(),
            list: InnerList::default(),
            config: Config::default(),
        }
    }
}

impl CloneWithFocus for Connections {
    fn clone_with_focus(&self, focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            focus,
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
        Self {
            focus,
            connection_manager,
            list: InnerList::new("Connections", config.clone()),
            config,
        }
    }

    fn get_selected_conn(&self) -> Option<Ref<Connection>> {
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
}

impl Component for Connections {
    fn is_focused(&self) -> bool {
        self.focus.get() == TabFocus::ConnScr(ConnScrFocus::ConnList)
    }

    fn focus(&self) {
        self.focus.set(TabFocus::ConnScr(ConnScrFocus::ConnList));
    }

    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = InnerList::base_commands();
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

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        let mut out = self
            .list
            .handle_base_command(command, self.connection_manager.connections().len());
        match command {
            Command::Confirm => {
                if let Some(conn) = self.get_selected_conn() {
                    out.push(Event::ConnectionSelected(conn.clone()).into());
                    out.push(Message::to_client(ClientAction::Connect(conn.clone())).into());
                }
            }
            Command::CreateNew => {
                out.push(Message::to_conn_scr(ConnScreenAction::StartNewConn).into());
            }
            Command::Edit => {
                if let Some(conn) = self.get_selected_conn() {
                    out.push(
                        Message::to_conn_scr(ConnScreenAction::StartEditingConn(conn.clone()))
                            .into(),
                    );
                }
            }
            Command::Delete => {
                out.push(
                    Message::to_tab(TabAction::RequestConfirmation(
                        ConfirmKind::DeleteConnection,
                    ))
                    .into(),
                );
            }
            _ => {}
        }
        out
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        let mut out = vec![];

        // only process the confirmation if this component is focused
        if self.is_focused() && matches!(event, Event::ConfirmYes(Command::Delete)) {
            let Some(index_to_delete) = self.list.state.selected() else {
                return out;
            };

            if matches!(
                self.connection_manager.delete_connection(index_to_delete),
                Ok(())
            ) {
                out.push(Event::ConnectionDeleted.into());
            } else {
                out.push(
                    Event::ErrorOccurred(
                        "An error occurred while saving connection preferences".into(),
                    )
                    .into(),
                );
            }
        }
        out
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
            let items: Vec<ListItem> = self
                .connection_manager
                .connections()
                .iter()
                .map(|item| {
                    let masked_conn_str = Self::mask_password(&item.connection_str);

                    let text = Text::from(vec![
                        Line::from(item.name.clone())
                            .fg(self.config.color_map.get(&ColorKey::FgPrimary)),
                        Line::from(format!(" {masked_conn_str}"))
                            .fg(self.config.color_map.get(&ColorKey::FgSecondary)),
                    ]);
                    ListItem::new(text)
                })
                .collect();

            self.list.render(frame, area, items, self.is_focused());
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
        self.connection_manager.set_connections(storage.connections);

        if let Some(conn) = storage.selected_conn {
            self.select_conn(&conn);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
