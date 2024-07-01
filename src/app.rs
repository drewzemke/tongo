use crate::connection::Connection;
use crate::screens::connection_screen::ConnectionScreen;
use crate::screens::primary_screen::PrimaryScreen;
use crate::state::{Mode, Screen, State};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use std::time::{Duration, Instant};

pub struct App<'a> {
    state: State<'a>,
}

const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

impl<'a> App<'a> {
    pub fn new(connection: Option<Connection>) -> Self {
        let mut state = State::new();
        match connection {
            Some(connection) => {
                state.set_conn_str(connection.connection_str);
                state.screen = Screen::Primary;
            }
            None => state.screen = Screen::Connection,
        }

        Self { state }
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.state.screen {
            Screen::Primary => {
                PrimaryScreen::default().render(frame.size(), frame.buffer_mut(), &mut self.state);
            }
            Screen::Connection => {
                ConnectionScreen::default().render(
                    frame.size(),
                    frame.buffer_mut(),
                    &mut self.state,
                );
            }
        }

        // show the cursor if we're editing something
        if self.state.mode == Mode::EditingFilter {
            let cursor_pos = self.state.filter_editor.cursor_pos;
            frame.set_cursor(cursor_pos.0, cursor_pos.1);
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        // initial draw call
        terminal.draw(|frame| self.draw(frame))?;

        let debounce: Option<Instant> = None;

        loop {
            // check for respones
            if let Ok(content) = self.state.response_recv.try_recv() {
                self.state.update_content(content);
            }

            let timeout =
                debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));
            let mut update = if crossterm::event::poll(timeout)? {
                let event = crossterm::event::read()?;
                self.handle_event(&event)
            } else {
                false
            };

            // exit if the app is in an exiting state
            if self.state.mode == Mode::Exiting {
                return Ok(());
            }

            if self.state.new_data {
                update = true;
                self.state.new_data = false;
            }

            if update {
                terminal.draw(|frame| {
                    self.draw(frame);
                })?;
            }
        }
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        let mut handle_events_in_screen = || match self.state.screen {
            Screen::Primary => PrimaryScreen::handle_event(event, &mut self.state),
            Screen::Connection => ConnectionScreen::handle_event(event, &mut self.state),
        };

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.state.mode = Mode::Exiting;
                    true
                }
                _ => handle_events_in_screen(),
            },
            _ => handle_events_in_screen(),
        }
    }
}
