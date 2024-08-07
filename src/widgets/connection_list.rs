#![allow(clippy::module_name_repetitions)]

use super::list_widget::{list_nav_down, list_nav_up, ListWidget};
use crate::{
    command::{Command, CommandGroup},
    components::{list::ListComponent, Component, ComponentCommand},
    connection::Connection,
    event::Event,
    state::{State, WidgetFocus},
};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::{prelude::*, widgets::ListState};

#[derive(Debug, Default)]
pub struct ConnectionListState {
    pub items: Vec<Connection>,
    pub state: ListState,
}

#[derive(Debug, Default)]
pub struct ConnectionList<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> ListWidget for ConnectionList<'a> {
    type Item = Connection;
    type State = State<'a>;

    fn title() -> &'static str {
        "Connections"
    }

    fn list_state(state: &mut Self::State) -> &mut ListState {
        &mut state.connection_list.state
    }

    fn items(state: &Self::State) -> std::slice::Iter<Self::Item> {
        state.connection_list.items.iter()
    }

    fn item_to_str(item: &Self::Item) -> Text<'static> {
        let masked_conn_str = ConnectionList::mask_password(&item.connection_str);

        Text::from(vec![
            Line::from(item.name.clone()),
            Line::from(format!(" {masked_conn_str}")).gray(),
        ])
    }

    fn is_focused(state: &Self::State) -> bool {
        state.focus == WidgetFocus::DatabaseList
    }

    fn on_event(event: &CrosstermEvent, state: &mut Self::State) -> bool {
        if let CrosstermEvent::Key(key) = event {
            if key.code == KeyCode::Char('D') {
                let Some(index_to_delete) = state.connection_list.state.selected() else {
                    return false;
                };
                state.connection_list.items.remove(index_to_delete);
                Connection::write_to_storage(&state.connection_list.items).unwrap_or_else(|_| {
                    state.status_bar.message =
                        Some("An error occurred while saving connection preferences".to_string());
                });
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
pub struct ConnectionListV2 {
    pub items: Vec<Connection>,
    pub state: ListState,
}

impl Component for ConnectionListV2 {
    fn commands(&self) -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::NavUp, Command::NavDown], "↑↓/jk", "navigate"),
            CommandGroup::new(vec![Command::Confirm], "enter", "connect"),
            CommandGroup::new(vec![Command::CreateNew], "n", "new conn."),
            CommandGroup::new(vec![Command::Delete], "D", "delete conn."),
        ]
    }

    fn handle_command(&mut self, command: ComponentCommand) -> Vec<Event> {
        if let ComponentCommand::Command(command) = command {
            match command {
                // TODO: these should be passed to a `handle_command` in ListComponent
                Command::NavUp => {
                    list_nav_up(&mut self.state, self.items.len());
                    vec![Event::ListSelectionChanged]
                }
                Command::NavDown => {
                    list_nav_down(&mut self.state, self.items.len());
                    vec![Event::ListSelectionChanged]
                }
                Command::Confirm => self.get_selected_conn_str().map_or_else(Vec::new, |conn| {
                    vec![Event::ConnectionSelected(conn.clone())]
                }),
                Command::CreateNew => vec![Event::NewConnectionStarted],
                Command::Delete => todo!(),
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        ListComponent::render(self, area, frame.buffer_mut());
    }
}

impl ListComponent for ConnectionListV2 {
    type Item = Connection;

    fn title() -> &'static str {
        "Connections"
    }

    fn items(&self) -> std::slice::Iter<Self::Item> {
        self.items.iter()
    }

    fn item_to_str(item: &Self::Item) -> Text<'static> {
        let masked_conn_str = ConnectionList::mask_password(&item.connection_str);

        Text::from(vec![
            Line::from(item.name.clone()),
            Line::from(format!(" {masked_conn_str}")).gray(),
        ])
    }

    fn is_focused(&self) -> bool {
        true
    }

    fn list_state(&mut self) -> &mut ListState {
        &mut self.state
    }
}

impl ConnectionListV2 {
    fn get_selected_conn_str(&self) -> Option<&Connection> {
        self.state
            .selected()
            .and_then(|index| self.items.get(index))
    }
}

impl<'a> ConnectionList<'a> {
    pub fn mask_password(conn_str: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        let conn_str = "mongodb://user:D1fficultP%40ssw0rd@cluster0.example.mongodb.net/";
        let masked_str = ConnectionList::mask_password(conn_str);
        let expected = "mongodb://user:******@cluster0.example.mongodb.net/";

        assert_eq!(masked_str, expected);
    }

    #[test]
    fn test_mask_password_with_srv() {
        let conn_str = "mongodb+srv://user:D1fficultP%40ssw0rd@cluster0.example.mongodb.net/";
        let masked_str = ConnectionList::mask_password(conn_str);
        let expected = "mongodb+srv://user:******@cluster0.example.mongodb.net/";

        assert_eq!(masked_str, expected);
    }

    #[test]
    fn test_mask_password_no_passwd() {
        let conn_str = "mongodb://cluster0.example.mongodb.net/";
        let masked_str = ConnectionList::mask_password(conn_str);
        let expected = "mongodb://cluster0.example.mongodb.net/";

        assert_eq!(masked_str, expected);
    }
}
