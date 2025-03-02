use super::{Component, ComponentCommand};
use crate::system::{
    command::{Command, CommandGroup},
    event::Event,
};
use ratatui::{prelude::*, widgets::Tabs};

#[derive(Debug, Default)]
pub struct TabBar {
    tabs: Vec<()>,
    current_tab_idx: usize,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            tabs: vec![()],
            current_tab_idx: 0,
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
                    return vec![Event::TabCreated];
                }
                Command::NextTab => {
                    self.current_tab_idx = (self.current_tab_idx + 1) % self.tabs.len();
                    return vec![Event::TabChanged];
                }
                Command::PreviousTab => {
                    self.current_tab_idx =
                        (self.current_tab_idx + self.tabs.len() - 1) % self.tabs.len();
                    return vec![Event::TabChanged];
                }
                _ => {}
            }
        }

        vec![]
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let tab_names = (1..self.tabs.len() + 1).map(|i| format!("[Tab {i}]"));
        let tabs_widget = Tabs::new(tab_names)
            .style(Style::default().gray())
            .highlight_style(Style::default().green())
            .divider(symbols::border::PLAIN.vertical_left)
            .select(self.current_tab_idx);
        frame.render_widget(tabs_widget, area);
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
