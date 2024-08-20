#![allow(clippy::cast_possible_truncation)]

use crate::{
    app::AppFocus,
    client::PAGE_SIZE,
    command::{Command, CommandGroup},
    components::ComponentCommand,
    event::Event,
    screens::primary_screen::PrimaryScreenFocus,
    tree::{top_level_document, MongoKey},
};
use mongodb::bson::{Bson, Document};
use ratatui::{
    prelude::*,
    widgets::{Block, Scrollbar, ScrollbarOrientation},
};
use std::{cell::RefCell, rc::Rc};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use super::{Component, UniqueType};

#[derive(Debug, Default)]
pub struct Documents<'a> {
    app_focus: Rc<RefCell<AppFocus>>,
    pub state: TreeState<MongoKey>,
    pub items: Vec<TreeItem<'a, MongoKey>>,

    #[allow(clippy::struct_field_names)]
    pub documents: Vec<Bson>,

    pub page: usize,
    pub count: u64,
}

impl<'a> Component<UniqueType> for Documents<'a> {
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
                "←↑↓→/hjkl",
                "navigate",
            ),
            CommandGroup::new(vec![Command::ExpandCollapse], "space", "expand/collapse"),
            CommandGroup::new(
                vec![Command::NextPage, Command::PreviousPage],
                "n/p",
                "next/prev page",
            ),
            CommandGroup::new(vec![Command::InsertDoc], "I", "insert doc"),
            CommandGroup::new(vec![Command::EditDoc], "E", "edit doc"),
            CommandGroup::new(vec![Command::DuplicateDoc], "C", "duplicate doc"),
            CommandGroup::new(vec![Command::DeleteDoc], "D", "delete doc"),
        ]
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        let ComponentCommand::Command(command) = command else {
            return vec![];
        };

        let mut out = vec![];
        match command {
            Command::NavLeft => {
                self.state.key_left();
                out.push(Event::ListSelectionChanged);
            }
            Command::NavUp => {
                self.state.key_up();
                out.push(Event::ListSelectionChanged);
            }
            Command::NavDown => {
                self.state.key_down();
                out.push(Event::ListSelectionChanged);
            }
            Command::NavRight => {
                self.state.key_right();
                out.push(Event::ListSelectionChanged);
            }
            Command::ExpandCollapse => {
                self.state.toggle_selected();
                out.push(Event::ListSelectionChanged);
            }
            Command::NextPage => {
                let end = (self.page + 1) * PAGE_SIZE;
                if end < self.count as usize {
                    self.page += 1;
                    out.push(Event::DocumentPageChanged(self.page));
                }
            }
            Command::PreviousPage => {
                if self.page > 0 {
                    self.page -= 1;
                    out.push(Event::DocumentPageChanged(self.page));
                }
            }
            _ => {}
        }
        out
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        match event {
            Event::DocumentsUpdated(docs) => {
                self.documents.clone_from(docs);

                let items: Vec<_> = docs
                    .iter()
                    .filter_map(|bson| bson.as_document().map(top_level_document))
                    .collect();

                // if reset_selection {
                // reset state to have all top-level documents expanded
                let mut state = TreeState::default();
                for item in &items {
                    state.open(vec![item.identifier().clone()]);
                }
                self.state = state;
                // }

                self.items = items;
            }
            Event::CountUpdated(count) => {
                self.count = *count;
            }
            Event::DocumentPageChanged(page) => {
                self.page = *page;
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

        let start = self.page * PAGE_SIZE + 1;
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

impl<'a> Documents<'a> {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>) -> Self {
        Self {
            app_focus,
            ..Default::default()
        }
    }

    pub fn selected_doc(&self) -> Option<&Document> {
        let id = self.state.selected().first()?;

        let bson = self
            .items
            .iter()
            .position(|tree_item| tree_item.identifier() == id)
            .and_then(|index| self.documents.get(index));

        if let Some(Bson::Document(doc)) = bson {
            Some(doc)
        } else {
            None
        }
    }
}
