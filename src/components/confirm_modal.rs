use super::ComponentCommand;
use crate::{
    app::AppFocus,
    components::Component,
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
};
use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default)]
pub struct ConfirmModal {
    app_focus: Rc<RefCell<AppFocus>>,
    command: Option<Command>,
}
impl ConfirmModal {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>) -> Self {
        Self {
            app_focus,
            ..Default::default()
        }
    }

    pub fn show_with(&mut self, command: Command) {
        self.command = Some(command);
        self.focus();
    }
}

impl Component for ConfirmModal {
    fn is_focused(&self) -> bool {
        *self.app_focus.borrow() == AppFocus::ConfirmModal
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::ConfirmModal;
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let Some(command) = self.command else {
            return;
        };

        let layout = Layout::default()
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .horizontal_margin(5)
            .split(area);

        let content = Paragraph::new(format!("{command:?}")).block(
            Block::bordered()
                .title("Are you sure?")
                .border_style(Style::default().fg(Color::Magenta)),
        );

        frame.render_widget(Clear, layout[1]);
        frame.render_widget(content, layout[1]);
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::Confirm], "yes"),
            CommandGroup::new(vec![Command::Back], "no"),
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
