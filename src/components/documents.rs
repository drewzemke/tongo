#![allow(clippy::cast_possible_truncation)]

use super::{primary_screen::PrimaryScreenFocus, Component};
use crate::{
    app::AppFocus,
    client::PAGE_SIZE,
    components::ComponentCommand,
    system::{
        command::{Command, CommandGroup},
        event::Event,
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
use std::{cell::RefCell, rc::Rc};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Debug, Default)]
pub struct Documents<'a> {
    app_focus: Rc<RefCell<AppFocus>>,
    state: TreeState<MongoKey>,
    items: Vec<TreeItem<'a, MongoKey>>,

    #[allow(clippy::struct_field_names)]
    documents: Vec<Bson>,

    page: Rc<RefCell<usize>>,
    count: u64,
}

impl<'a> Documents<'a> {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>, doc_page: Rc<RefCell<usize>>) -> Self {
        Self {
            app_focus,
            page: doc_page,
            ..Default::default()
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

    fn page(&self) -> usize {
        *self.page.borrow()
    }
}

impl<'a> Component for Documents<'a> {
    fn is_focused(&self) -> bool {
        *self.app_focus.borrow() == AppFocus::PrimaryScreen(PrimaryScreenFocus::DocTree)
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::PrimaryScreen(PrimaryScreenFocus::DocTree);
    }
    fn commands(&self) -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(
                vec![
                    Command::NavLeft,
                    Command::NavUp,
                    Command::NavDown,
                    Command::NavRight,
                ],
                "navigate",
            ),
            CommandGroup::new(vec![Command::ExpandCollapse], "expand/collapse"),
            CommandGroup::new(
                vec![Command::NextPage, Command::PreviousPage],
                "next/prev page",
            ),
            CommandGroup::new(
                vec![Command::FirstPage, Command::LastPage],
                "first/last page",
            ),
            CommandGroup::new(vec![Command::Refresh], "refresh"),
            CommandGroup::new(vec![Command::Yank], "yank selected"),
            CommandGroup::new(vec![Command::EditDoc], "edit doc"),
            CommandGroup::new(vec![Command::InsertDoc], "insert doc"),
            CommandGroup::new(vec![Command::DuplicateDoc], "duplicate doc"),
            CommandGroup::new(vec![Command::DeleteDoc], "delete doc"),
        ]
    }

    #[allow(clippy::too_many_lines)]
    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        let ComponentCommand::Command(command) = command else {
            return vec![];
        };

        let mut out = vec![];
        match command {
            Command::NavLeft => {
                if self.state.key_left() {
                    out.push(Event::ListSelectionChanged);
                }
            }
            Command::NavUp => {
                if self.state.key_up() {
                    out.push(Event::ListSelectionChanged);
                }
            }
            Command::NavDown => {
                if self.state.key_down() {
                    out.push(Event::ListSelectionChanged);
                }
            }
            Command::NavRight => {
                if self.state.key_right() {
                    out.push(Event::ListSelectionChanged);
                }
            }
            Command::ExpandCollapse => {
                if self.state.toggle_selected() {
                    out.push(Event::ListSelectionChanged);
                }
            }
            Command::NextPage => {
                let end = (self.page() + 1) * PAGE_SIZE;
                if end < self.count as usize {
                    *self.page.borrow_mut() += 1;
                    out.push(Event::DocumentPageChanged);
                }
            }
            Command::PreviousPage => {
                if self.page() > 0 {
                    *self.page.borrow_mut() -= 1;
                    out.push(Event::DocumentPageChanged);
                }
            }
            Command::FirstPage => {
                *self.page.borrow_mut() = 0;
                out.push(Event::DocumentPageChanged);
            }
            Command::LastPage => {
                let last_page = (self.count as usize).div_ceil(PAGE_SIZE) - 1;
                *self.page.borrow_mut() = last_page;
                out.push(Event::DocumentPageChanged);
            }
            Command::Refresh => {
                out.push(Event::RefreshRequested);
            }
            Command::EditDoc => {
                let Some(doc) = self.selected_doc() else {
                    return out;
                };

                out.push(Event::ReturnedFromAltScreen);
                match edit_doc(doc.clone()) {
                    Ok(new_doc) => out.push(Event::DocumentEdited(new_doc)),
                    Err(err) => out.push(Event::ErrorOccurred(err.to_string())),
                };
            }
            Command::InsertDoc => {
                let doc = doc! { "_id" : ObjectId::new() };

                out.push(Event::ReturnedFromAltScreen);

                match edit_doc(doc) {
                    Ok(new_doc) => out.push(Event::DocumentCreated(new_doc)),
                    Err(err) => out.push(Event::ErrorOccurred(err.to_string())),
                };
            }
            Command::DuplicateDoc => {
                let Some(doc) = self.selected_doc() else {
                    return out;
                };

                let mut duplicated_doc = doc.clone();
                let _ = duplicated_doc.insert("_id", ObjectId::new());

                out.push(Event::ReturnedFromAltScreen);
                match edit_doc(duplicated_doc) {
                    Ok(new_doc) => out.push(Event::DocumentCreated(new_doc)),
                    Err(err) => out.push(Event::ErrorOccurred(err.to_string())),
                };
            }
            // TODO: this needs some sort of confirmation
            Command::DeleteDoc => {
                let Some(doc) = self.selected_doc() else {
                    return out;
                };

                out.push(Event::DocumentDeleted(doc.clone()));
            }
            Command::Yank => {
                if let Some(bson) = self.selected_bson() {
                    if send_bson_to_clipboard(bson).is_ok() {
                        out.push(Event::DataSentToClipboard);
                    };
                }
            }
            _ => {}
        }
        out
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        match event {
            Event::DocumentsUpdated { docs, reset_state } => {
                self.documents.clone_from(docs);

                let items: Vec<_> = docs
                    .iter()
                    .filter_map(|bson| bson.as_document().map(top_level_document))
                    .collect();

                if *reset_state {
                    // reset state to have all top-level documents expanded
                    let mut state = TreeState::default();
                    for item in &items {
                        state.open(vec![item.identifier().clone()]);
                    }
                    self.state = state;
                }

                self.items = items;
            }
            Event::CountUpdated(count) => {
                self.count = *count;
            }

            _ => (),
        }
        vec![]
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let border_color = if self.is_focused() {
            Color::Green
        } else {
            Color::White
        };

        let start = *self.page.borrow() * PAGE_SIZE + 1;
        let end = (start + PAGE_SIZE - 1).min(self.count as usize);

        let title = format!("Documents ({start}-{end} of {})", self.count);

        let widget = Tree::new(&self.items)
            .expect("all item identifiers are unique")
            .block(
                Block::bordered()
                    .title(title)
                    .border_style(Style::default().fg(border_color)),
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ))
            .highlight_style(Style::new().black().on_white().bold());

        frame.render_stateful_widget(widget, area, &mut self.state);
    }
}
