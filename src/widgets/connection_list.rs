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

#[derive(Debug)]
pub struct ConnectionListState {
    pub items: Vec<Connection>,
    pub state: ListState,
}

// HACK
impl Default for ConnectionListState {
    fn default() -> Self {
        // HACK
        let fake_list = [
            Connection::new("Local".to_string(), "mongodb://localhost:27017".to_string()),
            Connection::new(
                "Other Connection".to_string(),
                "mongodb://not-real-db.com".to_string(),
            ),
            Connection::new(
                "Another!!!".to_string(),
                "mongodb://localhost:27017".to_string(),
            ),
            Connection::new("Nice.".to_string(), "mongodb://localhost:27017".to_string()),
        ];

        Self {
            items: fake_list.to_vec(),
            state: ListState::default(),
        }
    }
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
