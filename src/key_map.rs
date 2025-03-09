use crate::{
    config::Config,
    system::command::{Command, CommandGroup},
};
use anyhow::{bail, Result};
use crossterm::event::KeyCode;
use itertools::Itertools;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};
use std::collections::HashMap;

/// # Errors
/// If the input key is not recognized.
#[expect(clippy::missing_panics_doc)]
pub fn key_code_from_str(s: &str) -> Result<KeyCode> {
    match s {
        "enter" | "Enter" | "return" | "Return" => Ok(KeyCode::Enter),
        "esc" | "Esc" => Ok(KeyCode::Esc),
        "up" | "Up" => Ok(KeyCode::Up),
        "down" | "Down" => Ok(KeyCode::Down),
        "left" | "Left" => Ok(KeyCode::Left),
        "right" | "Right" => Ok(KeyCode::Right),
        "space" | "Space" => Ok(KeyCode::Char(' ')),
        "bksp" | "backspace" | "Backspace" => Ok(KeyCode::Backspace),
        "tab" | "Tab" => Ok(KeyCode::Tab),

        // just assume that any string of length 1 should
        // refer to that character
        s if s.len() == 1 => Ok(KeyCode::Char(
            s.chars()
                .next()
                .expect("strings of len 1 have a first char"),
        )),
        _ => bail!(format!("Key not recognized: \"{s}\"")),
    }
}

#[derive(Debug, Clone)]
pub struct KeyMap {
    map: HashMap<Command, KeyCode>,
}

impl Default for KeyMap {
    // TODO: find a way to make this typesafe, so that an error is shown
    // when a command isn't mapped here
    fn default() -> Self {
        let map = [
            (Command::ShowHelpModal, KeyCode::Char('?')),
            (Command::NavUp, KeyCode::Up),
            (Command::NavDown, KeyCode::Down),
            (Command::NavLeft, KeyCode::Left),
            (Command::NavRight, KeyCode::Right),
            (Command::FocusUp, KeyCode::Char('K')),
            (Command::FocusDown, KeyCode::Char('J')),
            (Command::FocusLeft, KeyCode::Char('H')),
            (Command::FocusRight, KeyCode::Char('L')),
            (Command::CreateNew, KeyCode::Char('A')),
            (Command::Edit, KeyCode::Char('E')),
            (Command::Confirm, KeyCode::Enter),
            (Command::Reset, KeyCode::Char('R')),
            (Command::Refresh, KeyCode::Char('r')),
            (Command::ExpandCollapse, KeyCode::Char(' ')),
            (Command::NextPage, KeyCode::Char('n')),
            (Command::PreviousPage, KeyCode::Char('p')),
            (Command::FirstPage, KeyCode::Char('P')),
            (Command::LastPage, KeyCode::Char('N')),
            (Command::Delete, KeyCode::Char('D')),
            (Command::Back, KeyCode::Esc),
            (Command::Quit, KeyCode::Char('q')),
            (Command::DuplicateDoc, KeyCode::Char('C')),
            (Command::Yank, KeyCode::Char('y')),
            (Command::NewTab, KeyCode::Char('T')),
            (Command::NextTab, KeyCode::Char(']')), // TODO: make these "tab" and "shift+tab" once modifiers are a thing
            (Command::PreviousTab, KeyCode::Char('[')),
            (Command::CloseTab, KeyCode::Char('X')),
            (Command::DuplicateTab, KeyCode::Char('S')), // ctrl+shift T or something?
            (Command::GotoTab(1), KeyCode::Char('1')),
            (Command::GotoTab(2), KeyCode::Char('2')),
            (Command::GotoTab(3), KeyCode::Char('3')),
            (Command::GotoTab(4), KeyCode::Char('4')),
            (Command::GotoTab(5), KeyCode::Char('5')),
            (Command::GotoTab(6), KeyCode::Char('6')),
            (Command::GotoTab(7), KeyCode::Char('7')),
            (Command::GotoTab(8), KeyCode::Char('8')),
            (Command::GotoTab(9), KeyCode::Char('9')),
        ]
        .into();

        Self { map }
    }
}

impl KeyMap {
    /// # Errors
    /// If the key part of the config cannot be parsed into valid keys and
    /// commands
    pub fn try_from_config(config: &Config) -> Result<Self> {
        let mut key_map = Self::default();

        for (command_str, key_str) in &config.keys {
            let command = Command::try_from_str(command_str)?;
            let key = key_code_from_str(key_str)?;
            key_map.map.insert(command, key);
        }

        Ok(key_map)
    }

    /// Gets the command corresponding to a key based on the loaded keymap,
    /// making sure that the command is one of the commands that the currently-focused
    /// component will respond to
    #[cfg(test)]
    #[must_use]
    pub fn command_for_key_unfiltered(&self, key: KeyCode) -> Option<&Command> {
        self.map
            .iter()
            .find_map(|(cmd, &k)| if k == key { Some(cmd) } else { None })
    }

    /// Gets the command corresponding to a key based on the loaded keymap,
    /// making sure that the command is one of the commands that the currently-focused
    /// component will respond to
    #[must_use]
    pub fn command_for_key(
        &self,
        key: KeyCode,
        available_commands: &[CommandGroup],
    ) -> Option<Command> {
        let commands = available_commands.iter().flat_map(|group| &group.commands);

        self.map
            .iter()
            .find_map(|(cmd, &k)| {
                if k == key && commands.clone().contains(cmd) {
                    Some(cmd)
                } else {
                    None
                }
            })
            .copied()
    }

    fn command_to_key_str(&self, command: Command) -> String {
        let Some(key) = self.map.get(&command) else {
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
    #[must_use]
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

#[expect(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_default_key_map() {
        let config = Config {
            keys: HashMap::new(),
        };
        let key_map = KeyMap::try_from_config(&config).unwrap();

        assert_eq!(
            key_map.command_for_key_unfiltered(KeyCode::Up),
            Some(&Command::NavUp)
        );
        assert_eq!(key_map.command_for_key_unfiltered(KeyCode::Char('k')), None);
    }

    #[test]
    fn create_overridden_key_map() {
        let config = Config {
            keys: HashMap::from([("nav_up".to_string(), "k".to_string())]),
        };
        let key_map = KeyMap::try_from_config(&config).unwrap();

        assert_eq!(key_map.command_for_key_unfiltered(KeyCode::Up), None);
        assert_eq!(
            key_map.command_for_key_unfiltered(KeyCode::Char('k')),
            Some(&Command::NavUp)
        );
    }

    #[test]
    fn create_key_map_from_default_config_file() {
        let file = include_str!("../assets/default-config.toml");

        // uncomment every line in the file under the [keys] header
        let file = file
            .lines()
            .map(|line| {
                if line.starts_with('#') && line.contains('=') {
                    line.strip_prefix("# ").unwrap()
                } else {
                    line
                }
            })
            .join("\n");

        let config = Config::read_from_string(&file).unwrap();
        let key_map_res = KeyMap::try_from_config(&config);

        assert!(key_map_res.is_ok());
        let key_map = key_map_res.unwrap();

        // make sure the loaded config matches the actual default keymap
        let default_key_map = KeyMap::default();
        for key in default_key_map.map.values() {
            assert_eq!(
                key_map.command_for_key_unfiltered(*key),
                default_key_map.command_for_key_unfiltered(*key),
                "default keymap and default config file disagree for key '{key}'"
            );
        }
    }

    #[test]
    fn bad_config_files() {
        let config = Config {
            keys: HashMap::from([("not-a-command".to_string(), "k".to_string())]),
        };

        let key_map_res = KeyMap::try_from_config(&config);
        assert!(key_map_res.is_err());

        let config = Config {
            keys: HashMap::from([("nav_up".to_string(), "not-a-key".to_string())]),
        };

        let key_map_res = KeyMap::try_from_config(&config);
        assert!(key_map_res.is_err());
    }
}
