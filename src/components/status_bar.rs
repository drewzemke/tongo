use super::ComponentCommand;
use crate::{
    components::Component,
    system::{command::CommandGroup, event::Event},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph, Wrap},
};

const DEBUG_RENDER_COUNT: bool = false;

#[derive(Debug, Default)]
pub struct StatusBar {
    pub commands: Vec<CommandGroup>,
    pub message: Option<String>,

    renders: usize,
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

        let content = Paragraph::new(content)
            .wrap(Wrap::default())
            .block(Block::default().padding(Padding::horizontal(1)));

        if DEBUG_RENDER_COUNT {
            self.renders += 1;
            let render_count_content = Paragraph::new(format!("{}", &self.renders));

            let layout =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(4)]).split(area);
            frame.render_widget(content, layout[0]);
            frame.render_widget(render_count_content, layout[1]);
        } else {
            frame.render_widget(content, area);
        }
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
