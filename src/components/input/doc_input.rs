use super::{InnerInput, InputFormatter};
use crate::{
    components::{
        primary_screen::PrimScrFocus,
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

#[derive(Debug, Default, Clone, Copy)]
pub enum DocInputKind {
    #[default]
    Filter,
    Projection,
    Sort,
}

#[derive(Debug, Default, Clone)]
pub struct DocumentInput {
    kind: DocInputKind,
    input: InnerInput<DocInputFormatter>,
    focus: Rc<Cell<TabFocus>>,
    config: Config,
}

const DEFAULT_DOC: &str = "{}";

impl CloneWithFocus for DocumentInput {
    fn clone_with_focus(&self, focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            focus,
            ..self.clone()
        }
    }
}

impl DocumentInput {
    pub fn new(
        kind: DocInputKind,
        focus: Rc<Cell<TabFocus>>,
        cursor_pos: Rc<Cell<(u16, u16)>>,
        config: Config,
    ) -> Self {
        let mut input = InnerInput::new(
            "Document",
            cursor_pos,
            config.clone(),
            DocInputFormatter::new(config.clone()),
        );
        input.set_value(DEFAULT_DOC);
        Self {
            kind,
            input,
            focus,
            config,
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

    pub const fn name(&self) -> &'static str {
        match self.kind {
            DocInputKind::Filter => "filter",
            DocInputKind::Projection => "projection",
            DocInputKind::Sort => "sort",
        }
    }

    fn get_doc(&self) -> Option<Document> {
        let doc_str = self.input.value();
        json5::from_str::<serde_json::Value>(doc_str)
            .ok()
            .and_then(|value| mongodb::bson::to_document(&value).ok())
    }

    const fn doc_updated_event(&self, doc: Document) -> Event {
        match self.kind {
            DocInputKind::Filter => Event::DocFilterUpdated(doc),
            DocInputKind::Projection => Event::DocProjectionUpdated(doc),
            DocInputKind::Sort => Event::DocSortUpdated(doc),
        }
    }
}

impl Component for DocumentInput {
    fn is_focused(&self) -> bool {
        matches!(self.focus.get(), TabFocus::PrimScr(PrimScrFocus::QueryIn(f)) if f == self.kind)
    }

    fn focus(&self) {
        self.focus
            .set(TabFocus::PrimScr(PrimScrFocus::QueryIn(self.kind.into())));
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
                CommandGroup::new(vec![Command::Confirm], "edit input")
                    .in_cat(CommandCategory::DocInputActions),
                CommandGroup::new(vec![Command::Reset], "reset input")
                    .in_cat(CommandCategory::DocInputActions),
            ]
        }
    }

    fn handle_command(&mut self, command: &Command, queue: &mut SignalQueue) {
        if self.input.is_editing() {
            match command {
                Command::Confirm => {
                    if let Some(doc) = self.get_doc() {
                        self.input.stop_editing();
                        queue.push(Event::DocumentPageChanged(0));
                        queue.push(self.doc_updated_event(doc));
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
                    self.input.set_value(DEFAULT_DOC);
                    queue.push(self.doc_updated_event(Document::default()));
                }
                _ => {}
            }
        }
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event, queue: &mut SignalQueue) {
        self.input.handle_raw_event(event, queue);
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let focused = matches!(
            self.focus.get(),
            TabFocus::PrimScr(PrimScrFocus::QueryIn(..))
        );

        self.input.render_without_block(frame, area, focused);

        // render an indicator symbol to show if the document is valid.
        // first determine what symbol and color we'll use for the indicator
        let valid_doc = self.get_doc().is_some();
        let (symbol, color) = if valid_doc {
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

impl PersistedComponent for DocumentInput {
    type StorageType = String;

    fn persist(&self) -> Self::StorageType {
        self.input.value().to_string()
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.input.set_value(&storage);
    }
}

#[derive(Debug, Default, Clone)]
struct DocInputFormatter {
    labeler: JsonLabeler,
    config: Config,
    text: Text<'static>,
}

impl DocInputFormatter {
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

impl InputFormatter for DocInputFormatter {
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
