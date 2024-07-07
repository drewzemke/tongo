#![allow(clippy::module_name_repetitions)]

use super::list_widget::ListWidget;
use crate::{
    connection::Connection,
    state::{Mode, Screen, State, WidgetFocus},
};
use crossterm::event::{Event, KeyCode};
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

    fn on_select(state: &mut Self::State) {
        let conn = state
            .connection_list
            .state
            .selected()
            .and_then(|index| state.connection_list.items.get(index));
        if let Some(conn) = conn {
            state.set_conn_str(conn.connection_str.clone());
            state.screen = Screen::Primary;
            state.mode = Mode::Navigating;
            state.focus = WidgetFocus::DatabaseList;
        }
    }

    fn on_event(event: &Event, state: &mut Self::State) -> bool {
        if let Event::Key(key) = event {
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
