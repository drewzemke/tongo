use super::InnerList;
use crate::{
    app::AppFocus,
    components::{primary_screen::PrimaryScreenFocus, Component, ComponentCommand, ListType},
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
};
use mongodb::results::CollectionSpecification;
use ratatui::{prelude::*, widgets::ListItem};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default)]
pub struct Collections {
    app_focus: Rc<RefCell<AppFocus>>,
    pub items: Vec<CollectionSpecification>,
    list: InnerList,
}

impl Collections {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>) -> Self {
        Self {
            app_focus,
            list: InnerList::new("Collections"),
            ..Default::default()
        }
    }

    fn get_selected(&self) -> Option<&CollectionSpecification> {
        self.list
            .state
            .selected()
            .and_then(|index| self.items.get(index))
    }
}

impl Component<ListType> for Collections {
    fn is_focused(&self) -> bool {
        *self.app_focus.borrow() == AppFocus::PrimaryScreen(PrimaryScreenFocus::CollList)
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::PrimaryScreen(PrimaryScreenFocus::CollList);
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
            out.push(Event::CollectionSelected);
        }
        out
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        match event {
            Event::ListSelectionChanged => {
                if self.is_focused() {
                    if let Some(coll) = self.get_selected() {
                        out.push(Event::CollectionHighlighted(coll.clone()));
                    }
                }
            }
            Event::CollectionsUpdated(colls) => {
                self.items.clone_from(colls);
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
