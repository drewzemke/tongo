#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]

use self::{
    coll_list::CollList,
    db_list::DbList,
    main_view::MainView,
    state::{State, WidgetFocus},
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
            if let Ok(content) = self.state.response_recv.try_recv() {
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
                            self.state.focus = match self.state.focus {
                                WidgetFocus::DatabaseList => WidgetFocus::CollectionList,
                                m => m,
                            };
                            true
                        }
                        KeyCode::Char('K') => {
                            self.state.focus = match self.state.focus {
                                WidgetFocus::CollectionList => WidgetFocus::DatabaseList,
                                m => m,
                            };
                            true
                        }
                        KeyCode::Char('H') => {
                            self.state.focus = match self.state.focus {
                                WidgetFocus::MainView => WidgetFocus::CollectionList,
                                m => m,
                            };
                            true
                        }
                        KeyCode::Char('L') => {
                            self.state.focus = WidgetFocus::MainView;
                            true
                        }
                        KeyCode::Esc => {
                            self.state.focus = match self.state.focus {
                                WidgetFocus::DatabaseList | WidgetFocus::CollectionList => {
                                    WidgetFocus::DatabaseList
                                }
                                WidgetFocus::MainView => WidgetFocus::CollectionList,
                            };
                            true
                        }
                        _ => match self.state.focus {
                            WidgetFocus::DatabaseList => {
                                DbList::handle_event(&event, &mut self.state)
                            }
                            WidgetFocus::CollectionList => {
                                CollList::handle_event(&event, &mut self.state)
                            }
                            WidgetFocus::MainView => {
                                MainView::handle_event(&event, &mut self.state)
                            }
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
