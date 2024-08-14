use super::ListComponent;
use crate::{
    command::{Command, CommandGroup},
    components::ComponentCommand,
    event::Event,
};
use mongodb::results::DatabaseSpecification;
use ratatui::{prelude::*, widgets::ListState};

#[derive(Debug, Default)]
pub struct DbList {
    pub items: Vec<DatabaseSpecification>,
    pub state: ListState,
}

impl ListComponent for DbList {
    type Item = DatabaseSpecification;

    fn title() -> &'static str {
        "Databases"
    }

    fn items(&self) -> std::slice::Iter<Self::Item> {
        self.items.iter()
    }

    fn item_to_str(item: &Self::Item) -> Text<'static> {
        item.name.clone().into()
    }

    fn is_focused(&self) -> bool {
        true
    }

    fn focus(&self) {}

    fn list_state(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![CommandGroup::new(vec![Command::Confirm], "enter", "select")]
    }

    fn handle_command(&mut self, _command: &ComponentCommand) -> Vec<Event> {
        vec![]
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        match event {
            Event::ListSelectionChanged => {
                if let Some(db) = self.get_selected() {
                    out.push(Event::DatabaseSelected(db.clone()));
                }
            }
            Event::DatabasesUpdated(dbs) => {
                self.items.clone_from(dbs);
            }
            _ => (),
        }
        out
    }
}

impl DbList {
    fn get_selected(&self) -> Option<&DatabaseSpecification> {
        self.state
            .selected()
            .and_then(|index| self.items.get(index))
    }
}
