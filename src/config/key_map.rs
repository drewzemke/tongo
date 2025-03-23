use anyhow::{anyhow, bail, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use itertools::Itertools;
use std::{collections::HashMap, ops::Not};
use strum::IntoEnumIterator;

use crate::system::command::{Command, CommandGroup};

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
            modifiers: event
                .modifiers
                .intersection(KeyModifiers::not(KeyModifiers::SHIFT)),
        }
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code_str = match self.code {
            KeyCode::Enter => "enter",
            KeyCode::Left => "←",
            KeyCode::Right => "→",
            KeyCode::Up => "↑",
            KeyCode::Down => "↓",
            KeyCode::Home => "home",
            KeyCode::End => "end",
            KeyCode::Tab => "tab",
            KeyCode::BackTab => "bktab",
            KeyCode::Delete => "del",
            KeyCode::Esc => "esc",
            KeyCode::CapsLock => "caps",
            KeyCode::Char(c) => match c {
                ' ' => "space",
                c => &c.to_string(),
            },
            _ => "?",
        };

        let alt_str = if self.modifiers.contains(KeyModifiers::ALT) {
            "A-"
        } else {
            ""
        };

        let ctrl_str = if self.modifiers.contains(KeyModifiers::CONTROL) {
            "C-"
        } else {
            ""
        };

        write!(f, "{ctrl_str}{alt_str}{code_str}")
    }
}

impl TryFrom<&str> for Key {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        let parts: Vec<&str> = s.split('-').collect();
        let key_str = parts
            .last()
            .ok_or_else(|| anyhow!("Key not recognized: \"{s}\""))?;
        let mut modifiers = KeyModifiers::empty();

        for part in &parts[..parts.len() - 1] {
            match *part {
                "C" | "c" => modifiers.extend(KeyModifiers::CONTROL),
                "A" | "a" => modifiers.extend(KeyModifiers::ALT),
                _ => bail!("Modifier \"{part}\" in key \"{s}\" not recognized"),
            }
        }

        let code = match *key_str {
            "enter" | "Enter" | "return" | "Return" => KeyCode::Enter,
            "esc" | "Esc" => KeyCode::Esc,
            "up" | "Up" => KeyCode::Up,
            "down" | "Down" => KeyCode::Down,
            "left" | "Left" => KeyCode::Left,
            "right" | "Right" => KeyCode::Right,
            "space" | "Space" => KeyCode::Char(' '),
            "bksp" | "backspace" | "Backspace" => KeyCode::Backspace,
            "tab" | "Tab" => KeyCode::Tab,
            "backtab" | "BackTab" => KeyCode::BackTab,

            // just assume that any string of length 1 should refer to that character
            s if s.len() == 1 => KeyCode::Char(
                s.chars()
                    .next()
                    .ok_or_else(|| anyhow!("Key not recognized: \"{s}\""))?,
            ),
            _ => bail!("Key not recognized: \"{s}\""),
        };

        Ok(Self { code, modifiers })
    }
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
        "search" => Ok(Command::Search),
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

impl TryFrom<HashMap<String, String>> for KeyMap {
    type Error = anyhow::Error;

    fn try_from(map: HashMap<String, String>) -> Result<Self, Self::Error> {
        let mut key_map = Self::default();

        for (command_str, key_str) in &map {
            let command = string_to_command(command_str)?;
            let key = Key::try_from(key_str.as_str())?;
            key_map.map.insert(command, key);
        }

        Ok(key_map)
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        let mut map = HashMap::default();

        // NOTE: creating the default map by iterating and matching like this
        // guarantees that every command is mapped to a key
        for command in Command::iter() {
            let key_code = match command {
                Command::ShowHelpModal => KeyCode::Char('?'),
                Command::NavUp => KeyCode::Up,
                Command::NavDown => KeyCode::Down,
                Command::NavLeft => KeyCode::Left,
                Command::NavRight => KeyCode::Right,
                Command::FocusUp => KeyCode::Char('K'),
                Command::FocusDown => KeyCode::Char('J'),
                Command::FocusLeft => KeyCode::Char('H'),
                Command::FocusRight => KeyCode::Char('L'),
                Command::CreateNew => KeyCode::Char('A'),
                Command::Edit => KeyCode::Char('E'),
                Command::Confirm => KeyCode::Enter,
                Command::Reset => KeyCode::Char('R'),
                Command::Refresh => KeyCode::Char('r'),
                Command::ExpandCollapse => KeyCode::Char(' '),
                Command::NextPage => KeyCode::Char('n'),
                Command::PreviousPage => KeyCode::Char('p'),
                Command::FirstPage => KeyCode::Char('P'),
                Command::LastPage => KeyCode::Char('N'),
                Command::Delete => KeyCode::Char('D'),
                Command::Search => KeyCode::Char('/'),
                Command::Back => KeyCode::Esc,
                Command::Quit => KeyCode::Char('q'),
                Command::DuplicateDoc => KeyCode::Char('C'),
                Command::Yank => KeyCode::Char('y'),
                Command::NewTab => KeyCode::Char('T'),
                Command::NextTab => KeyCode::Char(']'),
                Command::PreviousTab => KeyCode::Char('['),
                Command::CloseTab => KeyCode::Char('X'),
                Command::DuplicateTab => KeyCode::Char('S'),
                Command::GotoTab(1) => KeyCode::Char('1'),
                Command::GotoTab(2) => KeyCode::Char('2'),
                Command::GotoTab(3) => KeyCode::Char('3'),
                Command::GotoTab(4) => KeyCode::Char('4'),
                Command::GotoTab(5) => KeyCode::Char('5'),
                Command::GotoTab(6) => KeyCode::Char('6'),
                Command::GotoTab(7) => KeyCode::Char('7'),
                Command::GotoTab(8) => KeyCode::Char('8'),
                Command::GotoTab(9) => KeyCode::Char('9'),
                Command::GotoTab(_) => KeyCode::Null,
            };

            map.insert(command, key_code.into());
        }

        Self { map }
    }
}

impl KeyMap {
    #[must_use]
    pub fn key_for_command(&self, command: Command) -> Option<Key> {
        self.map.get(&command).copied()
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
}

#[expect(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::RawConfig;

    #[test]
    fn create_default_key_map() {
        let config = RawConfig::default();
        let key_map = KeyMap::try_from(config.keys).unwrap();

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
        let config = RawConfig {
            keys: HashMap::from([("nav_up".to_string(), "k".to_string())]),
            ..Default::default()
        };
        let key_map = KeyMap::try_from(config.keys).unwrap();

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
        let file = include_str!("../../assets/default-config.toml");

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

        let config = RawConfig::try_from((Some(file), None)).unwrap();
        let key_map_res = KeyMap::try_from(config.keys);

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
        let config = RawConfig {
            keys: HashMap::from([("not-a-command".to_string(), "k".to_string())]),
            ..Default::default()
        };

        let key_map_res = KeyMap::try_from(config.keys);
        assert!(key_map_res.is_err());

        let config = RawConfig {
            keys: HashMap::from([("nav_up".to_string(), "not-a-key".to_string())]),
            ..Default::default()
        };

        let key_map_res = KeyMap::try_from(config.keys);
        assert!(key_map_res.is_err());
    }

    #[test]
    fn parse_keys_with_control_modifier() {
        assert_eq!(
            Key::try_from("C-enter").expect("should be able to parse key"),
            Key {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::CONTROL
            }
        );

        assert_eq!(
            Key::try_from("C-K").expect("should be able to parse key"),
            Key {
                code: KeyCode::Char('K'),
                modifiers: KeyModifiers::CONTROL
            }
        );
    }

    #[test]
    fn parse_keys_with_alt_modifier() {
        assert_eq!(
            Key::try_from("A-enter").expect("should be able to parse key"),
            Key {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::ALT
            }
        );

        assert_eq!(
            Key::try_from("A-L").expect("should be able to parse key"),
            Key {
                code: KeyCode::Char('L'),
                modifiers: KeyModifiers::ALT
            }
        );
    }

    #[test]
    fn parse_keys_with_shift_modifier() {
        assert_eq!(
            Key::try_from("A").expect("should be able to parse key"),
            Key {
                code: KeyCode::Char('A'),
                modifiers: KeyModifiers::empty()
            }
        );
    }

    #[test]
    fn parse_keys_with_multiple_modifiers() {
        assert_eq!(
            Key::try_from("A-C-k").expect("should be able to parse key"),
            Key {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::CONTROL | KeyModifiers::ALT
            }
        );
    }
}
