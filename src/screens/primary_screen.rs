use crate::{
    app::AppFocus,
    command::CommandGroup,
    components::{
        list::{coll_list::CollList, db_list::DbList},
        Component, ComponentCommand, UniqueType,
    },
    event::Event,
    state::{Mode, Screen, State, WidgetFocus},
    widgets::{filter_input::FilterInput, input_widget::InputWidget, main_view::MainView},
};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::prelude::*;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PrimaryScreenFocus {
    #[default]
    DbList,
    CollList,
}

#[derive(Debug, Default)]
pub struct PrimaryScreenV2 {
    app_focus: Rc<RefCell<AppFocus>>,
    db_list: DbList,
    coll_list: CollList,
}

impl PrimaryScreenV2 {
    pub fn new(app_focus: Rc<RefCell<AppFocus>>) -> Self {
        Self {
            app_focus,
            ..Default::default()
        }
    }

    /// Narrows the shared `AppFocus` variable into the focus enum for this componenent
    fn internal_focus(&self) -> Option<PrimaryScreenFocus> {
        match &*self.app_focus.borrow() {
            AppFocus::ConnScreen(..) => None,
            AppFocus::PrimaryScreen(focus) => Some(focus.clone()),
        }
    }
}

impl Component<UniqueType> for PrimaryScreenV2 {
    fn commands(&self) -> Vec<CommandGroup> {
        match self.internal_focus() {
            Some(PrimaryScreenFocus::DbList) => self.db_list.commands(),
            Some(PrimaryScreenFocus::CollList) => self.coll_list.commands(),
            None => vec![],
        }
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        match self.internal_focus() {
            Some(PrimaryScreenFocus::DbList) => self.db_list.handle_command(command),
            Some(PrimaryScreenFocus::CollList) => self.coll_list.handle_command(command),
            None => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = vec![];
        out.append(&mut self.db_list.handle_event(event));
        out.append(&mut self.coll_list.handle_event(event));
        out
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Min(20)])
            .split(area);
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

        self.db_list.render(frame, sidebar_top);
        self.coll_list.render(frame, sidebar_btm);
        // FilterInput::render(main_view_top, buf, state);
        // MainView::default().render(main_view_btm, buf, state);
    }

    fn focus(&self) {
        *self.app_focus.borrow_mut() = AppFocus::PrimaryScreen(PrimaryScreenFocus::default());
    }

    fn is_focused(&self) -> bool {
        matches!(*self.app_focus.borrow(), AppFocus::PrimaryScreen(..))
    }
}

#[derive(Debug, Default)]
pub struct PrimaryScreen<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> PrimaryScreen<'a> {
    pub fn handle_event(event: &CrosstermEvent, state: &mut State) -> bool {
        match state.mode {
            Mode::EditingFilter => FilterInput::handle_event(event, state),
            Mode::Navigating => match event {
                CrosstermEvent::Key(key) => match key.code {
                    KeyCode::Char('q') => {
                        state.mode = Mode::Exiting;
                        true
                    }
                    KeyCode::Char('J') => {
                        state.focus = match state.focus {
                            WidgetFocus::DatabaseList => WidgetFocus::CollectionList,
                            WidgetFocus::FilterEditor => WidgetFocus::MainView,
                            m => m,
                        };
                        true
                    }
                    KeyCode::Char('K') => {
                        state.focus = match state.focus {
                            WidgetFocus::CollectionList => WidgetFocus::DatabaseList,
                            WidgetFocus::MainView => WidgetFocus::FilterEditor,
                            m => m,
                        };
                        true
                    }
                    KeyCode::Char('H') => {
                        state.focus = match state.focus {
                            WidgetFocus::MainView => WidgetFocus::CollectionList,
                            WidgetFocus::FilterEditor => WidgetFocus::DatabaseList,
                            m => m,
                        };
                        true
                    }
                    KeyCode::Char('L') => {
                        state.focus = match state.focus {
                            WidgetFocus::CollectionList => WidgetFocus::MainView,
                            WidgetFocus::DatabaseList => WidgetFocus::FilterEditor,
                            m => m,
                        };
                        true
                    }
                    KeyCode::Esc => {
                        state.screen = Screen::Connection;
                        state.mode = Mode::Navigating;
                        state.focus = WidgetFocus::ConnectionList;
                        true
                    }
                    _ => match state.focus {
                        // WidgetFocus::DatabaseList => DbList::handle_event(event, state),
                        // WidgetFocus::CollectionList => CollList::handle_event(event, state),
                        WidgetFocus::MainView => MainView::handle_event(event, state),
                        WidgetFocus::FilterEditor => FilterInput::handle_event(event, state),
                        _ => false,
                    },
                },
                CrosstermEvent::Resize(_, _) => true,
                _ => false,
            },
            _ => false,
        }
    }
}
