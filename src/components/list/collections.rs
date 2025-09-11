use super::InnerList;
use crate::{
    components::{
        confirm_modal::ConfirmKind,
        input::input_modal::InputKind,
        primary_screen::PrimScrFocus,
        tab::{CloneWithFocus, TabFocus},
        Component,
    },
    config::Config,
    model::collection::Collection,
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
pub struct Collections {
    focus: Rc<Cell<TabFocus>>,
    pub items: Vec<Collection>,
    list: InnerList,
}

impl CloneWithFocus for Collections {
    fn clone_with_focus(&self, focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            focus,
            ..self.clone()
        }
    }
}

impl Collections {
    pub fn new(focus: Rc<Cell<TabFocus>>, config: Config) -> Self {
        Self {
            focus,
            list: InnerList::new("Collections", config),
            ..Default::default()
        }
    }

    fn get_selected(&self) -> Option<&Collection> {
        self.list
            .state
            .selected()
            .and_then(|index| self.items.get(index))
    }

    // TODO: remove? only used for hydration
    fn select(&mut self, collection: Option<Collection>) {
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
        out.append(&mut vec![
            CommandGroup::new(vec![Command::Confirm], "select collection")
                .in_cat(CommandCategory::ConnActions),
            CommandGroup::new(vec![Command::CreateNew], "new collection")
                .in_cat(CommandCategory::ConnActions),
            CommandGroup::new(vec![Command::Delete], "drop collection")
                .in_cat(CommandCategory::ConnActions),
        ]);
        out
    }

    fn handle_command(&mut self, command: &Command, queue: &mut SignalQueue) {
        self.list.handle_base_command(command, self.items.len(), queue);
        match command {
            Command::Confirm => {
                if let Some(coll) = self.get_selected() {
                    queue.push(Event::DocumentPageChanged(0));
                    queue.push(Event::CollectionSelected(coll.clone()));
                }
            }
            Command::CreateNew => queue.push(Message::to_tab(TabAction::RequestInput(InputKind::NewCollectionName))),
            Command::Delete => {
                if self.get_selected().is_some() {
                    queue.push(Message::to_tab(TabAction::RequestConfirmation(
                            ConfirmKind::DropCollection,
                        )));
                }
            }
            _ => {}
        }
    }

    fn handle_event(&mut self, event: &Event, queue: &mut SignalQueue) {
        match event {
            Event::ListSelectionChanged => {
                if self.is_focused() {
                    if let Some(coll) = self.get_selected() {
                        queue.push(Event::CollectionHighlighted(coll.clone()));
                    }
                }
            }
            Event::CollectionsUpdated(colls) => {
                self.items.clone_from(colls);

                if self.list.state.selected().is_none() {
                    if let Some(first_coll) = colls.first() {
                        // try to select the first thing
                        self.list.state.select(Some(0));
                        queue.push(Event::CollectionHighlighted(first_coll.clone()));
                    }
                }
            }
            Event::ConfirmYes(Command::Delete) => {
                if self.is_focused() {
                    if let Some(coll) = self.get_selected() {
                        queue.push(Message::to_client(ClientAction::DropCollection(
                            coll.clone(),
                        )));
                    }
                }
            }
            _ => (),
        }
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
    selected_coll: Option<Collection>,
    all_colls: Vec<Collection>,
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

    fn get_dummy_collection() -> Collection {
        Collection::new("test_collection".to_string())
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
