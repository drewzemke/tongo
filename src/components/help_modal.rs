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
use itertools::Itertools;
use ratatui::{
    layout::Offset,
    prelude::*,
    widgets::{Block, Clear, Paragraph, Row, Table, TableState, Wrap},
};
use std::{collections::HashMap, rc::Rc};

const HELP_MODAL_WIDTH: u16 = 60;

#[derive(Debug, Default, Clone)]
pub struct HelpModal {
    command_manager: CommandManager,
    key_map: Rc<KeyMap>,
    state: TableState,
    categorized_groups: HashMap<CommandCategory, Vec<CommandGroup>>,
}

impl HelpModal {
    pub fn new(command_manager: CommandManager, key_map: Rc<KeyMap>) -> Self {
        Self {
            command_manager,
            key_map,
            state: TableState::default(),
            categorized_groups: HashMap::default(),
        }
    }

    fn compute_cats(&mut self) {
        self.categorized_groups = self
            .command_manager
            .groups()
            .into_iter()
            .into_group_map_by(|g| g.category);
    }
}

impl Component for HelpModal {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let title = "Available Commands";
        let block = Block::bordered()
            .title(title)
            .border_style(Style::default().fg(Color::Green));

        let layout = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Percentage(75),
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
        let content_area = block_area.inner(Margin::new(1, 1));
        frame.render_widget(block, block_area);

        let mut sub_area = content_area;
        for category in CommandCategory::all() {
            if category == CommandCategory::Hidden {
                continue;
            }

            if let Some(groups) = self.categorized_groups.get(&category) {
                let sub_layout =
                    Layout::horizontal(vec![Constraint::Length(12), Constraint::Fill(1)])
                        .horizontal_margin(1)
                        .split(sub_area);

                // render the category name
                let cat_name = Paragraph::new(format!("{category}")).wrap(Wrap::default());
                frame.render_widget(cat_name, sub_layout[0]);

                let rows = groups.iter().map(|group| {
                    let hint_style = Style::default();
                    let key_hint: String = group
                        .commands
                        .iter()
                        .map(|c| self.key_map.command_to_key_str(*c))
                        .collect();

                    Row::new(vec![
                        Span::styled(key_hint, hint_style.add_modifier(Modifier::BOLD))
                            .into_right_aligned_line(),
                        Span::styled(group.name, hint_style.fg(Color::Gray))
                            .into_left_aligned_line(),
                    ])
                });

                // render the table
                let table = Table::new(rows, vec![Constraint::Length(11), Constraint::Fill(1)])
                    .row_highlight_style(Style::default().bold().black().on_white());

                frame.render_stateful_widget(table, sub_layout[1], &mut self.state);

                // move the drawing area down so that the next category is drawn below this one
                sub_area = sub_area.offset(Offset {
                    x: 0,
                    #[expect(clippy::cast_possible_wrap)]
                    #[expect(clippy::cast_possible_truncation)]
                    y: groups.len() as i32 + 2,
                });
            }
        }
    }

    fn commands(&self) -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(vec![Command::NavUp, Command::NavDown], "navigate"),
            CommandGroup::new(vec![Command::Back], "close"),
        ]
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        match command {
            Command::Back => vec![Message::to_app(AppAction::CloseHelpModal).into()],
            Command::NavUp => {
                self.state.select_previous();
                vec![Event::ListSelectionChanged.into()]
            }
            Command::NavDown => {
                self.state.select_next();
                vec![Event::ListSelectionChanged.into()]
            }
            _ => vec![],
        }
    }

    fn handle_event(&mut self, event: &Event) -> Vec<Signal> {
        if matches!(event, Event::HelpModalToggled) {
            self.state = TableState::default();
            self.compute_cats();
        }

        vec![]
    }
}
