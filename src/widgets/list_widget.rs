use crossterm::event::{Event, KeyCode};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};
use std::slice::Iter;

pub trait ListWidget {
    type Item;
    type State;

    fn title() -> &'static str;

    fn items(state: &Self::State) -> Iter<Self::Item>;

    fn item_to_str(item: &Self::Item) -> Text<'static>;

    fn is_focused(state: &Self::State) -> bool;

    fn list_state(state: &mut Self::State) -> &mut ListState;

    fn on_change(_state: &mut Self::State) {}

    fn on_select(_state: &mut Self::State) {}

    fn on_event(_event: &Event, _state: &mut Self::State) -> bool {
        false
    }

    fn render(area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let focused = Self::is_focused(state);
        let border_color = if focused { Color::Green } else { Color::White };

        let items: Vec<ListItem> = Self::items(state)
            .map(|item| ListItem::new(Self::item_to_str(item)))
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(Self::title())
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(Style::default().bold().reversed().white());

        StatefulWidget::render(list, area, buf, Self::list_state(state));
    }

    fn handle_event(event: &Event, state: &mut Self::State) -> bool {
        let updated = if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    Self::list_state(state).select_next();
                    Self::on_change(state);
                    true
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    Self::list_state(state).select_previous();
                    Self::on_change(state);
                    true
                }
                KeyCode::Enter => {
                    Self::on_select(state);
                    true
                }
                _ => false,
            }
        } else {
            false
        };

        Self::on_event(event, state) || updated
    }
}
