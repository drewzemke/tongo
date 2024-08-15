use super::ListComponent;
use crate::{
    command::{Command, CommandGroup},
    components::ComponentCommand,
    connection::Connection,
    event::Event,
};
use ratatui::{prelude::*, widgets::ListState};

#[derive(Debug, Default)]
pub struct Connections {
    pub items: Vec<Connection>,
    pub state: ListState,
}

impl ListComponent for Connections {
    type Item = Connection;

    fn title() -> &'static str {
        "Connections"
    }

    fn items(&self) -> std::slice::Iter<Self::Item> {
        self.items.iter()
    }

    fn item_to_str(item: &Self::Item) -> Text<'static> {
        let masked_conn_str = Self::mask_password(&item.connection_str);

        Text::from(vec![
            Line::from(item.name.clone()),
            Line::from(format!(" {masked_conn_str}")).gray(),
        ])
    }

    fn is_focused(&self) -> bool {
        true
    }

    fn focus(&self) {}

    fn list_state(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::Confirm], "enter", "connect"),
            CommandGroup::new(vec![Command::CreateNew], "n", "new conn."),
            CommandGroup::new(vec![Command::Delete], "D", "delete conn."),
        ]
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        let ComponentCommand::Command(command) = command else {
            return vec![];
        };
        match command {
            Command::Confirm => self.get_selected_conn_str().map_or_else(Vec::new, |conn| {
                vec![Event::ConnectionSelected(conn.clone())]
            }),
            Command::CreateNew => vec![Event::NewConnectionStarted],
            Command::Delete => {
                let Some(index_to_delete) = self.state.selected() else {
                    return vec![];
                };
                self.items.remove(index_to_delete);
                let write_result = Connection::write_to_storage(&self.items);
                if write_result.is_ok() {
                    vec![Event::ConnectionDeleted]
                } else {
                    vec![Event::ErrorOccurred(
                        "An error occurred while saving connection preferences".to_string(),
                    )]
                }
            }
            _ => vec![],
        }
    }
}

impl Connections {
    fn get_selected_conn_str(&self) -> Option<&Connection> {
        self.state
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
