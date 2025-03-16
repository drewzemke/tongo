use super::{confirm_modal::ConfirmKind, primary_screen::PrimScrFocus, tab::TabFocus, Component};
use crate::{
    client::PAGE_SIZE,
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        message::{ClientAction, Message, TabAction},
        Signal,
    },
    utils::{
        clipboard::send_bson_to_clipboard,
        edit_doc::edit_doc,
        mongo_tree::{top_level_document, MongoKey},
    },
};
use mongodb::bson::{doc, oid::ObjectId, Bson, Document};
use ratatui::{
    prelude::*,
    widgets::{Block, Scrollbar, ScrollbarOrientation},
};
use serde::{Deserialize, Serialize};
use std::{cell::Cell, rc::Rc};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Debug, Default)]
pub struct Documents<'a> {
    focus: Rc<Cell<TabFocus>>,
    state: TreeState<MongoKey>,
    items: Vec<TreeItem<'a, MongoKey>>,

    #[expect(clippy::struct_field_names)]
    documents: Vec<Bson>,

    page: usize,
    count: u64,
}

impl Clone for Documents<'_> {
    fn clone(&self) -> Self {
        let mut documents = Self {
            focus: self.focus.clone(),
            state: TreeState::default(),
            items: self.items.clone(),
            documents: self.documents.clone(),
            page: self.page,
            count: self.count,
        };
        documents.reset_state();
        documents
    }
}

impl Documents<'_> {
    pub fn new(focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            focus,
            ..Default::default()
        }
    }

    fn reset_state(&mut self) {
        // reset state to have all top-level documents expanded
        let mut state = TreeState::default();
        for item in &self.items {
            state.open(vec![item.identifier().clone()]);
        }
        self.state = state;

        if let Some(first_item) = self.items.first() {
            // try to select the first thing
            self.state.select(vec![first_item.identifier().clone()]);
        }
    }

    fn set_docs(&mut self, docs: &Vec<Bson>, reset_state: bool) {
        self.documents.clone_from(docs);

        let items: Vec<_> = docs
            .iter()
            .filter_map(|bson| bson.as_document().map(top_level_document))
            .collect();

        self.items = items;

        if reset_state {
            self.reset_state();
        }
    }

    fn selected_doc_as_bson(&self) -> Option<&Bson> {
        let id = self.state.selected().first()?;

        self.items
            .iter()
            .position(|tree_item| tree_item.identifier() == id)
            .and_then(|index| self.documents.get(index))
    }

    fn selected_doc(&self) -> Option<&Document> {
        self.selected_doc_as_bson()
            .and_then(|bson| bson.as_document())
    }

    // TODO: this definitely needs tests
    // TODO: ... and a better name
    fn selected_bson(&self) -> Option<&Bson> {
        let mut bson = self.selected_doc_as_bson()?;

        // ignore the first element, which is always the doc id
        let path = &self.state.selected()[1..];

        for key in path {
            match (bson, key) {
                (Bson::Document(doc), MongoKey::String(key)) => {
                    bson = doc.get(key)?;
                }
                (Bson::Array(array), MongoKey::Usize(index)) => {
                    bson = array.get(*index)?;
                }
                _ => break,
            }
        }

        Some(bson)
    }
}

impl Component for Documents<'_> {
    fn is_focused(&self) -> bool {
        self.focus.get() == TabFocus::PrimScr(PrimScrFocus::DocTree)
    }

    fn focus(&self) {
        self.focus.set(TabFocus::PrimScr(PrimScrFocus::DocTree));
    }

    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = vec![
            CommandGroup::new(
                vec![
                    Command::NavLeft,
                    Command::NavDown,
                    Command::NavUp,
                    Command::NavRight,
                ],
                "navigate",
            )
            .in_cat(CommandCategory::DocNav),
            CommandGroup::new(vec![Command::ExpandCollapse], "expand/collapse")
                .in_cat(CommandCategory::DocNav),
            CommandGroup::new(
                vec![Command::PreviousPage, Command::NextPage],
                "previous/next page",
            )
            .in_cat(CommandCategory::DocNav),
            CommandGroup::new(
                vec![Command::FirstPage, Command::LastPage],
                "first/last page",
            )
            .in_cat(CommandCategory::DocNav),
            CommandGroup::new(vec![Command::Refresh], "refresh queries")
                .in_cat(CommandCategory::DocActions),
            CommandGroup::new(vec![Command::CreateNew], "insert document")
                .in_cat(CommandCategory::DocActions),
        ];

        if self.selected_doc().is_some() {
            out.append(&mut vec![
                CommandGroup::new(vec![Command::DuplicateDoc], "duplicate document")
                    .in_cat(CommandCategory::DocActions),
                CommandGroup::new(vec![Command::Delete], "delete document")
                    .in_cat(CommandCategory::DocActions),
                CommandGroup::new(vec![Command::Edit], "edit document")
                    .in_cat(CommandCategory::DocActions),
                CommandGroup::new(vec![Command::Yank], "copy to clipboard")
                    .in_cat(CommandCategory::DocActions),
            ]);
        }

        out
    }

    #[expect(clippy::too_many_lines)]
    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        let mut out = vec![];
        match command {
            Command::NavLeft => {
                if self.state.key_left() {
                    out.push(Event::ListSelectionChanged.into());
                }
            }
            Command::NavUp => {
                if self.state.key_up() {
                    out.push(Event::ListSelectionChanged.into());
                }
            }
            Command::NavDown => {
                if self.state.key_down() {
                    out.push(Event::ListSelectionChanged.into());
                }
            }
            Command::NavRight => {
                if self.state.key_right() {
                    out.push(Event::ListSelectionChanged.into());
                }
            }
            Command::ExpandCollapse => {
                if self.state.toggle_selected() {
                    out.push(Event::ListSelectionChanged.into());
                }
            }
            Command::NextPage => {
                let end = (self.page + 1) * PAGE_SIZE;

                #[expect(clippy::cast_possible_truncation)]
                if end < self.count as usize {
                    self.page += 1;
                    out.push(Event::DocumentPageChanged(self.page).into());
                }
            }
            Command::PreviousPage => {
                if self.page > 0 {
                    self.page -= 1;
                    out.push(Event::DocumentPageChanged(self.page).into());
                }
            }
            Command::FirstPage => {
                self.page = 0;
                out.push(Event::DocumentPageChanged(self.page).into());
            }
            Command::LastPage => {
                #[expect(clippy::cast_possible_truncation)]
                let last_page = (self.count as usize).div_ceil(PAGE_SIZE) - 1;
                self.page = last_page;
                out.push(Event::DocumentPageChanged(self.page).into());
            }
            Command::Refresh => {
                out.push(Message::to_client(ClientAction::RefreshQueries).into());
            }
            Command::Edit => {
                let Some(doc) = self.selected_doc() else {
                    return out;
                };

                out.push(Event::ReturnedFromAltScreen.into());
                match edit_doc(doc.clone()) {
                    Ok(new_doc) => {
                        out.push(Message::to_client(ClientAction::UpdateDoc(new_doc)).into());
                    }
                    Err(err) => out.push(Event::ErrorOccurred(err.to_string()).into()),
                }
            }
            Command::CreateNew => {
                let doc = doc! { "_id" : ObjectId::new() };

                out.push(Event::ReturnedFromAltScreen.into());

                match edit_doc(doc) {
                    Ok(new_doc) => {
                        out.push(Message::to_client(ClientAction::InsertDoc(new_doc)).into());
                    }
                    Err(err) => out.push(Event::ErrorOccurred(err.to_string()).into()),
                }
            }
            Command::DuplicateDoc => {
                let Some(doc) = self.selected_doc() else {
                    return out;
                };

                let mut duplicated_doc = doc.clone();
                let _ = duplicated_doc.insert("_id", ObjectId::new());

                out.push(Event::ReturnedFromAltScreen.into());
                match edit_doc(duplicated_doc) {
                    Ok(new_doc) => {
                        out.push(Message::to_client(ClientAction::InsertDoc(new_doc)).into());
                    }
                    Err(err) => out.push(Event::ErrorOccurred(err.to_string()).into()),
                }
            }
            Command::Delete => {
                out.push(
                    Message::to_tab(TabAction::RequestConfirmation(ConfirmKind::DeleteDoc)).into(),
                );
            }
            Command::Yank => {
                if let Some(bson) = self.selected_bson() {
                    if send_bson_to_clipboard(bson).is_ok() {
                        out.push(Event::DataSentToClipboard.into());
                    }
                }
            }
            _ => {}
        }
        out
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        let mut out = vec![];
        match event {
            Event::DocumentsUpdated { docs, reset_state } => {
                self.set_docs(docs, *reset_state);
                out.push(Event::ListSelectionChanged.into());
            }
            Event::CountUpdated(count) => {
                self.count = *count;
            }
            Event::ConfirmYes(Command::Delete) => {
                if self.is_focused() {
                    if let Some(doc) = self.selected_doc() {
                        return vec![
                            Message::to_client(ClientAction::DeleteDoc(doc.clone())).into()
                        ];
                    }
                }
            }
            Event::DocumentPageChanged(page) => {
                self.page = *page;
            }
            Event::CollectionDropConfirmed(dropped_selected) => {
                if *dropped_selected {
                    self.documents = vec![];
                    self.items = vec![];
                }
            }

            _ => (),
        }
        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let (border_color, highlight_color) = if self.is_focused() {
            (Color::Green, Color::White)
        } else {
            (Color::White, Color::Gray)
        };

        let start = self.page * PAGE_SIZE + 1;
        #[expect(clippy::cast_possible_truncation)]
        let end = (start + PAGE_SIZE - 1).min(self.count as usize);

        let title = format!("Documents ({start}-{end} of {})", self.count);

        let widget = Tree::new(&self.items)
            .expect("all item identifiers are unique")
            .block(
                Block::bordered()
                    .title(format!(" {title} "))
                    .border_style(Style::default().fg(border_color)),
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ))
            .highlight_style(Style::default().bold().black().bg(highlight_color));

        frame.render_stateful_widget(widget, area, &mut self.state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedDocuments {
    selection: Vec<MongoKey>,
    page: usize,
    docs: Vec<Bson>,
    count: u64,
}

impl PersistedComponent for Documents<'_> {
    type StorageType = PersistedDocuments;

    fn persist(&self) -> Self::StorageType {
        PersistedDocuments {
            selection: self.state.selected().to_vec(),
            page: self.page,
            docs: self.documents.clone(),
            count: self.count,
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.page = storage.page;
        self.count = storage.count;
        self.set_docs(&storage.docs, true);

        // FIXME: get this working again
        // (tests will pass with this stuff uncommented, but the stored selection
        // gets overridden by the queries that get executed after creating a new client)
        // self.state.select(storage.selection);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::testing::ComponentTestHarness;
    use mongodb::bson::bson;

    #[test]
    fn select_first_item_on_new_data() {
        let mut test = ComponentTestHarness::new(Documents::default());

        let doc = bson!({ "_id": "document-id" });
        let docs = vec![doc];

        test.given_event(Event::DocumentsUpdated {
            docs,
            reset_state: true,
        });

        assert_eq!(
            test.component().state.selected(),
            vec![MongoKey::String("document-id".into())]
        );
    }

    #[test]
    fn persisting_and_hydrate() {
        let doc = bson!({ "_id": "document-id" });
        let docs = vec![doc];
        let mut component = Documents {
            documents: docs,
            ..Default::default()
        };
        component
            .state
            .select(vec![MongoKey::String("document-id".into())]);

        let persisted_component = component.persist();

        let mut new_component = Documents::default();
        new_component.hydrate(persisted_component);

        assert_eq!(component.documents, new_component.documents);

        // FIXME: restore this
        // assert_eq!(component.state.selected(), new_component.state.selected());
    }
}
