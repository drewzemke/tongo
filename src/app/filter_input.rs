use super::state::State;
use crossterm::event::Event;
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug, Default)]
pub struct FilterInput<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for FilterInput<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // figure the right amount to scroll the input by
        let input_scroll = state.filter.visual_scroll(area.width as usize - 2);
        #[allow(clippy::cast_possible_truncation)]
        let input_widget = Paragraph::new(state.filter.value())
            .scroll((0, input_scroll as u16))
            .block(
                Block::default()
                    .title("New Todo")
                    .border_style(Style::default().fg(Color::Yellow))
                    .borders(Borders::ALL),
            );

        Clear.render(area, buf);
        input_widget.render(area, buf);
    }
}

impl<'a> FilterInput<'a> {
    pub fn reset(state: &mut State) {
        state.filter.reset();
    }

    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        state.filter.handle_event(event).is_some()
    }

    pub fn cursor_position(state: &State, area: Rect) -> (u16, u16) {
        let input_scroll = state.filter.visual_scroll(area.width as usize - 2);
        #[allow(clippy::cast_possible_truncation)]
        (
            area.x + (state.filter.visual_cursor().max(input_scroll) - input_scroll) as u16 + 1,
            area.y + 1,
        )
    }
}
