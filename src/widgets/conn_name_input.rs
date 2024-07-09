use super::input_widget::InputWidget;
use crate::state::{Mode, Screen, State, WidgetFocus};
use tui_input::Input;

#[derive(Debug, Default)]
pub struct ConnNameEditorState {
    pub input: Input,
    pub cursor_pos: (u16, u16),
}

#[derive(Debug, Default)]
pub struct ConnNameInput<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> InputWidget for ConnNameInput<'a> {
    type State = State<'a>;

    fn title() -> &'static str {
        "Name"
    }

    fn is_focused(state: &Self::State) -> bool {
        state.mode == Mode::CreatingNewConnection
            && state.focus == WidgetFocus::ConnectionNameEditor
    }

    fn is_editing(state: &Self::State) -> bool {
        state.mode == Mode::CreatingNewConnection
            && state.focus == WidgetFocus::ConnectionNameEditor
    }

    fn input(state: &mut Self::State) -> &mut Input {
        &mut state.conn_name_editor.input
    }

    fn cursor_pos(state: &mut Self::State) -> &mut (u16, u16) {
        &mut state.conn_name_editor.cursor_pos
    }

    fn on_edit_start(state: &mut Self::State) {
        state.screen = Screen::Connection;
        state.mode = Mode::CreatingNewConnection;
        state.focus = WidgetFocus::ConnectionStringEditor;
    }

    fn on_edit_end(state: &mut Self::State, confirmed: bool) {
        state.screen = Screen::Connection;

        if confirmed {
            state.mode = Mode::CreatingNewConnection;
            state.focus = WidgetFocus::ConnectionStringEditor;
        } else {
            state.mode = Mode::Navigating;
            state.focus = WidgetFocus::ConnectionList;
        }
    }

    fn on_tab(state: &mut Self::State) {
        Self::on_edit_end(state, true);
    }
}
