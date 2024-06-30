use crate::state::{State, WidgetFocus};
use crossterm::event::{Event, KeyCode};
use mongodb::results::DatabaseSpecification;
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};

#[derive(Debug, Default)]
pub struct DatabaseListState {
    pub items: Vec<DatabaseSpecification>,
    pub state: ListState,
}

#[derive(Debug, Default)]
pub struct DbList<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for DbList<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let focused = state.focus == WidgetFocus::DatabaseList;
        let border_color = if focused { Color::Green } else { Color::White };

        let items: Vec<ListItem> = state
            .db_list
            .items
            .iter()
            .map(|db| ListItem::new(db.name.clone()))
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("Databases")
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(Style::default().bold().reversed().white());

        StatefulWidget::render(list, area, buf, &mut state.db_list.state);
    }
}

impl<'a> DbList<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    state.db_list.state.select_next();
                    state.exec_get_collections();
                    true
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    state.db_list.state.select_previous();
                    state.exec_get_collections();
                    true
                }
                KeyCode::Enter => {
                    state.focus = WidgetFocus::CollectionList;
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
}
