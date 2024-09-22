use crate::{
    components::ComponentCommand,
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};

pub mod collections;
pub mod connections;
pub mod databases;

#[derive(Debug, Default)]
pub struct InnerList {
    title: &'static str,
    pub state: ListState,
}

impl InnerList {
    fn new(title: &'static str) -> Self {
        Self {
            title,
            ..Default::default()
        }
    }

    fn base_commands() -> Vec<CommandGroup> {
        vec![CommandGroup::new(
            vec![Command::NavUp, Command::NavDown],
            "navigate",
        )]
    }

    fn handle_base_command(&mut self, command: &ComponentCommand, num_items: usize) -> Vec<Event> {
        let mut out = vec![];
        if let ComponentCommand::Command(command) = command {
            match command {
                Command::NavUp => {
                    // jump to the bottom if we're at the top
                    if self.state.selected() == Some(0) {
                        self.state.select(Some(num_items.saturating_sub(1)));
                    } else {
                        self.state.select_previous();
                    }
                    out.push(Event::ListSelectionChanged);
                }
                Command::NavDown => {
                    // jump to the top if we're at the bottom
                    if self.state.selected() == Some(num_items.saturating_sub(1)) {
                        self.state.select_first();
                    } else {
                        self.state.select_next();
                    }
                    out.push(Event::ListSelectionChanged);
                }
                _ => {}
            }
        }
        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, items: Vec<ListItem>, focused: bool) {
        let (border_color, highlight_color) = if focused {
            (Color::Green, Color::White)
        } else {
            (Color::White, Color::Gray)
        };

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(self.title)
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(Style::default().bold().black().bg(highlight_color));

        StatefulWidget::render(list, area, frame.buffer_mut(), &mut self.state);
    }
}
