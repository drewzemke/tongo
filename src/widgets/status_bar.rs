#![allow(clippy::module_name_repetitions)]

use crate::{command::CommandGroup, components::Component};
use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph, Wrap},
};

#[derive(Debug, Default)]
pub struct StatusBarState {
    pub message: Option<String>,
}

#[derive(Debug, Default)]
pub struct StatusBar {
    pub commands: Vec<CommandGroup>,
    pub message: Option<String>,
}

impl Component for StatusBar {
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
}
