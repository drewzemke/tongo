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
    widgets::{Block, Clear, Row, Table},
};
use std::rc::Rc;

const HELP_MODAL_WIDTH: u16 = 60;

#[derive(Debug, Default, Clone)]
pub struct HelpModal {
    command_manager: CommandManager,
    key_map: Rc<KeyMap>,
    categorized_groups: Vec<(CommandCategory, Vec<CommandGroup>)>,
}

impl HelpModal {
    pub fn new(command_manager: CommandManager, key_map: Rc<KeyMap>) -> Self {
        Self {
            command_manager,
            key_map,
            categorized_groups: vec![],
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

        // counts how many cells we've drawn in total, so we can decide when
        // to move the subarea and whether to draw each cell in the left or the
        // right column
        let mut grid_cells_drawn = 0;

        //
        let mut grid_row_height = 0;

        for (category, groups) in &self.categorized_groups {
            let grid_cell_area = {
                let sub_area_layout = Layout::horizontal(vec![
                    Constraint::Fill(1),
                    Constraint::Length(2),
                    Constraint::Fill(1),
                ])
                .split(sub_area);

                // render on the left or right of this row
                if grid_cells_drawn % 2 == 0 {
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

            frame.render_widget(table, grid_cell_layout[2].inner(Margin::new(1, 0)));

            // compute the row height as the maximum of the heights of the two cells in the row
            grid_row_height = grid_row_height.max(groups.len());

            if grid_cells_drawn % 2 == 1 {
                // move the drawing area down so that the next row is below this one
                sub_area = sub_area.offset(Offset {
                    x: 0,
                    #[expect(clippy::cast_possible_wrap)]
                    #[expect(clippy::cast_possible_truncation)]
                    y: grid_row_height as i32 + 3,
                });

                grid_row_height = 0;
            }

            grid_cells_drawn += 1;
        }
    }

    fn commands(&self) -> Vec<CommandGroup> {
        // FIXME: need to work on navigation / state management, then add this back in
        // CommandGroup::new(vec![Command::NavUp, Command::NavDown], "navigate"),
        vec![CommandGroup::new(vec![Command::Back], "close").in_cat(CommandCategory::StatusBarOnly)]
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        match command {
            Command::Back => vec![Message::to_app(AppAction::CloseHelpModal).into()],
            Command::NavUp => {
                vec![Event::ListSelectionChanged.into()]
            }
            Command::NavDown => {
                vec![Event::ListSelectionChanged.into()]
            }
            _ => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        if matches!(event, Event::HelpModalToggled) {
            self.compute_cats();
        }

        vec![]
    }
}
