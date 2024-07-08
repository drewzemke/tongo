#![allow(clippy::cast_possible_truncation)]

use crossterm::event::{Event, KeyCode};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};
use tui_input::{backend::crossterm::EventHandler, Input};

pub trait InputWidget {
    type State;

    fn title() -> &'static str;

    fn is_focused(state: &Self::State) -> bool;

    fn is_editing(state: &Self::State) -> bool;

    fn input(state: &mut Self::State) -> &mut Input;

    fn cursor_pos(state: &mut Self::State) -> &mut (u16, u16);

    fn on_cancel(_state: &mut Self::State) {}

    fn on_tab(_state: &mut Self::State) {}

    fn on_confirm(_state: &mut Self::State) {}

    fn render(area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let border_color = if Self::is_focused(state) {
            if Self::is_editing(state) {
                Color::Yellow
            } else {
                Color::Green
            }
        } else {
            Color::White
        };

        // figure the right amount to scroll the input by
        let input_scroll = Self::input(state).visual_scroll(area.width as usize - 2);
        let input_widget = Paragraph::new(Self::input(state).value())
            .scroll((0, input_scroll as u16))
            .block(
                Block::default()
                    .title(Self::title())
                    .border_style(Style::default().fg(border_color))
                    .borders(Borders::ALL),
            );
        Clear.render(area, buf);
        input_widget.render(area, buf);

        // update cursor position
        *Self::cursor_pos(state) = (
            area.x
                + (Self::input(state).visual_cursor().max(input_scroll) - input_scroll) as u16
                + 1,
            area.y + 1,
        );
    }

    fn handle_event(event: &Event, state: &mut Self::State) -> bool {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Esc => {
                    Self::on_cancel(state);
                    true
                }
                KeyCode::Enter => {
                    Self::on_confirm(state);
                    true
                }
                KeyCode::Tab => {
                    Self::on_tab(state);
                    true
                }
                _ => Self::input(state).handle_event(event).is_some(),
            },
            _ => false,
        }
    }
}
