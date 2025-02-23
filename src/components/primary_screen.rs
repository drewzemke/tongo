use super::{
    connection_screen::ConnScrFocus,
    documents::PersistedDocuments,
    list::{collections::PersistedCollections, databases::PersistedDatabases},
};
use crate::{
    app::AppFocus,
    components::{
        documents::Documents,
        input::filter::FilterInput,
        list::{collections::Collections, databases::Databases},
        Component, ComponentCommand,
    },
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
};
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimScrFocus {
    #[default]
    DbList,
    CollList,
    DocTree,
    FilterIn,
}

#[derive(Debug, Default)]
pub struct PrimaryScreen<'a> {
    app_focus: Rc<RefCell<AppFocus>>,
    db_list: Databases,
    coll_list: Collections,
    doc_tree: Documents<'a>,
    filter_input: FilterInput,
}

impl PrimaryScreen<'_> {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>, cursor_pos: Rc<Cell<(u16, u16)>>) -> Self {
        let db_list = Databases::new(app_focus.clone());
        let coll_list = Collections::new(app_focus.clone());
        let doc_tree = Documents::new(app_focus.clone());
        let filter_input = FilterInput::new(app_focus.clone(), cursor_pos);
        Self {
            app_focus,
            db_list,
            coll_list,
            doc_tree,
            filter_input,
        }
    }

    /// Narrows the shared `AppFocus` variable into the focus enum for this componenent
    fn internal_focus(&self) -> Option<PrimScrFocus> {
        match &*self.app_focus.borrow() {
            AppFocus::PrimScr(focus) => Some(focus.clone()),
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

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        // we need to pass the command to the currently-focused component first,
        // the way this component handles the command might change the focus
        let mut out = match self.internal_focus() {
            Some(PrimScrFocus::DbList) => self.db_list.handle_command(command),
            Some(PrimScrFocus::CollList) => self.coll_list.handle_command(command),
            Some(PrimScrFocus::DocTree) => self.doc_tree.handle_command(command),
            Some(PrimScrFocus::FilterIn) => self.filter_input.handle_command(command),
            None => vec![],
        };

        if let ComponentCommand::Command(command) = command {
            match command {
                Command::FocusLeft => match self.internal_focus() {
                    Some(PrimScrFocus::DocTree) => {
                        self.coll_list.focus();
                        out.push(Event::FocusedChanged);
                    }
                    Some(PrimScrFocus::FilterIn) => {
                        self.db_list.focus();
                        out.push(Event::FocusedChanged);
                    }
                    _ => {}
                },
                Command::FocusUp => match self.internal_focus() {
                    Some(PrimScrFocus::CollList) => {
                        self.db_list.focus();
                        out.push(Event::FocusedChanged);
                    }
                    Some(PrimScrFocus::DocTree) => {
                        self.filter_input.focus();
                        out.push(Event::FocusedChanged);
                    }
                    _ => {}
                },
                Command::FocusDown => match self.internal_focus() {
                    Some(PrimScrFocus::DbList) => {
                        self.coll_list.focus();
                        out.push(Event::FocusedChanged);
                    }
                    Some(PrimScrFocus::FilterIn) => {
                        self.doc_tree.focus();
                        out.push(Event::FocusedChanged);
                    }
                    _ => {}
                },
                Command::FocusRight => match self.internal_focus() {
                    Some(PrimScrFocus::DbList) => {
                        self.filter_input.focus();
                        out.push(Event::FocusedChanged);
                    }
                    Some(PrimScrFocus::CollList) => {
                        self.doc_tree.focus();
                        out.push(Event::FocusedChanged);
                    }
                    _ => {}
                },
                Command::Back => match self.internal_focus() {
                    Some(PrimScrFocus::DbList) => {
                        *self.app_focus.borrow_mut() = AppFocus::ConnScr(ConnScrFocus::ConnList);
                        out.push(Event::FocusedChanged);
                    }
                    Some(PrimScrFocus::CollList) => {
                        self.db_list.focus();
                        out.push(Event::FocusedChanged);
                    }
                    Some(PrimScrFocus::DocTree) => {
                        self.coll_list.focus();
                        out.push(Event::FocusedChanged);
                    }
                    Some(PrimScrFocus::FilterIn) => {
                        self.doc_tree.focus();
                        out.push(Event::FocusedChanged);
                    }
                    None => {}
                },
                _ => {}
            }
        }
        out
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        match event {
            Event::DatabaseSelected => self.coll_list.focus(),
            Event::CollectionSelected(..) | Event::DocFilterUpdated(..) => self.doc_tree.focus(),
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
        *self.app_focus.borrow_mut() = AppFocus::PrimScr(PrimScrFocus::default());
    }

    fn is_focused(&self) -> bool {
        matches!(*self.app_focus.borrow(), AppFocus::PrimScr(..))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedPrimaryScreen {
    db_list: PersistedDatabases,
    coll_list: PersistedCollections,
    doc_tree: PersistedDocuments,
}

impl PersistedComponent for PrimaryScreen<'_> {
    type StorageType = PersistedPrimaryScreen;

    fn persist(&self) -> Self::StorageType {
        PersistedPrimaryScreen {
            db_list: self.db_list.persist(),
            coll_list: self.coll_list.persist(),
            doc_tree: self.doc_tree.persist(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) -> Vec<Event> {
        let mut out = vec![];

        out.append(&mut self.db_list.hydrate(storage.db_list));
        out.append(&mut self.coll_list.hydrate(storage.coll_list));
        out.append(&mut self.doc_tree.hydrate(storage.doc_tree));

        out
    }
}
