use crate::{
    components::Component,
    system::{
        command::{Command, CommandGroup},
        message::{AppAction, Message},
        Signal,
    },
};
use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};

const HELP_MODAL_WIDTH: u16 = 40;
const HELP_MODAL_HEIGHT: u16 = 3;

#[derive(Debug, Default, Clone)]
pub struct HelpModal {}

impl HelpModal {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Component for HelpModal {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let title = "Available Commands";

        let layout = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Length(HELP_MODAL_HEIGHT + 4),
            Constraint::Fill(1),
        ])
        .split(area);
        let layout = Layout::horizontal(vec![
            Constraint::Fill(1),
            Constraint::Length(HELP_MODAL_WIDTH + 6),
            Constraint::Fill(1),
        ])
        .split(layout[1]);

        let content = Paragraph::new("Oh hello!").wrap(Wrap { trim: true }).block(
            Block::bordered()
                .title(title)
                .border_style(Style::default().fg(Color::Magenta)),
        );

        frame.render_widget(Clear, layout[1]);
        frame.render_widget(content, layout[1].inner(Margin::new(2, 1)));
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![CommandGroup::new(vec![Command::Back], "close")]
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        if matches!(command, Command::Back) {
            vec![Message::to_app(AppAction::CloseHelpModal).into()]
        } else {
            vec![]
        }
    }
}
