use super::connection_screen::ConnScreenFocus;
use crate::{
    app::AppFocus,
    components::{
        documents::Documents,
        input::filter::FilterInput,
        list::{collections::Collections, databases::Databases},
        Component, ComponentCommand,
    },
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
};
use ratatui::prelude::*;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PrimaryScreenFocus {
    #[default]
    DbList,
    CollList,
    DocTree,
    FilterInput,
}

#[derive(Debug, Default)]
pub struct PrimaryScreen<'a> {
    app_focus: Rc<RefCell<AppFocus>>,
    db_list: Databases,
    coll_list: Collections,
    doc_tree: Documents<'a>,
    filter_input: FilterInput,
}

impl<'a> PrimaryScreen<'a> {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>, cursor_pos: Rc<RefCell<(u16, u16)>>) -> Self {
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
    fn internal_focus(&self) -> Option<PrimaryScreenFocus> {
        match &*self.app_focus.borrow() {
            AppFocus::ConnScreen(..) => None,
            AppFocus::PrimaryScreen(focus) => Some(focus.clone()),
        }
    }
}

impl<'a> Component for PrimaryScreen<'a> {
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
            Some(PrimaryScreenFocus::DbList) => out.append(&mut self.db_list.commands()),
            Some(PrimaryScreenFocus::CollList) => out.append(&mut self.coll_list.commands()),
            Some(PrimaryScreenFocus::DocTree) => out.append(&mut self.doc_tree.commands()),
            Some(PrimaryScreenFocus::FilterInput) => out.append(&mut self.filter_input.commands()),
            None => {}
        }
        out
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        if let ComponentCommand::Command(command) = command {
            match command {
                Command::FocusLeft => match self.internal_focus() {
                    Some(PrimaryScreenFocus::DocTree) => {
                        self.coll_list.focus();
                        return vec![Event::FocusedChanged];
                    }
                    Some(PrimaryScreenFocus::FilterInput) => {
                        self.db_list.focus();
                        return vec![Event::FocusedChanged];
                    }
                    _ => return vec![],
                },
                Command::FocusUp => match self.internal_focus() {
                    Some(PrimaryScreenFocus::CollList) => {
                        self.db_list.focus();
                        return vec![Event::FocusedChanged];
                    }
                    Some(PrimaryScreenFocus::DocTree) => {
                        self.filter_input.focus();
                        return vec![Event::FocusedChanged];
                    }
                    _ => return vec![],
                },
                Command::FocusDown => match self.internal_focus() {
                    Some(PrimaryScreenFocus::DbList) => {
                        self.coll_list.focus();
                        return vec![Event::FocusedChanged];
                    }
                    Some(PrimaryScreenFocus::FilterInput) => {
                        self.doc_tree.focus();
                        return vec![Event::FocusedChanged];
                    }
                    _ => return vec![],
                },
                Command::FocusRight => match self.internal_focus() {
                    Some(PrimaryScreenFocus::DbList) => {
                        self.filter_input.focus();
                        return vec![Event::FocusedChanged];
                    }
                    Some(PrimaryScreenFocus::CollList) => {
                        self.doc_tree.focus();
                        return vec![Event::FocusedChanged];
                    }
                    _ => return vec![],
                },
                Command::Back => match self.internal_focus() {
                    Some(PrimaryScreenFocus::DbList) => {
                        *self.app_focus.borrow_mut() =
                            AppFocus::ConnScreen(ConnScreenFocus::ConnList);
                        return vec![Event::FocusedChanged];
                    }
                    Some(PrimaryScreenFocus::CollList) => {
                        self.db_list.focus();
                        return vec![Event::FocusedChanged];
                    }
                    Some(PrimaryScreenFocus::DocTree) => {
                        self.coll_list.focus();
                        return vec![Event::FocusedChanged];
                    }
                    Some(PrimaryScreenFocus::FilterInput) => {
                        self.doc_tree.focus();
                        return vec![Event::FocusedChanged];
                    }
                    None => {}
                },
                _ => {}
            }
        };
        match self.internal_focus() {
            Some(PrimaryScreenFocus::DbList) => self.db_list.handle_command(command),
            Some(PrimaryScreenFocus::CollList) => self.coll_list.handle_command(command),
            Some(PrimaryScreenFocus::DocTree) => self.doc_tree.handle_command(command),
            Some(PrimaryScreenFocus::FilterInput) => self.filter_input.handle_command(command),
            None => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        match event {
            Event::DatabaseSelected => self.coll_list.focus(),
            Event::CollectionSelected | Event::DocFilterUpdated(..) => self.doc_tree.focus(),
            _ => {}
        };
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
        *self.app_focus.borrow_mut() = AppFocus::PrimaryScreen(PrimaryScreenFocus::default());
    }

    fn is_focused(&self) -> bool {
        matches!(*self.app_focus.borrow(), AppFocus::PrimaryScreen(..))
    }
}
