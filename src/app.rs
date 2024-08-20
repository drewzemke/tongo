use crate::{
    client::Client,
    command::{Command, CommandGroup},
    components::{
        list::connections::Connections, status_bar::StatusBar, Component, ComponentCommand,
        UniqueType,
    },
    connection::Connection,
    event::Event,
    screens::{
        connection_screen::{ConnScreenFocus, ConnectionScreen},
        primary_screen::{PrimaryScreenFocus, PrimaryScreenV2},
    },
    state::{Mode, Screen, State, WidgetFocus},
};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame, Terminal,
};
use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppFocus {
    ConnScreen(ConnScreenFocus),
    PrimaryScreen(PrimaryScreenFocus),
}

impl Default for AppFocus {
    fn default() -> Self {
        Self::ConnScreen(ConnScreenFocus::ConnList)
    }
}

#[derive(Default)]
pub struct App<'a> {
    state: State<'a>,
    raw_mode: bool,
    client: Client,

    // components
    conn_screen: ConnectionScreen,
    primary_screen: PrimaryScreenV2<'a>,
    status_bar: StatusBar,

    // shared data
    focus: Rc<RefCell<AppFocus>>,
    cursor_pos: Rc<RefCell<(u16, u16)>>,
    // commands: Vec<CommandGroup>,
}

const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

impl<'a> App<'a> {
    pub fn new(connection: Option<Connection>, all_connections: Vec<Connection>) -> Self {
        let mut state = State::new();

        let focus = if let Some(connection) = connection {
            state.set_conn_str(connection.connection_str);
            // state.conn_str_editor.input = state
            //     .conn_str_editor
            //     .input
            //     .with_value(connection.connection_str);
            AppFocus::PrimaryScreen(PrimaryScreenFocus::DbList)
        } else {
            state.screen = Screen::Connection;
            state.mode = Mode::Navigating;
            state.focus = WidgetFocus::ConnectionList;
            AppFocus::ConnScreen(ConnScreenFocus::ConnList)
        };
        let focus = Rc::new(RefCell::new(focus));

        let cursor_pos = Rc::new(RefCell::new((0, 0)));
        let connection_list = Connections::new(focus.clone(), all_connections);

        let primary_screen = PrimaryScreenV2::new(focus.clone(), cursor_pos.clone());
        let connection_screen =
            ConnectionScreen::new(connection_list, focus.clone(), cursor_pos.clone());

        Self {
            state,

            raw_mode: false,
            primary_screen,
            conn_screen: connection_screen,

            // commands: vec![],
            focus,
            cursor_pos,

            ..Default::default()
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
                // FIXME: how do I make it so that we don't _always_ redraw?
                // this causes a redraw every frame
                vec![Event::Tick]
            };

            let mut update = true;

            // process events
            let mut events_deque = VecDeque::from(events);
            while let Some(event) = events_deque.pop_front() {
                let new_events = self.handle_event(&event);
                for new_event in new_events {
                    events_deque.push_back(new_event);
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

        vec![]
    }
}

impl<'a> Component<UniqueType> for App<'a> {
    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = if self.raw_mode {
            vec![]
        } else {
            vec![CommandGroup::new(vec![Command::Quit], "q", "quit")]
        };

        out.append(&mut self.client.commands());
        out.append(&mut self.status_bar.commands());

        match *self.focus.borrow() {
            AppFocus::ConnScreen(_) => out.append(&mut self.conn_screen.commands()),
            AppFocus::PrimaryScreen(_) => out.append(&mut self.primary_screen.commands()),
        }
        out
    }

    #[must_use]
    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        if matches!(command, ComponentCommand::Command(Command::Quit)) {
            self.state.mode = Mode::Exiting;
            return vec![];
        }
        let mut out = vec![];
        out.append(&mut self.client.handle_command(command));
        out.append(&mut self.conn_screen.handle_command(command));
        out.append(&mut self.primary_screen.handle_command(command));
        out
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        match event {
            Event::ConnectionCreated(conn) | Event::ConnectionSelected(conn) => {
                self.client.set_conn_str(conn.connection_str.clone());
                self.primary_screen.focus();
            }
            Event::ErrorOccurred(error) => {
                self.status_bar.message = Some(error.clone());
            }
            Event::RawModeEntered => {
                self.raw_mode = true;
            }
            Event::RawModeExited => {
                self.raw_mode = false;
            }
            _ => {}
        };
        out.append(&mut self.client.handle_event(event));
        out.append(&mut self.conn_screen.handle_event(event));
        out.append(&mut self.primary_screen.handle_event(event));
        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let frame_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Length(1)])
            .split(area);
        let content = frame_layout[0];
        let btm_line = frame_layout[1];

        match &*self.focus.borrow() {
            AppFocus::PrimaryScreen(..) => self.primary_screen.render(frame, content),
            AppFocus::ConnScreen(..) => self.conn_screen.render(frame, content),
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

    fn focus(&self) {}

    fn is_focused(&self) -> bool {
        true
    }
}
