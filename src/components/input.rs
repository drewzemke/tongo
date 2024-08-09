#![allow(clippy::cast_possible_truncation)]

use crate::{
    components::{Component, ComponentCommand, InputType},
    event::Event,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Padding, Paragraph},
};
use tui_input::{backend::crossterm::EventHandler, Input};

#[allow(clippy::module_name_repetitions)]
pub mod connection_name_input;

#[allow(clippy::module_name_repetitions)]
pub trait InputComponent {
    fn title() -> &'static str;

    fn is_focused(&self) -> bool;

    fn is_editing(&self) -> bool;

    fn input(&mut self) -> &mut Input;

    fn cursor_pos(&mut self) -> &mut (u16, u16);

    fn commands(&self) -> Vec<crate::command::CommandGroup> {
        vec![]
    }

    fn handle_command(&mut self, _command: &ComponentCommand) -> Vec<Event> {
        vec![]
    }
}

impl<T: InputComponent> Component<InputType> for T {
    fn commands(&self) -> Vec<crate::command::CommandGroup> {
        InputComponent::commands(self)
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        let mut out = vec![];
        if self.is_editing() {
            if let ComponentCommand::RawEvent(event) = command {
                self.input().handle_event(event);
                out.push(Event::InputKeyPressed);
            }
        }
        out.append(&mut InputComponent::handle_command(self, command));
        out
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        matches!(event, Event::InputKeyPressed)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let border_color = if self.is_focused() {
            if self.is_editing() {
                Color::Yellow
            } else {
                Color::Green
            }
        } else {
            Color::White
        };

        // figure the right amount to scroll the input by
        let input_scroll = self.input().visual_scroll(area.width as usize - 5);

        // create the text
        let input_str = self.input().value().to_string();
        let text = Text::from(input_str);

        let input_widget = Paragraph::new(text).scroll((0, input_scroll as u16)).block(
            Block::default()
                .title(Self::title())
                .border_style(Style::default().fg(border_color))
                .padding(Padding::horizontal(1))
                .borders(Borders::ALL),
        );
        frame.render_widget(Clear, area);
        frame.render_widget(input_widget, area);

        // update cursor position
        *self.cursor_pos() = (
            area.x + (self.input().visual_cursor().max(input_scroll) - input_scroll) as u16 + 2,
            area.y + 1,
        );
    }
}
