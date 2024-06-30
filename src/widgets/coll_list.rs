use crate::state::{State, WidgetFocus};
use crossterm::event::{Event, KeyCode};
use mongodb::results::CollectionSpecification;
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};

#[derive(Debug, Default)]
pub struct CollectionListState {
    pub items: Vec<CollectionSpecification>,
    pub state: ListState,
}

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
            .coll_list
            .items
            .iter()
            .map(|coll| ListItem::new(coll.name.clone()))
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("Collections")
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(Style::default().bold().reversed().white());

        StatefulWidget::render(list, area, buf, &mut state.coll_list.state);
    }
}

impl<'a> CollList<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    state.coll_list.state.select_next();
                    true
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    state.coll_list.state.select_previous();
                    true
                }
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
}
