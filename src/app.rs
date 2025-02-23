use crate::{
    client::Client,
    components::{
        confirm_modal::ConfirmModal,
        connection_screen::{ConnScrFocus, ConnectionScreen, PersistedConnectionScreen},
        list::connections::Connections,
        primary_screen::{PersistedPrimaryScreen, PrimScrFocus, PrimaryScreen},
        status_bar::StatusBar,
        Component, ComponentCommand,
    },
    connection::Connection,
    key_map::KeyMap,
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandGroup},
        event::Event,
    },
    utils::storage::{FileStorage, Storage},
};

use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppFocus {
    ConnScr(ConnScrFocus),
    PrimScr(PrimScrFocus),
    ConfModal,
    NotFocused,
}

impl Default for AppFocus {
    fn default() -> Self {
        Self::ConnScr(ConnScrFocus::ConnList)
    }
}

#[derive(Debug)]
pub struct App<'a> {
    // components
    client: Client,
    conn_screen: ConnectionScreen,
    primary_screen: PrimaryScreen<'a>,
    status_bar: StatusBar,
    confirm_modal: ConfirmModal,

    // used when displaying the confirm modal or while the app is unfocused
    // FIXME: should these be seperate?
    background_focus: Option<AppFocus>,

    // shared data
    focus: Rc<RefCell<AppFocus>>,
    cursor_pos: Rc<Cell<(u16, u16)>>,
    storage: Rc<dyn Storage>,

    // config
    // FIXME - wait a minute, why do we need a `RefCell` here??
    key_map: Rc<RefCell<KeyMap>>,

    // flags
    raw_mode: bool,
    force_clear: bool,
    exiting: bool,
}

impl Default for App<'_> {
    fn default() -> Self {
        Self {
            client: Client::default(),
            conn_screen: ConnectionScreen::default(),
            primary_screen: PrimaryScreen::default(),
            status_bar: StatusBar::default(),
            confirm_modal: ConfirmModal::default(),
            background_focus: None,
            focus: Rc::new(RefCell::new(AppFocus::default())),
            cursor_pos: Rc::new(Cell::new((0, 0))),
            storage: Rc::new(FileStorage::default()),
            key_map: Rc::new(RefCell::new(KeyMap::default())),
            raw_mode: false,
            force_clear: false,
            exiting: false,
        }
    }
}

const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

impl App<'_> {
    // TODO: organize this function a bit better
    // TODO?: all_connections can be stored in the persisted connection list rather than
    // read in from a separate file
    pub fn new(
        connection: Option<Connection>,
        all_connections: Vec<Connection>,
        key_map: KeyMap,
        storage: Rc<dyn Storage>,
    ) -> Self {
        let client = Client::default();

        let initial_focus = if let Some(conn) = connection {
            client.set_conn_str(conn.connection_str);
            AppFocus::PrimScr(PrimScrFocus::DbList)
        } else {
            AppFocus::ConnScr(ConnScrFocus::ConnList)
        };

        // initialize shared data
        let focus = Rc::new(RefCell::new(initial_focus));
        let cursor_pos = Rc::new(Cell::new((0, 0)));
        let key_map = Rc::new(RefCell::new(key_map));

        let status_bar = StatusBar::new(key_map.clone());

        let confirm_modal = ConfirmModal::new(focus.clone());

        let primary_screen = PrimaryScreen::new(focus.clone(), cursor_pos.clone());

        let connection_list = Connections::new(focus.clone(), all_connections, storage.clone());
        let connection_screen = ConnectionScreen::new(
            connection_list,
            focus.clone(),
            cursor_pos.clone(),
            storage.clone(),
        );

        Self {
            client,

            raw_mode: false,

            status_bar,
            primary_screen,
            conn_screen: connection_screen,
            confirm_modal,

            // commands: vec![],
            focus,
            cursor_pos,

            key_map,
            storage,

            ..Default::default()
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // initial draw call
        terminal.draw(|frame| self.render(frame, frame.area()))?;

        let debounce: Option<Instant> = None;

        loop {
            let timeout =
                debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));

            // if a key is presssed, process it and send it through the system.
            // if no key is pressed, process a `tick` event and send it
            let events = if crossterm::event::poll(timeout)? {
                let event = crossterm::event::read()?;
                self.handle_terminal_event(&event)
            } else {
                vec![Event::Tick]
            };

            // process events
            let should_render = self.process_events(events);

            // once all the events are processed for this loop, tell the client to execute
            // any operations it decided to do during event processing loop
            self.client.exec_queued_ops();

            // save state if we're about to exit
            if self.exiting {
                self.persist_self()?;
                return Ok(());
            }

            if self.force_clear {
                terminal.clear()?;
                self.force_clear = false;
            }

            if should_render {
                terminal.draw(|frame| {
                    self.render(frame, frame.area());
                })?;
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn process_events(&mut self, events: Vec<Event>) -> bool {
        let mut should_render = false;
        let mut events_deque = VecDeque::from(events);

        while let Some(event) = events_deque.pop_front() {
            let is_nontrivial_event = !matches!(event, Event::Tick);

            // set the render flag to true if we get an event that isn't `Event::Tick`
            should_render = should_render || is_nontrivial_event;

            if is_nontrivial_event {
                tracing::debug!("Processing event {event}");
            }

            let new_events = self.handle_event(&event);
            for new_event in new_events {
                events_deque.push_back(new_event);
            }
        }

        should_render
    }

    fn handle_terminal_event(&mut self, event: &CrosstermEvent) -> Vec<Event> {
        // NOTE: for now we only deal with key events
        if let CrosstermEvent::Key(key) = event {
            // always quit on Control-C
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.exiting = true;
                return vec![];
            }

            // if in raw mode, check for enter or escape
            // otherwise just pass the whole event
            //
            // FIXME: these should be configurable!
            if self.raw_mode {
                if key.code == KeyCode::Enter {
                    return self.handle_command(&ComponentCommand::Command(Command::Confirm));
                }
                if key.code == KeyCode::Esc {
                    return self.handle_command(&ComponentCommand::Command(Command::Back));
                }
                return self.handle_command(&ComponentCommand::RawEvent(event.clone()));
            }

            // map the key to a command if we're not in raw mode,
            // making sure it's one of the currently-available commands
            let command = self
                .key_map
                .borrow()
                .command_for_key(key.code, &self.commands());

            // handle the command
            if let Some(command) = command {
                return self.handle_command(&ComponentCommand::Command(command));
            }
        }

        match event {
            CrosstermEvent::Resize(..) => {
                // returning a nontrivial event triggers a redraw
                vec![Event::ScreenResized]
            }
            CrosstermEvent::FocusGained => vec![Event::AppFocusGained],
            CrosstermEvent::FocusLost => vec![Event::AppFocusLost],
            _ => vec![],
        }
    }

    fn persist_self(&self) -> Result<()> {
        let stored_app = self.persist();
        self.storage.write_last_session(&stored_app)?;
        Ok(())
    }
}

impl Component for App<'_> {
    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = if self.raw_mode {
            vec![]
        } else {
            vec![CommandGroup::new(vec![Command::Quit], "quit")]
        };

        out.append(&mut self.client.commands());
        out.append(&mut self.status_bar.commands());

        match *self.focus.borrow() {
            AppFocus::ConnScr(_) => out.append(&mut self.conn_screen.commands()),
            AppFocus::PrimScr(_) => out.append(&mut self.primary_screen.commands()),
            AppFocus::ConfModal => out.append(&mut self.confirm_modal.commands()),
            AppFocus::NotFocused => {}
        }
        out
    }

    #[must_use]
    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        if matches!(command, ComponentCommand::Command(Command::Quit)) {
            tracing::info!("Quit command received. Exiting...");
            self.exiting = true;
            return vec![];
        }

        // HACK: need to clone here to avoid borrow error with the focus `RefCell`
        // TODO: refactor to use `Cell` instead of `RefCell`, since AppFocus is Copy
        let app_focus = self.focus.borrow().clone();
        match app_focus {
            AppFocus::ConnScr(_) => self.conn_screen.handle_command(command),
            AppFocus::PrimScr(_) => self.primary_screen.handle_command(command),
            AppFocus::ConfModal => self.confirm_modal.handle_command(command),
            AppFocus::NotFocused => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        match event {
            Event::ConnectionCreated(..) | Event::ConnectionSelected(..) => {
                self.primary_screen.focus();
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
            Event::ConfirmationRequested(command) => {
                self.background_focus = Some(self.focus.borrow().clone());
                self.confirm_modal.show_with(*command);
            }
            Event::ConfirmationYes(..) | Event::ConfirmationNo => {
                *self.focus.borrow_mut() = self.background_focus.take().unwrap_or_default();
            }
            _ => {}
        }
        out.append(&mut self.client.handle_event(event));
        out.append(&mut self.conn_screen.handle_event(event));
        out.append(&mut self.primary_screen.handle_event(event));
        out.append(&mut self.status_bar.handle_event(event));
        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let frame_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Length(1)])
            .split(area);
        let content = frame_layout[0];
        let btm_line = frame_layout[1];

        // render a screen based on current focus
        match &*self.focus.borrow() {
            AppFocus::PrimScr(..) => self.primary_screen.render(frame, content),
            AppFocus::ConnScr(..) => self.conn_screen.render(frame, content),
            AppFocus::ConfModal => {
                match self.background_focus {
                    Some(AppFocus::PrimScr(..)) => self.primary_screen.render(frame, content),
                    Some(AppFocus::ConnScr(..)) => self.conn_screen.render(frame, content),
                    _ => {}
                }
                self.confirm_modal.render(frame, content);
            }
            AppFocus::NotFocused => {}
        }

        // status bar
        // TODO: avoid a second call to `commands()` here?
        self.status_bar.commands = self.commands();
        self.status_bar.render(frame, btm_line);

        // show the cursor if we're editing something
        if self.raw_mode {
            let (x, y) = self.cursor_pos.get();
            frame.set_cursor_position((x, y));
        }
    }

    /// Not used.
    fn focus(&self) {}

    /// Not used.
    fn is_focused(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedApp {
    focus: AppFocus,
    conn_screen: PersistedConnectionScreen,
    primary_screen: PersistedPrimaryScreen,
}

impl PersistedComponent for App<'_> {
    type StorageType = PersistedApp;

    fn persist(&self) -> Self::StorageType {
        // don't save focus as any of the input components, it gets weird
        let focus = match *self.focus.borrow() {
            AppFocus::ConnScr(..) => AppFocus::ConnScr(ConnScrFocus::ConnList),
            AppFocus::PrimScr(ref focus) => {
                let ps_focus = match focus {
                    PrimScrFocus::FilterIn => PrimScrFocus::DocTree,
                    f => f.clone(),
                };
                AppFocus::PrimScr(ps_focus)
            }
            AppFocus::ConfModal | AppFocus::NotFocused => {
                self.background_focus.clone().unwrap_or_default()
            }
        };

        PersistedApp {
            focus,
            conn_screen: self.conn_screen.persist(),
            primary_screen: self.primary_screen.persist(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        *self.focus.borrow_mut() = storage.focus;

        self.conn_screen.hydrate(storage.conn_screen.clone());
        self.primary_screen.hydrate(storage.primary_screen);

        if let Some(Connection { connection_str, .. }) = storage.conn_screen.conn_list.selected_conn
        {
            self.client.set_conn_str(connection_str);
        }
    }
}
