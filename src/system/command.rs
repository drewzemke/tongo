use anyhow::{bail, Result};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Command {
    NavUp,
    NavDown,
    NavLeft,
    NavRight,

    FocusUp,
    FocusDown,
    FocusLeft,
    FocusRight,

    Confirm,
    Reset,
    Refresh,
    ExpandCollapse,

    NextPage,
    PreviousPage,
    FirstPage,
    LastPage,

    CreateNew,
    Edit,
    Delete,
    Back,
    Quit,

    DuplicateDoc,
    Yank,

    NewTab,
    NextTab,
    PreviousTab,
    CloseTab,
    DuplicateTab,
    GotoTab(usize),
}

impl Command {
    pub fn try_from_str(value: &str) -> Result<Self> {
        // TODO: better names?
        match value {
            "nav_up" => Ok(Self::NavUp),
            "nav_down" => Ok(Self::NavDown),
            "nav_left" => Ok(Self::NavLeft),
            "nav_right" => Ok(Self::NavRight),

            "focus_up" => Ok(Self::FocusUp),
            "focus_down" => Ok(Self::FocusDown),
            "focus_left" => Ok(Self::FocusLeft),
            "focus_right" => Ok(Self::FocusRight),

            "create_new" => Ok(Self::CreateNew),
            "edit" => Ok(Self::Edit),
            "confirm" => Ok(Self::Confirm),
            "reset" => Ok(Self::Reset),
            "refresh" => Ok(Self::Refresh),
            "expand_collapse" => Ok(Self::ExpandCollapse),

            "next_page" => Ok(Self::NextPage),
            "previous_page" => Ok(Self::PreviousPage),
            "first_page" => Ok(Self::FirstPage),
            "last_page" => Ok(Self::LastPage),

            "delete" => Ok(Self::Delete),
            "back" => Ok(Self::Back),
            "quit" => Ok(Self::Quit),

            "duplicate_doc" => Ok(Self::DuplicateDoc),
            "yank" => Ok(Self::Yank),

            "new_tab" => Ok(Self::NewTab),
            "next_tab" => Ok(Self::NextTab),
            "previous_tab" => Ok(Self::PreviousTab),
            "close_tab" => Ok(Self::CloseTab),
            "duplicate_tab" => Ok(Self::DuplicateTab),

            "goto_tab_1" => Ok(Self::GotoTab(1)),
            "goto_tab_2" => Ok(Self::GotoTab(2)),
            "goto_tab_3" => Ok(Self::GotoTab(3)),
            "goto_tab_4" => Ok(Self::GotoTab(4)),
            "goto_tab_5" => Ok(Self::GotoTab(5)),
            "goto_tab_6" => Ok(Self::GotoTab(6)),
            "goto_tab_7" => Ok(Self::GotoTab(7)),
            "goto_tab_8" => Ok(Self::GotoTab(8)),
            "goto_tab_9" => Ok(Self::GotoTab(9)),
            _ => bail!(format!("Command not recognized: \"{value}\"")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandGroup {
    pub commands: Vec<Command>,
    pub name: &'static str,
}

impl CommandGroup {
    pub const fn new(commands: Vec<Command>, name: &'static str) -> Self {
        Self { commands, name }
    }
}
