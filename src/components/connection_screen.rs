use super::{
    input::{conn_name_input::ConnNameInput, conn_str_input::ConnStrInput},
    list::connections::Connections,
    Component, ComponentCommand,
};
use crate::{
    app::AppFocus,
    connection::Connection,
    system::{command::CommandGroup, event::Event},
};
use ratatui::prelude::*;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ConnScreenFocus {
    #[default]
    ConnList,
    NameInput,
    StringInput,
}

#[derive(Debug, Default)]
pub struct ConnectionScreen {
    app_focus: Rc<RefCell<AppFocus>>,
    conn_list: Connections,
    conn_name_input: ConnNameInput,
    conn_str_input: ConnStrInput,
}

impl ConnectionScreen {
    pub fn new(
        connection_list: Connections,
        app_focus: Rc<RefCell<AppFocus>>,
        cursor_pos: Rc<RefCell<(u16, u16)>>,
    ) -> Self {
        let conn_name_input = ConnNameInput::new(app_focus.clone(), cursor_pos.clone());
        let conn_str_input = ConnStrInput::new(app_focus.clone(), cursor_pos);

        Self {
            app_focus,
            conn_list: connection_list,
            conn_name_input,
            conn_str_input,
        }
    }

    /// Narrows the shared `AppFocus` variable into the focus enum for this componenent
    fn internal_focus(&self) -> Option<ConnScreenFocus> {
        match &*self.app_focus.borrow() {
            AppFocus::ConnScreen(focus) => Some(focus.clone()),
            AppFocus::PrimaryScreen(..) => None,
        }
    }
}

impl Component for ConnectionScreen {
    fn commands(&self) -> Vec<CommandGroup> {
        match self.internal_focus() {
            Some(ConnScreenFocus::ConnList) => self.conn_list.commands(),
            Some(ConnScreenFocus::NameInput) => self.conn_name_input.commands(),
            Some(ConnScreenFocus::StringInput) => self.conn_str_input.commands(),
            None => vec![],
        }
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        match self.internal_focus() {
            Some(ConnScreenFocus::ConnList) => self.conn_list.handle_command(command),
            Some(ConnScreenFocus::NameInput) => self.conn_name_input.handle_command(command),
            Some(ConnScreenFocus::StringInput) => self.conn_str_input.handle_command(command),
            None => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        match event {
            Event::NewConnectionStarted => {
                self.conn_name_input.focus();
                self.conn_name_input.start_editing();
                out.push(Event::RawModeEntered);
            }
            Event::FocusedForward => match self.internal_focus() {
                Some(ConnScreenFocus::NameInput) => {
                    self.conn_name_input.stop_editing();
                    self.conn_str_input.focus();
                    self.conn_str_input.start_editing();
                }
                Some(ConnScreenFocus::StringInput) => {
                    let conn = Connection::new(
                        self.conn_name_input.value().to_string(),
                        self.conn_str_input.value().to_string(),
                    );
                    out.push(Event::ConnectionCreated(conn));
                }
                Some(ConnScreenFocus::ConnList) | None => {}
            },
            Event::FocusedBackward => match self.internal_focus() {
                Some(ConnScreenFocus::NameInput) => {
                    self.conn_list.focus();
                    self.conn_name_input.stop_editing();
                }
                Some(ConnScreenFocus::StringInput) => {
                    self.conn_name_input.focus();
                    self.conn_name_input.start_editing();
                    self.conn_str_input.stop_editing();
                }
                Some(ConnScreenFocus::ConnList) | None => {}
            },
            Event::ConnectionCreated(conn) => {
                self.conn_list.items.push(conn.clone());
                Connection::write_to_storage(&self.conn_list.items).unwrap_or_else(|_| {
                    out.push(Event::ErrorOccurred(
                        "Could not save updated connections.".to_string(),
                    ));
                });
            }
            _ => {}
        };
        out.append(&mut self.conn_list.handle_event(event));
        out.append(&mut self.conn_name_input.handle_event(event));
        out.append(&mut self.conn_str_input.handle_event(event));
        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.internal_focus() == Some(ConnScreenFocus::NameInput)
            || self.internal_focus() == Some(ConnScreenFocus::StringInput)
        {
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

            self.conn_list.render(frame, frame_left);
            self.conn_name_input.render(frame, right_layout[1]);
            self.conn_str_input.render(frame, right_layout[3]);
        } else {
            self.conn_list.render(frame, area);
        }
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::ConnScreen(ConnScreenFocus::default());
    }

    fn is_focused(&self) -> bool {
        matches!(*self.app_focus.borrow(), AppFocus::ConnScreen(..))
    }
}
