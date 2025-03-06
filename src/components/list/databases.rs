use std::{cell::RefCell, rc::Rc};

use super::InnerList;
use crate::{
    components::{
        confirm_modal::ConfirmKind, input::input_modal::InputKind, primary_screen::PrimScrFocus,
        tab::TabFocus, Component, ComponentCommand,
    },
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
};
use mongodb::results::DatabaseSpecification;
use ratatui::{
    prelude::{Frame, Rect},
    widgets::ListItem,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
pub struct Databases {
    focus: Rc<RefCell<TabFocus>>,
    items: Vec<DatabaseSpecification>,
    list: InnerList,
}

impl Databases {
    pub fn new(focus: Rc<RefCell<TabFocus>>) -> Self {
        Self {
            focus,
            list: InnerList::new("Databases"),
            ..Default::default()
        }
    }

    fn get_selected(&self) -> Option<&DatabaseSpecification> {
        self.list
            .state
            .selected()
            .and_then(|index| self.items.get(index))
    }

    fn select(&mut self, database: Option<DatabaseSpecification>) {
        let index = database
            .and_then(|database| self.items.iter().position(|db| *db.name == database.name));
        self.list.state.select(index);
    }
}

impl Component for Databases {
    fn is_focused(&self) -> bool {
        *self.focus.borrow() == TabFocus::PrimScr(PrimScrFocus::DbList)
    }

    fn focus(&self) {
        *self.focus.borrow_mut() = TabFocus::PrimScr(PrimScrFocus::DbList);
    }

    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = InnerList::base_commands();
        out.push(CommandGroup::new(vec![Command::Confirm], "select"));
        out.push(CommandGroup::new(vec![Command::CreateNew], "new database"));
        out.push(CommandGroup::new(vec![Command::Delete], "drop"));
        out
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        let mut out = self.list.handle_base_command(command, self.items.len());
        let ComponentCommand::Command(command) = command else {
            return vec![];
        };
        if matches!(command, Command::Confirm) {}
        match command {
            Command::Confirm => {
                if let Some(db) = self.get_selected() {
                    out.push(Event::DatabaseSelected(db.clone()));
                }
            }
            Command::CreateNew => out.push(Event::InputRequested(InputKind::NewDatabaseName)),
            Command::Delete => {
                if self.get_selected().is_some() {
                    out.push(Event::ConfirmationRequested(ConfirmKind::DropDatabase));
                }
            }
            _ => {}
        }
        out
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        match event {
            Event::ListSelectionChanged => {
                if self.is_focused() {
                    if let Some(db) = self.get_selected() {
                        out.push(Event::DatabaseHighlighted(db.clone()));
                    }
                }
            }
            Event::DatabasesUpdated(dbs) => {
                self.items.clone_from(dbs);

                if self.list.state.selected().is_none() {
                    if let Some(first_db) = dbs.first() {
                        // try to select the first thing
                        self.list.state.select(Some(0));
                        out.push(Event::DatabaseHighlighted(first_db.clone()));
                    }
                }
            }
            Event::ConfirmationYes(Command::Delete) => {
                if self.is_focused() {
                    if let Some(db) = self.get_selected() {
                        return vec![Event::DatabaseDropped(db.clone())];
                    }
                }
            }
            _ => (),
        }
        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| ListItem::new(item.name.clone()))
            .collect();

        self.list.render(frame, area, items, self.is_focused());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedDatabases {
    selected_db: Option<DatabaseSpecification>,
    all_dbs: Vec<DatabaseSpecification>,
}

impl PersistedComponent for Databases {
    type StorageType = PersistedDatabases;

    fn persist(&self) -> Self::StorageType {
        PersistedDatabases {
            selected_db: self.get_selected().cloned(),
            all_dbs: self.items.clone(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.items = storage.all_dbs;
        self.select(storage.selected_db);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        components::{confirm_modal::ConfirmKind, input::input_modal::InputKind},
        testing::ComponentTestHarness,
    };
    use serde_json::json;

    fn get_dummy_database() -> DatabaseSpecification {
        let db_spec_json = json!({
            "name": "test_db",
            "sizeOnDisk": 1024,
            "empty": false,
            "shards": null
        });

        serde_json::from_value(db_spec_json).expect("should be able to parse database from json")
    }

    #[test]
    fn select_first_item_on_new_data() {
        let mut test = ComponentTestHarness::new(Databases::default());

        let db_spec = get_dummy_database();
        test.given_event(Event::DatabasesUpdated(vec![db_spec]));

        assert_eq!(test.component().list.state.selected(), Some(0));
    }

    #[test]
    fn create_database() {
        let db_spec = get_dummy_database();
        let component = Databases {
            items: vec![db_spec],
            ..Default::default()
        };
        let mut test = ComponentTestHarness::new(component);

        test.given_command(Command::CreateNew);
        test.expect_event(|e| matches!(e, Event::InputRequested(InputKind::NewDatabaseName)));
    }

    #[test]
    fn drop_database() {
        let db_spec = get_dummy_database();
        let component = Databases {
            items: vec![db_spec],
            ..Default::default()
        };
        let mut test = ComponentTestHarness::new(component);

        test.given_command(Command::NavDown);
        test.given_command(Command::Delete);
        test.expect_event(|e| matches!(e, Event::ConfirmationRequested(ConfirmKind::DropDatabase)));
    }

    #[test]
    fn persisting_and_hydrate() {
        let db_spec = get_dummy_database();
        let mut component = Databases {
            items: vec![db_spec],
            ..Default::default()
        };
        component.list.state.select(Some(0));

        let persisted_component = component.persist();

        let mut new_component = Databases::default();
        new_component.hydrate(persisted_component);

        assert_eq!(component.items[0].name, new_component.items[0].name);
        assert_eq!(
            component.list.state.selected(),
            new_component.list.state.selected()
        );
    }
}
