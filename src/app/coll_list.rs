use crossterm::event::{Event, KeyCode};
use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, List, ListItem, StatefulWidget},
};

use super::state::{State, WidgetFocus};

#[derive(Debug, Default)]
pub struct CollList<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for CollList<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let focused = state.focus == WidgetFocus::CollectionList;
        let border_color = if focused { Color::Green } else { Color::White };

        let items: Vec<ListItem> = state
            .colls
            .iter()
            .map(|coll| ListItem::new(coll.name.clone()))
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("Collections")
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::White),
            );

        StatefulWidget::render(list, area, buf, &mut state.coll_state);
    }
}

impl<'a> CollList<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => Self::next(state),
                KeyCode::Char('k') | KeyCode::Up => Self::previous(state),
                KeyCode::Enter => {
                    state.exec_query();
                    state.exec_count();
                    state.focus = WidgetFocus::MainView;
                    false
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn next(state: &mut State) -> bool {
        let i = match state.coll_state.selected() {
            Some(i) => {
                if i >= state.colls.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.coll_state.select(Some(i));
        true
    }

    fn previous(state: &mut State) -> bool {
        let i = match state.coll_state.selected() {
            Some(i) => {
                if i == 0 {
                    state.colls.len() - 1
                } else {
                    i - 1
                }
            }
            None => state.colls.len() - 1,
        };
        state.coll_state.select(Some(i));
        true
    }
}
