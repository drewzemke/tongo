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
use mongodb::results::CollectionSpecification;
use ratatui::{prelude::*, widgets::ListItem};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default)]
pub struct Collections {
    app_focus: Rc<RefCell<AppFocus>>,
    pub items: Vec<CollectionSpecification>,
    list: InnerList,

    // HACK: (?) used to store the coll that should be selected
    // the next time the colls are updated
    pending_selection: Option<CollectionSpecification>,
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
    fn select(&mut self, collection: Option<CollectionSpecification>) {
        let index = collection.and_then(|collection| {
            self.items
                .iter()
                .position(|coll| *coll.name == collection.name)
        });
        self.list.state.select(index);
    }
}

impl Component for Collections {
    fn is_focused(&self) -> bool {
        *self.app_focus.borrow() == AppFocus::PrimaryScreen(PrimaryScreenFocus::CollList)
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::PrimaryScreen(PrimaryScreenFocus::CollList);
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
            if let Some(coll) = self.get_selected() {
                out.push(Event::DocumentPageChanged(0));
                out.push(Event::CollectionSelected(coll.clone()));
            }
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

                if self.pending_selection.is_some() {
                    let coll = self.pending_selection.take();
                    self.select(coll);
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
pub struct PersistedCollections {
    selected_coll: Option<CollectionSpecification>,
}

impl PersistedComponent for Collections {
    type StorageType = PersistedCollections;

    fn persist(&self) -> Self::StorageType {
        PersistedCollections {
            selected_coll: self.get_selected().cloned(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) -> Vec<Event> {
        // TODO: do we need to do this?
        self.pending_selection = storage.selected_coll;

        let mut out = vec![];
        if let Some(ref coll) = self.pending_selection {
            out.push(Event::CollectionHighlighted(coll.clone()));
            out.push(Event::CollectionSelected(coll.clone()));
        }
        out
    }
}
