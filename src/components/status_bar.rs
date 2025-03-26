use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::time::{Duration, Instant};

use crate::{
    components::Component,
    config::{color_map::ColorKey, Config},
    system::{
        command::{CommandCategory, CommandManager},
        event::Event,
        Signal,
    },
};

const DEBUG_RENDER_COUNT: bool = false;

const ERROR_MESSAGE_DURATION: Duration = Duration::from_secs(7);
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

    config: Config,

    // NOTE: used for debugging
    renders: usize,
}
impl StatusBar {
    pub fn new(command_manager: CommandManager, config: Config) -> Self {
        Self {
            command_manager,
            config,
            ..Default::default()
        }
    }

    fn message_widget(&self) -> Option<Paragraph<'_>> {
        let message = self.message.as_ref()?;
        let (prefix, color) = match message.kind {
            MessageKind::Error => (
                "● Error: ",
                self.config.color_map.get(&ColorKey::IndicatorError),
            ),
            MessageKind::Info => ("● ", self.config.color_map.get(&ColorKey::IndicatorInfo)),
            MessageKind::Success => (
                "● Success: ",
                self.config.color_map.get(&ColorKey::IndicatorSuccess),
            ),
        };

        let content = Line::from(vec![
            prefix.to_string().fg(color),
            message.content.clone().into(),
        ]);

        let paragraph = Paragraph::new(content).wrap(Wrap { trim: true });
        Some(paragraph)
    }

    #[expect(clippy::cast_possible_truncation)]
    pub fn height(&self, width: u16) -> u16 {
        self.message_widget()
            .map_or(1, |p| p.line_count(width.saturating_sub(2)))
            .min(5) as u16
    }
}

impl Component for StatusBar {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // render the bg color
        let bg = Block::default()
            .borders(Borders::NONE)
            .bg(self.config.color_map.get(&ColorKey::PanelInactiveBg));
        frame.render_widget(bg, area);

        // render the message if there is one, otherwise show commands and app name
        if let Some(message_content) = self.message_widget() {
            frame.render_widget(message_content, area.inner(Margin::new(1, 0)));
        } else {
            let layout = Layout::horizontal([Constraint::Fill(1), Constraint::Length(16)])
                .horizontal_margin(1)
                .split(area);

            let primary = self.config.color_map.get(&ColorKey::FgPrimary);
            let secondary = self.config.color_map.get(&ColorKey::FgSecondary);

            let commands = Line::from(
                self.command_manager
                    .groups()
                    .into_iter()
                    .filter(|group| group.category == CommandCategory::StatusBarOnly)
                    .flat_map(|group| {
                        let key_hint: String = group
                            .commands
                            .iter()
                            .map(|c| self.config.key_map.key_for_command(*c))
                            .map(|k| k.map(|k| format!("{k}")))
                            .map(Option::unwrap_or_default)
                            .collect();

                        vec![
                            key_hint.bold().fg(primary),
                            ": ".fg(secondary),
                            group.name.fg(secondary),
                            "  ".into(),
                        ]
                    })
                    .collect::<Vec<Span>>(),
            );

            let right_content = if DEBUG_RENDER_COUNT {
                self.renders += 1;
                Line::from(format!("{}", &self.renders)).right_aligned()
            } else {
                Line::from(format!("tongo v{}", env!("CARGO_PKG_VERSION")))
                    .fg(self.config.color_map.get(&ColorKey::AppName))
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
                self.message = Some(Message::error(&error.to_string()));
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
