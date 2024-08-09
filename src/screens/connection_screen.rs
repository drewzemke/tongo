use crate::command::CommandGroup;
use crate::components::input::connection_name_input::ConnectionNameInput;
use crate::components::list::connection_list::ConnectionList;
use crate::components::{Component, ComponentCommand, UniqueType};
use crate::event::Event;
use ratatui::prelude::*;

#[derive(Debug, Default, PartialEq, Eq)]
enum Focus {
    #[default]
    ConnectionList,
    NameInput,
    StringInput,
}

#[derive(Debug, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct ConnectionScreen {
    focus: Focus,
    connection_list: ConnectionList,
    connection_name_input: ConnectionNameInput,
}

impl ConnectionScreen {
    pub fn new(connection_list: ConnectionList) -> Self {
        Self {
            connection_list,
            ..Default::default()
        }
    }
}

impl Component<UniqueType> for ConnectionScreen {
    fn commands(&self) -> Vec<CommandGroup> {
        match self.focus {
            Focus::ConnectionList => self.connection_list.commands(),
            Focus::NameInput => self.connection_name_input.commands(),
            Focus::StringInput => todo!(),
        }
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        match self.focus {
            Focus::ConnectionList => self.connection_list.handle_command(command),
            Focus::NameInput => self.connection_name_input.handle_command(command),
            Focus::StringInput => todo!(),
        }
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        let internal_update = match event {
            Event::NewConnectionStarted => {
                self.focus = Focus::NameInput;
                self.connection_name_input.editing = true;
                true
            }
            Event::FocusedForward => match self.focus {
                Focus::NameInput => {
                    self.focus = Focus::StringInput;
                    self.connection_name_input.editing = false;
                    true
                }
                Focus::StringInput => todo!(),
                Focus::ConnectionList => false,
            },
            Event::FocusedBackward => match self.focus {
                Focus::NameInput => {
                    self.focus = Focus::ConnectionList;
                    self.connection_name_input.editing = false;
                    true
                }
                Focus::StringInput => todo!(),
                Focus::ConnectionList => false,
            },
            _ => false,
        };
        let conn_list_update = self.connection_list.handle_event(event);
        let conn_name_input_update = self.connection_name_input.handle_event(event);
        internal_update || conn_list_update || conn_name_input_update
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.focus == Focus::NameInput || self.focus == Focus::StringInput {
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

            self.connection_list.render(frame, frame_left);
            self.connection_name_input.render(frame, right_layout[1]);
            // ConnStrInput::render(right_layout[3], buf, state);
        } else {
            self.connection_list.render(frame, area);
        }
    }
}
