use super::InnerList;
use crate::{
    app::AppFocus,
    components::{primary_screen::PrimScrFocus, Component, ComponentCommand},
    persistence::PersistedComponent,
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
        *self.app_focus.borrow() == AppFocus::PrimScr(PrimScrFocus::CollList)
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::PrimScr(PrimScrFocus::CollList);
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

                if self.list.state.selected().is_none() {
                    if let Some(first_coll) = colls.first() {
                        // try to select the first thing
                        self.list.state.select(Some(0));
                        out.push(Event::CollectionHighlighted(first_coll.clone()));
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
        self.select(storage.selected_coll);
        vec![]
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::testing::ComponentTestHarness;
    use anyhow::Result;
    use serde_json::json;

    #[test]
    fn select_first_item_on_new_data() -> Result<()> {
        let mut test = ComponentTestHarness::new(Collections::default());

        let coll_spec_json = json!({
            "name": "test_collection",
            "type": "collection",
            "options": {},
            "info": { "readOnly": false, "uuid": null },
            "id_index": null
        });
        let coll_spec: CollectionSpecification = serde_json::from_value(coll_spec_json)?;

        test.given_event(Event::CollectionsUpdated(vec![coll_spec]));

        assert_eq!(test.component().list.state.selected(), Some(0));

        Ok(())
    }
}
