#![allow(clippy::cast_possible_truncation)]

use crate::{
    connection::Connection,
    state::{Mode, Screen, State, WidgetFocus},
};
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
        let editing = state.mode == Mode::CreatingNewConnection
            && state.focus == WidgetFocus::ConnectionStringEditor;
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
            Mode::CreatingNewConnection => match event {
                Event::Key(key) => match key.code {
                    KeyCode::Esc | KeyCode::Tab => {
                        state.screen = Screen::Connection;
                        state.mode = Mode::CreatingNewConnection;
                        state.focus = WidgetFocus::ConnectionNameEditor;
                        true
                    }
                    KeyCode::Enter => {
                        let input = state.conn_str_editor.input.value();
                        state.set_conn_str(input.to_string());

                        let new_conn = Connection::new(
                            state.conn_name_editor.input.value().to_string(),
                            state.conn_str_editor.input.value().to_string(),
                        );
                        state.connection_list.items.push(new_conn);
                        // QUESTION: is this the right place for this file call?
                        Connection::write_to_storage(&state.connection_list.items).unwrap_or_else(
                            |_| {
                                state.status_bar.message = Some(
                                    "An error occurred while saving connection preferences"
                                        .to_string(),
                                );
                            },
                        );

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
