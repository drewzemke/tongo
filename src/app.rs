#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]

use self::{
    coll_list::CollList,
    db_list::DbList,
    filter_input::FilterInput,
    main_view::MainView,
    state::{Mode, State, WidgetFocus},
    status_bar::StatusBar,
};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use mongodb::Client;
use ratatui::prelude::*;
use std::time::{Duration, Instant};

mod coll_list;
mod db_list;
mod filter_input;
mod main_view;
mod state;
mod status_bar;

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
        // TODO: change status bar visibility based on whether there's an error?
        let frame_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Length(1)])
            .split(frame.size());
        let content = frame_layout[0];
        let btm_line = frame_layout[1];

        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Min(20)])
            .split(content);
        let sidebar = content_layout[0];
        let main_view = content_layout[1];

        let sidebar_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(sidebar);
        let sidebar_top = sidebar_layout[0];
        let sidebar_btm = sidebar_layout[1];

        let main_view_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Percentage(100)])
            .split(main_view);
        let main_view_top = main_view_layout[0];
        let main_view_btm = main_view_layout[1];

        DbList::default().render(sidebar_top, frame.buffer_mut(), &mut self.state);
        CollList::default().render(sidebar_btm, frame.buffer_mut(), &mut self.state);
        FilterInput::default().render(main_view_top, frame.buffer_mut(), &mut self.state);
        MainView::default().render(main_view_btm, frame.buffer_mut(), &mut self.state);
        StatusBar::default().render(btm_line, frame.buffer_mut(), &mut self.state);

        // show the cursor if we're editing something
        if self.state.mode == Mode::EditingFilter {
            let cursor_position = FilterInput::cursor_position(&self.state, main_view_top);
            frame.set_cursor(cursor_position.0, cursor_position.1);
        }
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
        match self.state.mode {
            Mode::EditingFilter => FilterInput::handle_event(event, &mut self.state),
            Mode::Navigating => match event {
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.state.mode = Mode::Exiting;
                        true
                    }
                    KeyCode::Char('q') => {
                        self.state.mode = Mode::Exiting;
                        true
                    }
                    KeyCode::Char('J') => {
                        self.state.focus = match self.state.focus {
                            WidgetFocus::DatabaseList => WidgetFocus::CollectionList,
                            WidgetFocus::FilterEditor => WidgetFocus::MainView,
                            m => m,
                        };
                        true
                    }
                    KeyCode::Char('K') => {
                        self.state.focus = match self.state.focus {
                            WidgetFocus::CollectionList => WidgetFocus::DatabaseList,
                            WidgetFocus::MainView => WidgetFocus::FilterEditor,
                            m => m,
                        };
                        true
                    }
                    KeyCode::Char('H') => {
                        self.state.focus = match self.state.focus {
                            WidgetFocus::MainView => WidgetFocus::CollectionList,
                            WidgetFocus::FilterEditor => WidgetFocus::DatabaseList,
                            m => m,
                        };
                        true
                    }
                    KeyCode::Char('L') => {
                        self.state.focus = match self.state.focus {
                            WidgetFocus::CollectionList => WidgetFocus::MainView,
                            WidgetFocus::DatabaseList => WidgetFocus::FilterEditor,
                            m => m,
                        };
                        true
                    }
                    _ => match self.state.focus {
                        WidgetFocus::DatabaseList => DbList::handle_event(event, &mut self.state),
                        WidgetFocus::CollectionList => {
                            CollList::handle_event(event, &mut self.state)
                        }
                        WidgetFocus::MainView => MainView::handle_event(event, &mut self.state),
                        WidgetFocus::FilterEditor => {
                            FilterInput::handle_event(event, &mut self.state)
                        }
                    },
                },
                Event::Resize(_, _) => true,
                _ => false,
            },
            Mode::Exiting => false,
        }
    }
}
