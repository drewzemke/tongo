use crate::command::CommandGroup;
use crate::components::connection_list::ConnectionList;
use crate::components::{Component, ComponentCommand};
use crate::event::Event;
use crate::state::{Mode, State, WidgetFocus};
use crate::widgets::conn_name_input::ConnNameInput;
use crate::widgets::conn_str_input::ConnStrInput;
use crate::widgets::input_widget::InputWidget;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
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

            // ConnectionList::render(frame_left, buf, state);
            ConnNameInput::render(right_layout[1], buf, state);
            ConnStrInput::render(right_layout[3], buf, state);
        } else {
            // ConnectionList::render(area, buf, state);
        }
    }
}

impl<'a> ConnectionScreen<'a> {
    pub fn handle_event(event: &CrosstermEvent, state: &mut State) -> bool {
        match state.mode {
            Mode::CreatingNewConnection => match state.focus {
                WidgetFocus::ConnectionStringEditor => ConnStrInput::handle_event(event, state),
                WidgetFocus::ConnectionNameEditor => ConnNameInput::handle_event(event, state),
                _ => false,
            },
            Mode::Navigating => match event {
                CrosstermEvent::Key(key) => match key.code {
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
                        // WidgetFocus::ConnectionList => ConnectionList::handle_event(event, state),
                        _ => false,
                    },
                },
                CrosstermEvent::Resize(_, _) => true,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Debug, Default)]
enum ModeV2 {
    #[default]
    Navigating,
    CreatingNewConnection,
}

#[derive(Debug, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct ConnectionScreenV2 {
    pub mode: ModeV2,
    pub connection_list: ConnectionList,
}

impl Component for ConnectionScreenV2 {
    fn commands(&self) -> Vec<CommandGroup> {
        // TODO: should depend on mode
        let mut out = vec![];
        out.append(&mut self.connection_list.commands());
        out
    }

    fn handle_command(&mut self, command: ComponentCommand) -> Vec<Event> {
        self.connection_list.handle_command(command)
    }

    fn handle_event(&mut self, event: Event) -> bool {
        match &event {
            Event::NewConnectionStarted => {
                self.mode = ModeV2::CreatingNewConnection;
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // TODO: should depend on mode
        self.connection_list.render(frame, area);
    }
}
