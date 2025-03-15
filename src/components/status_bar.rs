use crate::{
    components::Component,
    key_map::KeyMap,
    system::{
        command::{CommandCategory, CommandManager},
        event::Event,
        Signal,
    },
};
use ratatui::prelude::*;
use std::{
    rc::Rc,
    time::{Duration, Instant},
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
    command_manager: CommandManager,
    message: Option<Message>,

    key_map: Rc<KeyMap>,

    // DEBUG:
    renders: usize,
}
impl StatusBar {
    pub fn new(command_manager: CommandManager, key_map: Rc<KeyMap>) -> Self {
        Self {
            command_manager,
            key_map,
            ..Default::default()
        }
    }
}

impl Component for StatusBar {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // render the message if there is one, otherwise show commands and app name
        if let Some(message) = &self.message {
            let (prefix, style) = match message.kind {
                MessageKind::Error => ("● Error: ", Style::default().red()),
                MessageKind::Info => ("● ", Style::default().blue()),
                MessageKind::Success => ("● Success: ", Style::default().green()),
            };
            let content = Line::from(vec![
                Span::styled(prefix, style),
                Span::from(message.content.clone()),
            ]);

            frame.render_widget(content, area.inner(Margin::new(1, 0)));
        } else {
            let layout = Layout::horizontal([Constraint::Fill(1), Constraint::Length(16)])
                .horizontal_margin(1)
                .split(area);

            let commands = Line::from(
                self.command_manager
                    .groups()
                    .into_iter()
                    .filter(|group| group.category == CommandCategory::StatusBarOnly)
                    .flat_map(|group| self.key_map.cmd_group_to_span(&group))
                    .collect::<Vec<Span>>(),
            );

            let right_content = if DEBUG_RENDER_COUNT {
                self.renders += 1;
                Line::from(format!("{}", &self.renders)).right_aligned()
            } else {
                Line::from(format!("tongo v{}", env!("CARGO_PKG_VERSION")).magenta())
                    .right_aligned()
            };

            frame.render_widget(commands, layout[0]);
            frame.render_widget(right_content, layout[1]);
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        // handle the event
        match event {
            Event::ErrorOccurred(error) => {
                self.message = Some(Message::error(error));
            }
            Event::DocUpdateComplete => {
                self.message = Some(Message::success("Document updated."));
            }
            Event::DocInsertComplete => {
                self.message = Some(Message::success("Document created."));
            }
            Event::DocDeleteComplete => {
                self.message = Some(Message::success("Document deleted."));
            }
            Event::CollectionCreationConfirmed => {
                self.message = Some(Message::success("Collection created."));
            }
            Event::CollectionDropConfirmed(_) => {
                self.message = Some(Message::success("Collection dropped."));
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
            return vec![Event::StatusMessageCleared.into()];
        }

        vec![]
    }
}
