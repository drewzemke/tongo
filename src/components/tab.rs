use crate::{
    client::{Client, PersistedClient},
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
    system::{command::CommandGroup, event::Event},
    utils::storage::Storage,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabFocus {
    ConnScr(ConnScrFocus),
    PrimScr(PrimScrFocus),
    ConfModal,
    NotFocused,
}

impl Default for TabFocus {
    fn default() -> Self {
        Self::ConnScr(ConnScrFocus::ConnList)
    }
}

#[derive(Debug)]
pub struct Tab<'a> {
    // components
    client: Client,
    conn_screen: ConnectionScreen,
    primary_screen: PrimaryScreen<'a>,
    status_bar: StatusBar,
    confirm_modal: ConfirmModal,

    // used when displaying the confirm modal or while the app is unfocused
    focus: Rc<RefCell<TabFocus>>,
    background_focus: Option<TabFocus>,
}

impl Default for Tab<'_> {
    fn default() -> Tab<'static> {
        Tab {
            client: Client::default(),
            conn_screen: ConnectionScreen::default(),
            primary_screen: PrimaryScreen::default(),
            status_bar: StatusBar::default(),
            confirm_modal: ConfirmModal::default(),
            focus: Rc::new(RefCell::new(TabFocus::default())),
            background_focus: None,
        }
    }
}

impl Tab<'_> {
    // TODO: organize this function a bit better
    // TODO?: all_connections can be stored in the persisted connection list rather than
    // read in from a separate file
    pub fn new(
        connection: Option<Connection>,
        all_connections: Vec<Connection>,
        key_map: KeyMap,
        storage: Rc<dyn Storage>,
        cursor_pos: Rc<Cell<(u16, u16)>>,
    ) -> Tab<'static> {
        let client = Client::default();

        let initial_focus = if let Some(conn) = connection {
            client.set_conn_str(conn.connection_str);
            TabFocus::PrimScr(PrimScrFocus::DbList)
        } else {
            TabFocus::ConnScr(ConnScrFocus::ConnList)
        };

        // initialize shared data
        let focus = Rc::new(RefCell::new(initial_focus));
        let key_map = Rc::new(RefCell::new(key_map));

        let status_bar = StatusBar::new(key_map.clone());

        let confirm_modal = ConfirmModal::new(focus.clone());

        let primary_screen = PrimaryScreen::new(focus.clone(), cursor_pos.clone());

        let connection_list = Connections::new(focus.clone(), all_connections, storage.clone());
        let conn_screen = ConnectionScreen::new(
            connection_list,
            focus.clone(),
            cursor_pos.clone(),
            storage.clone(),
        );

        Tab {
            client,

            status_bar,
            primary_screen,
            conn_screen,
            confirm_modal,

            focus,

            ..Default::default()
        }
    }

    pub fn exec_queued_ops(&mut self) {
        self.client.exec_queued_ops();
    }
}

impl Component for Tab<'_> {
    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = vec![];

        out.append(&mut self.client.commands());
        out.append(&mut self.status_bar.commands());

        match *self.focus.borrow() {
            TabFocus::ConnScr(_) => out.append(&mut self.conn_screen.commands()),
            TabFocus::PrimScr(_) => out.append(&mut self.primary_screen.commands()),
            TabFocus::ConfModal => out.append(&mut self.confirm_modal.commands()),
            TabFocus::NotFocused => {}
        }
        out
    }

    #[must_use]
    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        // HACK: need to clone here to avoid borrow error with the focus `RefCell`
        // TODO: refactor to use `Cell` instead of `RefCell`, since AppFocus is Copy
        let app_focus = self.focus.borrow().clone();
        match app_focus {
            TabFocus::ConnScr(_) => self.conn_screen.handle_command(command),
            TabFocus::PrimScr(_) => self.primary_screen.handle_command(command),
            TabFocus::ConfModal => self.confirm_modal.handle_command(command),
            TabFocus::NotFocused => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        match event {
            Event::ConnectionCreated(..) | Event::ConnectionSelected(..) => {
                self.primary_screen.focus();
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
            TabFocus::PrimScr(..) => self.primary_screen.render(frame, content),
            TabFocus::ConnScr(..) => self.conn_screen.render(frame, content),
            TabFocus::ConfModal => {
                match self.background_focus {
                    Some(TabFocus::PrimScr(..)) => self.primary_screen.render(frame, content),
                    Some(TabFocus::ConnScr(..)) => self.conn_screen.render(frame, content),
                    _ => {}
                }
                self.confirm_modal.render(frame, content);
            }
            TabFocus::NotFocused => {}
        }

        // status bar
        // TODO: avoid a second call to `commands()` here?
        self.status_bar.commands = self.commands();
        self.status_bar.render(frame, btm_line);
    }

    /// Not used.
    fn focus(&self) {}

    /// Not used.
    fn is_focused(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTab {
    focus: TabFocus,
    client: PersistedClient,
    conn_screen: PersistedConnectionScreen,
    primary_screen: PersistedPrimaryScreen,
}

impl PersistedComponent for Tab<'_> {
    type StorageType = PersistedTab;

    fn persist(&self) -> Self::StorageType {
        // don't save focus as any of the input components, it gets weird
        let focus = match *self.focus.borrow() {
            TabFocus::ConnScr(..) => TabFocus::ConnScr(ConnScrFocus::ConnList),
            TabFocus::PrimScr(ref focus) => {
                let ps_focus = match focus {
                    PrimScrFocus::FilterIn => PrimScrFocus::DocTree,
                    f => f.clone(),
                };
                TabFocus::PrimScr(ps_focus)
            }
            TabFocus::ConfModal | TabFocus::NotFocused => {
                self.background_focus.clone().unwrap_or_default()
            }
        };

        PersistedTab {
            focus,
            client: self.client.persist(),
            conn_screen: self.conn_screen.persist(),
            primary_screen: self.primary_screen.persist(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        *self.focus.borrow_mut() = storage.focus;
        self.conn_screen.hydrate(storage.conn_screen.clone());
        self.primary_screen.hydrate(storage.primary_screen);

        self.client.hydrate(storage.client);
        if let Some(conn) = storage.conn_screen.conn_list.selected_conn {
            self.client.set_conn_str(conn.connection_str);
        }
    }
}
