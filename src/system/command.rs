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

    // TODO: merge with `CreatNew`
    InsertDoc,
    // TODO: merge with `Edit`
    EditDoc,
    DuplicateDoc,
    // TODO: merge with `Delete`
    DeleteDoc,
    Yank,

    NewTab,
    NextTab,
    PreviousTab,
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

            "insert_doc" => Ok(Self::InsertDoc),
            "edit_doc" => Ok(Self::EditDoc),
            "duplicate_doc" => Ok(Self::DuplicateDoc),
            "delete_doc" => Ok(Self::DeleteDoc),
            "yank" => Ok(Self::Yank),

            "new_tab" => Ok(Self::NewTab),
            "next_tab" => Ok(Self::NextTab),
            "previous_tab" => Ok(Self::PreviousTab),
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
