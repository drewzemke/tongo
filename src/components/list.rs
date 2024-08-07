use crossterm::event::{Event, KeyCode};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};
use std::slice::Iter;

pub trait ListComponent {
    type Item;

    fn title() -> &'static str;

    fn items(&self) -> Iter<Self::Item>;

    fn item_to_str(item: &Self::Item) -> Text<'static>;

    fn is_focused(&self) -> bool;

    fn list_state(&mut self) -> &mut ListState;

    fn on_change(&mut self) {}

    fn on_select(&mut self) {}

    fn on_event(&mut self, _event: &Event) -> bool {
        false
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
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

        StatefulWidget::render(list, area, buf, self.list_state());
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        let updated = if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    let num_items = self.items().len();
                    let list_state = self.list_state();
                    // jump to the top if we're at the bottom
                    if list_state.selected() == Some(num_items - 1) {
                        list_state.select_first();
                    } else {
                        list_state.select_next();
                    }
                    self.on_change();
                    true
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let num_items = self.items().len();
                    let list_state = self.list_state();
                    // jump to the bottom if we're at the top
                    if list_state.selected() == Some(0) {
                        list_state.select(Some(num_items.saturating_sub(1)));
                    } else {
                        list_state.select_previous();
                    }
                    self.on_change();
                    true
                }
                KeyCode::Enter => {
                    self.on_select();
                    true
                }
                _ => false,
            }
        } else {
            false
        };

        self.on_event(event) || updated
    }
}
