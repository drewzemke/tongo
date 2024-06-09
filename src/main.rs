use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use futures::TryStreamExt;
use mongodb::{bson::Bson, options::ClientOptions, Client};
use ratatui::{
    layout::Position,
    prelude::*,
    widgets::{Block, Scrollbar, ScrollbarOrientation},
};
use std::time::{Duration, Instant};
use tui::{restore_terminal, setup_terminal};
use tui_tree_widget::{Tree, TreeItem, TreeState};

const DB_NAME: &str = "deeb";
const COLLECTION_NAME: &str = "stuff";

mod tree;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    client_options.app_name = Some("Mongo Stuff".to_string());

    let client = Client::with_options(client_options)?;

    let db = client.database(DB_NAME);

    let items: Vec<TreeItem<String>> = db
        .collection::<Bson>(COLLECTION_NAME)
        .find(None, None)
        .await?
        .try_collect::<Vec<Bson>>()
        .await?
        .iter()
        .map(|x| tree::top_level_document(x.as_document().unwrap()))
        .collect();

    let mut terminal = setup_terminal()?;

    // App
    let app = App::new(items);
    let res = run_app(&mut terminal, app);

    restore_terminal(terminal)?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

struct App<'a> {
    state: TreeState<String>,
    items: Vec<TreeItem<'a, String>>,
}

impl<'a> App<'a> {
    fn new(items: Vec<TreeItem<'a, String>>) -> Self {
        Self {
            state: TreeState::default(),
            items,
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.size();
        let widget = Tree::new(&self.items)
            .expect("all item identifiers are unique")
            .block(Block::bordered().title("Collection"))
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
}

const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    terminal.draw(|frame| app.draw(frame))?;

    let mut debounce: Option<Instant> = None;

    loop {
        let timeout = debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));
        if crossterm::event::poll(timeout)? {
            let update = match crossterm::event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(())
                    }
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('\n' | ' ') => app.state.toggle_selected(),
                    KeyCode::Left => app.state.key_left(),
                    KeyCode::Right => app.state.key_right(),
                    KeyCode::Down => app.state.key_down(),
                    KeyCode::Up => app.state.key_up(),
                    KeyCode::Esc => app.state.select(Vec::new()),
                    KeyCode::Home => app.state.select_first(),
                    KeyCode::End => app.state.select_last(),
                    KeyCode::PageDown => app.state.scroll_down(3),
                    KeyCode::PageUp => app.state.scroll_up(3),
                    _ => false,
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollDown => app.state.scroll_down(1),
                    MouseEventKind::ScrollUp => app.state.scroll_up(1),
                    MouseEventKind::Down(_button) => {
                        app.state.click_at(Position::new(mouse.column, mouse.row))
                    }
                    _ => false,
                },
                Event::Resize(_, _) => true,
                _ => false,
            };
            if update {
                debounce.get_or_insert_with(Instant::now);
            }
        }
        if debounce.is_some_and(|debounce| debounce.elapsed() > DEBOUNCE) {
            terminal.draw(|frame| {
                app.draw(frame);
            })?;
            debounce = None;
        }
    }
}
