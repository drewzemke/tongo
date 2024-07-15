#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_name_repetitions)]
// TODO: remove
#![allow(clippy::too_many_lines)]

use std::io;

use crate::{
    state::{State, WidgetFocus},
    tree::MongoKey,
};
use anyhow::Context;
use crossterm::{
    event::{Event, KeyCode, MouseEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use edit::Builder;
use mongodb::bson::{doc, oid::ObjectId, Bson};
use ratatui::{
    layout::Position,
    prelude::*,
    widgets::{Block, Scrollbar, ScrollbarOrientation},
};
use serde_json::Value;
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
                    // TODO: better error handling
                    let id = state.main_view.state.selected().first();
                    let Some(id) = id else { return false };

                    let filter = doc! { "_id" : Bson::from(id)};
                    let doc = state
                        .main_view
                        .items
                        .iter()
                        .position(|tree_item| tree_item.identifier() == id)
                        .and_then(|index| state.main_view.documents.get(index));
                    let Some(doc) = doc else { return false };

                    //////
                    let doc_string = mongodb::bson::from_bson::<Value>(doc.clone())
                        .context("converting doc to json")
                        .and_then(|json| {
                            serde_json::to_string_pretty(&json).context("converting json to string")
                        });
                    let Ok(doc_string) = doc_string else {
                        return false;
                    };

                    io::stdout()
                        .execute(LeaveAlternateScreen)
                        .expect("prep terminal");
                    let updated_string =
                        edit::edit_with_builder(doc_string, Builder::new().suffix(".json"))
                            .context("editing string");
                    io::stdout()
                        .execute(EnterAlternateScreen)
                        .expect("reset terminal");
                    state.clear_screen = true;

                    let new_doc = updated_string
                        .and_then(|s| {
                            serde_json::from_str::<serde_json::Value>(&s)
                                .context("converting string to json")
                        })
                        .and_then(|value| {
                            mongodb::bson::to_document(&value).context("converting json to doc")
                        });
                    let Ok(new_doc) = new_doc else {
                        return false;
                    };
                    //////

                    let update = doc! { "$set": new_doc };

                    state.exec_update_one(filter, update);
                    false
                }

                // duplicate current doc
                KeyCode::Char('C') => {
                    // TODO: better error handling
                    let id = state.main_view.state.selected().first();
                    let Some(id) = id else { return false };

                    let doc = state
                        .main_view
                        .items
                        .iter()
                        .position(|tree_item| tree_item.identifier() == id)
                        .and_then(|index| state.main_view.documents.get(index));
                    let Some(Bson::Document(doc)) = doc else {
                        return false;
                    };

                    let mut duplicated_doc = doc.clone();
                    let _ = duplicated_doc.insert("_id", ObjectId::new());
                    let new_doc = Bson::Document(duplicated_doc);

                    //////
                    let doc_string = mongodb::bson::from_bson::<Value>(new_doc)
                        .context("converting doc to json")
                        .and_then(|json| {
                            serde_json::to_string_pretty(&json).context("converting json to string")
                        });
                    let Ok(doc_string) = doc_string else {
                        return false;
                    };

                    io::stdout()
                        .execute(LeaveAlternateScreen)
                        .expect("prep terminal");
                    let updated_string =
                        edit::edit_with_builder(doc_string, Builder::new().suffix(".json"))
                            .context("editing string");
                    io::stdout()
                        .execute(EnterAlternateScreen)
                        .expect("reset terminal");
                    state.clear_screen = true;

                    let new_doc = updated_string
                        .and_then(|s| {
                            serde_json::from_str::<serde_json::Value>(&s)
                                .context("converting string to json")
                        })
                        .and_then(|value| {
                            mongodb::bson::to_document(&value).context("converting json to doc")
                        });
                    let Ok(new_doc) = new_doc else {
                        return false;
                    };
                    //////

                    state.exec_insert_one(new_doc);
                    false
                }

                // insert new doc
                KeyCode::Char('I') => {
                    // TODO: better error handling
                    let doc = doc! { "_id" : ObjectId::new() };

                    //////
                    let doc_string = mongodb::bson::from_document::<Value>(doc)
                        .context("converting doc to json")
                        .and_then(|json| {
                            serde_json::to_string_pretty(&json).context("converting json to string")
                        });
                    let Ok(doc_string) = doc_string else {
                        return false;
                    };

                    io::stdout()
                        .execute(LeaveAlternateScreen)
                        .expect("prep terminal");
                    let updated_string =
                        edit::edit_with_builder(doc_string, Builder::new().suffix(".json"))
                            .context("editing string");
                    io::stdout()
                        .execute(EnterAlternateScreen)
                        .expect("reset terminal");
                    state.clear_screen = true;

                    let new_doc = updated_string
                        .and_then(|s| {
                            serde_json::from_str::<serde_json::Value>(&s)
                                .context("converting string to json")
                        })
                        .and_then(|value| {
                            mongodb::bson::to_document(&value).context("converting json to doc")
                        });
                    let Ok(new_doc) = new_doc else {
                        return false;
                    };
                    //////

                    state.exec_insert_one(new_doc);
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
