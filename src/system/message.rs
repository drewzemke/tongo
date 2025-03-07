#[derive(Debug, Clone, strum_macros::Display)]
pub enum Action {
    EnterRawMode,
    ExitRawMode,
}

#[derive(Debug, Clone, strum_macros::Display)]
pub enum Target {
    App,
    Tab,
}

#[derive(Debug, Clone)]
pub struct Message(Action, Target);

impl Message {
    pub fn action(&self) -> &Action {
        &self.0
    }

    pub fn target(&self) -> &Target {
        &self.1
    }
}
