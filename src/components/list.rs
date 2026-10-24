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

/// A trait for datatypes used in the generic list.
pub trait InnerListItem: Default + Clone + PartialEq {
    fn to_list_item(&self, config: &Config) -> ListItem<'_>;
}

#[derive(Debug, Default, Clone)]
pub struct InnerList<T> {
    title: &'static str,
    config: Config,
    state: ListState,
    items: Vec<T>,
}

impl<T> InnerList<T>
where
    T: InnerListItem,
{
    pub fn new(title: &'static str, config: Config) -> Self {
        Self {
            title,
            config,
            ..Default::default()
        }
    }

    pub fn set_items(&mut self, items: &[T])
    where
        T: Clone,
    {
        self.items = items.to_vec();
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn select(&mut self, item: Option<T>)
    where
        T: PartialEq,
    {
        let index = item.and_then(|item| self.items.iter().position(|it| *it == item));
        self.state.select(index);
    }

    pub fn selected(&self) -> Option<&T> {
        self.state
            .selected()
            .and_then(|index| self.items.get(index))
    }

    pub const fn num_items(&self) -> usize {
        self.items.len()
    }

    #[expect(clippy::unused_self)]
    pub fn base_commands(&self) -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::NavUp, Command::NavDown], "navigate")
                .in_cat(CommandCategory::AppNav),
        ]
    }

    pub fn handle_base_command(
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

    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
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

        let list = List::new(
            self.items
                .iter()
                .map(|item| item.to_list_item(&self.config)),
        )
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
