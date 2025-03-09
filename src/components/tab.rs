use crate::{
    client::{Client, PersistedClient},
    components::{
        confirm_modal::ConfirmModal,
        connection_screen::{ConnScrFocus, ConnectionScreen, PersistedConnectionScreen},
        list::connections::Connections,
        primary_screen::{PersistedPrimaryScreen, PrimScrFocus, PrimaryScreen},
        Component,
    },
    connection::{Connection, ConnectionManager},
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandGroup},
        event::Event,
        message::{AppAction, Message, TabAction},
        Signal,
    },
};
use ratatui::{layout::Rect, Frame};
use serde::{Deserialize, Serialize};
use std::{cell::Cell, rc::Rc};

use super::input::input_modal::InputModal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabFocus {
    ConnScr(ConnScrFocus),
    PrimScr(PrimScrFocus),
    ConfModal,
    InputModal,
    NotFocused,
}

impl Default for TabFocus {
    fn default() -> Self {
        Self::ConnScr(ConnScrFocus::ConnList)
    }
}

#[derive(Debug, Clone)]
pub struct Tab<'a> {
    // components
    client: Client,
    conn_screen: ConnectionScreen,
    primary_screen: PrimaryScreen<'a>,
    confirm_modal: ConfirmModal,
    input_modal: InputModal,

    // used when displaying the confirm modal or while the app is unfocused
    focus: Rc<Cell<TabFocus>>,
    background_focus: Option<TabFocus>,
}

impl Default for Tab<'_> {
    fn default() -> Tab<'static> {
        Tab {
            client: Client::default(),
            conn_screen: ConnectionScreen::default(),
            primary_screen: PrimaryScreen::default(),
            confirm_modal: ConfirmModal::default(),
            input_modal: InputModal::default(),
            focus: Rc::new(Cell::new(TabFocus::default())),
            background_focus: None,
        }
    }
}

impl Tab<'_> {
    // TODO: organize this function a bit better
    // TODO?: all_connections can be stored in the persisted connection list rather than
    // read in from a separate file
    pub fn new(
        selected_connection: Option<Connection>,
        connection_manager: ConnectionManager,
        cursor_pos: Rc<Cell<(u16, u16)>>,
    ) -> Tab<'static> {
        let client = Client::default();

        let initial_focus = if let Some(conn) = selected_connection {
            client.connect(conn.connection_str);
            TabFocus::PrimScr(PrimScrFocus::DbList)
        } else {
            TabFocus::ConnScr(ConnScrFocus::ConnList)
        };

        // initialize shared data
        let focus = Rc::new(Cell::new(initial_focus));

        let confirm_modal = ConfirmModal::new(focus.clone());
        let input_modal = InputModal::new(focus.clone(), cursor_pos.clone());

        let primary_screen = PrimaryScreen::new(focus.clone(), cursor_pos.clone());

        let connection_list = Connections::new(focus.clone(), connection_manager.clone());
        let conn_screen = ConnectionScreen::new(
            connection_list,
            focus.clone(),
            cursor_pos,
            connection_manager,
        );

        Tab {
            client,

            conn_screen,
            primary_screen,
            confirm_modal,
            input_modal,

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

        match self.focus.get() {
            TabFocus::ConnScr(_) => out.append(&mut self.conn_screen.commands()),
            TabFocus::PrimScr(_) => out.append(&mut self.primary_screen.commands()),
            TabFocus::ConfModal => out.append(&mut self.confirm_modal.commands()),
            TabFocus::InputModal => out.append(&mut self.input_modal.commands()),
            TabFocus::NotFocused => {}
        }
        out
    }

    #[must_use]
    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        match self.focus.get() {
            TabFocus::ConnScr(_) => self.conn_screen.handle_command(command),
            TabFocus::PrimScr(_) => self.primary_screen.handle_command(command),
            TabFocus::ConfModal => self.confirm_modal.handle_command(command),
            TabFocus::InputModal => self.input_modal.handle_command(command),
            TabFocus::NotFocused => vec![],
        }
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event) -> Vec<Signal> {
        match self.focus.get() {
            TabFocus::ConnScr(_) => self.conn_screen.handle_raw_event(event),
            TabFocus::PrimScr(_) => self.primary_screen.handle_raw_event(event),
            TabFocus::ConfModal => self.confirm_modal.handle_raw_event(event),
            TabFocus::InputModal => self.input_modal.handle_raw_event(event),
            TabFocus::NotFocused => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        let mut out = vec![];
        match event {
            Event::ConnectionCreated(..) | Event::ConnectionSelected(..) => {
                self.primary_screen.focus();
            }
            Event::ConfirmYes(..)
            | Event::ConfirmNo
            | Event::InputConfirmed(..)
            | Event::InputCanceled => {
                self.focus
                    .set(self.background_focus.take().unwrap_or_default());
            }
            _ => {}
        }
        out.append(&mut self.client.handle_event(event));
        out.append(&mut self.conn_screen.handle_event(event));
        out.append(&mut self.primary_screen.handle_event(event));
        out
    }

    fn handle_message(&mut self, message: &Message) -> Vec<Signal> {
        if message.read_as_client().is_some() {
            self.client.handle_message(message)
        } else if message.read_as_conn_scr().is_some() {
            self.conn_screen.handle_message(message)
        } else {
            match message.read_as_tab() {
                Some(TabAction::RequestConfirmation(kind)) => {
                    self.background_focus = Some(self.focus.get());
                    self.confirm_modal.show_with(*kind);
                }
                Some(TabAction::RequestInput(kind)) => {
                    self.background_focus = Some(self.focus.get());
                    self.input_modal.show_with(*kind);
                    return vec![Message::to_app(AppAction::EnterRawMode).into()];
                }
                _ => {}
            }
            vec![]
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // render a screen based on current focus
        match self.focus.get() {
            TabFocus::PrimScr(..) => self.primary_screen.render(frame, area),
            TabFocus::ConnScr(..) => self.conn_screen.render(frame, area),
            TabFocus::ConfModal => {
                match self.background_focus {
                    Some(TabFocus::PrimScr(..)) => self.primary_screen.render(frame, area),
                    Some(TabFocus::ConnScr(..)) => self.conn_screen.render(frame, area),
                    _ => {}
                }
                self.confirm_modal.render(frame, area);
            }
            TabFocus::InputModal => {
                match self.background_focus {
                    Some(TabFocus::PrimScr(..)) => self.primary_screen.render(frame, area),
                    Some(TabFocus::ConnScr(..)) => self.conn_screen.render(frame, area),
                    _ => {}
                }
                self.input_modal.render(frame, area);
            }
            TabFocus::NotFocused => {}
        }
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
        let focus = match self.focus.get() {
            TabFocus::ConnScr(..) => TabFocus::ConnScr(ConnScrFocus::ConnList),
            TabFocus::PrimScr(ref focus) => {
                let ps_focus = match focus {
                    PrimScrFocus::FilterIn => PrimScrFocus::DocTree,
                    f => *f,
                };
                TabFocus::PrimScr(ps_focus)
            }
            TabFocus::ConfModal | TabFocus::InputModal | TabFocus::NotFocused => {
                self.background_focus.unwrap_or_default()
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
        self.focus.set(storage.focus);
        self.conn_screen.hydrate(storage.conn_screen.clone());
        self.primary_screen.hydrate(storage.primary_screen);

        self.client.hydrate(storage.client);
        if let Some(conn) = storage.conn_screen.conn_list.selected_conn {
            self.client.connect(conn.connection_str);
        }
    }
}
