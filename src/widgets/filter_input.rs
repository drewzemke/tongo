#![allow(clippy::cast_possible_truncation)]

use super::input_widget::InputWidget;
use crate::{
    json_labeler::{JsonLabel, JsonLabels},
    state::{Mode, State, WidgetFocus},
};
use crossterm::event::{Event, KeyCode};
use mongodb::bson::Document;
use tui_input::Input;

#[derive(Debug)]
pub struct FilterEditorState {
    pub input: Input,
    pub filter: Option<Document>,
    pub json_labels: JsonLabels,
    pub cursor_pos: (u16, u16),
}

const DEFAULT_FILTER: &str = "{}";

impl Default for FilterEditorState {
    fn default() -> Self {
        Self {
            input: Input::default().with_value(DEFAULT_FILTER.to_string()),
            filter: None,
            json_labels: vec![(DEFAULT_FILTER.to_string(), JsonLabel::Punctuation)],
            cursor_pos: (0, 0),
        }
    }
}

#[derive(Debug, Default)]
pub struct FilterInput<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> InputWidget for FilterInput<'a> {
    type State = State<'a>;

    fn title() -> &'static str {
        "Filter"
    }

    fn is_focused(state: &Self::State) -> bool {
        state.focus == WidgetFocus::FilterEditor
    }

    fn is_editing(state: &Self::State) -> bool {
        state.mode == Mode::EditingFilter
    }

    fn json_labels(state: &Self::State) -> Option<&JsonLabels> {
        Some(&state.filter_editor.json_labels)
    }

    fn input(state: &mut Self::State) -> &mut Input {
        &mut state.filter_editor.input
    }

    fn cursor_pos(state: &mut Self::State) -> &mut (u16, u16) {
        &mut state.filter_editor.cursor_pos
    }

    fn on_edit_start(state: &mut Self::State) {
        state.mode = Mode::EditingFilter;
    }

    fn on_change(state: &mut Self::State) {
        // update json labels
        let value = state.filter_editor.input.value().to_string();
        state.filter_editor.json_labels = state.json_labeler.label_line(&value).unwrap_or_default();
    }

    fn on_edit_end(state: &mut Self::State, confirmed: bool) {
        if confirmed {
            let filter_str = state.filter_editor.input.value();
            let filter = serde_json::from_str::<serde_json::Value>(filter_str)
                .ok()
                .and_then(|value| mongodb::bson::to_document(&value).ok());

            if let Some(doc) = filter {
                state.filter_editor.filter = Some(doc);
                state.exec_query(true, true);
                state.exec_count();
                state.mode = Mode::Navigating;
                state.focus = WidgetFocus::MainView;
            }
        } else {
            state.mode = Mode::Navigating;
        }
    }

    fn on_event(event: &Event, state: &mut Self::State) -> bool {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('R') {
                state.filter_editor.input = Input::default().with_value(DEFAULT_FILTER.to_string());
                Self::on_change(state);
                Self::on_edit_end(state, true);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
