use super::state::{Mode, State, WidgetFocus};
use crossterm::event::{Event, KeyCode};
use mongodb::bson::Document;
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Debug, Default)]
pub struct FilterEditorState {
    pub input: Input,
    pub filter: Option<Document>,
}

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
        let input_scroll = state
            .filter_editor
            .input
            .visual_scroll(area.width as usize - 2);
        #[allow(clippy::cast_possible_truncation)]
        let input_widget = Paragraph::new(state.filter_editor.input.value())
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
                    KeyCode::Enter => {
                        if let Some(doc) = Self::process_input(state) {
                            state.filter_editor.filter = Some(doc);
                            state.exec_query();
                            state.exec_count();
                            state.mode = Mode::Navigating;
                            state.focus = WidgetFocus::MainView;
                        }
                        true
                    }
                    KeyCode::Esc => {
                        state.mode = Mode::Navigating;
                        true
                    }
                    _ => state.filter_editor.input.handle_event(event).is_some(),
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
        let input_scroll = state
            .filter_editor
            .input
            .visual_scroll(area.width as usize - 2);
        #[allow(clippy::cast_possible_truncation)]
        (
            area.x
                + (state.filter_editor.input.visual_cursor().max(input_scroll) - input_scroll)
                    as u16
                + 1,
            area.y + 1,
        )
    }

    fn process_input(state: &State) -> Option<Document> {
        let filter_str = state.filter_editor.input.value();
        let filter = serde_json::from_str::<serde_json::Value>(filter_str).ok()?;
        mongodb::bson::to_document(&filter).ok()
    }
}
