use std::{cell::RefCell, rc::Rc};

use super::ListComponent;
use crate::{
    app::AppFocus,
    command::{Command, CommandGroup},
    components::ComponentCommand,
    event::Event,
    screens::primary_screen::PrimaryScreenFocus,
};
use mongodb::results::DatabaseSpecification;
use ratatui::{prelude::*, widgets::ListState};

#[derive(Debug, Default)]
pub struct Databases {
    app_focus: Rc<RefCell<AppFocus>>,
    pub items: Vec<DatabaseSpecification>,
    pub state: ListState,
}

impl ListComponent for Databases {
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
        *self.app_focus.borrow() == AppFocus::PrimaryScreen(PrimaryScreenFocus::DbList)
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::PrimaryScreen(PrimaryScreenFocus::DbList);
    }

    fn list_state(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![CommandGroup::new(vec![Command::Confirm], "enter", "select")]
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        let ComponentCommand::Command(command) = command else {
            return vec![];
        };
        let mut out = vec![];
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
            }
            _ => (),
        }
        out
    }
}

impl Databases {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>) -> Self {
        Self {
            app_focus,
            ..Default::default()
        }
    }

    fn get_selected(&self) -> Option<&DatabaseSpecification> {
        self.state
            .selected()
            .and_then(|index| self.items.get(index))
    }
}
