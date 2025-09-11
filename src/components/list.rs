use crate::{
    config::{color_map::ColorKey, Config},
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        signal::SignalQueue,
    },
};
use ratatui::{
    prelude::*,
    style::Style,
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};

pub mod collections;
pub mod connections;
pub mod databases;

#[derive(Debug, Default, Clone)]
pub struct InnerList {
    title: &'static str,
    config: Config,
    pub state: ListState,
}

impl InnerList {
    fn new(title: &'static str, config: Config) -> Self {
        Self {
            title,
            config,
            ..Default::default()
        }
    }

    fn base_commands() -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::NavUp, Command::NavDown], "navigate")
                .in_cat(CommandCategory::AppNav),
        ]
    }

    fn handle_base_command(
        &mut self,
        command: &Command,
        num_items: usize,
        queue: &mut SignalQueue,
    ) {
        match command {
            Command::NavUp => {
                // jump to the bottom if we're at the top
                if self.state.selected() == Some(0) {
                    self.state.select(Some(num_items.saturating_sub(1)));
                } else {
                    self.state.select_previous();
                }
                queue.push(Event::ListSelectionChanged);
            }
            Command::NavDown => {
                // jump to the top if we're at the bottom
                if self.state.selected() == Some(num_items.saturating_sub(1)) {
                    self.state.select_first();
                } else {
                    self.state.select_next();
                }
                queue.push(Event::ListSelectionChanged);
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, items: Vec<ListItem>, focused: bool) {
        let (border_color, bg_color, highlight_text, highlight_bg, unselected_fg) = if focused {
            (
                self.config.color_map.get(&ColorKey::PanelActiveBorder),
                self.config.color_map.get(&ColorKey::PanelActiveBg),
                self.config.color_map.get(&ColorKey::SelectionFg),
                self.config.color_map.get(&ColorKey::SelectionBg),
                self.config.color_map.get(&ColorKey::FgPrimary),
            )
        } else {
            (
                self.config.color_map.get(&ColorKey::PanelInactiveBorder),
                self.config.color_map.get(&ColorKey::PanelInactiveBg),
                self.config.color_map.get(&ColorKey::FgPrimary),
                self.config.color_map.get(&ColorKey::PanelInactiveBg),
                self.config.color_map.get(&ColorKey::FgSecondary),
            )
        };

        let list = List::new(items)
            .block(
                Block::bordered()
                    .bg(bg_color)
                    .title(format!(" {} ", self.title))
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().fg(unselected_fg))
            .highlight_style(Style::default().bold().fg(highlight_text).bg(highlight_bg));

        StatefulWidget::render(list, area, frame.buffer_mut(), &mut self.state);
    }
}
