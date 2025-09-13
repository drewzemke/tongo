use std::{cell::Cell, rc::Rc};

use ratatui::{layout::Offset, prelude::*, symbols, widgets::Block};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        input::filter::FilterInput,
        primary_screen::PrimScrFocus,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryInFocus {
    #[default]
    Filter,
    Project,
    Sort,
}

#[derive(Debug, Default, Clone)]
pub struct QueryInput {
    focus: Rc<Cell<TabFocus>>,
    config: Config,

    filter_input: FilterInput,
    projection_input: FilterInput,
    sort_input: FilterInput,

    expanded: bool,
}

impl CloneWithFocus for QueryInput {
    fn clone_with_focus(&self, focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            filter_input: self.filter_input.clone_with_focus(focus.clone()),
            projection_input: self.projection_input.clone_with_focus(focus.clone()),
            sort_input: self.sort_input.clone_with_focus(focus.clone()),
            focus,
            config: self.config.clone(),
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
        let filter_input = FilterInput::new(focus.clone(), cursor_pos.clone(), config.clone());
        let projection_input = FilterInput::new(focus.clone(), cursor_pos.clone(), config.clone());
        let sort_input = FilterInput::new(focus.clone(), cursor_pos, config.clone());
        Self {
            focus,
            config,
            filter_input,
            projection_input,
            sort_input,
            expanded: false,
        }
    }

    pub const fn is_editing(&self) -> bool {
        self.filter_input.is_editing()
    }

    pub const fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Narrows the shared `AppFocus` variable into the focus enum for this componenent
    fn internal_focus(&self) -> Option<QueryInFocus> {
        match self.focus.get() {
            TabFocus::PrimScr(PrimScrFocus::QueryIn(focus)) => Some(focus),
            _ => None,
        }
    }

    /// Returns whether or not the focus state changed, which would not
    /// be the case if the focus is already at the top of the component
    pub fn focus_up(&self) -> bool {
        if !self.is_expanded() {
            return false;
        }
        match self.internal_focus() {
            Some(QueryInFocus::Project) => {
                self.filter_input.focus();
                true
            }
            Some(QueryInFocus::Sort) => {
                self.projection_input.focus();
                true
            }
            _ => false,
        }
    }

    /// Returns whether or not the focus state changed, which would not
    /// be the case if the focus is already at the bottom of the component
    pub fn focus_down(&self) -> bool {
        if !self.is_expanded() {
            return false;
        }
        match self.internal_focus() {
            Some(QueryInFocus::Filter) => {
                self.projection_input.focus();
                true
            }
            Some(QueryInFocus::Project) => {
                self.sort_input.focus();
                true
            }
            _ => false,
        }
    }

    pub fn focus_last(&self) {
        if self.is_expanded() {
            self.sort_input.focus();
        } else {
            self.filter_input.focus();
        }
    }

    /// Returns (border color, background color) for the given subcomponent
    fn get_color_for(&self, input: QueryInFocus) -> (Color, Color) {
        self.internal_focus().map_or_else(
            || {
                (
                    self.config.color_map.get(&ColorKey::PanelInactiveBorder),
                    self.config.color_map.get(&ColorKey::PanelInactiveBg),
                )
            },
            |focus| {
                let border_color = if focus == input {
                    if self.is_editing() {
                        self.config.color_map.get(&ColorKey::PanelActiveInputBorder)
                    } else {
                        self.config.color_map.get(&ColorKey::PanelActiveBorder)
                    }
                } else {
                    self.config.color_map.get(&ColorKey::PanelInactiveBorder)
                };
                (
                    border_color,
                    self.config.color_map.get(&ColorKey::PanelActiveBg),
                )
            },
        )
    }

    /// Returns (border color, background color) based on:
    /// - whether the subcomponent to which this component belongs is focused
    /// - whether the entire query component is focused
    /// - whether an input is in editing mode
    fn get_colors(&self, subcomponent_focused: bool) -> (Color, Color) {
        let component_focused = self.internal_focus().is_some();
        let is_editing = self.is_editing();

        if component_focused {
            if subcomponent_focused {
                if is_editing {
                    (
                        self.config.color_map.get(&ColorKey::PanelActiveInputBorder),
                        self.config.color_map.get(&ColorKey::PanelActiveBg),
                    )
                } else {
                    (
                        self.config.color_map.get(&ColorKey::PanelActiveBorder),
                        self.config.color_map.get(&ColorKey::PanelActiveBg),
                    )
                }
            } else {
                (
                    self.config.color_map.get(&ColorKey::PanelInactiveBorder),
                    self.config.color_map.get(&ColorKey::PanelActiveBg),
                )
            }
        } else {
            (
                self.config.color_map.get(&ColorKey::PanelInactiveBorder),
                self.config.color_map.get(&ColorKey::PanelInactiveBg),
            )
        }
    }

    /// renders the component in a given rect (that is assumed to be of height 3),
    /// along with the side borders
    fn render_subcomponent(&mut self, subcomponent: QueryInFocus, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.internal_focus(), Some(f) if f == subcomponent);
        let (border_color, bg_color) = self.get_colors(focused);
        let border_style = Style::default().fg(border_color).bg(bg_color);

        let label = match subcomponent {
            QueryInFocus::Filter => "filt",
            QueryInFocus::Project => "proj",
            QueryInFocus::Sort => "sort",
        };
        let label_and_border = format!("{}{label}:", symbols::line::NORMAL.vertical);

        #[expect(clippy::cast_possible_truncation)]
        let layout = Layout::horizontal([
            Constraint::Length(label_and_border.len() as u16 - 1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);
        let label_area = layout[0];
        let input_area = layout[1];
        let right_border_area = layout[2];

        frame.render_widget(Line::from(label_and_border).style(border_style), label_area);

        let input = match subcomponent {
            QueryInFocus::Filter => &mut self.filter_input,
            QueryInFocus::Project => &mut self.projection_input,
            QueryInFocus::Sort => &mut self.sort_input,
        };
        input.render(frame, input_area);

        frame.render_widget(
            Line::from(symbols::line::NORMAL.vertical).style(border_style),
            right_border_area,
        );
    }

    /// renders at the top of `area`
    fn render_top_border(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.internal_focus(), Some(QueryInFocus::Filter));
        let (border_color, bg_color) = self.get_colors(focused);
        let border_style = Style::default().fg(border_color).bg(bg_color);

        let title = " Query ";
        let title_text_color = if self.internal_focus().is_some() {
            self.config.color_map.get(&ColorKey::FgPrimary)
        } else {
            self.config.color_map.get(&ColorKey::PanelInactiveBorder)
        };
        let title_style = Style::default().fg(title_text_color).bg(bg_color);

        let buf = frame.buffer_mut();

        buf.set_string(area.x, area.y, symbols::line::NORMAL.top_left, border_style);
        buf.set_string(area.x + 1, area.y, title, title_style);

        let horiz_line = symbols::line::HORIZONTAL.repeat(area.width as usize - title.len() - 2);
        #[expect(clippy::cast_possible_truncation)]
        buf.set_string(
            area.x + 1 + title.len() as u16,
            area.y,
            &horiz_line,
            border_style,
        );

        buf.set_string(
            area.x + area.width - 1,
            area.y,
            symbols::line::NORMAL.top_right,
            border_style,
        );
    }

    /// renders at the bottom of `area`
    fn render_bottom_border(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.internal_focus(), Some(QueryInFocus::Sort));
        let (border_color, bg_color) = self.get_colors(focused);
        let style = Style::default().fg(border_color).bg(bg_color);

        let horizontal_line = symbols::line::HORIZONTAL.repeat((area.width - 2) as usize);
        let border_line = format!(
            "{}{}{}",
            symbols::line::NORMAL.bottom_left,
            horizontal_line,
            symbols::line::NORMAL.bottom_right
        );
        frame
            .buffer_mut()
            .set_string(area.x, area.y + 2, &border_line, style);
    }

    /// renders at the top of `area`
    fn render_horizontal_divider1(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(
            self.internal_focus(),
            Some(QueryInFocus::Filter | QueryInFocus::Project)
        );
        let (border_color, bg_color) = self.get_colors(focused);
        let style = Style::default().fg(border_color).bg(bg_color);

        let horizontal_line = symbols::line::HORIZONTAL.repeat((area.width - 2) as usize);
        let border_line = format!(
            "{}{}{}",
            symbols::line::NORMAL.vertical_right,
            horizontal_line,
            symbols::line::NORMAL.vertical_left
        );
        frame
            .buffer_mut()
            .set_string(area.x, area.y, &border_line, style);
    }

    /// renders at the top of `area`
    fn render_horizontal_divider2(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(
            self.internal_focus(),
            Some(QueryInFocus::Project | QueryInFocus::Sort)
        );
        let (border_color, bg_color) = self.get_colors(focused);
        let style = Style::default().fg(border_color).bg(bg_color);

        let horizontal_line = symbols::line::HORIZONTAL.repeat((area.width - 2) as usize);
        let border_line = format!(
            "{}{}{}",
            symbols::line::NORMAL.vertical_right,
            horizontal_line,
            symbols::line::NORMAL.vertical_left
        );
        frame
            .buffer_mut()
            .set_string(area.x, area.y, &border_line, style);
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
        match self.internal_focus() {
            Some(QueryInFocus::Filter) => {
                out.append(&mut self.filter_input.commands());
            }
            Some(QueryInFocus::Project) => {
                out.append(&mut self.projection_input.commands());
            }
            Some(QueryInFocus::Sort) => {
                out.append(&mut self.sort_input.commands());
            }
            None => {}
        }
        out
    }

    fn handle_command(&mut self, command: &Command, queue: &mut SignalQueue) {
        if matches!(command, Command::ExpandCollapse) {
            self.expanded = !self.expanded;
            queue.push(Event::QueryInputExpanded);
        } else {
            match self.internal_focus() {
                Some(QueryInFocus::Filter) => self.filter_input.handle_command(command, queue),
                Some(QueryInFocus::Project) => self.projection_input.handle_command(command, queue),
                Some(QueryInFocus::Sort) => self.sort_input.handle_command(command, queue),
                None => {}
            }
        }
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event, queue: &mut SignalQueue) {
        match self.internal_focus() {
            Some(QueryInFocus::Filter) => self.filter_input.handle_raw_event(event, queue),
            Some(QueryInFocus::Project) => self.projection_input.handle_raw_event(event, queue),
            Some(QueryInFocus::Sort) => self.sort_input.handle_raw_event(event, queue),
            None => {}
        }
    }

    fn focus(&self) {
        self.filter_input.focus();
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.expanded {
            let layout = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(area);
            let mut rect = layout[0];

            self.render_subcomponent(
                QueryInFocus::Filter,
                frame,
                rect.offset(Offset { x: 0, y: 1 }),
            );
            self.render_top_border(frame, rect);

            rect = rect.offset(Offset { x: 0, y: 2 });

            self.render_subcomponent(
                QueryInFocus::Project,
                frame,
                rect.offset(Offset { x: 0, y: 1 }),
            );
            self.render_horizontal_divider1(frame, rect);

            rect = rect.offset(Offset { x: 0, y: 2 });

            self.render_subcomponent(
                QueryInFocus::Sort,
                frame,
                rect.offset(Offset { x: 0, y: 1 }),
            );
            self.render_horizontal_divider2(frame, rect);
            self.render_bottom_border(frame, rect);
        } else {
            let (border_color, bg_color) = self.get_color_for(QueryInFocus::Filter);

            self.filter_input.render(frame, area);

            let block = Block::default()
                .bg(bg_color)
                .title(" Query ")
                .border_style(Style::default().fg(border_color));
            block.render(area, frame.buffer_mut());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedQueryInput {
    filter_input: String,
    projection_input: String,
    sort_input: String,
    expanded: bool,
}

impl PersistedComponent for QueryInput {
    type StorageType = PersistedQueryInput;

    fn persist(&self) -> Self::StorageType {
        PersistedQueryInput {
            filter_input: self.filter_input.persist(),
            projection_input: self.projection_input.persist(),
            sort_input: self.sort_input.persist(),
            expanded: self.expanded,
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.filter_input.hydrate(storage.filter_input);
        self.projection_input.hydrate(storage.projection_input);
        self.sort_input.hydrate(storage.sort_input);
        self.expanded = storage.expanded;
    }
}
