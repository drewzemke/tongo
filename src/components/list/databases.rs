use std::{cell::RefCell, rc::Rc};

use super::InnerList;
use crate::{
    app::AppFocus,
    components::{primary_screen::PrimaryScreenFocus, Component, ComponentCommand},
    sessions::PersistedComponent,
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

#[derive(Debug, Default)]
pub struct Databases {
    app_focus: Rc<RefCell<AppFocus>>,
    items: Vec<DatabaseSpecification>,
    list: InnerList,

    // HACK: (?) used to store the db that should be selected
    // the next time the dbs are updated
    pending_selection: Option<DatabaseSpecification>,
}

impl Databases {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>) -> Self {
        Self {
            app_focus,
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
        *self.app_focus.borrow() == AppFocus::PrimaryScreen(PrimaryScreenFocus::DbList)
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::PrimaryScreen(PrimaryScreenFocus::DbList);
    }

    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = InnerList::base_commands();
        out.push(CommandGroup::new(vec![Command::Confirm], "select"));
        out
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        let mut out = self.list.handle_base_command(command, self.items.len());
        let ComponentCommand::Command(command) = command else {
            return vec![];
        };
        if matches!(command, Command::Confirm) {
            out.push(Event::DatabaseSelected);
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

                if self.pending_selection.is_some() {
                    let db = self.pending_selection.take();
                    self.select(db);
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

#[derive(Serialize, Deserialize)]
pub struct PersistedDatabases {
    selected_db: Option<DatabaseSpecification>,
}

impl PersistedComponent for Databases {
    type StorageType = PersistedDatabases;

    fn persist(&self) -> Self::StorageType {
        PersistedDatabases {
            selected_db: self.get_selected().cloned(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) -> Vec<Event> {
        // TODO: do we need to do this?
        self.pending_selection = storage.selected_db;

        let mut out = vec![];
        if let Some(ref db) = self.pending_selection {
            out.push(Event::DatabaseHighlighted(db.clone()));
            out.push(Event::DatabaseSelected);
        }
        out
    }
}
