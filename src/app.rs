use crate::screens::primary_screen::PrimaryScreen;
use crate::state::{Mode, State};
use crossterm::event::Event;
use mongodb::Client;
use ratatui::prelude::*;
use std::time::{Duration, Instant};

pub struct App<'a> {
    state: State<'a>,
}

const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

impl<'a> App<'a> {
    pub fn new(client: Client) -> Self {
        Self {
            state: State::new(client),
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        PrimaryScreen::default().render(frame.size(), frame.buffer_mut(), &mut self.state);

        // show the cursor if we're editing something
        // FIXME: store cursor pos in state?? ugh
        // if self.state.mode == Mode::EditingFilter {
        //     let cursor_position = FilterInput::cursor_position(&self.state, main_view_top);
        //     frame.set_cursor(cursor_position.0, cursor_position.1);
        // }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        // initial draw call
        terminal.draw(|frame| self.draw(frame))?;

        // initial mongo calls
        self.state.exec_get_dbs();

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
        PrimaryScreen::handle_event(event, &mut self.state)
    }
}
