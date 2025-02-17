use crate::system::event::Event;
use crossterm::event::Event as CrosstermEvent;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Padding, Paragraph},
};
use std::{cell::Cell, rc::Rc};
use tui_input::{backend::crossterm::EventHandler, Input as TuiInput};

pub mod conn_name_input;
pub mod conn_str_input;
pub mod filter;

#[derive(Debug, Default)]
pub struct InnerInput<T: Default + std::fmt::Debug> {
    state: TuiInput,
    formatter: T,

    title: &'static str,
    cursor_pos: Rc<Cell<(u16, u16)>>,
    editing: bool,
}

impl<T> InnerInput<T>
where
    T: Default + InputFormatter + std::fmt::Debug,
{
    pub fn new(title: &'static str, cursor_pos: Rc<Cell<(u16, u16)>>, formatter: T) -> Self {
        Self {
            formatter,
            title,
            cursor_pos,
            ..Default::default()
        }
    }

    pub const fn start_editing(&mut self) {
        self.editing = true;
    }

    pub const fn stop_editing(&mut self) {
        self.editing = false;
    }

    const fn is_editing(&self) -> bool {
        self.editing
    }

    pub fn value(&self) -> &str {
        self.state.value()
    }

    pub fn set_value(&mut self, value: &str) {
        self.state = self.state.clone().with_value(value.to_string());
        self.formatter.on_change(value);
    }

    fn handle_raw_event(&mut self, event: &CrosstermEvent) -> Vec<Event> {
        if self.is_editing() {
            self.state.handle_event(event);
            self.formatter.on_change(self.state.value());
            vec![Event::InputKeyPressed]
        } else {
            vec![]
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_color = if focused {
            if self.is_editing() {
                Color::Yellow
            } else {
                Color::Green
            }
        } else {
            Color::White
        };

        // figure the right amount to scroll the input by
        let input_scroll = self.state.visual_scroll(area.width as usize - 5);

        // create the text
        let text = self.formatter.get_formatted();

        #[expect(clippy::cast_possible_truncation)]
        let input_widget = Paragraph::new(text).scroll((0, input_scroll as u16)).block(
            Block::default()
                .title(self.title)
                .border_style(Style::default().fg(border_color))
                .padding(Padding::horizontal(1))
                .borders(Borders::ALL),
        );
        frame.render_widget(Clear, area);
        frame.render_widget(input_widget, area);

        // update cursor position
        #[expect(clippy::cast_possible_truncation)]
        if self.is_editing() {
            let cursor_pos = (
                area.x + (self.state.visual_cursor().max(input_scroll) - input_scroll) as u16 + 2,
                area.y + 1,
            );
            self.cursor_pos.set(cursor_pos);
        }
    }
}

pub trait InputFormatter {
    fn on_change(&mut self, _text: &str) {}

    fn get_formatted(&self) -> Text;
}

#[derive(Default, Debug)]
pub struct DefaultFormatter {
    text: Text<'static>,
}

impl InputFormatter for DefaultFormatter {
    fn get_formatted(&self) -> Text {
        self.text.clone()
    }

    fn on_change(&mut self, text: &str) {
        self.text = text.to_string().into();
    }
}
