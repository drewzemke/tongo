use std::{cell::Cell, rc::Rc};

use super::InnerList;
use crate::{
    components::{
        confirm_modal::ConfirmKind, input::input_modal::InputKind, primary_screen::PrimScrFocus,
        tab::TabFocus, Component,
    },
    model::database::Database,
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        message::{ClientAction, Message, TabAction},
        Signal,
    },
};
use ratatui::{
    prelude::{Frame, Rect},
    widgets::ListItem,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
pub struct Databases {
    focus: Rc<Cell<TabFocus>>,
    items: Vec<Database>,
    list: InnerList,
}

impl Databases {
    pub fn new(focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            focus,
            list: InnerList::new("Databases"),
            ..Default::default()
        }
    }

    fn get_selected(&self) -> Option<&Database> {
        self.list
            .state
            .selected()
            .and_then(|index| self.items.get(index))
    }

    fn select(&mut self, database: Option<Database>) {
        let index = database
            .and_then(|database| self.items.iter().position(|db| *db.name == database.name));
        self.list.state.select(index);
    }
}

impl Component for Databases {
    fn is_focused(&self) -> bool {
        self.focus.get() == TabFocus::PrimScr(PrimScrFocus::DbList)
    }

    fn focus(&self) {
        self.focus.set(TabFocus::PrimScr(PrimScrFocus::DbList));
    }

    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = InnerList::base_commands();
        out.append(&mut vec![
            CommandGroup::new(vec![Command::Confirm], "select").in_cat(CommandCategory::DbActions),
            CommandGroup::new(vec![Command::CreateNew], "new database")
                .in_cat(CommandCategory::DbActions),
            CommandGroup::new(vec![Command::Delete], "drop").in_cat(CommandCategory::DbActions),
        ]);
        out
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        let mut out = self.list.handle_base_command(command, self.items.len());
        match command {
            Command::Confirm => {
                if let Some(db) = self.get_selected() {
                    out.push(Event::DatabaseSelected(db.clone()).into());
                }
            }
            Command::CreateNew => out
                .push(Message::to_tab(TabAction::RequestInput(InputKind::NewDatabaseName)).into()),
            Command::Delete => {
                if self.get_selected().is_some() {
                    out.push(
                        Message::to_tab(TabAction::RequestConfirmation(ConfirmKind::DropDatabase))
                            .into(),
                    );
                }
            }
            _ => {}
        }
        out
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        let mut out = vec![];
        match event {
            Event::ListSelectionChanged => {
                if self.is_focused() {
                    if let Some(db) = self.get_selected() {
                        out.push(Event::DatabaseHighlighted(db.clone()).into());
                    }
                }
            }
            Event::DatabasesUpdated(dbs) => {
                self.items.clone_from(dbs);

                if self.list.state.selected().is_none() {
                    if let Some(first_db) = dbs.first() {
                        // try to select the first thing
                        self.list.state.select(Some(0));
                        out.push(Event::DatabaseHighlighted(first_db.clone()).into());
                    }
                }
            }
            Event::ConfirmYes(Command::Delete) => {
                if self.is_focused() {
                    if let Some(db) = self.get_selected() {
                        return vec![
                            Message::to_client(ClientAction::DropDatabase(db.clone())).into()
                        ];
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
    selected_db: Option<Database>,
    all_dbs: Vec<Database>,
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

    fn get_dummy_database() -> Database {
        Database::new("test_db".to_string())
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
        test.expect_message(|m| {
            matches!(
                m.read_as_tab(),
                Some(TabAction::RequestInput(InputKind::NewDatabaseName))
            )
        });
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
        test.expect_message(|m| {
            matches!(
                m.read_as_tab(),
                Some(TabAction::RequestConfirmation(ConfirmKind::DropDatabase))
            )
        });
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
