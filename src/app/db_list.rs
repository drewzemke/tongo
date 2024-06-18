use crossterm::event::{Event, KeyCode};
use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, List, ListItem, StatefulWidget},
};

use super::state::{Mode, State};

#[derive(Debug, Default)]
pub struct DbList<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for DbList<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let focused = state.mode == Mode::ChoosingDatabase;
        let border_color = if focused { Color::Green } else { Color::White };

        let items: Vec<ListItem> = state
            .dbs
            .iter()
            .map(|db| ListItem::new(db.name.clone()))
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("Databases")
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::White),
            );

        StatefulWidget::render(list, area, buf, &mut state.db_state);
    }
}

impl<'a> DbList<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    Self::next(state);
                    state.exec_get_collections();
                    true
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    Self::previous(state);
                    state.exec_get_collections();
                    true
                }
                KeyCode::Enter => {
                    state.mode = Mode::ChoosingCollection;
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn next(state: &mut State) -> bool {
        let i = match state.db_state.selected() {
            Some(i) => {
                if i >= state.dbs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.db_state.select(Some(i));
        true
    }

    fn previous(state: &mut State) -> bool {
        let i = match state.db_state.selected() {
            Some(i) => {
                if i == 0 {
                    state.dbs.len() - 1
                } else {
                    i - 1
                }
            }
            None => state.dbs.len() - 1,
        };
        state.db_state.select(Some(i));
        true
    }
}
