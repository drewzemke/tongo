#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_name_repetitions)]

use crate::state::{State, WidgetFocus};
use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::{
    layout::Position,
    prelude::*,
    widgets::{Block, Scrollbar, ScrollbarOrientation},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

const PAGE_SIZE: usize = 5;

#[derive(Debug, Default)]
pub struct MainViewState<'a> {
    pub state: TreeState<String>,
    pub items: Vec<TreeItem<'a, String>>,
    pub page: usize,
    pub count: u64,
}

#[derive(Debug, Default)]
pub struct MainView<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for MainView<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let focused = state.focus == WidgetFocus::MainView;
        let border_color = if focused { Color::Green } else { Color::White };

        let start = state.main_view.page * PAGE_SIZE + 1;
        let end = (start + PAGE_SIZE - 1).min(state.main_view.count as usize);

        let title = format!("Documents ({start}-{end} of {})", state.main_view.count);

        let widget = Tree::new(&state.main_view.items)
            .expect("all item identifiers are unique")
            .block(
                Block::bordered()
                    .title(title)
                    .border_style(Style::default().fg(border_color)),
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ))
            .highlight_style(Style::new().black().on_white().bold());

        StatefulWidget::render(widget, area, buf, &mut state.main_view.state);
    }
}

impl<'a> MainView<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('\n' | ' ') => state.main_view.state.toggle_selected(),
                KeyCode::Left => state.main_view.state.key_left(),
                KeyCode::Right => state.main_view.state.key_right(),
                KeyCode::Down => state.main_view.state.key_down(),
                KeyCode::Up => state.main_view.state.key_up(),
                KeyCode::Home => state.main_view.state.select_first(),
                KeyCode::End => state.main_view.state.select_last(),
                KeyCode::PageDown => state.main_view.state.scroll_down(3),
                KeyCode::PageUp => state.main_view.state.scroll_up(3),
                // next page
                KeyCode::Char('n') => {
                    let end = (state.main_view.page + 1) * PAGE_SIZE;
                    if end < state.main_view.count as usize {
                        state.main_view.page += 1;
                        state.exec_query();
                        true
                    } else {
                        false
                    }
                }
                // previous page
                KeyCode::Char('p') => {
                    if state.main_view.page > 0 {
                        state.main_view.page -= 1;
                        state.exec_query();
                        true
                    } else {
                        false
                    }
                }
                // refresh
                KeyCode::Char('r') => {
                    state.exec_query();
                    state.exec_count();
                    false
                }
                _ => false,
            },

            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => state.main_view.state.scroll_down(1),
                MouseEventKind::ScrollUp => state.main_view.state.scroll_up(1),
                MouseEventKind::Down(_) => state
                    .main_view
                    .state
                    .click_at(Position::new(mouse.column, mouse.row)),
                _ => false,
            },
            Event::Resize(_, _) => true,
            _ => false,
        }
    }
}
