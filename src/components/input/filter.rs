use super::{InnerInput, InputFormatter};
use crate::{
    components::{
        primary_screen::PrimScrFocus,
        query_input::QueryInFocus,
        tab::{CloneWithFocus, TabFocus},
        Component,
    },
    config::{color_map::ColorKey, Config},
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        message::{AppAction, Message},
        signal::SignalQueue,
    },
    utils::json_labeler::{JsonLabel, JsonLabeler},
};
use mongodb::bson::Document;
use ratatui::{
    prelude::{Frame, Rect},
    style::Style,
    text::{Line, Span, Text},
};
use std::{cell::Cell, rc::Rc};

#[derive(Debug, Default, Clone)]
pub struct FilterInput {
    focus: Rc<Cell<TabFocus>>,
    config: Config,
    input: InnerInput<FilterInputFormatter>,
}

const DEFAULT_FILTER: &str = "{}";

impl CloneWithFocus for FilterInput {
    fn clone_with_focus(&self, focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            focus,
            ..self.clone()
        }
    }
}

impl FilterInput {
    pub fn new(
        focus: Rc<Cell<TabFocus>>,
        cursor_pos: Rc<Cell<(u16, u16)>>,
        config: Config,
    ) -> Self {
        let mut input = InnerInput::new(
            "Filter",
            cursor_pos,
            config.clone(),
            FilterInputFormatter::new(config.clone()),
        );
        input.set_value(DEFAULT_FILTER);
        Self {
            focus,
            config,
            input,
        }
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
        self.focus.get() == TabFocus::PrimScr(PrimScrFocus::QueryIn(QueryInFocus::Filter))
    }

    fn focus(&self) {
        self.focus.set(TabFocus::PrimScr(PrimScrFocus::QueryIn(
            QueryInFocus::Filter,
        )));
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

    fn handle_command(&mut self, command: &Command, queue: &mut SignalQueue) {
        if self.input.is_editing() {
            match command {
                Command::Confirm => {
                    if let Some(doc) = self.get_filter_doc() {
                        self.input.stop_editing();
                        queue.push(Event::DocumentPageChanged(0));
                        queue.push(Event::DocFilterUpdated(doc));
                        queue.push(Message::to_app(AppAction::ExitRawMode));
                    } else {
                        queue.push(Event::ErrorOccurred("Invalid filter.".into()));
                    }
                }
                Command::Back => {
                    self.stop_editing();
                    queue.push(Message::to_app(AppAction::ExitRawMode));
                }
                _ => {}
            }
        } else {
            match command {
                Command::Confirm => {
                    self.start_editing();
                    queue.push(Message::to_app(AppAction::EnterRawMode));
                }
                Command::Reset => {
                    self.input.set_value(DEFAULT_FILTER);
                    queue.push(Event::DocFilterUpdated(Document::default()));
                }
                _ => {}
            }
        }
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event, queue: &mut SignalQueue) {
        self.input.handle_raw_event(event, queue);
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.input.render_without_block(frame, area);

        // render an indicator symbol to show if the filter is valid.
        // first determine what symbol and color we'll use for the indicator
        let valid_filter = self.get_filter_doc().is_some();
        let (symbol, color) = if valid_filter {
            ("●", self.config.color_map.get(&ColorKey::IndicatorSuccess))
        } else if self.is_editing() {
            ("◯", self.config.color_map.get(&ColorKey::IndicatorError))
        } else {
            ("●", self.config.color_map.get(&ColorKey::IndicatorError))
        };

        frame
            .buffer_mut()
            .cell_mut((area.right() - 1, area.y))
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
    config: Config,
    text: Text<'static>,
}

impl FilterInputFormatter {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    fn style_for_label(&self, label: &JsonLabel) -> Style {
        let color_key = match label {
            JsonLabel::Punctuation => ColorKey::Punctuation,
            JsonLabel::Number => ColorKey::Number,
            JsonLabel::Key => ColorKey::Key,
            JsonLabel::Value => ColorKey::String,
            JsonLabel::Constant => ColorKey::Boolean,
            JsonLabel::DollarSignKey => ColorKey::MongoOperator,
            JsonLabel::Error => ColorKey::FgPrimary,
            JsonLabel::Whitespace => return Style::default(),
        };

        Style::default().fg(self.config.color_map.get(&color_key))
    }
}

impl InputFormatter for FilterInputFormatter {
    fn on_change(&mut self, text: &str) {
        let labels = self.labeler.label_line(text);
        let text = labels.map_or_else(
            |_| Text::from(text.to_string()),
            |labels| {
                let spans: Vec<_> = labels
                    .into_iter()
                    .map(|(s, label)| Span::styled(s, self.style_for_label(&label)))
                    .collect();
                let line = Line::from(spans);
                Text::from(line)
            },
        );

        self.text = text;
    }

    fn get_formatted(&self) -> Text<'_> {
        self.text.clone()
    }
}
