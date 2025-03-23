use super::{confirm_modal::ConfirmKind, primary_screen::PrimScrFocus, tab::TabFocus, Component};
use crate::{
    client::PAGE_SIZE,
    config::{color_map::ColorKey, Config},
    model::collection::Collection,
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        message::{AppAction, ClientAction, Message, PrimScreenAction, TabAction},
        Signal,
    },
    utils::{
        clipboard::send_bson_to_clipboard,
        doc_searcher::DocSearcher,
        edit_doc::edit_doc,
        mongo_tree::{MongoKey, MongoTreeBuilder},
    },
};
use layout::Flex;
use mongodb::bson::{doc, oid::ObjectId, Bson, Document};
use ratatui::{
    prelude::*,
    widgets::{Block, Scrollbar, ScrollbarOrientation},
};
use serde::{Deserialize, Serialize};
use std::{cell::Cell, rc::Rc};
use tui_input::{backend::crossterm::EventHandler, Input};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Debug, Default)]
enum Mode {
    #[default]
    Normal,
    SearchInput,
    SearchReview,
}

#[derive(Debug, Default)]
pub struct Documents<'a> {
    focus: Rc<Cell<TabFocus>>,
    config: Config,

    state: TreeState<MongoKey>,
    items: Vec<TreeItem<'a, MongoKey>>,
    mongo_tree_builder: MongoTreeBuilder<'a>,

    #[expect(clippy::struct_field_names)]
    documents: Vec<Bson>,
    collection: Option<Collection>,

    page: usize,
    count: u64,

    // search things
    mode: Mode,
    search_input: Input,
    searcher: DocSearcher,
}

impl Clone for Documents<'_> {
    fn clone(&self) -> Self {
        let documents = self.documents.clone();
        let mut searcher = DocSearcher::default();
        searcher.load_docs(&documents);

        let mut documents = Self {
            focus: self.focus.clone(),
            config: self.config.clone(),
            state: TreeState::default(),
            items: self.items.clone(),
            mongo_tree_builder: self.mongo_tree_builder.clone(),
            documents,
            collection: self.collection.clone(),
            mode: Mode::Normal,
            search_input: Input::default(),
            searcher,
            page: self.page,
            count: self.count,
        };
        documents.reset_state();
        documents
    }
}

impl Documents<'_> {
    pub fn new(focus: Rc<Cell<TabFocus>>, config: Config) -> Self {
        let mongo_tree_builder = MongoTreeBuilder::new(config.clone());
        Self {
            focus,
            config,
            mongo_tree_builder,
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
        self.searcher.load_docs(docs);

        let items: Vec<_> = docs
            .iter()
            .filter_map(|bson| {
                bson.as_document()
                    .map(|doc| self.mongo_tree_builder.build_tree_item(doc))
            })
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

    fn set_selection_to_search_match(&mut self) {
        if let Some(keys) = self.searcher.current_match() {
            tracing::trace!("selecting {keys:?}");
            self.state.select(keys.clone());

            // open the selected key and every parent so that the selected
            // item is visible
            for idx in (0..keys.len()).rev() {
                let suffix = keys[0..idx].to_vec();
                tracing::trace!("opening {suffix:?}");

                let not_already_open = self.state.open(suffix);
                if !not_already_open {
                    break;
                }
            }

            self.state.open(keys.clone());
        }
    }

    fn reset_search(&mut self) {
        self.search_input = Input::default();
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
        // handle search input mode separately
        if matches!(self.mode, Mode::SearchInput) {
            return vec![
                CommandGroup::new(vec![Command::Confirm], "end search")
                    .in_cat(CommandCategory::StatusBarOnly),
                CommandGroup::new(vec![Command::Back], "cancel")
                    .in_cat(CommandCategory::StatusBarOnly),
            ];
        }

        let mut out = if matches!(self.mode, Mode::Normal) {
            vec![
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
                CommandGroup::new(vec![Command::Refresh], "refresh")
                    .in_cat(CommandCategory::DocActions),
                CommandGroup::new(vec![Command::Search], "fuzzy search")
                    .in_cat(CommandCategory::DocNav),
            ]
        } else {
            // self.mode == Mode::SearchReview
            vec![
                CommandGroup::new(
                    vec![Command::PreviousPage, Command::NextPage],
                    "previous/next result",
                )
                .in_cat(CommandCategory::DocNav),
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
                CommandGroup::new(vec![Command::Back], "exit search")
                    .in_cat(CommandCategory::StatusBarOnly),
                CommandGroup::new(vec![Command::Back], "exit search")
                    .in_cat(CommandCategory::DocNav),
                CommandGroup::new(vec![Command::Refresh], "refresh")
                    .in_cat(CommandCategory::DocActions),
                CommandGroup::new(vec![Command::Search], "fuzzy search")
                    .in_cat(CommandCategory::DocNav),
            ]
        };

        out.push(
            CommandGroup::new(vec![Command::CreateNew], "insert document")
                .in_cat(CommandCategory::DocActions),
        );

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
            Command::NextPage => match self.mode {
                Mode::Normal => {
                    let end = (self.page + 1) * PAGE_SIZE;

                    #[expect(clippy::cast_possible_truncation)]
                    if end < self.count as usize {
                        self.page += 1;
                        out.push(Event::DocumentPageChanged(self.page).into());
                    }
                }
                Mode::SearchReview => {
                    self.searcher.next_match();
                    self.set_selection_to_search_match();
                    out.push(Event::ListSelectionChanged.into());
                }
                Mode::SearchInput => {}
            },
            Command::PreviousPage => match self.mode {
                Mode::Normal => {
                    if self.page > 0 {
                        self.page -= 1;
                        out.push(Event::DocumentPageChanged(self.page).into());
                    }
                }
                Mode::SearchReview => {
                    self.searcher.prev_match();
                    self.set_selection_to_search_match();
                    out.push(Event::ListSelectionChanged.into());
                }
                Mode::SearchInput => {}
            },
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
            Command::Search => {
                self.mode = Mode::SearchInput;
                out.push(Message::to_app(AppAction::EnterRawMode).into());
            }

            // only in search input mode
            Command::Confirm => {
                if matches!(self.mode, Mode::SearchInput) {
                    self.mode = Mode::SearchReview;
                    out.push(Message::to_app(AppAction::ExitRawMode).into());
                }
            }

            // search input mode or search review mode
            Command::Back => match self.mode {
                Mode::SearchInput => {
                    self.mode = Mode::Normal;
                    self.reset_search();
                    out.push(Message::to_app(AppAction::ExitRawMode).into());
                    out.push(Event::DocSearchUpdated.into());
                }
                Mode::SearchReview => {
                    self.mode = Mode::Normal;
                    self.reset_search();
                    out.push(Event::DocSearchUpdated.into());
                }
                Mode::Normal => {
                    out.push(
                        Message::to_prim_scr(PrimScreenAction::SetFocus(PrimScrFocus::CollList))
                            .into(),
                    );
                }
            },
            _ => {}
        }
        out
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event) -> Vec<Signal> {
        if matches!(self.mode, Mode::SearchInput) {
            self.search_input.handle_event(event);
            self.searcher.update_pattern(self.search_input.value());
            self.set_selection_to_search_match();
            vec![Event::DocSearchUpdated.into()]
        } else {
            vec![]
        }
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
            Event::CollectionSelected(coll) => {
                self.collection = Some(coll.clone());
            }

            Event::ConnectionSelected(_)
            | Event::ConnectionCreated(_)
            | Event::DatabaseSelected(_) => {
                self.collection = None;
            }

            _ => (),
        }
        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let (border_color, bg_color) = if self.is_focused() {
            (
                self.config.color_map.get(&ColorKey::PanelFocusedBorder),
                self.config.color_map.get(&ColorKey::PanelFocusedBg),
            )
        } else {
            (
                self.config.color_map.get(&ColorKey::PanelUnfocusedBorder),
                self.config.color_map.get(&ColorKey::PanelUnfocusedBg),
            )
        };

        // if no collection is selected, render a "no data" message
        let Some(coll) = &self.collection else {
            let block = Block::bordered()
                .title(" Documents ")
                .bg(bg_color)
                .border_style(Style::default().fg(border_color));
            frame.render_widget(block, area);

            let layout = Layout::vertical([1]).flex(Flex::Center).split(area);
            let widget = Line::from(
                "(no collection selected)".fg(self.config.color_map.get(&ColorKey::DocumentsNote)),
            )
            .centered();
            frame.render_widget(widget, layout[0]);

            return;
        };

        let start = self.page * PAGE_SIZE + 1;
        #[expect(clippy::cast_possible_truncation)]
        let end = (start + PAGE_SIZE - 1).min(self.count as usize);

        let title_left = format!("Documents in '{}'", coll.name);
        let title_right = format!("{start}-{end} of {}", self.count);

        let mut block = Block::bordered()
            .bg(bg_color)
            .title(Line::from(format!(" {title_left} ")).left_aligned())
            .title(Line::from(format!(" {title_right} ")).right_aligned())
            .border_style(Style::default().fg(border_color));

        let match_n = self.searcher.match_idx() + 1;
        let num_matches = self.searcher.num_matches();
        let match_word = if num_matches == 1 { "match" } else { "matches" };
        block = match self.mode {
            Mode::Normal => block,
            Mode::SearchInput => block
                .title_bottom(
                    Line::from(format!(" search: \"{}\" ", self.search_input.value()))
                        .left_aligned()
                        .fg(self.config.color_map.get(&ColorKey::DocumentsSearch)),
                )
                .title_bottom(
                    Line::from(format!(" {num_matches} {match_word} "))
                        .right_aligned()
                        .fg(self.config.color_map.get(&ColorKey::DocumentsSearch)),
                ),
            Mode::SearchReview => block
                .title_bottom(
                    Line::from(format!(" search: \"{}\" ", self.search_input.value()))
                        .left_aligned()
                        .fg(self.config.color_map.get(&ColorKey::DocumentsSearch)),
                )
                .title_bottom(
                    Line::from(format!(" match {match_n} of {num_matches} "))
                        .right_aligned()
                        .fg(self.config.color_map.get(&ColorKey::DocumentsSearch)),
                ),
        };

        let widget = Tree::new(&self.items)
            .expect("all item identifiers are unique")
            .block(block)
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ))
            .style(Style::default().fg(self.config.color_map.get(&ColorKey::FgPrimary)))
            .highlight_style(
                Style::default()
                    .bold()
                    .fg(self.config.color_map.get(&ColorKey::SelectionFg))
                    .bg(self.config.color_map.get(&ColorKey::SelectionBg)),
            );

        frame.render_stateful_widget(widget, area, &mut self.state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedDocuments {
    selection: Vec<MongoKey>,
    page: usize,
    docs: Vec<Bson>,
    collection: Option<Collection>,
    count: u64,
}

impl PersistedComponent for Documents<'_> {
    type StorageType = PersistedDocuments;

    fn persist(&self) -> Self::StorageType {
        PersistedDocuments {
            selection: self.state.selected().to_vec(),
            page: self.page,
            docs: self.documents.clone(),
            collection: self.collection.clone(),
            count: self.count,
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.page = storage.page;
        self.count = storage.count;
        self.set_docs(&storage.docs, true);
        self.collection = storage.collection;

        // FIXME: get this working again
        // (tests will pass with this stuff uncommented, but the stored selection
        // gets overridden by the queries that get executed after creating a new client)
        // self.state.select(storage.selection);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        model::{connection::Connection, database::Database},
        testing::ComponentTestHarness,
    };
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
    fn record_collection_changes() {
        let mut test = ComponentTestHarness::new(Documents::default());

        let collection = Collection::new("test".to_string());
        test.given_event(Event::CollectionSelected(collection.clone()));

        assert!(test
            .component()
            .collection
            .as_ref()
            .is_some_and(|c| c.name == "test"));

        test.given_event(Event::ConnectionSelected(Connection::default()));

        assert!(test.component().collection.is_none());

        test.given_event(Event::CollectionSelected(collection));
        test.given_event(Event::DatabaseSelected(Database::default()));

        assert!(test.component().collection.is_none());
    }

    #[test]
    fn persisting_and_hydrate() {
        let doc = bson!({ "_id": "document-id" });
        let docs = vec![doc];
        let coll = Collection::new("test!".to_string());
        let mut component = Documents {
            documents: docs,
            collection: Some(coll),
            ..Default::default()
        };
        component
            .state
            .select(vec![MongoKey::String("document-id".into())]);

        let persisted_component = component.persist();

        let mut new_component = Documents::default();
        new_component.hydrate(persisted_component);

        assert_eq!(component.documents, new_component.documents);
        assert_eq!(component.collection, new_component.collection);

        // FIXME: restore this
        // assert_eq!(component.state.selected(), new_component.state.selected());
    }
}
