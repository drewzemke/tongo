use super::{Component, ComponentCommand};
use crate::system::{
    command::{Command, CommandGroup},
    event::Event,
};
use ratatui::{prelude::*, widgets::Tabs};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

#[derive(Debug, Default)]
pub struct TabBar {
    tabs: Vec<()>,
    current_tab_idx: usize,

    scroll_state: ScrollViewState,
    // set after the first render
    visible_width: Option<u16>,
}

const TAB_NAME_SPACING: usize = 3;

impl TabBar {
    pub fn new() -> Self {
        Self {
            tabs: vec![()],
            current_tab_idx: 0,
            scroll_state: ScrollViewState::new(),
            visible_width: None,
        }
    }

    pub fn current_tab_idx(&self) -> usize {
        self.current_tab_idx
    }

    // HACK: instead of doing this, have this component implement PersistedComponent
    pub(crate) fn set_current_tab(&mut self, index: usize) {
        self.current_tab_idx = index;
    }

    pub fn num_tabs(&self) -> usize {
        self.tabs.len()
    }

    fn tab_names(&mut self) -> impl Iterator<Item = String> + Clone {
        (1..self.tabs.len() + 1).map(|i| format!("[Tab {i}]"))
    }

    fn next_tab(&mut self) {
        self.current_tab_idx = (self.current_tab_idx + 1) % self.tabs.len();
    }

    fn prev_tab(&mut self) {
        self.current_tab_idx = (self.current_tab_idx + self.tabs.len() - 1) % self.tabs.len();
    }

    /// scrolls the view so that the currently-selected tab is visible
    fn scroll_to_current_tab(&mut self) {
        // calculate the positions (relative to the left edge) of the first and last character
        // of this tab's name in the view
        let left_char_pos = self
            .tab_names()
            .take(self.current_tab_idx)
            .map(|s| s.len() + TAB_NAME_SPACING)
            .sum::<usize>() as u16;

        let right_char_pos = left_char_pos
            + self
                .tab_names()
                .nth(self.current_tab_idx)
                .unwrap_or_default()
                .len() as u16;

        let visible_width = self.visible_width.unwrap_or_default();

        let mut offset = self.scroll_state.offset().x;
        // make sure the right side of the name is visible
        offset = offset.max(right_char_pos.saturating_sub(visible_width) + 2);
        // make sure the left side of the name is visible
        offset = offset.min(left_char_pos);

        self.scroll_state.set_offset(Position::new(offset, 0));
    }
}

impl Component for TabBar {
    fn focus(&self) {
        // do nothing
    }

    fn is_focused(&self) -> bool {
        false
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::NewTab], "new tab"),
            CommandGroup::new(vec![Command::NextTab, Command::PreviousTab], "change tab"),
        ]
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Event> {
        if let ComponentCommand::Command(command) = command {
            match command {
                Command::NewTab => {
                    self.tabs.push(());
                    self.current_tab_idx = self.tabs.len() - 1;
                    self.scroll_to_current_tab();
                    return vec![Event::TabCreated];
                }
                Command::NextTab => {
                    self.next_tab();
                    self.scroll_to_current_tab();
                    return vec![Event::TabChanged];
                }
                Command::PreviousTab => {
                    self.prev_tab();
                    self.scroll_to_current_tab();
                    return vec![Event::TabChanged];
                }
                _ => {}
            }
        }

        vec![]
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.visible_width = Some(area.width);

        let tab_names = self.tab_names();

        let content_width = tab_names.clone().map(|s| s.len() + 3).sum::<usize>() - 2;
        let mut scroll_view = ScrollView::new(Size::new(content_width as u16, 1))
            .scrollbars_visibility(ScrollbarVisibility::Never);

        let tabs = Tabs::new(tab_names)
            .style(Style::default().gray())
            .highlight_style(Style::default().green())
            .divider(symbols::border::PLAIN.vertical_left)
            .select(self.current_tab_idx);
        scroll_view.render_widget(tabs, scroll_view.buf().area);

        frame.render_stateful_widget(scroll_view, area, &mut self.scroll_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::ComponentTestHarness;

    #[test]
    fn create_new_tab() {
        let mut test = ComponentTestHarness::new(TabBar::new());

        assert_eq!(test.component().current_tab_idx(), 0);

        test.given_command(Command::NewTab);
        test.expect_event(|e| matches!(e, Event::TabCreated));

        // after creating a new tab, it should be selected
        assert_eq!(test.component().current_tab_idx(), 1);
    }

    #[test]
    fn navigate_between_tabs() {
        let mut test = ComponentTestHarness::new(TabBar::new());

        // create two additional tabs
        test.given_command(Command::NewTab);
        test.given_command(Command::NewTab);

        // we should now be on the last tab (index 2)
        assert_eq!(test.component().current_tab_idx(), 2);

        // test moving forward
        test.given_command(Command::NextTab);
        test.expect_event(|e| matches!(e, Event::TabChanged));
        assert_eq!(test.component().current_tab_idx(), 0);

        // test moving backward
        test.given_command(Command::PreviousTab);
        test.expect_event(|e| matches!(e, Event::TabChanged));
        assert_eq!(test.component().current_tab_idx(), 2);
    }

    #[test]
    fn cycle_through_tabs() {
        let mut test = ComponentTestHarness::new(TabBar::new());

        // starting with one tab
        assert_eq!(test.component().current_tab_idx(), 0);

        // moving forward should wrap around to the beginning
        test.given_command(Command::NextTab);
        test.expect_event(|e| matches!(e, Event::TabChanged));
        assert_eq!(test.component().current_tab_idx(), 0);

        // moving backward should wrap around to the end
        test.given_command(Command::PreviousTab);
        test.expect_event(|e| matches!(e, Event::TabChanged));
        assert_eq!(test.component().current_tab_idx(), 0);
    }
}
