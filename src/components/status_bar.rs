use crate::{
    components::Component,
    system::{command::CommandGroup, event::Event},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph, Wrap},
};

use super::ComponentCommand;

#[derive(Debug, Default)]
pub struct StatusBar {
    pub commands: Vec<CommandGroup>,
    pub message: Option<String>,
}

impl Component for StatusBar {
    fn focus(&self) {}

    fn is_focused(&self) -> bool {
        false
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let content = self.message.as_ref().map_or_else(
            || {
                Line::from(
                    self.commands
                        .iter()
                        .flat_map(Into::<Vec<Span>>::into)
                        .collect::<Vec<Span>>(),
                )
            },
            |message| {
                Line::from(vec![
                    Span::styled("Error: ", Style::default().red()),
                    Span::from(message.clone()),
                ])
            },
        );

        let paragraph = Paragraph::new(content)
            .wrap(Wrap::default())
            .block(Block::default().padding(Padding::horizontal(1)));
        frame.render_widget(paragraph, area);
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![]
    }

    fn handle_command(&mut self, _command: &ComponentCommand) -> Vec<Event> {
        vec![]
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        if let Event::ErrorOccurred(error) = event {
            self.message = Some(error.clone());
        };
        vec![]
    }
}
