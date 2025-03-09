use super::InnerList;
use crate::{
    components::{
        confirm_modal::ConfirmKind, input::input_modal::InputKind, primary_screen::PrimScrFocus,
        tab::TabFocus, Component,
    },
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandGroup},
        event::Event,
        message::{ClientAction, Message, TabAction},
        Signal,
    },
};
use mongodb::results::CollectionSpecification;
use ratatui::{prelude::*, widgets::ListItem};
use serde::{Deserialize, Serialize};
use std::{cell::Cell, rc::Rc};

#[derive(Debug, Default, Clone)]
pub struct Collections {
    focus: Rc<Cell<TabFocus>>,
    pub items: Vec<CollectionSpecification>,
    list: InnerList,
}

impl Collections {
    pub fn new(focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            focus,
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

    // TODO: remove? only used for hydration
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
        self.focus.get() == TabFocus::PrimScr(PrimScrFocus::CollList)
    }

    fn focus(&self) {
        self.focus.set(TabFocus::PrimScr(PrimScrFocus::CollList));
    }

    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = InnerList::base_commands();
        out.push(CommandGroup::new(vec![Command::Confirm], "select"));
        out.push(CommandGroup::new(
            vec![Command::CreateNew],
            "new collection",
        ));
        out.push(CommandGroup::new(vec![Command::Delete], "drop"));
        out
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        let mut out = self.list.handle_base_command(command, self.items.len());
        match command {
            Command::Confirm => {
                if let Some(coll) = self.get_selected() {
                    out.push(Event::DocumentPageChanged(0).into());
                    out.push(Event::CollectionSelected(coll.clone()).into());
                }
            }
            Command::CreateNew => out.push(
                Message::to_tab(TabAction::RequestInput(InputKind::NewCollectionName)).into(),
            ),
            Command::Delete => {
                if self.get_selected().is_some() {
                    out.push(
                        Message::to_tab(TabAction::RequestConfirmation(
                            ConfirmKind::DropCollection,
                        ))
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
                    if let Some(coll) = self.get_selected() {
                        out.push(Event::CollectionHighlighted(coll.clone()).into());
                    }
                }
            }
            Event::CollectionsUpdated(colls) => {
                self.items.clone_from(colls);

                if self.list.state.selected().is_none() {
                    if let Some(first_coll) = colls.first() {
                        // try to select the first thing
                        self.list.state.select(Some(0));
                        out.push(Event::CollectionHighlighted(first_coll.clone()).into());
                    }
                }
            }
            Event::ConfirmYes(Command::Delete) => {
                if self.is_focused() {
                    if let Some(coll) = self.get_selected() {
                        return vec![Message::to_client(ClientAction::DropCollection(
                            coll.clone(),
                        ))
                        .into()];
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
    all_colls: Vec<CollectionSpecification>,
}

impl PersistedComponent for Collections {
    type StorageType = PersistedCollections;

    fn persist(&self) -> Self::StorageType {
        PersistedCollections {
            selected_coll: self.get_selected().cloned(),
            all_colls: self.items.clone(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.items = storage.all_colls;
        self.select(storage.selected_coll);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{components::input::input_modal::InputKind, testing::ComponentTestHarness};
    use serde_json::json;

    fn get_dummy_collection() -> CollectionSpecification {
        let coll_spec_json = json!({
            "name": "test_collection",
            "type": "collection",
            "options": {},
            "info": { "readOnly": false, "uuid": null },
            "id_index": null
        });

        serde_json::from_value(coll_spec_json)
            .expect("should be able to parse collection from json")
    }

    #[test]
    fn select_first_item_on_new_data() {
        let mut test = ComponentTestHarness::new(Collections::default());

        let coll_spec = get_dummy_collection();
        test.given_event(Event::CollectionsUpdated(vec![coll_spec]));

        assert_eq!(test.component().list.state.selected(), Some(0));
    }

    #[test]
    fn create_collection() {
        let coll_spec = get_dummy_collection();
        let component = Collections {
            items: vec![coll_spec],
            ..Default::default()
        };
        let mut test = ComponentTestHarness::new(component);

        test.given_command(Command::CreateNew);
        test.expect_message(|m| {
            matches!(
                m.read_as_tab(),
                Some(TabAction::RequestInput(InputKind::NewCollectionName))
            )
        });
    }

    #[test]
    fn drop_collection() {
        let coll_spec = get_dummy_collection();
        let component = Collections {
            items: vec![coll_spec],
            ..Default::default()
        };
        let mut test = ComponentTestHarness::new(component);

        test.given_command(Command::NavDown);
        test.given_command(Command::Delete);
        test.expect_message(|m| {
            matches!(
                m.read_as_tab(),
                Some(TabAction::RequestConfirmation(ConfirmKind::DropCollection))
            )
        });
    }

    #[test]
    fn persisting_and_hydrate() {
        let coll_spec = get_dummy_collection();
        let mut component = Collections {
            items: vec![coll_spec],
            ..Default::default()
        };
        component.list.state.select(Some(0));

        let persisted_component = component.persist();

        let mut new_component = Collections::default();
        new_component.hydrate(persisted_component);

        assert_eq!(component.items[0].name, new_component.items[0].name);
        assert_eq!(
            component.list.state.selected(),
            new_component.list.state.selected()
        );
    }
}
