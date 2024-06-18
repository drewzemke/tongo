use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::{
    layout::Position,
    prelude::*,
    widgets::{Block, Scrollbar, ScrollbarOrientation},
};
use tui_tree_widget::Tree;

use super::state::{Mode, State};

const PAGE_SIZE: usize = 5;

#[derive(Debug, Default)]
pub struct MainView<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for MainView<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let focused = state.mode == Mode::MainView;
        let border_color = if focused { Color::Green } else { Color::White };

        let start = state.page * PAGE_SIZE + 1;
        let end = (start + PAGE_SIZE - 1).min(state.count as usize);

        let title = format!("Documents ({start}-{end} of {})", state.count);

        let widget = Tree::new(&state.main_view_items)
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
            .highlight_style(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        StatefulWidget::render(widget, area, buf, &mut state.main_view_state);
    }
}

impl<'a> MainView<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('\n' | ' ') => state.main_view_state.toggle_selected(),
                KeyCode::Left => state.main_view_state.key_left(),
                KeyCode::Right => state.main_view_state.key_right(),
                KeyCode::Down => state.main_view_state.key_down(),
                KeyCode::Up => state.main_view_state.key_up(),
                KeyCode::Home => state.main_view_state.select_first(),
                KeyCode::End => state.main_view_state.select_last(),
                KeyCode::PageDown => state.main_view_state.scroll_down(3),
                KeyCode::PageUp => state.main_view_state.scroll_up(3),
                // next page
                KeyCode::Char('n') => {
                    let end = state.page * PAGE_SIZE + PAGE_SIZE - 1;
                    if end < state.count as usize {
                        state.page += 1;
                        state.exec_query();
                        true
                    } else {
                        false
                    }
                }
                // previous page
                KeyCode::Char('p') => {
                    if state.page > 0 {
                        state.page -= 1;
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
                MouseEventKind::ScrollDown => state.main_view_state.scroll_down(1),
                MouseEventKind::ScrollUp => state.main_view_state.scroll_up(1),
                MouseEventKind::Down(_) => state
                    .main_view_state
                    .click_at(Position::new(mouse.column, mouse.row)),
                _ => false,
            },
            Event::Resize(_, _) => true,
            _ => false,
        }
    }
}
