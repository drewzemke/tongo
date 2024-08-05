use crate::connection::Connection;

#[derive(Debug)]
pub enum Event {
    ListSelectionChanged,
    ConnectionSelected(Connection),
}
