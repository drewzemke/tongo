use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use super::ComponentCommand;
use crate::{
    components::Component,
    key_map::KeyMap,
    system::{command::CommandGroup, event::Event},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph, Wrap},
};

const DEBUG_RENDER_COUNT: bool = false;

// should this be configurable?
const ERROR_MESSAGE_DURATION: Duration = Duration::from_secs(5);
const INFO_MESSAGE_DURATION: Duration = Duration::from_secs(4);
const SUCCESS_MESSAGE_DURATION: Duration = Duration::from_secs(4);

#[derive(Debug)]
enum MessageKind {
    Error,
    Info,
    Success,
}

#[derive(Debug)]
struct Message {
    kind: MessageKind,
    content: String,
    start: Instant,
    duration: Duration,
}

impl Message {
    fn error(content: &str) -> Self {
        Self {
            kind: MessageKind::Error,
            content: content.to_string(),
            start: Instant::now(),
            duration: ERROR_MESSAGE_DURATION,
        }
    }
    fn info(content: &str) -> Self {
        Self {
            kind: MessageKind::Info,
            content: content.to_string(),
            start: Instant::now(),
            duration: INFO_MESSAGE_DURATION,
        }
    }
    fn success(content: &str) -> Self {
        Self {
            kind: MessageKind::Success,
            content: content.to_string(),
            start: Instant::now(),
            duration: SUCCESS_MESSAGE_DURATION,
        }
    }
}

#[derive(Debug, Default)]
pub struct StatusBar {
    pub commands: Vec<CommandGroup>,
    message: Option<Message>,

    key_map: Rc<RefCell<KeyMap>>,

    // DEBUG:
    renders: usize,
}
impl StatusBar {
    pub fn new(key_map: Rc<RefCell<KeyMap>>) -> Self {
        Self {
            key_map,
            ..Default::default()
        }
    }
}

impl Component for StatusBar {
    fn focus(&self) {}

    fn is_focused(&self) -> bool {
        false
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let content = self.message.as_ref().map_or_else(
            || {
                Line::from(
                    self.commands
                        .iter()
                        .flat_map(|group| self.key_map.borrow().cmd_group_to_span(group))
                        .collect::<Vec<Span>>(),
                )
            },
            |message| {
                let (prefix, style) = match message.kind {
                    MessageKind::Error => ("● Error: ", Style::default().red()),
                    MessageKind::Info => ("● ", Style::default().blue()),
                    MessageKind::Success => ("● Success: ", Style::default().green()),
                };
                Line::from(vec![
                    Span::styled(prefix, style),
                    Span::from(message.content.clone()),
                ])
            },
        );

        let content = Paragraph::new(content)
            .wrap(Wrap::default())
            .block(Block::default().padding(Padding::horizontal(1)));

        if DEBUG_RENDER_COUNT {
            self.renders += 1;
            let render_count_content = Paragraph::new(format!("{}", &self.renders));

            let layout =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(4)]).split(area);
            frame.render_widget(content, layout[0]);
            frame.render_widget(render_count_content, layout[1]);
        } else {
            frame.render_widget(content, area);
        }
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![]
    }

    fn handle_command(&mut self, _command: &ComponentCommand) -> Vec<Event> {
        vec![]
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        // handle the event
        match event {
            Event::ErrorOccurred(error) => {
                self.message = Some(Message::error(error));
            }
            Event::UpdateConfirmed => {
                self.message = Some(Message::success("Document updated."));
            }
            Event::InsertConfirmed => {
                self.message = Some(Message::success("Document created."));
            }
            Event::DeleteConfirmed => {
                self.message = Some(Message::success("Document deleted."));
            }
            Event::DataSentToClipboard => {
                self.message = Some(Message::info("Copied to clipboard."));
            }
            _ => (),
        }

        // check to see if it's time to clear the message
        if self.message.as_ref().is_some_and(
            |Message {
                 start, duration, ..
             }| start.elapsed() >= *duration,
        ) {
            self.message = None;
            return vec![Event::StatusMessageCleared];
        }

        vec![]
    }
}
