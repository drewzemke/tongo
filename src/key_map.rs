use crate::{
    config::Config,
    system::command::{Command, CommandGroup},
};
use anyhow::{anyhow, bail, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use itertools::Itertools;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<KeyCode> for Key {
    fn from(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::empty(),
        }
    }
}

impl From<KeyEvent> for Key {
    fn from(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

/// # Errors
/// If the input key is not recognized.
pub fn key_code_from_str(s: &str) -> Result<Key> {
    let key_code = match s {
        "enter" | "Enter" | "return" | "Return" => KeyCode::Enter,
        "esc" | "Esc" => KeyCode::Esc,
        "up" | "Up" => KeyCode::Up,
        "down" | "Down" => KeyCode::Down,
        "left" | "Left" => KeyCode::Left,
        "right" | "Right" => KeyCode::Right,
        "space" | "Space" => KeyCode::Char(' '),
        "bksp" | "backspace" | "Backspace" => KeyCode::Backspace,
        "tab" | "Tab" => KeyCode::Tab,

        // just assume that any string of length 1 should refer to that character
        s if s.len() == 1 => KeyCode::Char(
            s.chars()
                .next()
                .ok_or_else(|| anyhow!("Key not recognized: \"{s}\""))?,
        ),
        _ => bail!("Key not recognized: \"{s}\""),
    };

    Ok(key_code.into())
}

fn string_to_command(value: &str) -> Result<Command> {
    // TODO: better names?
    match value {
        "show_help_modal" => Ok(Command::ShowHelpModal),

        "nav_up" => Ok(Command::NavUp),
        "nav_down" => Ok(Command::NavDown),
        "nav_left" => Ok(Command::NavLeft),
        "nav_right" => Ok(Command::NavRight),

        "focus_up" => Ok(Command::FocusUp),
        "focus_down" => Ok(Command::FocusDown),
        "focus_left" => Ok(Command::FocusLeft),
        "focus_right" => Ok(Command::FocusRight),

        "create_new" => Ok(Command::CreateNew),
        "edit" => Ok(Command::Edit),
        "confirm" => Ok(Command::Confirm),
        "reset" => Ok(Command::Reset),
        "refresh" => Ok(Command::Refresh),
        "expand_collapse" => Ok(Command::ExpandCollapse),

        "next_page" => Ok(Command::NextPage),
        "previous_page" => Ok(Command::PreviousPage),
        "first_page" => Ok(Command::FirstPage),
        "last_page" => Ok(Command::LastPage),

        "delete" => Ok(Command::Delete),
        "back" => Ok(Command::Back),
        "quit" => Ok(Command::Quit),

        "duplicate_doc" => Ok(Command::DuplicateDoc),
        "yank" => Ok(Command::Yank),

        "new_tab" => Ok(Command::NewTab),
        "next_tab" => Ok(Command::NextTab),
        "previous_tab" => Ok(Command::PreviousTab),
        "close_tab" => Ok(Command::CloseTab),
        "duplicate_tab" => Ok(Command::DuplicateTab),

        "goto_tab_1" => Ok(Command::GotoTab(1)),
        "goto_tab_2" => Ok(Command::GotoTab(2)),
        "goto_tab_3" => Ok(Command::GotoTab(3)),
        "goto_tab_4" => Ok(Command::GotoTab(4)),
        "goto_tab_5" => Ok(Command::GotoTab(5)),
        "goto_tab_6" => Ok(Command::GotoTab(6)),
        "goto_tab_7" => Ok(Command::GotoTab(7)),
        "goto_tab_8" => Ok(Command::GotoTab(8)),
        "goto_tab_9" => Ok(Command::GotoTab(9)),
        _ => bail!(format!("Command not recognized: \"{value}\"")),
    }
}

#[derive(Debug, Clone)]
pub struct KeyMap {
    map: HashMap<Command, Key>,
}

impl Default for KeyMap {
    // TODO: find a way to make this typesafe, so that an error is shown
    // when a command isn't mapped here
    fn default() -> Self {
        let map = [
            (Command::ShowHelpModal, KeyCode::Char('?').into()),
            (Command::NavUp, KeyCode::Up.into()),
            (Command::NavDown, KeyCode::Down.into()),
            (Command::NavLeft, KeyCode::Left.into()),
            (Command::NavRight, KeyCode::Right.into()),
            (Command::FocusUp, KeyCode::Char('K').into()),
            (Command::FocusDown, KeyCode::Char('J').into()),
            (Command::FocusLeft, KeyCode::Char('H').into()),
            (Command::FocusRight, KeyCode::Char('L').into()),
            (Command::CreateNew, KeyCode::Char('A').into()),
            (Command::Edit, KeyCode::Char('E').into()),
            (Command::Confirm, KeyCode::Enter.into()),
            (Command::Reset, KeyCode::Char('R').into()),
            (Command::Refresh, KeyCode::Char('r').into()),
            (Command::ExpandCollapse, KeyCode::Char(' ').into()),
            (Command::NextPage, KeyCode::Char('n').into()),
            (Command::PreviousPage, KeyCode::Char('p').into()),
            (Command::FirstPage, KeyCode::Char('P').into()),
            (Command::LastPage, KeyCode::Char('N').into()),
            (Command::Delete, KeyCode::Char('D').into()),
            (Command::Back, KeyCode::Esc.into()),
            (Command::Quit, KeyCode::Char('q').into()),
            (Command::DuplicateDoc, KeyCode::Char('C').into()),
            (Command::Yank, KeyCode::Char('y').into()),
            (Command::NewTab, KeyCode::Char('T').into()),
            (Command::NextTab, KeyCode::Char(']').into()), // TODO: make these "tab" and "shift+tab" once modifiers are a thing
            (Command::PreviousTab, KeyCode::Char('[').into()),
            (Command::CloseTab, KeyCode::Char('X').into()),
            (Command::DuplicateTab, KeyCode::Char('S').into()), // ctrl+shift T or something?
            (Command::GotoTab(1), KeyCode::Char('1').into()),
            (Command::GotoTab(2), KeyCode::Char('2').into()),
            (Command::GotoTab(3), KeyCode::Char('3').into()),
            (Command::GotoTab(4), KeyCode::Char('4').into()),
            (Command::GotoTab(5), KeyCode::Char('5').into()),
            (Command::GotoTab(6), KeyCode::Char('6').into()),
            (Command::GotoTab(7), KeyCode::Char('7').into()),
            (Command::GotoTab(8), KeyCode::Char('8').into()),
            (Command::GotoTab(9), KeyCode::Char('9').into()),
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
            let command = string_to_command(command_str)?;
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
    pub fn command_for_key_unfiltered(&self, key: Key) -> Option<&Command> {
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
        key: Key,
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

    // TODO: use impl of Display for Key instead
    #[must_use]
    pub fn command_to_key_str(&self, command: Command) -> String {
        let Some(key) = self.map.get(&command) else {
            return "?".into();
        };

        match key.code {
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
    pub fn cmd_group_to_span(&self, group: &CommandGroup) -> Vec<Span> {
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
            key_map.command_for_key_unfiltered(KeyCode::Up.into()),
            Some(&Command::NavUp)
        );
        assert_eq!(
            key_map.command_for_key_unfiltered(KeyCode::Char('k').into()),
            None
        );
    }

    #[test]
    fn create_overridden_key_map() {
        let config = Config {
            keys: HashMap::from([("nav_up".to_string(), "k".to_string())]),
        };
        let key_map = KeyMap::try_from_config(&config).unwrap();

        assert!(key_map
            .command_for_key_unfiltered(KeyCode::Up.into())
            .is_none(),);
        assert_eq!(
            key_map.command_for_key_unfiltered(KeyCode::Char('k').into()),
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
