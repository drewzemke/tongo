use std::{cell::Cell, rc::Rc};

use crate::{
    components::{
        tab::{CloneWithFocus, TabFocus},
        Component,
    },
    config::{color_map::ColorKey, Config},
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        message::{AppAction, Message},
        signal::SignalQueue,
    },
    utils::list_searcher::ListSearcher,
};
use crossterm::event::Event as CrosstermEvent;
use ratatui::{
    prelude::*,
    style::Style,
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};
use tui_input::{backend::crossterm::EventHandler as _, Input};

pub mod collections;
pub mod connections;
pub mod databases;

/// A trait for datatypes used in the generic list.
pub trait InnerListItem: Default + Clone + PartialEq {
    fn to_list_item(&self, config: &Config) -> ListItem<'_>;
    fn to_search_string(&self) -> String;
}

#[derive(Debug, Default, Clone)]
enum Mode {
    #[default]
    Normal,
    SearchInput,
}

#[derive(Debug, Default, Clone)]
pub struct InnerList<T> {
    title: &'static str,
    config: Config,

    state: ListState,
    items: Vec<T>,

    focus: Rc<Cell<TabFocus>>,
    focused_when: TabFocus,

    mode: Mode,
    search_input: Input,
    searcher: ListSearcher,
    visible_indices: Vec<usize>,
    cursor_pos: Rc<Cell<(u16, u16)>>,
}

impl<T> InnerList<T>
where
    T: InnerListItem,
{
    pub fn new(
        title: &'static str,
        focus: Rc<Cell<TabFocus>>,
        focused_when: TabFocus,
        cursor_pos: Rc<Cell<(u16, u16)>>,
        config: Config,
    ) -> Self {
        Self {
            title,
            config,
            focus,
            focused_when,
            cursor_pos,
            ..Default::default()
        }
    }

    pub fn set_items(&mut self, items: &[T])
    where
        T: Clone,
    {
        // cache the currently-selected item so we can potentially reselect
        // it after updating
        let selected = self.selected().cloned();

        // update
        self.items = items.to_vec();
        self.filter_items_by_search();

        // restore selection
        self.select(selected);
    }

    pub fn all_items(&self) -> &[T] {
        &self.items
    }

    /// computes which items are visible based on the search filter and stores
    /// the indices of the visible items in `visible_indices`
    ///
    /// returns a reference to the best match (if there is one)
    fn filter_items_by_search(&mut self) -> Option<&T> {
        let query = self.search_input.value();
        let matches = self.searcher.matches(&self.items, query);

        let mut best_idx = None;
        let mut best_score = 0;
        self.visible_indices = matches
            .map(|(idx, score)| {
                if score > best_score {
                    best_idx = Some(idx);
                    best_score = score;
                }
                idx
            })
            .collect();

        best_idx.and_then(|idx| self.items.get(idx))
    }

    fn reset_search(&mut self) {
        let selected = self.selected().cloned();

        self.search_input = Input::default();
        self.filter_items_by_search();

        self.select(selected);
    }

    pub fn select(&mut self, item: Option<T>)
    where
        T: PartialEq,
    {
        if let Some(item) = item {
            let index = self
                .visible_indices
                .iter()
                .position(|&idx| self.items.get(idx).is_some_and(|it| *it == item));
            self.state.select(index);
        } else {
            self.state.select(None);
        }
    }

    pub fn selected(&self) -> Option<&T> {
        self.state
            .selected()
            .and_then(|index| self.visible_indices.get(index))
            .and_then(|&index| self.items.get(index))
    }
}

impl<T> CloneWithFocus for InnerList<T>
where
    T: InnerListItem,
{
    fn clone_with_focus(&self, focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            focus,
            ..self.clone()
        }
    }
}

impl<T> Component for InnerList<T>
where
    T: InnerListItem,
{
    fn commands(&self) -> Vec<CommandGroup> {
        // handle search input mode separately
        if matches!(self.mode, Mode::SearchInput) {
            return vec![
                CommandGroup::new(vec![Command::Confirm], "end search")
                    .in_cat(CommandCategory::StatusBarOnly),
                CommandGroup::new(vec![Command::Back], "cancel")
                    .in_cat(CommandCategory::StatusBarOnly),
            ];
        }

        let mut out = vec![
            CommandGroup::new(vec![Command::NavUp, Command::NavDown], "navigate")
                .in_cat(CommandCategory::AppNav),
            CommandGroup::new(vec![Command::Search], "fuzzy filter")
                .in_cat(CommandCategory::AppNav),
        ];

        if !self.search_input.value().is_empty() {
            out.push(
                CommandGroup::new(vec![Command::Reset], "reset filter")
                    .in_cat(CommandCategory::AppNav),
            );
        }

        out
    }

    fn handle_command(&mut self, command: &Command, queue: &mut SignalQueue) {
        let num_items = self.visible_indices.len();

        match command {
            Command::NavUp => {
                // jump to the bottom if we're at the top
                if self.state.selected().is_none() || self.state.selected() == Some(0) {
                    self.state.select(Some(num_items.saturating_sub(1)));
                } else {
                    self.state.select_previous();
                }
                queue.push(Event::ListSelectionChanged);
            }
            Command::NavDown => {
                // jump to the top if we're at the bottom
                if self.state.selected().is_none()
                    || self.state.selected() == Some(num_items.saturating_sub(1))
                {
                    self.state.select_first();
                } else {
                    self.state.select_next();
                }
                queue.push(Event::ListSelectionChanged);
            }

            Command::Search => {
                if matches!(self.mode, Mode::Normal) {
                    self.mode = Mode::SearchInput;
                    queue.push(Message::to_app(AppAction::EnterRawMode));
                }
            }

            // only in search input mode
            Command::Confirm => {
                if matches!(self.mode, Mode::SearchInput) {
                    self.mode = Mode::Normal;
                    queue.push(Message::to_app(AppAction::ExitRawMode));
                }
            }

            // only in search input mode
            Command::Back => {
                if matches!(self.mode, Mode::SearchInput) {
                    self.mode = Mode::Normal;
                    self.reset_search();
                    queue.push(Message::to_app(AppAction::ExitRawMode));
                }
            }

            Command::Reset => {
                self.reset_search();
                queue.push(Event::SearchUpdated);
            }
            _ => {}
        }
    }

    fn handle_raw_event(&mut self, event: &CrosstermEvent, queue: &mut SignalQueue) {
        if matches!(self.mode, Mode::SearchInput) {
            self.search_input.handle_event(event);
            let best = self.filter_items_by_search().cloned();
            self.select(best);
            queue.push(Event::SearchUpdated);
            queue.push(Event::ListSelectionChanged);
        }
    }

    fn focus(&self) {
        self.focus.set(self.focused_when);
    }

    fn is_focused(&self) -> bool {
        self.focus.get() == self.focused_when
    }

    fn is_capturing_input(&self) -> bool {
        matches!(self.mode, Mode::SearchInput)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let (border_color, bg_color, highlight_text, highlight_bg, unselected_fg) =
            if self.is_focused() {
                (
                    self.config.color_map.get(&ColorKey::PanelActiveBorder),
                    self.config.color_map.get(&ColorKey::PanelActiveBg),
                    self.config.color_map.get(&ColorKey::SelectionFg),
                    self.config.color_map.get(&ColorKey::SelectionBg),
                    self.config.color_map.get(&ColorKey::FgPrimary),
                )
            } else {
                (
                    self.config.color_map.get(&ColorKey::PanelInactiveBorder),
                    self.config.color_map.get(&ColorKey::PanelInactiveBg),
                    self.config.color_map.get(&ColorKey::FgPrimary),
                    self.config.color_map.get(&ColorKey::PanelInactiveBg),
                    self.config.color_map.get(&ColorKey::FgSecondary),
                )
            };

        let is_editing = matches!(self.mode, Mode::SearchInput);
        let has_search = !self.search_input.value().is_empty() || is_editing;

        let mut block = Block::bordered()
            .bg(bg_color)
            .title(format!(" {} ", self.title))
            .border_style(Style::default().fg(border_color));

        if has_search {
            block = block.title_bottom(
                Line::from(format!(" /{} ", self.search_input.value()))
                    .left_aligned()
                    .fg(self.config.color_map.get(&ColorKey::DocumentsSearch)),
            );
        }

        let visible_items = self.visible_indices.iter().map(|&i| &self.items[i]);

        let list = List::new(visible_items.map(|item| item.to_list_item(&self.config)))
            .block(block)
            .style(Style::default().fg(unselected_fg))
            .highlight_style(Style::default().bold().fg(highlight_text).bg(highlight_bg));

        StatefulWidget::render(list, area, frame.buffer_mut(), &mut self.state);

        // update cursor position
        #[expect(clippy::cast_possible_truncation)]
        if is_editing {
            let cursor_pos = (
                area.x + self.search_input.visual_cursor() as u16 + 3,
                area.y + area.height - 1,
            );
            self.cursor_pos.set(cursor_pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::primary_screen::PrimScrFocus, testing::ComponentTestHarness};

    impl<T> InnerList<T>
    where
        T: InnerListItem + Default + Clone,
    {
        fn with_items(items: &[T]) -> Self {
            let mut comp = Self::default();
            comp.set_items(items);
            comp
        }
    }

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    struct TestItem(u32);

    impl InnerListItem for TestItem {
        fn to_list_item(&self, _config: &Config) -> ListItem<'_> {
            ListItem::new(self.to_search_string())
        }

        fn to_search_string(&self) -> String {
            self.0.to_string()
        }
    }

    #[test]
    fn test_select_with_nothing_selected() {
        // down selects first item
        let component = InnerList::<TestItem>::with_items(&[TestItem(1), TestItem(2), TestItem(3)]);
        let mut test = ComponentTestHarness::new(component);

        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected(), Some(&TestItem(1)));

        // up selects last item
        let component = InnerList::<TestItem>::with_items(&[TestItem(1), TestItem(2), TestItem(3)]);
        let mut test = ComponentTestHarness::new(component);

        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected(), Some(&TestItem(3)));
    }

    #[test]
    fn test_select_an_item() {
        let component = InnerList::<TestItem>::with_items(&[TestItem(1), TestItem(2), TestItem(3)]);
        let mut test = ComponentTestHarness::new(component);

        // initialize selection
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected(), Some(&TestItem(1)));

        // select second item
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected(), Some(&TestItem(2)));

        test.expect_event(|e| matches!(e, Event::ListSelectionChanged));
    }

    #[test]
    fn test_selection_wraps_around_list() {
        let component = InnerList::<TestItem>::with_items(&[TestItem(1), TestItem(2), TestItem(3)]);
        let mut test = ComponentTestHarness::new(component);

        // initialize selection
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected(), Some(&TestItem(1)));

        // up to wrap around
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected(), Some(&TestItem(3)));

        // down to wrap back
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected(), Some(&TestItem(1)));
    }

    #[test]
    fn test_handles_empty_item_list() {
        let component = InnerList::<TestItem>::with_items(&[]);
        let mut test = ComponentTestHarness::new(component);

        // navigating doesn't panic and doesn't select anything
        test.given_command(Command::NavUp);
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected(), None);
    }

    #[test]
    fn test_select_sets_selection() {
        let mut component =
            InnerList::<TestItem>::with_items(&[TestItem(1), TestItem(2), TestItem(3)]);

        // select an item in the list
        component.select(Some(TestItem(2)));
        assert_eq!(component.selected(), Some(&TestItem(2)));

        // selecting an item not in the list should clear the selection
        component.select(Some(TestItem(4)));
        assert_eq!(component.selected(), None);

        component.select(Some(TestItem(3)));
        assert_eq!(component.selected(), Some(&TestItem(3)));

        // selecting None should also clear the selection
        component.select(None);
        assert_eq!(component.selected(), None);
    }

    #[test]
    fn test_set_items_updates_items() {
        let mut component = InnerList::<TestItem>::with_items(&[]);

        component.set_items(&[TestItem(1), TestItem(2), TestItem(3)]);
        assert_eq!(
            component.all_items(),
            &[TestItem(1), TestItem(2), TestItem(3)]
        );

        component.set_items(&[TestItem(4)]);
        assert_eq!(component.all_items(), &[TestItem(4)]);
    }

    #[test]
    fn test_set_items_keeps_selection() {
        let mut component =
            InnerList::<TestItem>::with_items(&[TestItem(1), TestItem(2), TestItem(3)]);

        component.select(Some(TestItem(2)));
        assert_eq!(component.selected(), Some(&TestItem(2)));

        component.set_items(&[TestItem(1), TestItem(2), TestItem(3), TestItem(4)]);
        assert_eq!(component.selected(), Some(&TestItem(2)));

        component.set_items(&[TestItem(2), TestItem(3), TestItem(4)]);
        assert_eq!(component.selected(), Some(&TestItem(2)));

        component.set_items(&[TestItem(3), TestItem(4)]);
        assert_eq!(component.selected(), None);
    }

    #[test]
    fn test_focus_follows_shared_cell() {
        let focus = Rc::new(Cell::new(TabFocus::default()));
        let db_focus = TabFocus::PrimScr(PrimScrFocus::DbList);
        let component = InnerList::<TestItem>::new(
            "Test",
            focus.clone(),
            db_focus,
            Rc::default(),
            Config::default(),
        );
        assert!(!component.is_focused());

        component.focus();
        assert_eq!(focus.get(), db_focus);
        assert!(component.is_focused());

        // something else takes focus
        focus.set(TabFocus::PrimScr(PrimScrFocus::CollList));
        assert!(!component.is_focused());
    }

    #[test]
    fn test_clone_with_focus_uses_new_cell() {
        let db_focus = TabFocus::PrimScr(PrimScrFocus::DbList);
        let original_focus = Rc::new(Cell::new(TabFocus::default()));
        let original = InnerList::<TestItem>::new(
            "Test",
            original_focus.clone(),
            db_focus,
            Rc::default(),
            Config::default(),
        );

        let cloned_focus = Rc::new(Cell::new(TabFocus::default()));
        let cloned = original.clone_with_focus(cloned_focus);

        cloned.focus();
        assert!(cloned.is_focused());
        assert!(!original.is_focused());
        assert_eq!(original_focus.get(), TabFocus::default());
    }

    #[test]
    fn test_search_filters_items() {
        let component =
            InnerList::<TestItem>::with_items(&[TestItem(12), TestItem(22), TestItem(31)]);
        let mut test = ComponentTestHarness::new(component);

        test.given_command(Command::Search);
        test.expect_message(|m| matches!(m.read_as_app(), Some(AppAction::EnterRawMode)));

        test.given_string("2");
        test.expect_event(|e| matches!(e, Event::SearchUpdated));

        test.given_command(Command::Confirm);
        test.expect_message(|m| matches!(m.read_as_app(), Some(AppAction::ExitRawMode)));

        assert_eq!(test.component().visible_indices, vec![0, 1]);
    }

    fn searchable_list() -> ComponentTestHarness<InnerList<TestItem>> {
        let component = InnerList::<TestItem>::with_items(&[
            TestItem(123),
            TestItem(2),
            TestItem(321),
            TestItem(11),
            TestItem(13),
        ]);
        ComponentTestHarness::new(component)
    }

    fn search_for(test: &mut ComponentTestHarness<InnerList<TestItem>>, query: &str) {
        test.given_command(Command::Search);
        test.given_string(query);
    }

    #[test]
    fn test_search_selects_best_match() {
        let mut test = searchable_list();

        search_for(&mut test, "2");
        // "2" is a weaker match in 123 and 321
        assert_eq!(test.component().selected(), Some(&TestItem(2)));
    }

    #[test]
    fn test_search_emits_selection_changed() {
        let mut test = searchable_list();

        search_for(&mut test, "2");
        test.expect_event(|e| matches!(e, Event::ListSelectionChanged));
    }

    #[test]
    fn test_navigation_stays_within_filtered_items() {
        let mut test = searchable_list();

        // visible: 123, 13 (best match)
        search_for(&mut test, "13");
        test.given_command(Command::Confirm);
        assert_eq!(test.component().selected(), Some(&TestItem(13)));

        // wraps within the two visible items
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected(), Some(&TestItem(123)));
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected(), Some(&TestItem(13)));
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected(), Some(&TestItem(123)));
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected(), Some(&TestItem(13)));
    }

    #[test]
    fn test_search_with_no_matches() {
        let mut test = searchable_list();

        search_for(&mut test, "9");
        assert_eq!(test.component().selected(), None);

        test.given_command(Command::Confirm);
        test.given_command(Command::NavDown);
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected(), None);
    }

    #[test]
    fn test_confirm_keeps_filter_and_selection() {
        let mut test = searchable_list();

        search_for(&mut test, "13");
        test.given_command(Command::Confirm);

        assert_eq!(test.component().selected(), Some(&TestItem(13)));
        assert_eq!(test.component().visible_indices, vec![0, 4]);
    }

    #[test]
    fn test_cancel_clears_filter_and_keeps_selection() {
        let mut test = searchable_list();

        search_for(&mut test, "13");
        test.given_command(Command::Back);

        assert_eq!(test.component().selected(), Some(&TestItem(13)));
        assert_eq!(test.component().visible_indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_set_items_while_filtered_keeps_filter_and_selection() {
        let mut test = searchable_list();

        search_for(&mut test, "13");
        test.given_command(Command::Confirm);

        test.component_mut()
            .set_items(&[TestItem(13), TestItem(2), TestItem(123), TestItem(4)]);

        assert_eq!(test.component().selected(), Some(&TestItem(13)));
        assert_eq!(test.component().visible_indices, vec![0, 2]);
    }

    #[test]
    fn test_selecting_hidden_item_clears_selection() {
        let mut test = searchable_list();

        search_for(&mut test, "13");
        test.given_command(Command::Confirm);

        // 2 exists but is filtered out
        test.component_mut().select(Some(TestItem(2)));
        assert_eq!(test.component().selected(), None);
    }
}
