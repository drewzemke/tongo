use crate::state::{Mode, Screen, State, WidgetFocus};
use crate::widgets::input_widget::InputWidget;
use crate::widgets::list_widget::ListWidget;
use crate::widgets::{
    coll_list::CollList, db_list::DbList, filter_input::FilterInput, main_view::MainView,
};
use crossterm::event::{Event, KeyCode};
use ratatui::prelude::*;

#[derive(Debug, Default)]
pub struct PrimaryScreen<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for PrimaryScreen<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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

        DbList::render(sidebar_top, buf, state);
        CollList::render(sidebar_btm, buf, state);
        FilterInput::render(main_view_top, buf, state);
        MainView::default().render(main_view_btm, buf, state);
    }
}

impl<'a> PrimaryScreen<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        match state.mode {
            Mode::EditingFilter => FilterInput::handle_event(event, state),
            Mode::Navigating => match event {
                Event::Key(key) => match key.code {
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
                        WidgetFocus::DatabaseList => DbList::handle_event(event, state),
                        WidgetFocus::CollectionList => CollList::handle_event(event, state),
                        WidgetFocus::MainView => MainView::handle_event(event, state),
                        WidgetFocus::FilterEditor => FilterInput::handle_event(event, state),
                        _ => false,
                    },
                },
                Event::Resize(_, _) => true,
                _ => false,
            },
            _ => false,
        }
    }
}
