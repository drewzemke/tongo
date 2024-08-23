use super::{Input, InputFormatter};
use crate::{
    app::AppFocus,
    components::{Component, ComponentCommand, InputType},
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
use std::{cell::RefCell, rc::Rc};
use tui_input::Input as TuiInput;

#[derive(Debug, Default)]
pub struct FilterInput {
    input: Input<FilterInputFormatter>,
}

const DEFAULT_FILTER: &str = "{}";

impl FilterInput {
    pub fn new(
        title: &'static str,
        cursor_pos: Rc<RefCell<(u16, u16)>>,
        app_focus: Rc<RefCell<AppFocus>>,
        focused_when: AppFocus,
    ) -> Self {
        let mut input = Input::new(
            title,
            cursor_pos,
            vec![],
            app_focus,
            focused_when,
            vec![],
            vec![],
            FilterInputFormatter::default(),
        );

        input.inner_input = input.inner_input.with_value(DEFAULT_FILTER.to_string());
        input.formatter.on_change(DEFAULT_FILTER);

        Self { input }
    }

    pub const fn is_editing(&self) -> bool {
        self.input.is_editing()
    }
}

impl Component<InputType> for FilterInput {
    fn focus(&self) {
        self.input.focus();
    }

    fn is_focused(&self) -> bool {
        self.input.is_focused()
    }

    fn commands(&self) -> Vec<CommandGroup> {
        if self.input.is_editing() {
            vec![
                CommandGroup::new(vec![Command::Confirm], "enter", "execute query"),
                CommandGroup::new(vec![Command::Back], "esc", "cancel"),
            ]
        } else {
            vec![
                CommandGroup::new(vec![Command::Confirm], "enter", "edit filter"),
                CommandGroup::new(vec![Command::Reset], "R", "reset filter"),
            ]
        }
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        if self.input.is_editing() {
            match command {
                ComponentCommand::RawEvent(..) => {
                    self.input.handle_command(command);
                    vec![Event::InputKeyPressed]
                }
                ComponentCommand::Command(command) => match command {
                    Command::Confirm => {
                        let filter_str = self.input.inner_input.value();
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
                        self.input.stop_editing();
                        vec![Event::RawModeExited]
                    }
                    _ => vec![],
                },
            }
        } else if let ComponentCommand::Command(command) = command {
            match command {
                Command::Confirm => {
                    self.input.start_editing();
                    vec![Event::RawModeEntered]
                }
                Command::Reset => {
                    self.input.inner_input = TuiInput::new("{}".to_string());
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
        self.input.render(frame, area);
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
