#![allow(clippy::cast_possible_truncation)]

use crate::{
    connection::Connection,
    state::{Mode, Screen, State, WidgetFocus},
};
use tui_input::Input;

use super::input_widget::InputWidget;

#[derive(Debug, Default)]
pub struct ConnStrEditorState {
    pub input: Input,
    pub cursor_pos: (u16, u16),
}

#[derive(Debug, Default)]
pub struct ConnStrInput<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> InputWidget for ConnStrInput<'a> {
    type State = State<'a>;

    fn title() -> &'static str {
        "Connection String"
    }

    fn is_focused(state: &Self::State) -> bool {
        state.mode == Mode::CreatingNewConnection
            && state.focus == WidgetFocus::ConnectionStringEditor
    }

    fn is_editing(state: &Self::State) -> bool {
        state.mode == Mode::CreatingNewConnection
            && state.focus == WidgetFocus::ConnectionStringEditor
    }

    fn input(state: &mut Self::State) -> &mut Input {
        &mut state.conn_str_editor.input
    }

    fn cursor_pos(state: &mut Self::State) -> &mut (u16, u16) {
        &mut state.conn_str_editor.cursor_pos
    }

    fn on_edit_start(state: &mut Self::State) {
        state.screen = Screen::Connection;
        state.mode = Mode::CreatingNewConnection;
        state.focus = WidgetFocus::ConnectionStringEditor;
    }

    fn on_edit_end(state: &mut Self::State, confirmed: bool) {
        if confirmed {
            let input = state.conn_str_editor.input.value();
            state.set_conn_str(input.to_string());

            let new_conn = Connection::new(
                state.conn_name_editor.input.value().to_string(),
                state.conn_str_editor.input.value().to_string(),
            );
            state.connection_list.items.push(new_conn);
            // QUESTION: is this the right place for this file call?
            Connection::write_to_storage(&state.connection_list.items).unwrap_or_else(|_| {
                state.status_bar.message =
                    Some("An error occurred while saving connection preferences".to_string());
            });

            state.screen = Screen::Primary;
            state.mode = Mode::Navigating;
            state.focus = WidgetFocus::DatabaseList;
        } else {
            state.screen = Screen::Connection;
            state.mode = Mode::CreatingNewConnection;
            state.focus = WidgetFocus::ConnectionNameEditor;
        }
    }

    fn on_tab(state: &mut Self::State) {
        Self::on_edit_end(state, false);
    }
}
