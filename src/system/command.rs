use std::{cell::RefCell, rc::Rc};

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

    ShowHelpModal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    DocNav,
    DocActions,
    CollActions,
    DbActions,
    FilterInputActions,
    ConnActions,
    TabActions,
    AppNav,
    StatusBarOnly,
    Hidden,
}

impl std::fmt::Display for CommandCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DocNav => write!(f, "Document Navigation"),
            Self::DocActions => write!(f, "Document Actions"),
            Self::CollActions => write!(f, "Collection"),
            Self::DbActions => write!(f, "Database"),
            Self::FilterInputActions => write!(f, "Filter"),
            Self::ConnActions => write!(f, "Connection"),
            Self::TabActions => write!(f, "Tab"),
            Self::AppNav => write!(f, "Navigation"),
            _ => write!(f, ""),
        }
    }
}

impl CommandCategory {
    pub const fn help_modal_categories() -> [Self; 8] {
        [
            Self::DocNav,
            Self::DocActions,
            Self::CollActions,
            Self::DbActions,
            Self::FilterInputActions,
            Self::ConnActions,
            Self::TabActions,
            Self::AppNav,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct CommandGroup {
    pub commands: Vec<Command>,
    pub name: &'static str,
    pub category: CommandCategory,
}

impl CommandGroup {
    pub const fn new(commands: Vec<Command>, name: &'static str) -> Self {
        Self {
            commands,
            name,
            category: CommandCategory::Hidden,
        }
    }

    pub const fn in_cat(mut self, category: CommandCategory) -> Self {
        self.category = category;
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct CommandManager {
    commands: Rc<RefCell<Vec<CommandGroup>>>,
}

impl CommandManager {
    pub fn groups(&self) -> Vec<CommandGroup> {
        self.commands.borrow().clone()
    }

    pub fn set_commands(&self, commands: Vec<CommandGroup>) {
        *self.commands.borrow_mut() = commands;
    }
}
