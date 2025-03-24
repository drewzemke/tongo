use super::Component;
use crate::{
    config::{color_map::ColorKey, Config},
    model::connection::Connection,
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        Signal,
    },
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Tabs},
};
use serde::{Deserialize, Serialize};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

// other options for this: • · ⋅ ⋮ → ⟩ ⌲ ∕ ⟿ ⇝ ⇢ ▸ ⌁ ⁘ ⁙ ∶ ∷ ⟫ ⟾ ◈ ⟡ ⟜ ⟝ ‣
const TAB_NAME_SPACING: usize = 3;
const TAB_NAME_SEP: &str = "‣";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct TabSpec {
    connection: Option<String>,
    db: Option<String>,
    coll: Option<String>,
}

impl TabSpec {
    fn name(&self) -> String {
        self.connection.as_ref().map_or_else(
            || String::from("New Tab"),
            |conn_name| {
                let mut name = conn_name.clone();
                if let Some(db_name) = &self.db {
                    name = name + TAB_NAME_SEP + db_name;
                    if let Some(coll_name) = &self.coll {
                        name = name + TAB_NAME_SEP + coll_name;
                    }
                }
                name
            },
        )
    }
}

#[derive(Debug)]
pub struct TabBar {
    tabs: Vec<TabSpec>,
    current_tab_idx: usize,
    config: Config,

    scroll_state: ScrollViewState,
    // set after the first render
    visible_width: Option<u16>,
}

impl Default for TabBar {
    fn default() -> Self {
        Self {
            tabs: vec![TabSpec::default()],
            current_tab_idx: 0,
            config: Config::default(),
            scroll_state: ScrollViewState::new(),
            visible_width: None,
        }
    }
}

impl TabBar {
    pub fn new(selected_connection: Option<Connection>, config: Config) -> Self {
        let base_tab = TabSpec {
            connection: selected_connection.map(|c| c.name),
            ..Default::default()
        };

        Self {
            tabs: vec![base_tab],
            current_tab_idx: 0,
            config,
            scroll_state: ScrollViewState::new(),
            visible_width: None,
        }
    }

    pub const fn current_tab_idx(&self) -> usize {
        self.current_tab_idx
    }

    pub fn num_tabs(&self) -> usize {
        self.tabs.len()
    }

    fn tab_names(&self) -> impl Iterator<Item = String> + Clone + use<'_> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(idx, tab_spec)| format!("[{}] {}", idx + 1, tab_spec.name()))
    }

    fn next_tab(&mut self) {
        self.current_tab_idx = (self.current_tab_idx + 1) % self.tabs.len();
    }

    fn prev_tab(&mut self) {
        self.current_tab_idx = (self.current_tab_idx + self.tabs.len() - 1) % self.tabs.len();
    }

    /// scrolls the view so that the currently-selected tab is visible
    #[expect(clippy::cast_possible_truncation)]
    fn scroll_to_current_tab(&mut self) {
        let current_tab_idx = self.current_tab_idx;
        // calculate the positions (relative to the left edge) of the first and last character
        // of this tab's name in the view
        let left_char_pos = self
            .tab_names()
            .take(current_tab_idx)
            .map(|s| s.chars().count() + TAB_NAME_SPACING)
            .sum::<usize>() as u16;

        let current_tab_name = self.tab_names().nth(current_tab_idx).unwrap_or_default();
        let right_char_pos = left_char_pos + current_tab_name.chars().count() as u16;

        let visible_width = self.visible_width.unwrap_or_default();

        let mut offset = self.scroll_state.offset().x;
        // make sure the right side of the name is visible
        offset = offset.max(right_char_pos.saturating_sub(visible_width));
        // make sure the left side of the name is visible
        offset = offset.min(left_char_pos);

        self.scroll_state.set_offset(Position::new(offset, 0));
    }
}

impl Component for TabBar {
    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = vec![
            CommandGroup::new(vec![Command::NewTab], "new tab").in_cat(CommandCategory::TabActions),
            CommandGroup::new(vec![Command::DuplicateTab], "duplicate tab")
                .in_cat(CommandCategory::TabActions),
        ];

        if self.tabs.len() > 1 {
            out.append(&mut vec![
                CommandGroup::new(
                    vec![Command::PreviousTab, Command::NextTab],
                    "previous/next tab",
                )
                .in_cat(CommandCategory::TabActions),
                CommandGroup::new(vec![Command::CloseTab], "close tab")
                    .in_cat(CommandCategory::TabActions),
                CommandGroup::new(
                    vec![
                        Command::GotoTab(1),
                        Command::GotoTab(2),
                        Command::GotoTab(3),
                        Command::GotoTab(4),
                        Command::GotoTab(5),
                        Command::GotoTab(6),
                        Command::GotoTab(7),
                        Command::GotoTab(8),
                        Command::GotoTab(9),
                    ],
                    "goto tab",
                )
                .in_cat(CommandCategory::TabActions),
            ]);
        }

        out
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        match command {
            Command::NewTab => {
                self.tabs.push(TabSpec::default());
                self.current_tab_idx = self.tabs.len() - 1;
                self.scroll_to_current_tab();
                vec![Event::TabCreated.into(), Event::TabChanged.into()]
            }
            Command::NextTab => {
                self.next_tab();
                self.scroll_to_current_tab();
                vec![Event::TabChanged.into()]
            }
            Command::PreviousTab => {
                self.prev_tab();
                self.scroll_to_current_tab();
                vec![Event::TabChanged.into()]
            }
            Command::CloseTab => {
                // do nothing if this is the last tab
                if self.tabs.len() == 1 {
                    return vec![];
                }

                let next_tab_idx = self.current_tab_idx.saturating_sub(1);
                self.tabs.remove(self.current_tab_idx);
                self.current_tab_idx = next_tab_idx;
                self.scroll_to_current_tab();

                vec![Event::TabClosed.into(), Event::TabChanged.into()]
            }
            Command::DuplicateTab => {
                let Some(current_tab) = self.tabs.get_mut(self.current_tab_idx) else {
                    return vec![];
                };

                let new_tab = current_tab.clone();
                self.tabs.push(new_tab);
                self.current_tab_idx = self.tabs.len() - 1;
                self.scroll_to_current_tab();
                vec![Event::TabCreated.into(), Event::TabChanged.into()]
            }
            Command::GotoTab(num) => {
                let new_idx = num - 1;
                if new_idx >= self.tabs.len() {
                    return vec![];
                }

                self.current_tab_idx = new_idx;
                self.scroll_to_current_tab();
                vec![Event::TabChanged.into()]
            }
            _ => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        let Some(current_tab) = self.tabs.get_mut(self.current_tab_idx) else {
            return vec![];
        };

        match event {
            Event::ConnectionSelected(conn) | Event::ConnectionCreated(conn) => {
                current_tab.connection = Some(conn.name.clone());
                current_tab.db = None;
                current_tab.coll = None;
                self.scroll_to_current_tab();
            }
            Event::DatabaseSelected(db) => {
                current_tab.db = Some(db.name.clone());
                current_tab.coll = None;
                self.scroll_to_current_tab();
            }
            Event::CollectionSelected(coll) => {
                current_tab.coll = Some(coll.name.clone());
                self.scroll_to_current_tab();
            }
            _ => {}
        }

        vec![]
    }

    #[expect(clippy::cast_possible_truncation)]
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let bg = Block::default()
            .borders(Borders::NONE)
            .bg(self.config.color_map.get(&ColorKey::PanelInactiveBg));
        frame.render_widget(bg, area);

        let area = area.inner(Margin::new(1, 0));

        // update the width and scrolling if this is the first render,
        // or if the screen was resized
        if self.visible_width.is_none_or(|w| w != area.width) {
            self.visible_width = Some(area.width);
            self.scroll_to_current_tab();
        }

        let tab_names = self.tab_names();

        let content_width = tab_names.clone().map(|s| s.len() + 3).sum::<usize>() - 2;
        let mut scroll_view = ScrollView::new(Size::new(content_width as u16, 1))
            .scrollbars_visibility(ScrollbarVisibility::Never);

        let tabs = Tabs::new(tab_names)
            .bg(self.config.color_map.get(&ColorKey::PanelInactiveBg))
            .fg(self.config.color_map.get(&ColorKey::TabInactive))
            .highlight_style(self.config.color_map.get(&ColorKey::TabActive))
            // remove padding (but add spaces to divider) to get better control over margins and scrolling
            .padding("", "")
            .divider(format!(" {} ", symbols::border::PLAIN.vertical_left))
            .select(self.current_tab_idx);
        scroll_view.render_widget(tabs, scroll_view.buf().area);

        frame.render_stateful_widget(scroll_view, area, &mut self.scroll_state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTabBar {
    tabs: Vec<TabSpec>,
    current_tab_idx: usize,
}

impl PersistedComponent for TabBar {
    type StorageType = PersistedTabBar;

    fn persist(&self) -> Self::StorageType {
        PersistedTabBar {
            tabs: self.tabs.clone(),
            current_tab_idx: self.current_tab_idx,
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.tabs = storage.tabs;
        self.current_tab_idx = storage.current_tab_idx;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{collection::Collection, connection::Connection, database::Database},
        testing::ComponentTestHarness,
    };

    #[test]
    fn create_new_tab() {
        let mut test = ComponentTestHarness::new(TabBar::default());

        assert_eq!(test.component().current_tab_idx(), 0);

        test.given_command(Command::NewTab);
        test.expect_event(|e| matches!(e, Event::TabCreated));

        // after creating a new tab, it should be selected
        assert_eq!(test.component().current_tab_idx(), 1);
    }

    #[test]
    fn navigate_between_tabs() {
        let mut test = ComponentTestHarness::new(TabBar::default());

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
        let mut test = ComponentTestHarness::new(TabBar::default());

        // create two additional tabs
        test.given_command(Command::NewTab);
        test.given_command(Command::NewTab);

        // we start on tab 3 since we just created the third tab
        // moving forward should wrap around to the beginning
        test.given_command(Command::NextTab);
        test.expect_event(|e| matches!(e, Event::TabChanged));
        assert_eq!(test.component().current_tab_idx(), 0);

        // moving backward should wrap around to the end
        test.given_command(Command::PreviousTab);
        test.expect_event(|e| matches!(e, Event::TabChanged));
        assert_eq!(test.component().current_tab_idx(), 2);
    }

    #[test]
    fn goto_tab() {
        let mut test = ComponentTestHarness::new(TabBar::default());

        // create two additional tabs
        test.given_command(Command::NewTab);
        test.given_command(Command::NewTab);
        test.given_command(Command::NewTab);

        // we start on tab 4 since we just created the third tab
        test.given_command(Command::GotoTab(1));
        test.expect_event(|e| matches!(e, Event::TabChanged));
        assert_eq!(test.component().current_tab_idx(), 0);

        test.given_command(Command::GotoTab(3));
        test.expect_event(|e| matches!(e, Event::TabChanged));
        assert_eq!(test.component().current_tab_idx(), 2);

        // a command to goto a nonexistent tab should do nothing
        test.given_command(Command::GotoTab(8));
        assert_eq!(test.component().current_tab_idx(), 2);
    }

    #[test]
    fn close_tabs() {
        let mut test = ComponentTestHarness::new(TabBar::default());

        // create three additional tabs
        test.given_command(Command::NewTab);
        test.given_command(Command::NewTab);
        test.given_command(Command::NewTab);

        // we should now be on the last tab (index 3)
        assert_eq!(test.component().current_tab_idx(), 3);

        // close current tab, should move to previous tab
        test.given_command(Command::CloseTab);
        test.expect_event(|e| matches!(e, Event::TabClosed));
        assert_eq!(test.component().current_tab_idx(), 2);

        // close tab when on first tab, should move to next tab
        test.given_command(Command::GotoTab(1));
        test.expect_event(|e| matches!(e, Event::TabChanged));
        test.given_command(Command::CloseTab);
        test.expect_event(|e| matches!(e, Event::TabClosed));
        assert_eq!(test.component().current_tab_idx(), 0);

        // cannot close last remaining tab
        test.given_command(Command::CloseTab);
        test.given_command(Command::CloseTab);
        test.given_command(Command::CloseTab);
        assert_eq!(test.component().current_tab_idx(), 0);
    }

    fn get_dummy_database() -> Database {
        Database::new("test_db".to_string())
    }

    fn get_dummy_collection() -> Collection {
        Collection::new("test_collection".to_string())
    }

    #[test]
    fn update_tab_names() {
        let mut test = ComponentTestHarness::new(TabBar::default());

        // add a tab to make sure changes are only made to the current tab
        test.given_command(Command::NewTab);

        // default name should be "New Tab"
        assert_eq!(test.component().tabs[1].name(), String::from("New Tab"));

        // creating a connection with the tab focused should change the name
        test.given_event(Event::ConnectionCreated(Connection::new(
            String::from("Cool Conn"),
            String::from("url"),
        )));
        assert_eq!(test.component().tabs[1].name(), String::from("Cool Conn"));

        // selecting a connection should also change the name
        test.given_event(Event::ConnectionSelected(Connection::new(
            String::from("Better Conn"),
            String::from("url2"),
        )));
        assert_eq!(test.component().tabs[1].name(), String::from("Better Conn"));

        // selecting a db should append that to the name
        test.given_event(Event::DatabaseSelected(get_dummy_database()));
        assert_eq!(
            test.component().tabs[1].name(),
            String::from("Better Conn‣test_db")
        );

        // selecting a coll should append to the name again
        test.given_event(Event::CollectionSelected(get_dummy_collection()));
        assert_eq!(
            test.component().tabs[1].name(),
            String::from("Better Conn‣test_db‣test_collection")
        );

        // selecting a db should reset the collection name
        test.given_event(Event::DatabaseSelected(get_dummy_database()));
        assert_eq!(
            test.component().tabs[1].name(),
            String::from("Better Conn‣test_db")
        );

        // selecting a connection should reset to just the connection
        test.given_event(Event::ConnectionSelected(Connection::new(
            String::from("Even Better Conn"),
            String::from("url3"),
        )));
        assert_eq!(
            test.component().tabs[1].name(),
            String::from("Even Better Conn")
        );

        // the original tab should not have a different name
        assert_eq!(test.component().tabs[0].name(), String::from("New Tab"));
    }

    #[test]
    fn duplicate_tab() {
        let mut test = ComponentTestHarness::new(TabBar::default());

        // select a connection, collection, and db
        test.given_event(Event::ConnectionSelected(Connection::new(
            String::from("Conn"),
            String::from("url"),
        )));
        test.given_event(Event::DatabaseSelected(get_dummy_database()));
        test.given_event(Event::CollectionSelected(get_dummy_collection()));
        assert_eq!(
            test.component().tabs[0].name(),
            String::from("Conn‣test_db‣test_collection")
        );

        // duplicate the tab
        test.given_command(Command::DuplicateTab);
        assert_eq!(test.component().current_tab_idx, 1);
        assert_eq!(
            test.component().tabs[1].name(),
            String::from("Conn‣test_db‣test_collection")
        );
        test.expect_event(|e| matches!(e, Event::TabCreated));
    }
}
