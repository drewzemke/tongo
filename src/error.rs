use mongodb::error::ErrorKind;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    EditDoc(#[from] crate::utils::edit_doc::EditDocError),
    Mongo(#[from] mongodb::error::Error),
    String(String),
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EditDoc(edit_doc_error) => write!(f, "{edit_doc_error}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Mongo(error) => match *error.kind.clone() {
                ErrorKind::BsonDeserialization(error) => write!(f, "{error}"),
                ErrorKind::BsonSerialization(error) => write!(f, "{error}"),
                ErrorKind::Command(command_error, ..) => write!(f, "{command_error}"),
                ErrorKind::ServerSelection { message, .. }
                | ErrorKind::DnsResolve { message, .. }
                | ErrorKind::Authentication { message, .. }
                | ErrorKind::InvalidArgument { message, .. }
                | ErrorKind::Internal { message, .. }
                | ErrorKind::ConnectionPoolCleared { message, .. }
                | ErrorKind::InvalidResponse { message, .. }
                | ErrorKind::Transaction { message, .. }
                | ErrorKind::IncompatibleServer { message, .. } => write!(f, "{message}"),
                ErrorKind::Io(error) => write!(f, "{error}"),
                _ => write!(f, "{error}"),
            },
        }
    }
}
