use crate::connection::Connection;

#[derive(Debug, Clone)]
pub enum Event {
    ListSelectionChanged,
    ConnectionSelected(Connection),
    ConnectionDeleted,
    ErrorOccurred(String),
    NewConnectionStarted,
    FocusedForward,
    FocusedBackward,
    RawModeEntered,
    RawModeExited,
    InputKeyPressed,
}
