use crate::command::{Command, CommandGroup};
use crate::components::list::connection_list::ConnectionList;
use crate::components::{Component, ComponentCommand, UniqueType};
use crate::connection::Connection;
use crate::event::Event;
use crate::screens::connection_screen::ConnectionScreen;
use crate::screens::primary_screen::PrimaryScreen;
use crate::state::{Mode, Screen, State, WidgetFocus};
use crate::widgets::status_bar::StatusBar;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct App<'a> {
    state: State<'a>,

    raw_mode: bool,
    // commands: Vec<CommandGroup>,
    cursor_pos: Rc<RefCell<(u16, u16)>>,
    connection_screen: ConnectionScreen,
    status_bar: StatusBar,
}

const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

impl<'a> App<'a> {
    pub fn new(connection: Option<Connection>, all_connections: Vec<Connection>) -> Self {
        let mut state = State::new();

        let connection_list = ConnectionList {
            items: all_connections,
            ..Default::default()
        };
        let connection_screen = ConnectionScreen::new(connection_list);

        if let Some(connection) = connection {
            state.set_conn_str(connection.connection_str.clone());
            state.conn_str_editor.input = state
                .conn_str_editor
                .input
                .with_value(connection.connection_str);
            state.screen = Screen::Primary;
        } else {
            state.screen = Screen::Connection;
            state.mode = Mode::Navigating;
            state.focus = WidgetFocus::ConnectionList;
        }

        Self {
            state,

            raw_mode: false,
            // commands: vec![],
            cursor_pos: Rc::new(RefCell::new((0, 0))),
            connection_screen,
            status_bar: StatusBar::default(),
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        // initial draw call
        terminal.draw(|frame| self.render(frame, frame.size()))?;

        let debounce: Option<Instant> = None;

        loop {
            // check for respones
            if let Ok(content) = self.state.response_recv.try_recv() {
                self.state.update_content(content);
            }

            let timeout =
                debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));

            let events = if crossterm::event::poll(timeout)? {
                let event = crossterm::event::read()?;
                self.handle_user_event(&event)
            } else {
                vec![]
            };

            let mut update = false;
            for event in events {
                if self.handle_event(&event) {
                    update = true;
                }
            }

            // exit if the app is in an exiting state
            if self.state.mode == Mode::Exiting {
                return Ok(());
            }

            if self.state.new_data {
                update = true;
                self.state.new_data = false;
            }

            if self.state.clear_screen {
                terminal.clear()?;
                self.state.clear_screen = false;
            }

            if update {
                terminal.draw(|frame| {
                    self.render(frame, frame.size());
                })?;
            }
        }
    }

    fn handle_user_event(&mut self, event: &CrosstermEvent) -> Vec<Event> {
        // NOTE: for now we only deal with key events
        if let CrosstermEvent::Key(key) = event {
            // always quit on Control-C
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.state.mode = Mode::Exiting;
                return vec![];
            }

            // if in raw mode, check for enter or escape
            // otherwise just pass the whole event
            if self.raw_mode {
                if key.code == KeyCode::Enter {
                    return self.handle_command(&ComponentCommand::Command(Command::Confirm));
                }
                if key.code == KeyCode::Esc {
                    return self.handle_command(&ComponentCommand::Command(Command::Back));
                }
                return self.handle_command(&ComponentCommand::RawEvent(event));
            }

            // map the key to a command if we're not in raw mode
            let command = self
                .commands()
                .iter()
                .flat_map(|group| &group.commands)
                .find(|command| command.key() == key.code)
                .copied();

            // handle the command
            if let Some(command) = command {
                return self.handle_command(&ComponentCommand::Command(command));
            }
        }

        // TODO: remove, eventually
        if let Screen::Primary = self.state.screen {
            PrimaryScreen::handle_event(event, &mut self.state);
        };

        // HACK shouldn't be returning something here
        vec![Event::ListSelectionChanged]
    }
}

impl<'a> Component<UniqueType> for App<'a> {
    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = if self.raw_mode {
            vec![]
        } else {
            vec![CommandGroup::new(vec![Command::Quit], "q", "quit")]
        };

        out.append(&mut self.status_bar.commands());

        // TODO: should be based on app state
        if self.state.screen == Screen::Connection {
            out.append(&mut self.connection_screen.commands());
        }
        out
    }

    #[must_use]
    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        if matches!(command, ComponentCommand::Command(Command::Quit)) {
            self.state.mode = Mode::Exiting;
            return vec![];
        }
        self.connection_screen.handle_command(command)
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        let internal_update = match event {
            Event::ConnectionSelected(connection) => {
                self.state.set_conn_str(connection.connection_str.clone());
                self.state.screen = Screen::Primary;
                self.state.mode = Mode::Navigating;
                self.state.focus = WidgetFocus::DatabaseList;
                true
            }
            Event::ListSelectionChanged => true,
            Event::ErrorOccurred(error) => {
                self.status_bar.message = Some(error.clone());
                true
            }
            Event::NewConnectionStarted | Event::RawModeEntered => {
                self.raw_mode = true;
                true
            }
            Event::RawModeExited => {
                self.raw_mode = false;
                true
            }
            _ => false,
        };
        let conn_scr_update = self.connection_screen.handle_event(event);

        internal_update || conn_scr_update
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let frame_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Length(1)])
            .split(area);
        let content = frame_layout[0];
        let btm_line = frame_layout[1];

        match self.state.screen {
            Screen::Primary => {
                PrimaryScreen::default().render(content, frame.buffer_mut(), &mut self.state);
            }
            Screen::Connection => {
                self.connection_screen.render(frame, content);
            }
        }

        // status bar
        // HACK suboptimal stuff while refactoring around commands
        self.status_bar
            .message
            .clone_from(&self.state.status_bar.message);
        self.status_bar.commands = self.commands();
        self.status_bar.render(frame, btm_line);

        // show the cursor if we're editing something
        if self.raw_mode {
            let (x, y) = *self.cursor_pos.borrow();
            frame.set_cursor(x, y);
        }
    }
}
