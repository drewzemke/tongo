use super::InnerList;
use crate::{
    components::{
        confirm_modal::ConfirmKind,
        input::input_modal::InputKind,
        list::InnerListItem,
        primary_screen::PrimScrFocus,
        tab::{CloneWithFocus, TabFocus},
        Component,
    },
    config::Config,
    model::database::Database,
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        message::{ClientAction, Message, TabAction},
        signal::SignalQueue,
    },
};
use ratatui::{prelude::*, widgets::ListItem};
use serde::{Deserialize, Serialize};
use std::{cell::Cell, rc::Rc};

#[derive(Debug, Default, Clone)]
pub struct Databases {
    focus: Rc<Cell<TabFocus>>,
    list: InnerList<Database>,
}

impl InnerListItem for Database {
    fn to_list_item(&self, _config: &Config) -> ListItem<'_> {
        ListItem::new(self.name.clone())
    }
}

impl CloneWithFocus for Databases {
    fn clone_with_focus(&self, focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            focus,
            ..self.clone()
        }
    }
}

impl Databases {
    pub fn new(focus: Rc<Cell<TabFocus>>, config: Config) -> Self {
        Self {
            focus,
            list: InnerList::new("Databases", config),
        }
    }

    #[cfg(test)]
    fn with_items(dbs: &[Database]) -> Self {
        let mut comp = Self::default();
        comp.list.set_items(dbs);
        comp
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
        let mut out = self.list.base_commands();
        out.append(&mut vec![
            CommandGroup::new(vec![Command::Confirm], "select database")
                .in_cat(CommandCategory::DbActions),
            CommandGroup::new(vec![Command::CreateNew], "new database")
                .in_cat(CommandCategory::DbActions),
            CommandGroup::new(vec![Command::Delete], "drop database")
                .in_cat(CommandCategory::DbActions),
        ]);
        out
    }

    fn handle_command(&mut self, command: &Command, queue: &mut SignalQueue) {
        self.list
            .handle_base_command(command, self.list.num_items(), queue);
        match command {
            Command::Confirm => {
                if let Some(db) = self.list.selected() {
                    queue.push(Event::DatabaseSelected(db.clone()));
                }
            }
            Command::CreateNew => queue.push(Message::to_tab(TabAction::RequestInput(
                InputKind::NewDatabaseName,
            ))),
            Command::Delete if self.list.selected().is_some() => {
                queue.push(Message::to_tab(TabAction::RequestConfirmation(
                    ConfirmKind::DropDatabase,
                )));
            }
            _ => {}
        }
    }

    fn handle_event(&mut self, event: &Event, queue: &mut SignalQueue) {
        match event {
            Event::ListSelectionChanged => {
                if self.is_focused() {
                    if let Some(db) = self.list.selected() {
                        queue.push(Event::DatabaseHighlighted(db.clone()));
                    }
                }
            }
            Event::DatabasesUpdated(dbs) => {
                self.list.set_items(dbs);

                if self.list.selected().is_none() {
                    if let Some(first_db) = dbs.first() {
                        // try to select the first thing
                        self.list.state.select(Some(0));
                        queue.push(Event::DatabaseHighlighted(first_db.clone()));
                    }
                }
            }
            Event::ConfirmYes(Command::Delete) if self.is_focused() => {
                if let Some(db) = self.list.selected() {
                    queue.push(Message::to_client(ClientAction::DropDatabase(db.clone())));
                }
            }
            _ => (),
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.list.render(frame, area, self.is_focused());
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
            selected_db: self.list.selected().cloned(),
            all_dbs: self.list.items().to_vec(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.list.set_items(&storage.all_dbs);
        self.list.select(storage.selected_db);
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
        let component = Databases::with_items(&[db_spec]);
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
        let component = Databases::with_items(&[db_spec]);
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
        let mut component = Databases::with_items(&[db_spec]);
        component.list.state.select(Some(0));

        let persisted_component = component.persist();

        let mut new_component = Databases::default();
        new_component.hydrate(persisted_component);

        assert_eq!(
            component.list.items()[0].name,
            new_component.list.items()[0].name
        );
        assert_eq!(
            component.list.state.selected(),
            new_component.list.state.selected()
        );
    }
}
