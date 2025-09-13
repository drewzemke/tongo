use std::{cell::Cell, rc::Rc};

use ratatui::{
    layout::Offset,
    prelude::*,
    widgets::{Block, Borders},
};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        input::filter::FilterInput,
        tab::{CloneWithFocus, TabFocus},
        Component,
    },
    config::{color_map::ColorKey, Config},
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        signal::SignalQueue,
    },
};

#[derive(Debug, Default, Clone)]
pub struct QueryInput {
    #[expect(dead_code)]
    focus: Rc<Cell<TabFocus>>,
    config: Config,
    filter_input: FilterInput,
    expanded: bool,
}

impl CloneWithFocus for QueryInput {
    fn clone_with_focus(&self, focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            filter_input: self.filter_input.clone_with_focus(focus.clone()),
            config: self.config.clone(),
            focus,
            expanded: self.expanded,
        }
    }
}

impl QueryInput {
    pub fn new(
        focus: Rc<Cell<TabFocus>>,
        cursor_pos: Rc<Cell<(u16, u16)>>,
        config: Config,
    ) -> Self {
        let filter_input = FilterInput::new(focus.clone(), cursor_pos, config.clone());
        Self {
            focus,
            config,
            filter_input,
            expanded: false,
        }
    }

    pub const fn is_editing(&self) -> bool {
        self.filter_input.is_editing()
    }

    pub const fn is_expanded(&self) -> bool {
        self.expanded
    }
}

impl Component for QueryInput {
    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = if self.expanded {
            vec![
                CommandGroup::new(vec![Command::ExpandCollapse], "hide adv. query")
                    .in_cat(CommandCategory::FilterInputActions),
            ]
        } else {
            vec![
                CommandGroup::new(vec![Command::ExpandCollapse], "show adv. query")
                    .in_cat(CommandCategory::FilterInputActions),
            ]
        };
        out.append(&mut self.filter_input.commands());
        out
    }

    fn handle_command(&mut self, command: &Command, queue: &mut SignalQueue) {
        if matches!(command, Command::ExpandCollapse) {
            self.expanded = !self.expanded;
            queue.push(Event::QueryInputExpanded);
        } else {
            self.filter_input.handle_command(command, queue);
        }
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event, queue: &mut SignalQueue) {
        self.filter_input.handle_raw_event(event, queue);
    }

    fn focus(&self) {
        self.filter_input.focus();
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.expanded {
            let layout = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(area);
            let mut subarea = layout[0];

            self.filter_input.render(frame, subarea);
            let block = Block::default().borders(Borders::ALL);
            block
                .clone()
                .title(" Query ")
                .render(subarea, frame.buffer_mut());

            let top_corners = symbols::border::Set {
                top_left: symbols::line::NORMAL.vertical_right,
                top_right: symbols::line::NORMAL.vertical_left,
                ..symbols::border::PLAIN
            };

            subarea = subarea.offset(Offset { x: 0, y: 2 });
            self.filter_input.render(frame, subarea);
            block
                .clone()
                .title(" Projection ")
                .border_set(top_corners)
                .render(subarea, frame.buffer_mut());

            subarea = subarea.offset(Offset { x: 0, y: 2 });
            self.filter_input.render(frame, subarea);
            block
                .title(" Sort ")
                .border_set(top_corners)
                .render(subarea, frame.buffer_mut());
        } else {
            let (border_color, bg_color) = if self.filter_input.is_focused() {
                let border_color = if self.is_editing() {
                    self.config.color_map.get(&ColorKey::PanelActiveInputBorder)
                } else {
                    self.config.color_map.get(&ColorKey::PanelActiveBorder)
                };
                (
                    border_color,
                    self.config.color_map.get(&ColorKey::PanelActiveBg),
                )
            } else {
                (
                    self.config.color_map.get(&ColorKey::PanelInactiveBorder),
                    self.config.color_map.get(&ColorKey::PanelInactiveBg),
                )
            };

            self.filter_input.render(frame, area);

            let block = Block::default()
                .bg(bg_color)
                .title(" Query ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color));
            block.render(area, frame.buffer_mut());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedQueryInput {
    filter_input: String,
    expanded: bool,
}

impl PersistedComponent for QueryInput {
    type StorageType = PersistedQueryInput;

    fn persist(&self) -> Self::StorageType {
        PersistedQueryInput {
            filter_input: self.filter_input.persist(),
            expanded: self.expanded,
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.filter_input.hydrate(storage.filter_input);
        self.expanded = storage.expanded;
    }
}
