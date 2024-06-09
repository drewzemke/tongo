use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::{
    layout::Position,
    prelude::*,
    widgets::{Block, Scrollbar, ScrollbarOrientation},
};
use std::time::{Duration, Instant};
use tui_tree_widget::{Tree, TreeItem, TreeState};

pub struct App<'a> {
    state: TreeState<String>,
    items: Vec<TreeItem<'a, String>>,
}

const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

impl<'a> App<'a> {
    pub fn new(items: Vec<TreeItem<'a, String>>) -> Self {
        // initial state has all top-level documents expanded
        let mut state = TreeState::default();
        for item in &items {
            state.open(vec![item.identifier().clone()]);
        }

        Self { state, items }
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

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        terminal.draw(|frame| self.draw(frame))?;

        let mut debounce: Option<Instant> = None;

        loop {
            let timeout =
                debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));
            if crossterm::event::poll(timeout)? {
                let update = match crossterm::event::read()? {
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
                };
                if update {
                    debounce.get_or_insert_with(Instant::now);
                }
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
