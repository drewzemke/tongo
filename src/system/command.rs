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

#[derive(Debug, Default, Clone)]
pub struct CommandManager {
    commands: Rc<RefCell<Vec<CommandGroup>>>,
}

impl CommandManager {
    pub fn groups(&self) -> Vec<CommandGroup> {
        self.commands.borrow().clone()
    }

    // pub fn commands(&self) -> Vec<Command> {
    //     self.commands
    //         .borrow()
    //         .iter()
    //         .flat_map(|group| group.commands.clone())
    //         .collect::<Vec<_>>()
    // }

    pub fn set_commands(&self, commands: Vec<CommandGroup>) {
        *self.commands.borrow_mut() = commands;
    }
}
