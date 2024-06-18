use super::state::{Mode, State, WidgetFocus};
use crossterm::event::{Event, KeyCode};
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
        let focused = state.focus == WidgetFocus::FilterEditor;
        let editing = state.mode == Mode::EditingFilter;
        let border_color = if focused {
            if editing {
                Color::Yellow
            } else {
                Color::Green
            }
        } else {
            Color::White
        };

        // figure the right amount to scroll the input by
        let input_scroll = state.filter.visual_scroll(area.width as usize - 2);
        #[allow(clippy::cast_possible_truncation)]
        let input_widget = Paragraph::new(state.filter.value())
            .scroll((0, input_scroll as u16))
            .block(
                Block::default()
                    .title("Filter")
                    .border_style(Style::default().fg(border_color))
                    .borders(Borders::ALL),
            );

        Clear.render(area, buf);
        input_widget.render(area, buf);
    }
}

impl<'a> FilterInput<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        match state.mode {
            Mode::EditingFilter => match event {
                Event::Key(key) => match key.code {
                    KeyCode::Esc => {
                        state.mode = Mode::Navigating;
                        true
                    }
                    _ => state.filter.handle_event(event).is_some(),
                },
                _ => false,
            },
            Mode::Navigating => match event {
                Event::Key(key) => match key.code {
                    KeyCode::Enter => {
                        state.mode = Mode::EditingFilter;
                        true
                    }
                    _ => false,
                },
                _ => false,
            },
            Mode::Exiting => false,
        }
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
