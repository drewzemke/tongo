use crate::system::{
    command::{Command, CommandCategory, CommandGroup},
    event::Event,
    Signal,
};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};

pub mod collections;
pub mod connections;
pub mod databases;

#[derive(Debug, Default, Clone)]
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
        vec![
            CommandGroup::new(vec![Command::NavUp, Command::NavDown], "navigate")
                .in_cat(CommandCategory::AppNav),
        ]
    }

    fn handle_base_command(&mut self, command: &Command, num_items: usize) -> Vec<Signal> {
        let mut out = vec![];
        match command {
            Command::NavUp => {
                // jump to the bottom if we're at the top
                if self.state.selected() == Some(0) {
                    self.state.select(Some(num_items.saturating_sub(1)));
                } else {
                    self.state.select_previous();
                }
                out.push(Event::ListSelectionChanged.into());
            }
            Command::NavDown => {
                // jump to the top if we're at the bottom
                if self.state.selected() == Some(num_items.saturating_sub(1)) {
                    self.state.select_first();
                } else {
                    self.state.select_next();
                }
                out.push(Event::ListSelectionChanged.into());
            }
            _ => {}
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
                    .title(format!(" {} ", self.title))
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(Style::default().bold().black().bg(highlight_color));

        StatefulWidget::render(list, area, frame.buffer_mut(), &mut self.state);
    }
}
