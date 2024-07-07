#![allow(clippy::module_name_repetitions)]

use crate::{
    connection::Connection,
    state::{Mode, Screen, State, WidgetFocus},
};
use crossterm::event::{Event, KeyCode};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};

#[derive(Debug, Default)]
pub struct ConnectionListState {
    pub items: Vec<Connection>,
    pub state: ListState,
}

#[derive(Debug, Default)]
pub struct ConnectionList<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for ConnectionList<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let focused = state.focus == WidgetFocus::ConnectionList;
        let border_color = if focused { Color::Green } else { Color::White };

        let items: Vec<ListItem> = state
            .connection_list
            .items
            .iter()
            .map(|conn| {
                ListItem::new(Text::from(vec![
                    Line::from(conn.name.clone()),
                    Line::from(format!(" {}", conn.connection_str)).gray(),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("Connections")
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(Style::default().bold().reversed().white());

        StatefulWidget::render(list, area, buf, &mut state.connection_list.state);
    }
}

impl<'a> ConnectionList<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    state.connection_list.state.select_next();
                    state.exec_get_collections();
                    true
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    state.connection_list.state.select_previous();
                    state.exec_get_collections();
                    true
                }
                KeyCode::Char('D') => {
                    let Some(index_to_delete) = state.connection_list.state.selected() else {
                        return false;
                    };
                    state.connection_list.items.remove(index_to_delete);
                    Connection::write_to_storage(&state.connection_list.items).unwrap_or_else(
                        |_| {
                            state.status_bar.message = Some(
                                "An error occurred while saving connection preferences".to_string(),
                            );
                        },
                    );
                    true
                }
                KeyCode::Enter => {
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
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
}
