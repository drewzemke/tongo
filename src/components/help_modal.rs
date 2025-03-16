use crate::{
    components::Component,
    key_map::KeyMap,
    system::{
        command::{Command, CommandCategory, CommandGroup, CommandManager},
        event::Event,
        message::{AppAction, Message},
        Signal,
    },
};
use ratatui::{
    layout::Offset,
    prelude::*,
    widgets::{Block, Clear, Row, Table, TableState},
};
use std::rc::Rc;

const HELP_MODAL_WIDTH: u16 = 60;

/// represents the location of the selected command. first usize is the category,
/// second usize is the group
#[derive(Debug, Default, Clone)]
struct State(Option<(usize, usize)>);

#[derive(Debug, Default, Clone)]
pub struct HelpModal {
    command_manager: CommandManager,
    key_map: Rc<KeyMap>,
    categorized_groups: Vec<(CommandCategory, Vec<CommandGroup>)>,
    state: State,
}

impl HelpModal {
    pub fn new(command_manager: CommandManager, key_map: Rc<KeyMap>) -> Self {
        Self {
            command_manager,
            key_map,
            categorized_groups: vec![],
            state: State::default(),
        }
    }

    fn compute_cats(&mut self) {
        let all_groups = self.command_manager.groups();
        let mut categorized_groups = vec![];

        for category in CommandCategory::help_modal_categories() {
            let groups = all_groups
                .iter()
                .filter(|group| group.category == category)
                .map(Clone::clone)
                .collect::<Vec<_>>();
            if !groups.is_empty() {
                categorized_groups.push((category, groups));
            }
        }

        self.categorized_groups = categorized_groups;
    }

    fn select_down(&mut self) {
        if let State(Some((cat_idx, group_idx))) = self.state {
            let current_cat_len = self
                .categorized_groups
                .get(cat_idx)
                .map_or(0, |(_, groups)| groups.len());

            let next_indices = if group_idx >= current_cat_len.saturating_sub(1) {
                // move select to the next category if there is one
                let num_cats = self.categorized_groups.len();
                if cat_idx < num_cats.saturating_sub(2) {
                    (cat_idx + 2, 0)
                } else {
                    (cat_idx, group_idx)
                }
            } else {
                (cat_idx, group_idx + 1)
            };

            self.state = State(Some(next_indices));
        } else {
            self.state = State(Some((0, 0)));
        }
    }

    fn select_up(&mut self) {
        if let State(Some((cat_idx, group_idx))) = self.state {
            let next_indices = if cat_idx == 0 && group_idx == 0 {
                (cat_idx, group_idx)
            } else if group_idx == 0 && cat_idx > 1 {
                let next_cat_len = self
                    .categorized_groups
                    .get(cat_idx.saturating_sub(2))
                    .map_or(cat_idx, |(_, groups)| groups.len());

                (cat_idx.saturating_sub(2), next_cat_len.saturating_sub(1))
            } else {
                (cat_idx, group_idx.saturating_sub(1))
            };

            self.state = State(Some(next_indices));
        } else {
            let last_cat_length = self
                .categorized_groups
                .last()
                .map_or(0, |(_, groups)| groups.len());

            let num_cats = self.categorized_groups.len();

            self.state = State(Some((
                num_cats.saturating_sub(1),
                last_cat_length.saturating_sub(1),
            )));
        }
    }

    fn select_right(&mut self) {
        if let State(Some((cat_idx, group_idx))) = self.state {
            let num_cats = self.categorized_groups.len();

            let next_indices = if cat_idx % 2 == 0 && cat_idx < num_cats.saturating_sub(1) {
                let next_cat_len = self
                    .categorized_groups
                    .get(cat_idx + 1)
                    .map_or(cat_idx, |(_, groups)| groups.len());

                // try to move directly to the left.
                // if the category to the left has fewer groups, just go to the last one
                (cat_idx + 1, group_idx.min(next_cat_len.saturating_sub(1)))
            } else {
                (cat_idx, group_idx)
            };

            self.state = State(Some(next_indices));
        } else {
            self.state = State(Some((0, 0)));
        }
    }

    fn select_left(&mut self) {
        if let State(Some((cat_idx, group_idx))) = self.state {
            let next_indices = if cat_idx % 2 == 1 {
                let next_cat_len = self
                    .categorized_groups
                    .get(cat_idx.saturating_sub(1))
                    .map_or(cat_idx, |(_, groups)| groups.len());

                (
                    cat_idx.saturating_sub(1),
                    group_idx.min(next_cat_len.saturating_sub(1)),
                )
            } else {
                (cat_idx, group_idx)
            };

            self.state = State(Some(next_indices));
        } else {
            let num_cats = self.categorized_groups.len();
            if num_cats > 1 {
                self.state = State(Some((1, 0)));
            } else {
                self.state = State(Some((0, 0)));
            }
        }
    }

    /// returns a unique `Command` in the current-selected `CommandGroup`.
    /// returns `None` if no group is selected or if the selected group has more
    /// than one `Command`
    fn selected_cmd(&self) -> Option<Command> {
        let State(state) = self.state;
        let (cat_idx, group_idx) = state?;

        let (_, groups) = self.categorized_groups.get(cat_idx)?;
        let group = groups.get(group_idx)?;
        if group.commands.len() == 1 {
            group.commands.first().copied()
        } else {
            None
        }
    }
}

impl Component for HelpModal {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // overall popup layout -- we render `Clear` onto a larger area than the popup in order
        // create some visual separation between the popup and the rest of the app
        let title = " Available Commands ";
        let block = Block::bordered()
            .title(title)
            .border_style(Style::default().fg(Color::Blue));

        let layout = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Length(25),
            Constraint::Fill(1),
        ])
        .split(area);
        let layout = Layout::horizontal(vec![
            Constraint::Fill(1),
            Constraint::Length(HELP_MODAL_WIDTH + 6),
            Constraint::Fill(1),
        ])
        .split(layout[1]);

        frame.render_widget(Clear, layout[1]);

        let block_area = layout[1].inner(Margin::new(2, 1));
        let content_area = block_area.inner(Margin::new(1, 2));
        frame.render_widget(block, block_area);

        // for the commands, we'll draw each currently-relevant
        // `CommandCategory` as its own little table, and those tables will be
        // arranged in a 2-column grid. the next three variables are used
        // for book-keeping while we're doing that

        // keeps track of where the next row of the grid should be
        let mut sub_area = content_area;

        //
        let mut grid_row_height = 0;

        // the index counts how many cells we've drawn in total, so we can
        // decide when to move the subarea and whether to draw each cell in the
        // left or the right column
        for (cat_idx, (category, groups)) in self.categorized_groups.iter().enumerate() {
            let grid_cell_area = {
                let sub_area_layout = Layout::horizontal(vec![
                    Constraint::Fill(1),
                    Constraint::Length(2),
                    Constraint::Fill(1),
                ])
                .split(sub_area);

                // render on the left or right of this row
                if cat_idx % 2 == 0 {
                    sub_area_layout[0]
                } else {
                    sub_area_layout[2]
                }
            };

            let grid_cell_layout = Layout::vertical(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .horizontal_margin(2)
            .split(grid_cell_area);

            // render the category name
            let cat_name = Line::from(format!("{category}"));
            frame.render_widget(cat_name, grid_cell_layout[0]);

            // draw a horizontal line
            let line = Line::from("─".repeat(grid_cell_layout[1].width as usize));
            frame.render_widget(line, grid_cell_layout[1]);

            let rows = groups.iter().map(|group| {
                let key_hint: String = group
                    .commands
                    .iter()
                    .map(|c| self.key_map.command_to_key_str(*c))
                    .collect();

                Row::new(vec![key_hint.bold(), group.name.gray()])
            });

            // render the table
            let table = Table::new(rows, vec![Constraint::Length(7), Constraint::Fill(1)])
                .row_highlight_style(Style::default().bold().black().on_white());

            // figure out if/where we should show a selected row in this table
            let mut table_state =
                if let State(Some((selected_cat_idx, selected_group_idx))) = self.state {
                    if selected_cat_idx == cat_idx {
                        TableState::default().with_selected(Some(selected_group_idx))
                    } else {
                        TableState::default()
                    }
                } else {
                    TableState::default()
                };

            frame.render_stateful_widget(
                table,
                grid_cell_layout[2].inner(Margin::new(1, 0)),
                &mut table_state,
            );

            // compute the row height as the maximum of the heights of the two cells in the row
            grid_row_height = grid_row_height.max(groups.len());

            if cat_idx % 2 == 1 {
                // move the drawing area down so that the next row is below this one
                sub_area = sub_area.offset(Offset {
                    x: 0,
                    #[expect(clippy::cast_possible_wrap)]
                    #[expect(clippy::cast_possible_truncation)]
                    y: grid_row_height as i32 + 3,
                });

                grid_row_height = 0;
            }
        }
    }

    fn commands(&self) -> Vec<CommandGroup> {
        let mut out = vec![CommandGroup::new(
            vec![
                Command::NavLeft,
                Command::NavDown,
                Command::NavUp,
                Command::NavRight,
            ],
            "navigate",
        )
        .in_cat(CommandCategory::StatusBarOnly)];

        if self.selected_cmd().is_some() {
            out.push(
                CommandGroup::new(vec![Command::Confirm], "do selected")
                    .in_cat(CommandCategory::StatusBarOnly),
            );
        }

        out.push(
            CommandGroup::new(vec![Command::Back], "close").in_cat(CommandCategory::StatusBarOnly),
        );
        out
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        match command {
            Command::Back => vec![Message::to_app(AppAction::CloseHelpModal).into()],
            Command::NavLeft => {
                self.select_left();
                vec![Event::ListSelectionChanged.into()]
            }
            Command::NavDown => {
                self.select_down();
                vec![Event::ListSelectionChanged.into()]
            }
            Command::NavUp => {
                self.select_up();
                vec![Event::ListSelectionChanged.into()]
            }
            Command::NavRight => {
                self.select_right();
                vec![Event::ListSelectionChanged.into()]
            }
            Command::Confirm => self.selected_cmd().map_or_else(Vec::new, |cmd| {
                vec![
                    Message::to_app(AppAction::CloseHelpModal).into(),
                    Message::to_app(AppAction::DoCommand(cmd)).into(),
                ]
            }),
            _ => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        if matches!(event, Event::HelpModalToggled) {
            self.compute_cats();
            self.state = State::default();
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::ComponentTestHarness;

    #[test]
    fn select_down_within_one_category() {
        let component = HelpModal {
            categorized_groups: vec![(
                CommandCategory::DocActions,
                vec![
                    CommandGroup::new(vec![Command::CreateNew], "new"),
                    CommandGroup::new(vec![Command::Edit], "edit"),
                ],
            )],
            ..Default::default()
        };

        let mut test = ComponentTestHarness::new(component);

        // nothing selected initially
        assert_eq!(test.component().selected_cmd(), None);

        // select first command
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected_cmd(), Some(Command::CreateNew));

        // move down to select next group
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected_cmd(), Some(Command::Edit));

        // moving down shouldn't change anything
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected_cmd(), Some(Command::Edit));
    }

    #[test]
    fn select_up_within_one_catgory() {
        let component = HelpModal {
            categorized_groups: vec![(
                CommandCategory::DocActions,
                vec![
                    CommandGroup::new(vec![Command::CreateNew], "new"),
                    CommandGroup::new(vec![Command::Edit], "edit"),
                ],
            )],
            ..Default::default()
        };

        let mut test = ComponentTestHarness::new(component);

        // nothing selected initially
        assert_eq!(test.component().selected_cmd(), None);

        // select last command
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected_cmd(), Some(Command::Edit));

        // move up
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected_cmd(), Some(Command::CreateNew));

        // moving up again shouldn't change anything
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected_cmd(), Some(Command::CreateNew));
    }

    #[test]
    fn select_down_through_multiple_categories() {
        let component = HelpModal {
            categorized_groups: vec![
                (
                    CommandCategory::DocActions,
                    vec![
                        CommandGroup::new(vec![Command::CreateNew], "new"),
                        CommandGroup::new(vec![Command::Edit], "edit"),
                    ],
                ),
                (
                    CommandCategory::DocNav,
                    vec![CommandGroup::new(vec![Command::ExpandCollapse], "exp/coll")],
                ),
                (
                    CommandCategory::AppNav,
                    vec![CommandGroup::new(vec![Command::FocusUp], "focus up")],
                ),
            ],
            ..Default::default()
        };

        let mut test = ComponentTestHarness::new(component);

        // nothing selected initially
        assert_eq!(test.component().selected_cmd(), None);

        // select first group in first category
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected_cmd(), Some(Command::CreateNew));

        // select second group in first category
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected_cmd(), Some(Command::Edit));

        // move down to select the *third* category, since the second cat is to
        // the right of the first
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected_cmd(), Some(Command::FocusUp));

        // moving down again shouldn't change anything
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected_cmd(), Some(Command::FocusUp));
    }

    #[test]
    fn select_up_through_multiple_categories() {
        let component = HelpModal {
            categorized_groups: vec![
                (
                    CommandCategory::DocActions,
                    vec![CommandGroup::new(vec![Command::CreateNew], "new")],
                ),
                (
                    CommandCategory::DocNav,
                    vec![CommandGroup::new(vec![Command::ExpandCollapse], "exp/coll")],
                ),
                (
                    CommandCategory::AppNav,
                    vec![
                        CommandGroup::new(vec![Command::FocusUp], "focus up"),
                        CommandGroup::new(vec![Command::FocusDown], "focus down"),
                    ],
                ),
            ],
            ..Default::default()
        };

        let mut test = ComponentTestHarness::new(component);

        // nothing selected initially
        assert_eq!(test.component().selected_cmd(), None);

        // select last group in last category
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected_cmd(), Some(Command::FocusDown));

        // select first group in last category
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected_cmd(), Some(Command::FocusUp));

        // move up to select the *first* category, skipping the second since
        // it's in a different column
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected_cmd(), Some(Command::CreateNew));

        // moving up again shouldn't change anything
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected_cmd(), Some(Command::CreateNew));
    }

    #[test]
    fn select_laterally_through_multiple_categories() {
        let component = HelpModal {
            categorized_groups: vec![
                (
                    CommandCategory::DocActions,
                    vec![
                        CommandGroup::new(vec![Command::CreateNew], "new"),
                        CommandGroup::new(vec![Command::Edit], "edit"),
                    ],
                ),
                (
                    CommandCategory::DocNav,
                    vec![CommandGroup::new(vec![Command::Reset], "reset")],
                ),
                (
                    CommandCategory::AppNav,
                    vec![
                        CommandGroup::new(vec![Command::FocusUp], "focus up"),
                        CommandGroup::new(vec![Command::FocusDown], "focus down"),
                    ],
                ),
            ],
            ..Default::default()
        };

        let mut test = ComponentTestHarness::new(component);

        // nothing selected initially
        assert_eq!(test.component().selected_cmd(), None);

        // moving right starts in the top left (first group of first category)
        test.given_command(Command::NavRight);
        assert_eq!(test.component().selected_cmd(), Some(Command::CreateNew));

        // move down
        test.given_command(Command::NavDown);
        assert_eq!(test.component().selected_cmd(), Some(Command::Edit));

        // move right to the second category; the first element should be selected
        test.given_command(Command::NavRight);
        assert_eq!(test.component().selected_cmd(), Some(Command::Reset));

        // moving right again should do nothing
        test.given_command(Command::NavRight);
        assert_eq!(test.component().selected_cmd(), Some(Command::Reset));

        // moving up should also do nothing
        test.given_command(Command::NavUp);
        assert_eq!(test.component().selected_cmd(), Some(Command::Reset));

        // move left back to the first category
        test.given_command(Command::NavLeft);
        assert_eq!(test.component().selected_cmd(), Some(Command::CreateNew));

        // moving left again should do nothing
        test.given_command(Command::NavLeft);
        assert_eq!(test.component().selected_cmd(), Some(Command::CreateNew));
    }

    #[test]
    fn execute_selected_cmd() {
        let component = HelpModal {
            categorized_groups: vec![(
                CommandCategory::DocActions,
                vec![CommandGroup::new(vec![Command::CreateNew], "new")],
            )],
            ..Default::default()
        };

        let mut test = ComponentTestHarness::new(component);

        test.given_command(Command::NavDown);
        test.given_command(Command::Confirm);

        test.expect_message(|m| matches!(m.read_as_app(), Some(AppAction::CloseHelpModal)));
        test.expect_message(|m| {
            matches!(
                m.read_as_app(),
                Some(AppAction::DoCommand(c)) if *c == Command::CreateNew
            )
        });
    }

    #[test]
    fn cannot_execute_groups_with_more_than_one_command() {
        let component = HelpModal {
            categorized_groups: vec![(
                CommandCategory::DocActions,
                vec![CommandGroup::new(
                    vec![Command::NavUp, Command::NavDown],
                    "nav",
                )],
            )],
            ..Default::default()
        };

        let mut test = ComponentTestHarness::new(component);

        test.given_command(Command::NavDown);
        test.given_command(Command::Confirm);

        test.expect_no_messages();
    }
}
