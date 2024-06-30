use crate::state::{Mode, State, WidgetFocus};
use crate::widgets::{
    coll_list::CollList, db_list::DbList, filter_input::FilterInput, main_view::MainView,
    status_bar::StatusBar,
};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;

#[derive(Debug, Default)]
pub struct PrimaryScreen<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for PrimaryScreen<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // TODO: change status bar visibility based on whether there's an error?
        let frame_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Length(1)])
            .split(area);
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

        DbList::default().render(sidebar_top, buf, state);
        CollList::default().render(sidebar_btm, buf, state);
        FilterInput::default().render(main_view_top, buf, state);
        MainView::default().render(main_view_btm, buf, state);
        StatusBar::default().render(btm_line, buf, state);
    }
}

impl<'a> PrimaryScreen<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        match state.mode {
            Mode::EditingFilter => FilterInput::handle_event(event, state),
            Mode::Navigating => match event {
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.mode = Mode::Exiting;
                        true
                    }
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
                    _ => match state.focus {
                        WidgetFocus::DatabaseList => DbList::handle_event(event, state),
                        WidgetFocus::CollectionList => CollList::handle_event(event, state),
                        WidgetFocus::MainView => MainView::handle_event(event, state),
                        WidgetFocus::FilterEditor => FilterInput::handle_event(event, state),
                    },
                },
                Event::Resize(_, _) => true,
                _ => false,
            },
            Mode::Exiting => false,
        }
    }
}
