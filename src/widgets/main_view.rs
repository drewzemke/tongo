#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_name_repetitions)]
// TODO: remove
#![allow(clippy::too_many_lines)]

use crate::{
    edit_doc::edit_doc,
    state::{State, WidgetFocus},
    tree::MongoKey,
};
use crossterm::event::{Event, KeyCode, MouseEventKind};
use mongodb::bson::{doc, oid::ObjectId, Bson, Document};
use ratatui::{
    layout::Position,
    prelude::*,
    widgets::{Block, Scrollbar, ScrollbarOrientation},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

const PAGE_SIZE: usize = 5;

#[derive(Debug, Default)]
pub struct MainViewState<'a> {
    pub state: TreeState<MongoKey>,
    pub items: Vec<TreeItem<'a, MongoKey>>,
    pub documents: Vec<Bson>,
    pub page: usize,
    pub count: u64,
}

impl<'a> MainViewState<'a> {
    pub fn selected_doc(&self) -> Option<&Document> {
        let id = self.state.selected().first()?;

        let bson = self
            .items
            .iter()
            .position(|tree_item| tree_item.identifier() == id)
            .and_then(|index| self.documents.get(index));

        if let Some(Bson::Document(doc)) = bson {
            Some(doc)
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
pub struct MainView<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for MainView<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let focused = state.focus == WidgetFocus::MainView;
        let border_color = if focused { Color::Green } else { Color::White };

        let start = state.main_view.page * PAGE_SIZE + 1;
        let end = (start + PAGE_SIZE - 1).min(state.main_view.count as usize);

        let title = format!("Documents ({start}-{end} of {})", state.main_view.count);

        let widget = Tree::new(&state.main_view.items)
            .expect("all item identifiers are unique")
            .block(
                Block::bordered()
                    .title(title)
                    .border_style(Style::default().fg(border_color)),
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ))
            .highlight_style(Style::new().black().on_white().bold());

        StatefulWidget::render(widget, area, buf, &mut state.main_view.state);
    }
}

impl<'a> MainView<'a> {
    pub fn handle_event(event: &Event, state: &mut State) -> bool {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char(' ') | KeyCode::Enter => state.main_view.state.toggle_selected(),
                KeyCode::Left | KeyCode::Char('h') => state.main_view.state.key_left(),
                KeyCode::Right | KeyCode::Char('l') => state.main_view.state.key_right(),
                KeyCode::Down | KeyCode::Char('j') => state.main_view.state.key_down(),
                KeyCode::Up | KeyCode::Char('k') => state.main_view.state.key_up(),
                KeyCode::Home => state.main_view.state.select_first(),
                KeyCode::End => state.main_view.state.select_last(),
                KeyCode::PageDown => state.main_view.state.scroll_down(3),
                KeyCode::PageUp => state.main_view.state.scroll_up(3),
                // next page
                KeyCode::Char('n') => {
                    let end = (state.main_view.page + 1) * PAGE_SIZE;
                    if end < state.main_view.count as usize {
                        state.main_view.page += 1;
                        state.exec_query(true, false);
                        true
                    } else {
                        false
                    }
                }
                // previous page
                KeyCode::Char('p') => {
                    if state.main_view.page > 0 {
                        state.main_view.page -= 1;
                        state.exec_query(true, false);
                        true
                    } else {
                        false
                    }
                }
                // refresh
                KeyCode::Char('r') => {
                    state.exec_query(false, false);
                    state.exec_count();
                    false
                }

                // delete current doc
                KeyCode::Char('D') => {
                    let filter = state
                        .main_view
                        .state
                        .selected()
                        .first()
                        .map(|id| doc! { "_id" : Bson::from(id)});
                    if let Some(filter) = filter {
                        state.exec_delete_one(filter);
                    }
                    false
                }

                // edit current doc
                KeyCode::Char('E') => {
                    let id = state.main_view.state.selected().first();
                    let Some(id) = id else { return false };

                    let Some(doc) = state.main_view.selected_doc() else {
                        return false;
                    };

                    let Ok(new_doc) = edit_doc(doc.clone()) else {
                        return false;
                    };

                    let filter = doc! { "_id" : Bson::from(id)};
                    let update = doc! { "$set": new_doc };

                    state.exec_update_one(filter, update);
                    state.clear_screen = true;
                    false
                }

                // duplicate current doc
                KeyCode::Char('C') => {
                    let Some(doc) = state.main_view.selected_doc() else {
                        return false;
                    };

                    let mut duplicated_doc = doc.clone();
                    let _ = duplicated_doc.insert("_id", ObjectId::new());

                    let Ok(new_doc) = edit_doc(duplicated_doc) else {
                        return false;
                    };

                    state.exec_insert_one(new_doc);
                    state.clear_screen = true;
                    false
                }

                // insert new doc
                KeyCode::Char('I') => {
                    let doc = doc! { "_id" : ObjectId::new() };

                    let Ok(new_doc) = edit_doc(doc) else {
                        return false;
                    };

                    state.exec_insert_one(new_doc);
                    state.clear_screen = true;
                    false
                }
                _ => false,
            },

            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => state.main_view.state.scroll_down(1),
                MouseEventKind::ScrollUp => state.main_view.state.scroll_up(1),
                MouseEventKind::Down(_) => state
                    .main_view
                    .state
                    .click_at(Position::new(mouse.column, mouse.row)),
                _ => false,
            },
            Event::Resize(_, _) => true,
            _ => false,
        }
    }
}
