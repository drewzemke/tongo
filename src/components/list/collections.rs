use super::ListComponent;
use crate::{
    app::AppFocus,
    command::{Command, CommandGroup},
    components::ComponentCommand,
    event::Event,
    screens::primary_screen::PrimaryScreenFocus,
};
use mongodb::results::CollectionSpecification;
use ratatui::{prelude::*, widgets::ListState};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default)]
pub struct Collections {
    app_focus: Rc<RefCell<AppFocus>>,
    pub items: Vec<CollectionSpecification>,
    pub state: ListState,
}

impl ListComponent for Collections {
    type Item = CollectionSpecification;

    fn title() -> &'static str {
        "Collections"
    }

    fn items(&self) -> std::slice::Iter<Self::Item> {
        self.items.iter()
    }

    fn item_to_str(item: &Self::Item) -> Text<'static> {
        item.name.clone().into()
    }

    fn is_focused(&self) -> bool {
        *self.app_focus.borrow() == AppFocus::PrimaryScreen(PrimaryScreenFocus::CollList)
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::PrimaryScreen(PrimaryScreenFocus::CollList);
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
            if let Some(coll) = self.get_selected() {
                out.push(Event::CollectionSelected(coll.clone()));
            }
        }
        out
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        match event {
            Event::ListSelectionChanged => {
                if let Some(coll) = self.get_selected() {
                    out.push(Event::CollectionHighlighted(coll.clone()));
                }
            }
            Event::CollectionsUpdated(colls) => {
                self.items.clone_from(colls);
            }
            _ => (),
        }
        out
    }
}

impl Collections {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>) -> Self {
        Self {
            app_focus,
            ..Default::default()
        }
    }

    fn get_selected(&self) -> Option<&CollectionSpecification> {
        self.state
            .selected()
            .and_then(|index| self.items.get(index))
    }
}
