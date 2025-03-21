use super::{InnerInput, InputFormatter};
use crate::{
    components::{primary_screen::PrimScrFocus, tab::TabFocus, Component},
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        message::{AppAction, Message},
        Signal,
    },
    utils::json_labeler::{JsonLabel, JsonLabeler},
};
use mongodb::bson::Document;
use ratatui::{
    prelude::{Frame, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
};
use std::{cell::Cell, rc::Rc};

#[derive(Debug, Default, Clone)]
pub struct FilterInput {
    focus: Rc<Cell<TabFocus>>,
    input: InnerInput<FilterInputFormatter>,
}

const DEFAULT_FILTER: &str = "{}";

impl FilterInput {
    pub fn new(focus: Rc<Cell<TabFocus>>, cursor_pos: Rc<Cell<(u16, u16)>>) -> Self {
        let mut input = InnerInput::new("Filter", cursor_pos, FilterInputFormatter::default());
        input.set_value(DEFAULT_FILTER);
        Self { focus, input }
    }

    pub const fn is_editing(&self) -> bool {
        self.input.is_editing()
    }

    pub const fn start_editing(&mut self) {
        self.input.start_editing();
    }

    pub const fn stop_editing(&mut self) {
        self.input.stop_editing();
    }

    fn get_filter_doc(&self) -> Option<Document> {
        let filter_str = self.input.value();
        json5::from_str::<serde_json::Value>(filter_str)
            .ok()
            .and_then(|value| mongodb::bson::to_document(&value).ok())
    }
}

impl Component for FilterInput {
    fn is_focused(&self) -> bool {
        self.focus.get() == TabFocus::PrimScr(PrimScrFocus::FilterIn)
    }

    fn focus(&self) {
        self.focus.set(TabFocus::PrimScr(PrimScrFocus::FilterIn));
    }

    fn commands(&self) -> Vec<CommandGroup> {
        if self.input.is_editing() {
            vec![
                CommandGroup::new(vec![Command::Confirm], "run query")
                    .in_cat(CommandCategory::StatusBarOnly),
                CommandGroup::new(vec![Command::Back], "cancel")
                    .in_cat(CommandCategory::StatusBarOnly),
            ]
        } else {
            vec![
                CommandGroup::new(vec![Command::Confirm], "edit filter")
                    .in_cat(CommandCategory::FilterInputActions),
                CommandGroup::new(vec![Command::Reset], "reset filter")
                    .in_cat(CommandCategory::FilterInputActions),
            ]
        }
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        if self.input.is_editing() {
            match command {
                Command::Confirm => {
                    if let Some(doc) = self.get_filter_doc() {
                        self.input.stop_editing();
                        vec![
                            Event::DocumentPageChanged(0).into(),
                            Event::DocFilterUpdated(doc).into(),
                            Message::to_app(AppAction::ExitRawMode).into(),
                        ]
                    } else {
                        vec![Event::ErrorOccurred("Invalid filter.".to_string()).into()]
                    }
                }
                Command::Back => {
                    self.stop_editing();
                    vec![Message::to_app(AppAction::ExitRawMode).into()]
                }
                _ => vec![],
            }
        } else {
            match command {
                Command::Confirm => {
                    self.start_editing();
                    vec![Message::to_app(AppAction::EnterRawMode).into()]
                }
                Command::Reset => {
                    self.input.set_value(DEFAULT_FILTER);
                    vec![Event::DocFilterUpdated(Document::default()).into()]
                }
                _ => vec![],
            }
        }
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event) -> Vec<Signal> {
        self.input.handle_raw_event(event)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.input.render(frame, area, self.is_focused());

        // render an indicator symbol to show if the filter is valid.
        // first determine what symbol and color we'll use for the indicator
        let valid_filter = self.get_filter_doc().is_some();
        let (symbol, color) = if valid_filter {
            ("●", Color::Green)
        } else if self.is_editing() {
            ("◯", Color::Red)
        } else {
            ("●", Color::Red)
        };

        frame
            .buffer_mut()
            .cell_mut((area.right() - 2, area.y + 1))
            .map(|cell| cell.set_symbol(symbol).set_fg(color));
    }
}

impl PersistedComponent for FilterInput {
    type StorageType = String;

    fn persist(&self) -> Self::StorageType {
        self.input.value().to_string()
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.input.set_value(&storage);
    }
}

#[derive(Debug, Default, Clone)]
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
                            JsonLabel::DollarSignKey => Style::default().magenta(),
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
