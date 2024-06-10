#![allow(clippy::cast_possible_wrap)]

use crate::tree::top_level_document;
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use futures::TryStreamExt;
use mongodb::{bson::Bson, options::FindOptions, Database};
use ratatui::{
    layout::Position,
    prelude::*,
    widgets::{Block, Scrollbar, ScrollbarOrientation},
};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

const PAGE_SIZE: usize = 5;

pub struct App<'a> {
    state: TreeState<String>,
    items: Vec<TreeItem<'a, String>>,

    collection_name: String,
    db: Database,

    page: usize,

    query_send: Sender<Vec<Bson>>,
    query_recv: Receiver<Vec<Bson>>,
}

const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

impl<'a> App<'a> {
    pub fn new(db: Database, collection_name: String) -> Self {
        let (query_send, query_recv) = mpsc::channel::<Vec<Bson>>();

        Self {
            state: TreeState::default(),
            items: vec![],
            collection_name,
            db,
            page: 0,
            query_send,
            query_recv,
        }
    }

    fn exec_query(&self) {
        let sender = self.query_send.clone();
        let db = self.db.clone();
        let collection_name = self.collection_name.clone();

        let skip = self.page * PAGE_SIZE;
        let mut options = FindOptions::default();
        options.skip = Some(skip as u64);

        options.limit = Some(PAGE_SIZE as i64);

        tokio::spawn(async move {
            let result: Vec<Bson> = db
                .collection::<Bson>(&collection_name)
                .find(None, options)
                .await
                .unwrap()
                .try_collect::<Vec<Bson>>()
                .await
                .unwrap();

            // FIXME: Need a way (maybe another channel) to communicate to the UI
            // that the sync failed
            sender
                .send(result)
                .expect("Error occurred while processing server response.");
        });
    }

    fn update_content(&mut self, content: &[Bson]) {
        let items: Vec<TreeItem<String>> = content
            .iter()
            .map(|x| top_level_document(x.as_document().unwrap()))
            .collect();

        // initial state has all top-level documents expanded
        let mut state = TreeState::default();
        for item in &items {
            state.open(vec![item.identifier().clone()]);
        }

        self.items = items;
        self.state = state;
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.size();
        let widget = Tree::new(&self.items)
            .expect("all item identifiers are unique")
            .block(Block::bordered().title(self.collection_name.clone()))
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ))
            .highlight_style(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_stateful_widget(widget, area, &mut self.state);
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        // initial query and draw call
        self.exec_query();
        terminal.draw(|frame| self.draw(frame))?;

        let mut debounce: Option<Instant> = None;

        loop {
            let timeout =
                debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));
            let mut update = if crossterm::event::poll(timeout)? {
                match crossterm::event::read()? {
                    Event::Key(key) => match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(())
                        }
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('\n' | ' ') => self.state.toggle_selected(),
                        KeyCode::Left => self.state.key_left(),
                        KeyCode::Right => self.state.key_right(),
                        KeyCode::Down => self.state.key_down(),
                        KeyCode::Up => self.state.key_up(),
                        KeyCode::Esc => self.state.select(Vec::new()),
                        KeyCode::Home => self.state.select_first(),
                        KeyCode::End => self.state.select_last(),
                        KeyCode::PageDown => self.state.scroll_down(3),
                        KeyCode::PageUp => self.state.scroll_up(3),
                        // next page
                        KeyCode::Char('n') => {
                            self.page += 1;
                            self.exec_query();
                            true
                        }
                        // previous page
                        KeyCode::Char('p') => {
                            if self.page > 0 {
                                self.page -= 1;
                                self.exec_query();
                                true
                            } else {
                                false
                            }
                        }
                        // refresh
                        KeyCode::Char('r') => {
                            self.exec_query();
                            false
                        }
                        _ => false,
                    },
                    Event::Mouse(mouse) => match mouse.kind {
                        MouseEventKind::ScrollDown => self.state.scroll_down(1),
                        MouseEventKind::ScrollUp => self.state.scroll_up(1),
                        MouseEventKind::Down(_button) => {
                            self.state.click_at(Position::new(mouse.column, mouse.row))
                        }
                        _ => false,
                    },
                    Event::Resize(_, _) => true,
                    _ => false,
                }
            } else {
                false
            };

            if let Ok(content) = self.query_recv.try_recv() {
                update = true;
                self.update_content(&content);
            }
            if update {
                debounce.get_or_insert_with(Instant::now);
            }
            if debounce.is_some_and(|debounce| debounce.elapsed() > DEBOUNCE) {
                terminal.draw(|frame| {
                    self.draw(frame);
                })?;
                debounce = None;
            }
        }
    }
}
