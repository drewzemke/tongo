use crate::{
    command::{Command, CommandGroup},
    components::{Component, ComponentCommand, ListType},
    event::Event,
};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};
use std::slice::Iter;

pub mod connection_list;

pub trait ListComponent {
    type Item;

    fn title() -> &'static str;

    fn items(&self) -> Iter<Self::Item>;

    fn item_to_str(item: &Self::Item) -> Text<'static>;

    fn focus(&self);

    fn is_focused(&self) -> bool;

    fn list_state(&mut self) -> &mut ListState;

    fn commands(&self) -> Vec<crate::command::CommandGroup> {
        vec![]
    }

    fn handle_command(&mut self, _command: &ComponentCommand) -> Vec<Event> {
        vec![]
    }
}

impl<T: ListComponent> Component<ListType> for T {
    fn commands(&self) -> Vec<crate::command::CommandGroup> {
        let mut out = vec![CommandGroup::new(
            vec![Command::NavUp, Command::NavDown],
            "↑↓/jk",
            "navigate",
        )];
        out.append(&mut ListComponent::commands(self));
        out
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        let mut out = vec![];
        if let ComponentCommand::Command(command) = command {
            match command {
                Command::NavUp => {
                    let num_items = self.items().len();
                    let list_state = self.list_state();
                    // jump to the bottom if we're at the top
                    if list_state.selected() == Some(0) {
                        list_state.select(Some(num_items.saturating_sub(1)));
                    } else {
                        list_state.select_previous();
                    }
                    out.push(Event::ListSelectionChanged);
                }
                Command::NavDown => {
                    let num_items = self.items().len();
                    let list_state = self.list_state();
                    // jump to the top if we're at the bottom
                    if list_state.selected() == Some(num_items - 1) {
                        list_state.select_first();
                    } else {
                        list_state.select_next();
                    }
                    out.push(Event::ListSelectionChanged);
                }
                _ => {}
            }
        }
        out.append(&mut ListComponent::handle_command(self, command));
        out
    }

    fn handle_event(&mut self, _event: &Event) -> Vec<Event> {
        vec![]
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let focused = self.is_focused();
        let border_color = if focused { Color::Green } else { Color::White };

        let items: Vec<ListItem> = self
            .items()
            .map(|item| ListItem::new(Self::item_to_str(item)))
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(Self::title())
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(Style::default().bold().reversed().white());

        StatefulWidget::render(list, area, frame.buffer_mut(), self.list_state());
    }

    fn focus(&self) {
        ListComponent::focus(self);
    }

    fn is_focused(&self) -> bool {
        ListComponent::is_focused(self)
    }
}
