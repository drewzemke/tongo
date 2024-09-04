use super::{InnerInput, InputFormatter};
use crate::{
    app::AppFocus,
    components::{primary_screen::PrimaryScreenFocus, Component, ComponentCommand},
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
    utils::json_labeler::{JsonLabel, JsonLabeler},
};
use mongodb::bson::Document;
use ratatui::{
    prelude::{Frame, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use tui_input::Input as TuiInput;

#[derive(Debug, Default)]
pub struct FilterInput {
    app_focus: Rc<RefCell<AppFocus>>,
    input: InnerInput<FilterInputFormatter>,
}

const DEFAULT_FILTER: &str = "{}";

impl FilterInput {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>, cursor_pos: Rc<Cell<(u16, u16)>>) -> Self {
        let mut input = InnerInput::new("Filter", cursor_pos, FilterInputFormatter::default());
        input.state = input.state.with_value(DEFAULT_FILTER.to_string());
        input.formatter.on_change(DEFAULT_FILTER);
        Self { app_focus, input }
    }

    pub const fn is_editing(&self) -> bool {
        self.input.is_editing()
    }

    pub fn value(&self) -> &str {
        self.input.state.value()
    }

    pub fn start_editing(&mut self) {
        self.input.start_editing();
    }

    pub fn stop_editing(&mut self) {
        self.input.stop_editing();
    }
}

impl Component for FilterInput {
    fn is_focused(&self) -> bool {
        *self.app_focus.borrow() == AppFocus::PrimaryScreen(PrimaryScreenFocus::FilterInput)
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::PrimaryScreen(PrimaryScreenFocus::FilterInput);
    }

    fn commands(&self) -> Vec<CommandGroup> {
        if self.input.is_editing() {
            vec![
                CommandGroup::new(vec![Command::Confirm], "execute query"),
                CommandGroup::new(vec![Command::Back], "cancel"),
            ]
        } else {
            vec![
                CommandGroup::new(vec![Command::Confirm], "edit filter"),
                CommandGroup::new(vec![Command::Reset], "reset filter"),
            ]
        }
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        if self.input.is_editing() {
            match command {
                ComponentCommand::RawEvent(event) => self.input.handle_raw_event(event),
                ComponentCommand::Command(command) => match command {
                    Command::Confirm => {
                        let filter_str = self.input.state.value();
                        let filter = serde_json::from_str::<serde_json::Value>(filter_str)
                            .ok()
                            .and_then(|value| mongodb::bson::to_document(&value).ok());

                        if let Some(doc) = filter {
                            self.input.stop_editing();
                            vec![Event::DocFilterUpdated(doc), Event::RawModeExited]
                        } else {
                            vec![]
                        }
                    }
                    Command::Back => {
                        self.stop_editing();
                        vec![Event::RawModeExited]
                    }
                    _ => vec![],
                },
            }
        } else if let ComponentCommand::Command(command) = command {
            match command {
                Command::Confirm => {
                    self.start_editing();
                    vec![Event::RawModeEntered]
                }
                Command::Reset => {
                    self.input.state = TuiInput::new(DEFAULT_FILTER.to_string());
                    self.input.formatter.on_change(DEFAULT_FILTER);
                    vec![Event::DocFilterUpdated(Document::default())]
                }
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    fn handle_event(&mut self, _event: &Event) -> Vec<Event> {
        vec![]
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.input.render(frame, area, self.is_focused());
    }
}

#[derive(Debug, Default)]
struct FilterInputFormatter {
    labeler: JsonLabeler,
    text: Text<'static>,
}

impl InputFormatter for FilterInputFormatter {
    fn on_change(&mut self, text: &str) {
        let labels = self.labeler.label_line(text);
        let text = labels.map_or_else(
            |_| Text::from(text.to_string()),
            |labels| {
                let spans: Vec<_> = labels
                    .into_iter()
                    .map(|(s, label)| {
                        let style = match label {
                            JsonLabel::Punctuation => Style::default().gray(),
                            JsonLabel::Number => Style::default().yellow(),
                            JsonLabel::Key => Style::default().white(),
                            JsonLabel::Value => Style::default().green(),
                            JsonLabel::Constant => Style::default().cyan(),
                            JsonLabel::Whitespace => Style::default(),
                            JsonLabel::Error => Style::default().on_red(),
                        };

                        Span::styled(s, style)
                    })
                    .collect();
                let line = Line::from(spans);
                Text::from(line)
            },
        );

        self.text = text;
    }

    fn get_formatted(&self) -> Text {
        self.text.clone()
    }
}
