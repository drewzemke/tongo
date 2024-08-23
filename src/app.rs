use crate::{
    client::Client,
    components::{
        connection_screen::{ConnScreenFocus, ConnectionScreen},
        list::connections::Connections,
        primary_screen::{PrimaryScreenFocus, PrimaryScreenV2},
        status_bar::StatusBar,
        Component, ComponentCommand, UniqueType,
    },
    connection::Connection,
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
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
    force_clear: bool,
    exiting: bool,
}

const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

impl<'a> App<'a> {
    pub fn new(connection: Option<Connection>, all_connections: Vec<Connection>) -> Self {
        let client = Client::default();

        let initial_focus = if let Some(conn) = connection {
            client.set_conn_str(conn.connection_str);
            AppFocus::PrimaryScreen(PrimaryScreenFocus::DbList)
        } else {
            AppFocus::ConnScreen(ConnScreenFocus::ConnList)
        };

        let focus = Rc::new(RefCell::new(initial_focus));
        let cursor_pos = Rc::new(RefCell::new((0, 0)));

        let primary_screen = PrimaryScreenV2::new(focus.clone(), cursor_pos.clone());

        let connection_list = Connections::new(focus.clone(), all_connections);
        let connection_screen =
            ConnectionScreen::new(connection_list, focus.clone(), cursor_pos.clone());

        Self {
            client,

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

            // process events
            let mut events_deque = VecDeque::from(events);
            while let Some(event) = events_deque.pop_front() {
                let new_events = self.handle_event(&event);
                for new_event in new_events {
                    events_deque.push_back(new_event);
                }
            }

            // exit if the app is in an exiting state
            if self.exiting {
                return Ok(());
            }

            if self.force_clear {
                terminal.clear()?;
                self.force_clear = false;
            }

            // TODO: find a way to only update when screen changes
            terminal.draw(|frame| {
                self.render(frame, frame.size());
            })?;
        }
    }

    fn handle_user_event(&mut self, event: &CrosstermEvent) -> Vec<Event> {
        // NOTE: for now we only deal with key events
        if let CrosstermEvent::Key(key) = event {
            // always quit on Control-C
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.exiting = true;
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
            self.exiting = true;
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
                // TODO: consume from within component
                self.client.set_conn_str(conn.connection_str.clone());
                self.primary_screen.focus();
            }
            Event::ErrorOccurred(error) => {
                // TODO: consume from within component
                self.status_bar.message = Some(error.clone());
            }
            Event::RawModeEntered => {
                self.raw_mode = true;
            }
            Event::RawModeExited => {
                self.raw_mode = false;
            }
            Event::ReturnedFromAltScreen => {
                self.force_clear = true;
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
        // TODO: handle some of this stuff in the status bar comp
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
