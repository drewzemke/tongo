use crate::{
    components::{
        status_bar::StatusBar,
        tab::{PersistedTab, Tab},
        tab_bar::{PersistedTabBar, TabBar},
        Component,
    },
    connection::{Connection, ConnectionManager},
    key_map::KeyMap,
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandGroup},
        event::Event,
        message::{AppAction, Message},
        Signal,
    },
    utils::storage::{FileStorage, Storage},
};
use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use ratatui::{backend::Backend, prelude::*};
use serde::{Deserialize, Serialize};
use std::{
    cell::Cell,
    collections::VecDeque,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct App<'a> {
    //components
    tabs: Vec<Tab<'a>>,
    tab_bar: TabBar,
    status_bar: StatusBar,

    // shared data
    cursor_pos: Rc<Cell<(u16, u16)>>,
    storage: Rc<dyn Storage>,
    connection_manager: ConnectionManager,

    // config
    key_map: Rc<KeyMap>,

    // flags
    raw_mode: bool,
    force_clear: bool,
    exiting: bool,
}

impl Default for App<'_> {
    fn default() -> Self {
        let storage = Rc::new(FileStorage::default());
        Self {
            tabs: vec![Tab::default()],
            tab_bar: TabBar::default(),
            status_bar: StatusBar::default(),
            cursor_pos: Rc::new(Cell::new((0, 0))),
            connection_manager: ConnectionManager::new(vec![], storage.clone()),
            storage,
            key_map: Rc::new(KeyMap::default()),
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
        selected_connection: Option<Connection>,
        connections: Vec<Connection>,
        key_map: KeyMap,
        storage: Rc<dyn Storage>,
    ) -> Self {
        // initialize shared data
        let cursor_pos = Rc::new(Cell::new((0, 0)));
        let connection_manager = ConnectionManager::new(connections, storage.clone());
        let key_map = Rc::new(key_map);

        let tab = Tab::new(
            selected_connection,
            connection_manager.clone(),
            cursor_pos.clone(),
        );

        let tab_bar = TabBar::new();
        let status_bar = StatusBar::new(key_map.clone());

        Self {
            tabs: vec![tab],
            tab_bar,
            status_bar,

            raw_mode: false,

            cursor_pos,

            key_map,
            storage,
            connection_manager,

            force_clear: false,
            exiting: false,
        }
    }

    fn create_tab(&self) -> Tab<'static> {
        Tab::new(
            None,
            self.connection_manager.clone(),
            self.cursor_pos.clone(),
        )
    }

    const fn current_tab_idx(&self) -> usize {
        self.tab_bar.current_tab_idx()
    }

    /// Starts and runs the main app loop until the user exits the program.
    ///
    /// # Errors
    /// If something goes wrong while drawing to the screen
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // initial draw call
        terminal.draw(|frame| self.render(frame, frame.area()))?;

        let debounce: Option<Instant> = None;

        loop {
            let timeout =
                debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));

            // if a key is presssed, process it and send it through the system.
            // if no key is pressed, process a `tick` event and send it
            let signals = if crossterm::event::poll(timeout)? {
                let event = crossterm::event::read()?;
                self.handle_raw_event(&event)
            } else {
                vec![Event::Tick.into()]
            };

            // process events and messages
            let should_render = self.process_signals(signals);

            // once all the signals are processed for this loop, tell the client to execute
            // any operations it decided to do during event processing loop
            for tab in &mut self.tabs {
                tab.exec_queued_ops();
            }

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
    fn process_signals(&mut self, signals: Vec<Signal>) -> bool {
        let mut should_render = false;
        let mut signals_deque = VecDeque::from(signals);

        while let Some(signal) = signals_deque.pop_front() {
            let is_nontrivial_event = !matches!(signal, Signal::Event(Event::Tick));

            // set the render flag to true if we get an event that isn't `Event::Tick`
            should_render = should_render || is_nontrivial_event;

            if is_nontrivial_event {
                tracing::debug!("Processing signal {signal}");
            }

            let new_signals = match signal {
                Signal::Event(event) => self.handle_event(&event),
                Signal::Message(message) => self.handle_message(&message),
            };

            for new_signal in new_signals {
                signals_deque.push_back(new_signal);
            }
        }

        should_render
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

        out.append(&mut self.status_bar.commands());
        out.append(&mut self.tab_bar.commands());

        if let Some(tab) = &mut self.tabs.get(self.current_tab_idx()) {
            out.append(&mut tab.commands());
        }

        out
    }

    #[must_use]
    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        match command {
            Command::Quit => {
                tracing::info!("Quit command received. Exiting...");
                self.exiting = true;
                return vec![];
            }
            Command::NewTab => {
                let tab = self.create_tab();
                self.tabs.push(tab);
            }
            Command::DuplicateTab => {
                if let Some(current_tab) = self.tabs.get(self.tab_bar.current_tab_idx()) {
                    let new_tab = current_tab.clone();
                    self.tabs.push(new_tab);
                }
            }
            Command::CloseTab => {
                if self.tabs.len() > 1 {
                    self.tabs.remove(self.tab_bar.current_tab_idx());
                }
            }
            _ => {}
        }

        // the tab bar sees every command (although it only handles a few of them)
        let mut out = self.tab_bar.handle_command(command);

        let index = self.current_tab_idx();
        if let Some(tab) = &mut self.tabs.get_mut(index) {
            out.append(&mut tab.handle_command(command));
        }
        out
    }

    fn handle_raw_event(&mut self, event: &CrosstermEvent) -> Vec<Signal> {
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
                    return self.handle_command(&Command::Confirm);
                }
                if key.code == KeyCode::Esc {
                    return self.handle_command(&Command::Back);
                }
                let index = self.current_tab_idx();
                if let Some(tab) = &mut self.tabs.get_mut(index) {
                    return tab.handle_raw_event(event);
                }
            }

            // map the key to a command if we're not in raw mode,
            // making sure it's one of the currently-available commands
            let command = self.key_map.command_for_key(key.code, &self.commands());

            // handle the command
            if let Some(command) = command {
                return self.handle_command(&command);
            }
        }

        match event {
            CrosstermEvent::Resize(..) => {
                // returning a nontrivial event triggers a redraw
                vec![Event::ScreenResized.into()]
            }
            CrosstermEvent::FocusGained => vec![Event::AppFocusGained.into()],
            CrosstermEvent::FocusLost => vec![Event::AppFocusLost.into()],
            _ => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        let mut out = vec![];
        if matches!(event, Event::ReturnedFromAltScreen) {
            self.force_clear = true;
        }

        out.append(&mut self.tab_bar.handle_event(event));

        let index = self.current_tab_idx();
        if let Some(tab) = self.tabs.get_mut(index) {
            out.append(&mut tab.handle_event(event));
        }

        out.append(&mut self.status_bar.handle_event(event));

        out
    }

    fn handle_message(&mut self, message: &Message) -> Vec<Signal> {
        match message.read_as_app() {
            Some(AppAction::EnterRawMode) => self.raw_mode = true,
            Some(AppAction::ExitRawMode) => self.raw_mode = false,
            _ => {
                let index = self.current_tab_idx();
                if let Some(tab) = self.tabs.get_mut(index) {
                    return tab.handle_message(message);
                }
            }
        }

        vec![]
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // split off bottom line(s) for the status bar
        let frame_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(1)])
            .split(area);
        let main_area = frame_layout[0];
        let status_bar_area = frame_layout[1];

        // split off top line for the tab bar
        let main_area = if self.tab_bar.num_tabs() > 1 {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Fill(1)])
                .split(main_area);
            let tab_area = layout[0].inner(Margin::new(1, 0));
            self.tab_bar.render(frame, tab_area);
            layout[1]
        } else {
            main_area
        };

        // render current tab
        let index = self.current_tab_idx();
        if let Some(tab) = &mut self.tabs.get_mut(index) {
            tab.render(frame, main_area);
        }

        // render status bar
        // TODO: avoid a second call to `commands()` here?
        self.status_bar.commands = self.commands();
        self.status_bar.render(frame, status_bar_area);

        // show the cursor if we're editing something
        if self.raw_mode {
            let (x, y) = self.cursor_pos.get();
            frame.set_cursor_position((x, y));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedApp {
    tabs: Vec<PersistedTab>,
    tab_bar: PersistedTabBar,
}

impl PersistedComponent for App<'_> {
    type StorageType = PersistedApp;

    fn persist(&self) -> Self::StorageType {
        let tabs = self.tabs.iter().map(Tab::persist).collect();
        PersistedApp {
            tabs,
            tab_bar: self.tab_bar.persist(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        // delete the default tab, then create and hydreate new tabs for each stored one
        self.tabs = vec![];
        for persisted_tab in storage.tabs {
            let mut tab = self.create_tab();
            tab.hydrate(persisted_tab);
            self.tabs.push(tab);
        }

        self.tab_bar.hydrate(storage.tab_bar);
    }
}
