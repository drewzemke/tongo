#[derive(Clone, Debug, Default)]
pub struct Connection {
    pub name: String,
    pub connection_str: String,
}

impl Connection {
    pub const fn new(name: String, connection_str: String) -> Self {
        Self {
            name,
            connection_str,
        }
    }
}
