use crate::state::{Mode, Screen, State, WidgetFocus};
use ratatui::prelude::*;

pub struct KeyHint {
    pub key: String,
    pub action: String,
}

impl KeyHint {
    fn new<S: Into<String>>(key: S, action: S) -> Self {
        Self {
            key: key.into(),
            action: action.into(),
        }
    }

    #[must_use]
    pub fn from_state(state: &State) -> Vec<Self> {
        match state.screen {
            Screen::Connection => match state.mode {
                Mode::Navigating => match state.focus {
                    WidgetFocus::ConnectionList => vec![
                        Self::new("↑↓/jk", "navigate"),
                        Self::new("enter", "connect"),
                        Self::new("n", "new conn."),
                        Self::new("D", "delete conn."),
                        Self::new("q", "quit"),
                    ],
                    _ => vec![],
                },
                Mode::CreatingNewConnection => match state.focus {
                    WidgetFocus::ConnectionNameEditor => vec![
                        Self::new("enter/tab", "next field"),
                        Self::new("esc", "back"),
                    ],
                    WidgetFocus::ConnectionStringEditor => vec![
                        Self::new("enter", "connect"),
                        Self::new("esc/tab", "prev field"),
                    ],
                    _ => vec![],
                },
                _ => vec![],
            },
            Screen::Primary => match state.mode {
                Mode::Navigating => match state.focus {
                    WidgetFocus::CollectionList | WidgetFocus::DatabaseList => vec![
                        Self::new("↑↓/jk", "navigate"),
                        Self::new("enter", "select"),
                        Self::new("HJKL", "change focus"),
                        Self::new("esc", "back"),
                        Self::new("q", "quit"),
                    ],
                    WidgetFocus::FilterEditor => vec![
                        Self::new("enter", "edit filter"),
                        Self::new("HJKL", "change focus"),
                        Self::new("esc", "back"),
                        Self::new("q", "quit"),
                    ],
                    WidgetFocus::MainView => vec![
                        Self::new("←↑↓→/hjkl", "navigate"),
                        Self::new("enter/space", "expand/collapse"),
                        Self::new("HJKL", "change focus"),
                        Self::new("esc", "back"),
                        Self::new("q", "quit"),
                    ],
                    _ => vec![],
                },
                Mode::EditingFilter => {
                    vec![
                        Self::new("enter", "execute query"),
                        Self::new("esc", "cancel"),
                    ]
                }
                _ => vec![],
            },
        }
    }
}

impl<'a> From<KeyHint> for Vec<Span<'a>> {
    fn from(hint: KeyHint) -> Self {
        let hint_style = Style::default();
        vec![
            Span::styled(hint.key, hint_style.add_modifier(Modifier::BOLD)),
            Span::styled(": ", hint_style),
            Span::styled(hint.action, hint_style.fg(Color::Gray)),
            Span::raw("  "),
        ]
    }
}
