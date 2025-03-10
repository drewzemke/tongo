use crate::{
    components::Component,
    key_map::KeyMap,
    system::{
        command::{Command, CommandGroup, CommandManager},
        event::Event,
        message::{AppAction, Message},
        Signal,
    },
};
use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Row, Table, TableState},
};
use std::rc::Rc;

const HELP_MODAL_WIDTH: u16 = 60;

#[derive(Debug, Default, Clone)]
pub struct HelpModal {
    command_manager: CommandManager,
    key_map: Rc<KeyMap>,
    state: TableState,
}

impl HelpModal {
    pub fn new(command_manager: CommandManager, key_map: Rc<KeyMap>) -> Self {
        Self {
            command_manager,
            key_map,
            state: TableState::default(),
        }
    }
}

impl Component for HelpModal {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let title = "Available Commands";

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

        let groups = self.command_manager.groups();
        let rows = groups.into_iter().map(|group| {
            let hint_style = Style::default();
            let key_hint: String = group
                .commands
                .iter()
                .map(|c| self.key_map.command_to_key_str(*c))
                .collect();

            Row::new(vec![
                Span::styled(key_hint, hint_style.add_modifier(Modifier::BOLD))
                    .into_right_aligned_line(),
                Span::styled(group.name, hint_style.fg(Color::Gray)).into_left_aligned_line(),
            ])
        });

        let table = Table::new(rows, Constraint::from_fills([1, 1]))
            .block(
                Block::bordered()
                    .title(title)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .row_highlight_style(Style::default().bold().black().on_white());

        frame.render_widget(Clear, layout[1]);
        frame.render_stateful_widget(table, layout[1].inner(Margin::new(2, 1)), &mut self.state);
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
}
