#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]

use self::{
    coll_list::CollList,
    db_list::DbList,
    main_view::MainView,
    state::{Mode, State},
};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use mongodb::Client;
use ratatui::prelude::*;
use std::time::{Duration, Instant};

mod coll_list;
mod db_list;
mod main_view;
mod state;

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
        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Min(20)])
            .split(frame.size());
        let sidebar = top_layout[0];
        let main_view = top_layout[1];

        let sidebar_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(sidebar);
        let sidebar_top = sidebar_layout[0];
        let sidebar_btm = sidebar_layout[1];

        DbList::default().render(sidebar_top, frame.buffer_mut(), &mut self.state);
        CollList::default().render(sidebar_btm, frame.buffer_mut(), &mut self.state);
        MainView::default().render(main_view, frame.buffer_mut(), &mut self.state);
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        // initial draw call
        terminal.draw(|frame| self.draw(frame))?;

        // initial mongo calls
        self.state.exec_get_dbs();

        let debounce: Option<Instant> = None;

        loop {
            // check for respones
            if let Ok(content) = self.state.query_recv.try_recv() {
                self.state.update_content(content);
            }

            let timeout =
                debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));
            let mut update = if crossterm::event::poll(timeout)? {
                let event = crossterm::event::read()?;
                match event {
                    Event::Key(key) => match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(())
                        }
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('J') => {
                            self.state.mode = match self.state.mode {
                                Mode::ChoosingDatabase => Mode::ChoosingCollection,
                                m => m,
                            };
                            true
                        }
                        KeyCode::Char('K') => {
                            self.state.mode = match self.state.mode {
                                Mode::ChoosingCollection => Mode::ChoosingDatabase,
                                m => m,
                            };
                            true
                        }
                        KeyCode::Char('H') => {
                            self.state.mode = match self.state.mode {
                                Mode::MainView => Mode::ChoosingCollection,
                                m => m,
                            };
                            true
                        }
                        KeyCode::Char('L') => {
                            self.state.mode = Mode::MainView;
                            true
                        }
                        KeyCode::Esc => {
                            self.state.mode = match self.state.mode {
                                Mode::ChoosingDatabase | Mode::ChoosingCollection => {
                                    Mode::ChoosingDatabase
                                }
                                Mode::MainView => Mode::ChoosingCollection,
                            };
                            true
                        }
                        _ => match self.state.mode {
                            Mode::ChoosingDatabase => DbList::handle_event(&event, &mut self.state),
                            Mode::ChoosingCollection => {
                                CollList::handle_event(&event, &mut self.state)
                            }
                            Mode::MainView => MainView::handle_event(&event, &mut self.state),
                        },
                    },
                    Event::Resize(_, _) => true,
                    _ => false,
                }
            } else {
                false
            };

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
}
