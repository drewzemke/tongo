use crate::{
    config::RawConfig,
    system::command::{Command, CommandGroup},
};
use anyhow::{anyhow, bail, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use itertools::Itertools;
use std::{collections::HashMap, ops::Not};

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
            (Command::Search, KeyCode::Char('/').into()),
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
    pub fn try_from_config(config: &RawConfig) -> Result<Self> {
        let mut key_map = Self::default();

        for (command_str, key_str) in &config.keys {
            let command = string_to_command(command_str)?;
            let key = Key::try_from(key_str.as_str())?;
            key_map.map.insert(command, key);
        }

        Ok(key_map)
    }

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

    #[test]
    fn create_default_key_map() {
        let config = RawConfig::default();
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
        let config = RawConfig {
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

        let config = RawConfig::try_from(&*file).unwrap();
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
        let config = RawConfig {
            keys: HashMap::from([("not-a-command".to_string(), "k".to_string())]),
        };

        let key_map_res = KeyMap::try_from_config(&config);
        assert!(key_map_res.is_err());

        let config = RawConfig {
            keys: HashMap::from([("nav_up".to_string(), "not-a-key".to_string())]),
        };

        let key_map_res = KeyMap::try_from_config(&config);
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
