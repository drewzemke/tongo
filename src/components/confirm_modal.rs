use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};
use std::{cell::Cell, rc::Rc};

use crate::{
    components::{tab::TabFocus, Component},
    config::{color_map::ColorKey, Config},
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        Signal,
    },
};

const CONFIRM_MODAL_WIDTH: u16 = 40;
const CONFIRM_MODAL_HEIGHT: u16 = 3;

#[derive(Debug, Clone, Copy)]
pub enum ConfirmKind {
    DeleteConnection,
    DeleteDoc,
    DropCollection,
    DropDatabase,
}

impl ConfirmKind {
    const fn command(self) -> Command {
        match self {
            Self::DropDatabase
            | Self::DropCollection
            | Self::DeleteDoc
            | Self::DeleteConnection => Command::Delete,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ConfirmModal {
    focus: Rc<Cell<TabFocus>>,
    kind: Option<ConfirmKind>,
    config: Config,
}
impl ConfirmModal {
    pub fn new(focus: Rc<Cell<TabFocus>>, config: Config) -> Self {
        Self {
            focus,
            config,
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
            Some(ConfirmKind::DropCollection) => Some((
                "Confirm Drop",
                "Are you sure you want to drop this collection? This cannot be undone.",
            )),
            Some(ConfirmKind::DropDatabase) => Some((
                "Confirm Drop",
                "Are you sure? If you drop the database in tongo, you drop it in real life.",
            )),

            None => None,
        }
    }
}

impl Component for ConfirmModal {
    fn is_focused(&self) -> bool {
        self.focus.get() == TabFocus::ConfModal
    }

    fn focus(&self) {
        self.focus.set(TabFocus::ConfModal);
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let Some((title, message)) = self.text_content() else {
            return;
        };

        let layout = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Length(CONFIRM_MODAL_HEIGHT + 2),
            Constraint::Fill(1),
        ])
        .split(area);
        let layout = Layout::horizontal(vec![
            Constraint::Fill(1),
            Constraint::Length(CONFIRM_MODAL_WIDTH + 2),
            Constraint::Fill(1),
        ])
        .split(layout[1]);

        let content = Paragraph::new(message).wrap(Wrap { trim: true }).block(
            Block::bordered()
                .border_style(self.config.color_map.get(&ColorKey::PopupBorder))
                .title(format!(" {title} "))
                .fg(self.config.color_map.get(&ColorKey::FgPrimary))
                .bg(self.config.color_map.get(&ColorKey::PopupBg)),
        );

        frame.render_widget(Clear, layout[1]);
        frame.render_widget(content, layout[1]);
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::Confirm], "confirm")
                .in_cat(CommandCategory::StatusBarOnly),
            CommandGroup::new(vec![Command::Back], "cancel").in_cat(CommandCategory::StatusBarOnly),
        ]
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        let Some(confirm_kind) = &self.kind else {
            return vec![];
        };

        let mut out = vec![];
        match command {
            Command::Confirm => out.push(Event::ConfirmYes(confirm_kind.command()).into()),
            Command::Back => out.push(Event::ConfirmNo.into()),
            _ => {}
        }
        out
    }
}
