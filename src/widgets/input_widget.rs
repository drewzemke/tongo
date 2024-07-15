#![allow(clippy::cast_possible_truncation)]

use crossterm::event::{Event, KeyCode};
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{Block, Borders, Clear, Padding, Paragraph},
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::json_labeler::{JsonLabel, JsonLabels};

pub trait InputWidget {
    type State;

    fn title() -> &'static str;

    fn json_labels(_state: &Self::State) -> Option<&JsonLabels> {
        None
    }

    fn is_focused(state: &Self::State) -> bool;

    fn is_editing(state: &Self::State) -> bool;

    fn input(state: &mut Self::State) -> &mut Input;

    fn cursor_pos(state: &mut Self::State) -> &mut (u16, u16);

    fn on_edit_start(_state: &mut Self::State) {}

    fn on_edit_end(_state: &mut Self::State, _confirmed: bool) {}

    fn on_change(_state: &mut Self::State) {}

    fn on_tab(_state: &mut Self::State) {}

    fn on_event(_event: &Event, _state: &mut Self::State) -> bool {
        false
    }

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
        let input_scroll = Self::input(state).visual_scroll(area.width as usize - 5);

        // create the text
        let input_str = Self::input(state).value().to_string();
        let text = Self::json_labels(state).map_or_else(
            || Text::from(input_str),
            |labels| {
                let spans: Vec<_> = labels
                    .iter()
                    .map(|(s, label)| {
                        let style = match label {
                            JsonLabel::Punctuation => Style::default().gray(),
                            JsonLabel::Number => Style::default().yellow(),
                            JsonLabel::Key => Style::default().white(),
                            JsonLabel::Value => Style::default().green(),
                            JsonLabel::Constant => Style::default().cyan(),
                            JsonLabel::Whitespace => Style::default(),
                            JsonLabel::Error => Style::default().on_red(),
                        };

                        Span::styled(s, style)
                    })
                    .collect();
                let line = Line::from(spans);
                Text::from(line)
            },
        );

        let input_widget = Paragraph::new(text).scroll((0, input_scroll as u16)).block(
            Block::default()
                .title(Self::title())
                .border_style(Style::default().fg(border_color))
                .padding(Padding::horizontal(1))
                .borders(Borders::ALL),
        );
        Clear.render(area, buf);
        input_widget.render(area, buf);

        // update cursor position
        *Self::cursor_pos(state) = (
            area.x
                + (Self::input(state).visual_cursor().max(input_scroll) - input_scroll) as u16
                + 2,
            area.y + 1,
        );
    }

    fn handle_event(event: &Event, state: &mut Self::State) -> bool {
        let updated = if Self::is_editing(state) {
            match event {
                Event::Key(key) => match key.code {
                    KeyCode::Esc => {
                        Self::on_edit_end(state, false);
                        true
                    }
                    KeyCode::Enter => {
                        Self::on_edit_end(state, true);
                        true
                    }
                    KeyCode::Tab => {
                        Self::on_tab(state);
                        true
                    }
                    _ => {
                        let updated = Self::input(state).handle_event(event);
                        Self::on_change(state);
                        updated.is_some()
                    }
                },
                _ => false,
            }
        } else {
            match event {
                Event::Key(key) => match key.code {
                    KeyCode::Enter => {
                        Self::on_edit_start(state);
                        true
                    }
                    KeyCode::Tab => {
                        Self::on_tab(state);
                        true
                    }
                    _ => false,
                },
                _ => false,
            }
        };

        Self::on_event(event, state) || updated
    }
}
