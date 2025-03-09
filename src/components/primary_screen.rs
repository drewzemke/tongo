use super::{
    connection_screen::ConnScrFocus,
    documents::PersistedDocuments,
    list::{collections::PersistedCollections, databases::PersistedDatabases},
    tab::TabFocus,
};
use crate::{
    components::{
        documents::Documents,
        input::filter::FilterInput,
        list::{collections::Collections, databases::Databases},
        Component,
    },
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandGroup},
        event::Event,
        Signal,
    },
};
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimScrFocus {
    #[default]
    DbList,
    CollList,
    DocTree,
    FilterIn,
}

#[derive(Debug, Default, Clone)]
pub struct PrimaryScreen<'a> {
    focus: Rc<Cell<TabFocus>>,
    db_list: Databases,
    coll_list: Collections,
    doc_tree: Documents<'a>,
    filter_input: FilterInput,
}

impl PrimaryScreen<'_> {
    pub fn new(focus: Rc<Cell<TabFocus>>, cursor_pos: Rc<Cell<(u16, u16)>>) -> Self {
        let db_list = Databases::new(focus.clone());
        let coll_list = Collections::new(focus.clone());
        let doc_tree = Documents::new(focus.clone());
        let filter_input = FilterInput::new(focus.clone(), cursor_pos);
        Self {
            focus,
            db_list,
            coll_list,
            doc_tree,
            filter_input,
        }
    }

    /// Narrows the shared `AppFocus` variable into the focus enum for this componenent
    fn internal_focus(&self) -> Option<PrimScrFocus> {
        match self.focus.get() {
            TabFocus::PrimScr(focus) => Some(focus),
            _ => None,
        }
    }
}

impl Component for PrimaryScreen<'_> {
    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = vec![];

        if !self.filter_input.is_editing() {
            out.push(CommandGroup::new(
                vec![
                    Command::FocusLeft,
                    Command::FocusDown,
                    Command::FocusUp,
                    Command::FocusRight,
                ],
                "change focus",
            ));
            out.push(CommandGroup::new(vec![Command::Back], "back"));
        }

        match self.internal_focus() {
            Some(PrimScrFocus::DbList) => out.append(&mut self.db_list.commands()),
            Some(PrimScrFocus::CollList) => out.append(&mut self.coll_list.commands()),
            Some(PrimScrFocus::DocTree) => out.append(&mut self.doc_tree.commands()),
            Some(PrimScrFocus::FilterIn) => out.append(&mut self.filter_input.commands()),
            None => {}
        }
        out
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        // we need to pass the command to the currently-focused component first,
        // the way this component handles the command might change the focus
        let mut out = match self.internal_focus() {
            Some(PrimScrFocus::DbList) => self.db_list.handle_command(command),
            Some(PrimScrFocus::CollList) => self.coll_list.handle_command(command),
            Some(PrimScrFocus::DocTree) => self.doc_tree.handle_command(command),
            Some(PrimScrFocus::FilterIn) => self.filter_input.handle_command(command),
            None => vec![],
        };

        match command {
            Command::FocusLeft => match self.internal_focus() {
                Some(PrimScrFocus::DocTree) => {
                    self.coll_list.focus();
                    out.push(Event::FocusedChanged.into());
                }
                Some(PrimScrFocus::FilterIn) => {
                    self.db_list.focus();
                    out.push(Event::FocusedChanged.into());
                }
                _ => {}
            },
            Command::FocusUp => match self.internal_focus() {
                Some(PrimScrFocus::CollList) => {
                    self.db_list.focus();
                    out.push(Event::FocusedChanged.into());
                }
                Some(PrimScrFocus::DocTree) => {
                    self.filter_input.focus();
                    out.push(Event::FocusedChanged.into());
                }
                _ => {}
            },
            Command::FocusDown => match self.internal_focus() {
                Some(PrimScrFocus::DbList) => {
                    self.coll_list.focus();
                    out.push(Event::FocusedChanged.into());
                }
                Some(PrimScrFocus::FilterIn) => {
                    self.doc_tree.focus();
                    out.push(Event::FocusedChanged.into());
                }
                _ => {}
            },
            Command::FocusRight => match self.internal_focus() {
                Some(PrimScrFocus::DbList) => {
                    self.filter_input.focus();
                    out.push(Event::FocusedChanged.into());
                }
                Some(PrimScrFocus::CollList) => {
                    self.doc_tree.focus();
                    out.push(Event::FocusedChanged.into());
                }
                _ => {}
            },
            Command::Back => match self.internal_focus() {
                Some(PrimScrFocus::DbList) => {
                    self.focus.set(TabFocus::ConnScr(ConnScrFocus::ConnList));
                    out.push(Event::FocusedChanged.into());
                }
                Some(PrimScrFocus::CollList) => {
                    self.db_list.focus();
                    out.push(Event::FocusedChanged.into());
                }
                Some(PrimScrFocus::DocTree) => {
                    self.coll_list.focus();
                    out.push(Event::FocusedChanged.into());
                }
                Some(PrimScrFocus::FilterIn) => {
                    self.doc_tree.focus();
                    out.push(Event::FocusedChanged.into());
                }
                None => {}
            },
            _ => {}
        }
        out
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event) -> Vec<Signal> {
        match self.internal_focus() {
            Some(PrimScrFocus::DbList) => self.db_list.handle_raw_event(event),
            Some(PrimScrFocus::CollList) => self.coll_list.handle_raw_event(event),
            Some(PrimScrFocus::DocTree) => self.doc_tree.handle_raw_event(event),
            Some(PrimScrFocus::FilterIn) => self.filter_input.handle_raw_event(event),
            None => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        let mut out = vec![];
        match event {
            Event::DatabaseSelected(_) => self.coll_list.focus(),
            Event::CollectionSelected(_) | Event::DocFilterUpdated(_) => self.doc_tree.focus(),
            _ => {}
        }
        out.append(&mut self.db_list.handle_event(event));
        out.append(&mut self.coll_list.handle_event(event));
        out.append(&mut self.doc_tree.handle_event(event));
        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Min(20)])
            .split(area);
        let sidebar = content_layout[0];
        let main_view = content_layout[1];

        let sidebar_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(sidebar);
        let sidebar_top = sidebar_layout[0];
        let sidebar_btm = sidebar_layout[1];

        let main_view_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Percentage(100)])
            .split(main_view);
        let main_view_top = main_view_layout[0];
        let main_view_btm = main_view_layout[1];

        self.db_list.render(frame, sidebar_top);
        self.coll_list.render(frame, sidebar_btm);
        self.doc_tree.render(frame, main_view_btm);
        self.filter_input.render(frame, main_view_top);
    }

    fn focus(&self) {
        self.focus.set(TabFocus::PrimScr(PrimScrFocus::default()));
    }

    fn is_focused(&self) -> bool {
        matches!(self.focus.get(), TabFocus::PrimScr(..))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedPrimaryScreen {
    db_list: PersistedDatabases,
    coll_list: PersistedCollections,
    doc_tree: PersistedDocuments,
    filter_input: String,
}

impl PersistedComponent for PrimaryScreen<'_> {
    type StorageType = PersistedPrimaryScreen;

    fn persist(&self) -> Self::StorageType {
        PersistedPrimaryScreen {
            db_list: self.db_list.persist(),
            coll_list: self.coll_list.persist(),
            doc_tree: self.doc_tree.persist(),
            filter_input: self.filter_input.persist(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.db_list.hydrate(storage.db_list);
        self.coll_list.hydrate(storage.coll_list);
        self.doc_tree.hydrate(storage.doc_tree);
        self.filter_input.hydrate(storage.filter_input);
    }
}
