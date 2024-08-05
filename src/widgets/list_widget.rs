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
                    let num_items = Self::items(state).len();
                    list_nav_down(Self::list_state(state), num_items);
                    Self::on_change(state);
                    true
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let num_items = Self::items(state).len();
                    list_nav_up(Self::list_state(state), num_items);
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

pub fn list_nav_up(list_state: &mut ListState, num_items: usize) -> bool {
    // jump to the bottom if we're at the top
    if list_state.selected() == Some(0) {
        list_state.select(Some(num_items.saturating_sub(1)));
    } else {
        list_state.select_previous();
    }
    true
}

pub fn list_nav_down(list_state: &mut ListState, num_items: usize) -> bool {
    if list_state.selected() == Some(num_items - 1) {
        list_state.select_first();
    } else {
        list_state.select_next();
    }
    true
}

pub fn list_draw<'a>(
    frame: &mut Frame,
    area: Rect,
    items: impl Iterator<Item = Text<'a>>,
    state: &mut ListState,
    title: &str,
    focused: bool,
) {
    let border_color = if focused { Color::Green } else { Color::White };

    let items: Vec<ListItem> = items.map(|item| ListItem::new(item.clone())).collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(title)
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(Style::default().bold().reversed().white());

    StatefulWidget::render(list, area, frame.buffer_mut(), state);
}
