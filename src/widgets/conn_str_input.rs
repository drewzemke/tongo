#![allow(clippy::cast_possible_truncation)]

use crate::state::{Mode, Screen, State, WidgetFocus};
use crossterm::event::{Event, KeyCode};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Debug, Default)]
pub struct ConnStrEditorState {
    pub input: Input,
    pub cursor_pos: (u16, u16),
}

#[derive(Debug, Default)]
pub struct ConnStrInput<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for ConnStrInput<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let editing = state.mode == Mode::EditingConnectionString;
        let border_color = if editing { Color::Yellow } else { Color::White };

        // figure the right amount to scroll the input by
        let input_scroll = state
            .conn_str_editor
            .input
            .visual_scroll(area.width as usize - 2);
        let input_widget = Paragraph::new(state.conn_str_editor.input.value())
            .scroll((0, input_scroll as u16))
            .block(
                Block::default()
                    .title("Connection String")
                    .border_style(Style::default().fg(border_color))
                    .borders(Borders::ALL),
            );

        // update cursor position if we're in an editing state
        state.conn_str_editor.cursor_pos = (
            area.x
                + (state
                    .conn_str_editor
                    .input
                    .visual_cursor()
                    .max(input_scroll)
                    - input_scroll) as u16
                + 1,
            area.y + 1,
        );

        Clear.render(area, buf);
        input_widget.render(area, buf);
    }
}

impl<'a> ConnStrInput<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        match state.mode {
            Mode::EditingConnectionString => match event {
                Event::Key(key) => match key.code {
                    KeyCode::Enter => {
                        let input = state.conn_str_editor.input.value();
                        state.set_conn_str(input.to_string());
                        state.screen = Screen::Primary;
                        state.mode = Mode::Navigating;
                        state.focus = WidgetFocus::DatabaseList;
                        true
                    }
                    _ => state.conn_str_editor.input.handle_event(event).is_some(),
                },
                _ => false,
            },
            _ => false,
        }
    }
}
