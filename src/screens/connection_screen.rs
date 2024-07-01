use crate::state::{Mode, State};
use crate::widgets::conn_str_input::ConnStrInput;
use crossterm::event::Event;
use ratatui::prelude::*;

#[derive(Debug, Default)]
pub struct ConnectionScreen<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for ConnectionScreen<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        ConnStrInput::default().render(area, buf, state);
    }
}

impl<'a> ConnectionScreen<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        match state.mode {
            Mode::EditingConnectionString | Mode::Navigating => {
                ConnStrInput::handle_event(event, state)
            }
            _ => false,
        }
    }
}
