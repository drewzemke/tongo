#![allow(clippy::cast_possible_truncation)]

use crate::{
    app::AppFocus,
    components::{Component, ComponentCommand, InputType},
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Padding, Paragraph},
};
use std::{cell::RefCell, rc::Rc};
use tui_input::{backend::crossterm::EventHandler, Input as TuiInput};

pub mod filter;

#[derive(Debug, Default)]
pub struct Input<T: Default + std::fmt::Debug> {
    #[allow(clippy::struct_field_names)]
    pub inner_input: TuiInput,
    cursor_pos: Rc<RefCell<(u16, u16)>>,
    editing: bool,

    app_focus: Rc<RefCell<AppFocus>>,
    focused_when: AppFocus,

    title: &'static str,
    commands: Vec<CommandGroup>,
    confirm_events: Vec<Event>,
    back_events: Vec<Event>,

    formatter: T,
}

impl<T> Input<T>
where
    T: Default + std::fmt::Debug,
{
    // TODO: builder pattern macro?? this is a bit big and difficult to read, especially at the call site
    pub fn new(
        title: &'static str,
        cursor_pos: Rc<RefCell<(u16, u16)>>,
        commands: Vec<CommandGroup>,
        app_focus: Rc<RefCell<AppFocus>>,
        focused_when: AppFocus,
        confirm_events: Vec<Event>,
        back_events: Vec<Event>,
        formatter: T,
    ) -> Self {
        Self {
            cursor_pos,
            app_focus,
            focused_when,
            title,
            commands,
            confirm_events,
            back_events,
            formatter,
            ..Default::default()
        }
    }

    pub fn start_editing(&mut self) {
        self.editing = true;
    }

    pub fn stop_editing(&mut self) {
        self.editing = false;
    }

    const fn is_editing(&self) -> bool {
        self.editing
    }
}

impl<T> Component<InputType> for Input<T>
where
    T: Default + InputFormatter + std::fmt::Debug,
{
    fn commands(&self) -> Vec<CommandGroup> {
        self.commands.clone()
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        if self.is_editing() {
            match command {
                ComponentCommand::RawEvent(event) => {
                    self.inner_input.handle_event(event);
                    self.formatter.on_change(self.inner_input.value());
                    vec![Event::InputKeyPressed]
                }
                ComponentCommand::Command(command) => match command {
                    Command::Confirm => self.confirm_events.clone(),
                    Command::Back => self.back_events.clone(),
                    _ => vec![],
                },
            }
        } else {
            vec![]
        }
    }

    fn handle_event(&mut self, _event: &Event) -> Vec<Event> {
        vec![]
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
        let input_scroll = self.inner_input.visual_scroll(area.width as usize - 5);

        // create the text
        let text = self.formatter.get_formatted();

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
        if self.is_editing() {
            let cursor_pos = (
                area.x
                    + (self.inner_input.visual_cursor().max(input_scroll) - input_scroll) as u16
                    + 2,
                area.y + 1,
            );
            *self.cursor_pos.borrow_mut() = cursor_pos;
        }
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = self.focused_when.clone();
    }

    fn is_focused(&self) -> bool {
        *self.app_focus.borrow() == self.focused_when
    }
}

trait InputFormatter {
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
