use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use edit::{edit_with_builder, Builder};
use mongodb::bson::{from_document, to_document, Document};
use serde_json::{from_str, Value};
use std::io::stdout;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum EditDocError {
    #[error("Could not convert doc to JSON: {0}")]
    DocToJson(String),

    #[error("Could not convert doc to JSON: {0}")]
    JsonToString(String),

    #[error("Error editing string: {0}")]
    EditString(String),

    #[error("Could not parse JSON: {0}")]
    StringToJson(String),

    #[error("Could not convert JSON into doc: {0}")]
    JsonToDoc(String),

    #[error("Terminal error occurred: {0}")]
    TerminalCommand(String),
}

impl From<mongodb::bson::de::Error> for EditDocError {
    fn from(err: mongodb::bson::de::Error) -> Self {
        Self::DocToJson(err.to_string())
    }
}

impl From<serde_json::Error> for EditDocError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonToString(err.to_string())
    }
}

impl From<std::io::Error> for EditDocError {
    fn from(err: std::io::Error) -> Self {
        Self::EditString(err.to_string())
    }
}

impl From<mongodb::bson::ser::Error> for EditDocError {
    fn from(err: mongodb::bson::ser::Error) -> Self {
        Self::JsonToDoc(err.to_string())
    }
}

/// Edit document in external editor
///
/// # Errors
/// Emits a `EditDocError` if something goes wrong. See that struct for details
/// on the possible cases.
pub fn edit_doc(doc: Document) -> Result<Document, EditDocError> {
    // prepare doc
    let doc_string = from_document::<Value>(doc)
        .map_err(|e| EditDocError::DocToJson(e.to_string()))
        .and_then(|json| {
            serde_json::to_string_pretty(&json)
                .map_err(|e| EditDocError::JsonToString(e.to_string()))
        })?;

    // setup terminal for external editor
    stdout()
        .execute(LeaveAlternateScreen)
        .map_err(|e| EditDocError::TerminalCommand(e.to_string()))?;

    // call function from `edit` crate
    let updated_string = edit_with_builder(doc_string, Builder::new().suffix(".json"))
        .map_err(|e| EditDocError::EditString(e.to_string()))?;

    // setup terminal for returning to app
    stdout()
        .execute(EnterAlternateScreen)
        .map_err(|e| EditDocError::TerminalCommand(e.to_string()))?;

    // convert back into a doc
    let new_doc_json = from_str::<Value>(&updated_string)
        .map_err(|e| EditDocError::StringToJson(e.to_string()))?;

    to_document(&new_doc_json).map_err(|e| EditDocError::JsonToDoc(e.to_string()))
}
