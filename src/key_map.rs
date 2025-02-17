use crate::system::command::{Command, CommandGroup};
use crossterm::event::KeyCode;
use itertools::Itertools;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct KeyMap {
    map: HashMap<KeyCode, Command>,
}

impl Default for KeyMap {
    fn default() -> Self {
        let map = [
            (KeyCode::Up, Command::NavUp),
            (KeyCode::Down, Command::NavDown),
            (KeyCode::Left, Command::NavLeft),
            (KeyCode::Right, Command::NavRight),
            (KeyCode::Char('K'), Command::FocusUp),
            (KeyCode::Char('J'), Command::FocusDown),
            (KeyCode::Char('H'), Command::FocusLeft),
            (KeyCode::Char('L'), Command::FocusRight),
            (KeyCode::Char('n'), Command::CreateNew),
            (KeyCode::Enter, Command::Confirm),
            (KeyCode::Char('R'), Command::Reset),
            (KeyCode::Char('r'), Command::Refresh),
            (KeyCode::Char(' '), Command::ExpandCollapse),
            (KeyCode::Char('n'), Command::NextPage),
            (KeyCode::Char('p'), Command::PreviousPage),
            (KeyCode::Char('P'), Command::FirstPage),
            (KeyCode::Char('N'), Command::LastPage),
            (KeyCode::Char('D'), Command::Delete),
            (KeyCode::Esc, Command::Back),
            (KeyCode::Char('q'), Command::Quit),
            (KeyCode::Char('I'), Command::InsertDoc),
            (KeyCode::Char('E'), Command::EditDoc),
            (KeyCode::Char('C'), Command::DuplicateDoc),
            (KeyCode::Char('D'), Command::DeleteDoc),
            (KeyCode::Char('y'), Command::Yank),
        ]
        .into();

        Self { map }
    }
}

impl KeyMap {
    /// Gets the command corresponding to a key based on the loaded keymap,
    /// making sure that the command is one of the commands that the currently-focused
    /// component will respond to
    pub fn get(&self, key: KeyCode, available_commands: &[CommandGroup]) -> Option<Command> {
        let command = self.map.get(&key)?;

        // QUESTION: should this check be elsewhere?
        if available_commands
            .iter()
            .flat_map(|group| &group.commands)
            .contains(command)
        {
            Some(*command)
        } else {
            None
        }
    }

    fn rev_lookup(&self, command: Command) -> Option<KeyCode> {
        self.map
            .iter()
            .find_map(|(key, com)| if command == *com { Some(key) } else { None })
            .copied()
    }

    fn command_to_key_str(&self, command: Command) -> String {
        let Some(key) = self.rev_lookup(command) else {
            return "?".into();
        };

        match key {
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Home => "home".to_string(),
            KeyCode::End => "end".to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::Delete => "del".to_string(),
            KeyCode::Esc => "esc".to_string(),
            KeyCode::CapsLock => "caps".to_string(),
            KeyCode::Char(c) => match c {
                ' ' => "space".to_string(),
                c => c.to_string(),
            },
            _ => "?".to_string(),
        }
    }

    /// Uses the current key configuration to build a string from a command group.
    /// Used for displaying key hints in the status bar.
    pub fn cmd_group_to_span<'a>(&self, group: &'a CommandGroup) -> Vec<Span<'a>> {
        let hint_style = Style::default();
        let key_hint: String = group
            .commands
            .iter()
            .map(|c| self.command_to_key_str(*c))
            .collect();

        vec![
            Span::styled(key_hint, hint_style.add_modifier(Modifier::BOLD)),
            Span::styled(": ", hint_style),
            Span::styled(group.name, hint_style.fg(Color::Gray)),
            Span::raw("  "),
        ]
    }
}
