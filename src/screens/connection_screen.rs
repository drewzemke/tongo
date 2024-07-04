use crate::state::{Mode, State, WidgetFocus};
use crate::widgets::conn_name_input::ConnNameInput;
use crate::widgets::conn_str_input::ConnStrInput;
use crate::widgets::connection_list::ConnectionList;
use crossterm::event::{Event, KeyCode};
use ratatui::prelude::*;

#[derive(Debug, Default)]
pub struct ConnectionScreen<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for ConnectionScreen<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.mode == Mode::CreatingNewConnection {
            let frame_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);
            let frame_left = frame_layout[0];
            let frame_right = frame_layout[1];

            let right_layout = Layout::default()
                .constraints(vec![
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ])
                .horizontal_margin(2)
                .split(frame_right);

            ConnectionList::default().render(frame_left, buf, state);
            ConnNameInput::default().render(right_layout[1], buf, state);
            ConnStrInput::default().render(right_layout[3], buf, state);
        } else {
            ConnectionList::default().render(area, buf, state);
        }
    }
}

impl<'a> ConnectionScreen<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        match state.mode {
            Mode::CreatingNewConnection => match state.focus {
                WidgetFocus::ConnectionStringEditor => ConnStrInput::handle_event(event, state),
                WidgetFocus::ConnectionNameEditor => ConnNameInput::handle_event(event, state),
                _ => false,
            },
            Mode::Navigating => match event {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => {
                        state.mode = Mode::Exiting;
                        true
                    }
                    KeyCode::Char('n') => {
                        state.mode = Mode::CreatingNewConnection;
                        state.focus = WidgetFocus::ConnectionNameEditor;
                        true
                    }
                    _ => match state.focus {
                        WidgetFocus::ConnectionList => ConnectionList::handle_event(event, state),
                        _ => false,
                    },
                },
                Event::Resize(_, _) => true,
                _ => false,
            },
            _ => false,
        }
    }
}
