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

#[derive(Debug, Default, Clone)]
pub struct ConfirmModal {
    focus: Rc<RefCell<TabFocus>>,
    command: Option<Command>,
}
impl ConfirmModal {
    pub fn new(focus: Rc<RefCell<TabFocus>>) -> Self {
        Self {
            focus,
            ..Default::default()
        }
    }

    pub fn show_with(&mut self, command: Command) {
        self.command = Some(command);
        self.focus();
    }

    const fn text_content(&self) -> Option<(&'static str, &'static str)> {
        match self.command {
            Some(Command::Delete) => Some((
                "Confirm Delete",
                "Are you sure you want to delete this connection?",
            )),
            Some(Command::DeleteDoc) => Some((
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

        let layout = Layout::default()
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(5),
                Constraint::Fill(1),
            ])
            .horizontal_margin(5)
            .split(area);

        let content = Paragraph::new(message).wrap(Wrap { trim: true }).block(
            Block::bordered()
                .title(title)
                .border_style(Style::default().fg(Color::Magenta)),
        );

        frame.render_widget(Clear, layout[1]);
        frame.render_widget(content, layout[1]);
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
        let Some(stored_command) = self.command else {
            return vec![];
        };

        let mut out = vec![];
        match command {
            Command::Confirm => out.push(Event::ConfirmationYes(stored_command)),
            Command::Back => out.push(Event::ConfirmationNo),
            _ => {}
        }
        out
    }
}
