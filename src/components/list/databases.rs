use std::{cell::RefCell, rc::Rc};

use super::InnerList;
use crate::{
    app::AppFocus,
    components::{primary_screen::PrimaryScreenFocus, Component, ComponentCommand, ListType},
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

#[derive(Debug, Default)]
pub struct Databases {
    app_focus: Rc<RefCell<AppFocus>>,
    items: Vec<DatabaseSpecification>,
    list: InnerList,
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
}

impl Component<ListType> for Databases {
    fn is_focused(&self) -> bool {
        *self.app_focus.borrow() == AppFocus::PrimaryScreen(PrimaryScreenFocus::DbList)
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::PrimaryScreen(PrimaryScreenFocus::DbList);
    }

    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = InnerList::base_commands();
        out.push(CommandGroup::new(vec![Command::Confirm], "enter", "select"));
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
