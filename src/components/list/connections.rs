use std::{cell::RefCell, rc::Rc};

use super::InnerList;
use crate::{
    app::AppFocus,
    components::{connection_screen::ConnScreenFocus, Component, ComponentCommand},
    connection::Connection,
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
};
use ratatui::{prelude::*, widgets::ListItem};

#[derive(Debug, Default)]
pub struct Connections {
    app_focus: Rc<RefCell<AppFocus>>,
    pub items: Vec<Connection>,
    list: InnerList,
}

impl Connections {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>, items: Vec<Connection>) -> Self {
        Self {
            app_focus,
            items,
            list: InnerList::new("Connections"),
        }
    }

    fn get_selected_conn_str(&self) -> Option<&Connection> {
        self.list
            .state
            .selected()
            .and_then(|index| self.items.get(index))
    }

    fn mask_password(conn_str: &str) -> String {
        let Some((before_slashes, after_slashes)) = conn_str.split_once("//") else {
            return String::from(conn_str);
        };
        let Some((user_and_pw, after_at)) = after_slashes.split_once('@') else {
            return String::from(conn_str);
        };
        let Some((user, _)) = user_and_pw.split_once(':') else {
            return String::from(conn_str);
        };
        format!("{before_slashes}//{user}:******@{after_at}")
    }
}

impl Component for Connections {
    fn is_focused(&self) -> bool {
        *self.app_focus.borrow() == AppFocus::ConnScreen(ConnScreenFocus::ConnList)
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::ConnScreen(ConnScreenFocus::ConnList);
    }

    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = InnerList::base_commands();
        out.append(&mut vec![
            CommandGroup::new(vec![Command::Confirm], "connect"),
            CommandGroup::new(vec![Command::CreateNew], "new conn."),
            CommandGroup::new(vec![Command::Delete], "delete conn."),
        ]);
        out
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        let mut out = self.list.handle_base_command(command, self.items.len());
        let ComponentCommand::Command(command) = command else {
            return vec![];
        };
        match command {
            Command::Confirm => {
                if let Some(conn) = self.get_selected_conn_str() {
                    out.push(Event::ConnectionSelected(conn.clone()));
                }
            }
            Command::CreateNew => {
                out.push(Event::NewConnectionStarted);
            }
            Command::Delete => {
                out.push(Event::ConfirmationRequested(*command));
            }
            _ => {}
        }
        out
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        // only process the confirmation if this component is focused
        if self.is_focused() && matches!(event, Event::ConfirmationYes(Command::Delete)) {
            let Some(index_to_delete) = self.list.state.selected() else {
                return out;
            };
            self.items.remove(index_to_delete);
            let write_result = Connection::write_to_storage(&self.items);
            if write_result.is_ok() {
                out.push(Event::ConnectionDeleted);
            } else {
                out.push(Event::ErrorOccurred(
                    "An error occurred while saving connection preferences".to_string(),
                ));
            }
        }
        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                let masked_conn_str = Self::mask_password(&item.connection_str);

                let text = Text::from(vec![
                    Line::from(item.name.clone()),
                    Line::from(format!(" {masked_conn_str}")).gray(),
                ]);
                ListItem::new(text)
            })
            .collect();

        self.list.render(frame, area, items, self.is_focused());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        let conn_str = "mongodb://user:D1fficultP%40ssw0rd@cluster0.example.mongodb.net/";
        let masked_str = Connections::mask_password(conn_str);
        let expected = "mongodb://user:******@cluster0.example.mongodb.net/";

        assert_eq!(masked_str, expected);
    }

    #[test]
    fn test_mask_password_with_srv() {
        let conn_str = "mongodb+srv://user:D1fficultP%40ssw0rd@cluster0.example.mongodb.net/";
        let masked_str = Connections::mask_password(conn_str);
        let expected = "mongodb+srv://user:******@cluster0.example.mongodb.net/";

        assert_eq!(masked_str, expected);
    }

    #[test]
    fn test_mask_password_no_passwd() {
        let conn_str = "mongodb://cluster0.example.mongodb.net/";
        let masked_str = Connections::mask_password(conn_str);
        let expected = "mongodb://cluster0.example.mongodb.net/";

        assert_eq!(masked_str, expected);
    }
}
