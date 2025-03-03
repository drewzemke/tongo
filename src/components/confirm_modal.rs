use super::{tab::TabFocus, ComponentCommand};
use crate::{
    components::Component,
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
};
use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};
use std::{cell::RefCell, rc::Rc};

const CONFIRM_MODAL_WIDTH: u16 = 40;
const CONFIRM_MODAL_HEIGHT: u16 = 3;

#[derive(Debug, Clone, Copy)]
pub enum ConfirmKind {
    DeleteConnection,
    DeleteDoc,
}

impl ConfirmKind {
    fn command(&self) -> Command {
        // match self {
        //     Self::DeleteConnection => Command::Delete,
        //     Self::DeleteDoc => Command::DeleteDoc,
        // }
        Command::Delete
    }
}

#[derive(Debug, Default, Clone)]
pub struct ConfirmModal {
    focus: Rc<RefCell<TabFocus>>,
    kind: Option<ConfirmKind>,
}
impl ConfirmModal {
    pub fn new(focus: Rc<RefCell<TabFocus>>) -> Self {
        Self {
            focus,
            ..Default::default()
        }
    }

    pub fn show_with(&mut self, kind: ConfirmKind) {
        self.kind = Some(kind);
        self.focus();
    }

    const fn text_content(&self) -> Option<(&'static str, &'static str)> {
        match self.kind {
            Some(ConfirmKind::DeleteConnection) => Some((
                "Confirm Delete",
                "Are you sure you want to delete this connection? This cannot be undone.",
            )),
            Some(ConfirmKind::DeleteDoc) => Some((
                "Confirm Delete",
                "Are you sure you want to delete this document? This cannot be undone.",
            )),
            _ => None,
        }
    }
}

impl Component for ConfirmModal {
    fn is_focused(&self) -> bool {
        *self.focus.borrow() == TabFocus::ConfModal
    }

    fn focus(&self) {
        *self.focus.borrow_mut() = TabFocus::ConfModal;
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let Some((title, message)) = self.text_content() else {
            return;
        };

        let layout = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Length(CONFIRM_MODAL_HEIGHT + 4),
            Constraint::Fill(1),
        ])
        .split(area);
        let layout = Layout::horizontal(vec![
            Constraint::Fill(1),
            Constraint::Length(CONFIRM_MODAL_WIDTH + 4),
            Constraint::Fill(1),
        ])
        .split(layout[1]);

        let content = Paragraph::new(message).wrap(Wrap { trim: true }).block(
            Block::bordered()
                .title(title)
                .border_style(Style::default().fg(Color::Magenta)),
        );

        frame.render_widget(Clear, layout[1]);
        frame.render_widget(content, layout[1].inner(Margin::new(1, 1)));
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::Confirm], "confirm"),
            CommandGroup::new(vec![Command::Back], "cancel"),
        ]
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        let ComponentCommand::Command(command) = command else {
            return vec![];
        };
        let Some(confirm_kind) = &self.kind else {
            return vec![];
        };

        let mut out = vec![];
        match command {
            Command::Confirm => out.push(Event::ConfirmationYes(confirm_kind.command())),
            Command::Back => out.push(Event::ConfirmationNo),
            _ => {}
        }
        out
    }
}
