#![allow(clippy::cast_possible_truncation)]

use super::input_widget::InputWidget;
use crate::state::{Mode, State, WidgetFocus};
use mongodb::bson::Document;
use tui_input::Input;

#[derive(Debug, Default)]
pub struct FilterEditorState {
    pub input: Input,
    pub filter: Option<Document>,
    pub cursor_pos: (u16, u16),
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

    fn input(state: &mut Self::State) -> &mut Input {
        &mut state.filter_editor.input
    }

    fn cursor_pos(state: &mut Self::State) -> &mut (u16, u16) {
        &mut state.filter_editor.cursor_pos
    }

    fn on_cancel(state: &mut Self::State) {
        state.mode = Mode::Navigating;
    }

    fn on_confirm(state: &mut Self::State) {
        let filter_str = state.filter_editor.input.value();
        let filter = serde_json::from_str::<serde_json::Value>(filter_str)
            .ok()
            .and_then(|value| mongodb::bson::to_document(&value).ok());

        if let Some(doc) = filter {
            state.filter_editor.filter = Some(doc);
            state.exec_query();
            state.exec_count();
            state.mode = Mode::Navigating;
            state.focus = WidgetFocus::MainView;
        }
    }
}
