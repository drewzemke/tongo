use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use edit::{edit_with_builder, Builder};
use mongodb::bson::{from_document, to_document, Document};
use serde_json::{from_str, Value};
use std::io::stdout;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EditDocError {
    #[error("Could not convert doc to JSON: {0}")]
    DocToJson(#[from] mongodb::bson::de::Error),

    #[error("Could not convert doc to JSON: {0}")]
    JsonToString(serde_json::Error),

    #[error("Error editing string: {0}")]
    EditString(#[from] std::io::Error),

    #[error("Could not parse JSON: {0}")]
    StringToJson(serde_json::Error),

    #[error("Could not convet JSON in to doc: {0}")]
    JsonToDoc(#[from] mongodb::bson::ser::Error),

    #[error("Terminal error occurred: {0}")]
    TerminalCommand(std::io::Error),
}

/// Edit document in external editor
///
/// # Errors
/// Emits a `EditDocError` if something goes wrong. See that struct for details
/// on the possible cases.
pub fn edit_doc(doc: Document) -> Result<Document, EditDocError> {
    // prepare doc
    let doc_string = from_document::<Value>(doc)
        .map_err(EditDocError::DocToJson)
        .and_then(|json| serde_json::to_string_pretty(&json).map_err(EditDocError::JsonToString))?;

    // setup terminal for external editor
    stdout()
        .execute(LeaveAlternateScreen)
        .map_err(EditDocError::TerminalCommand)?;

    // call function from `edit` crate
    let updated_string = edit_with_builder(doc_string, Builder::new().suffix(".json"))
        .map_err(EditDocError::EditString)?;

    // setup terminal for returning to app
    stdout()
        .execute(EnterAlternateScreen)
        .map_err(EditDocError::TerminalCommand)?;

    // convert back into a doc
    let new_doc_json = from_str::<Value>(&updated_string).map_err(EditDocError::StringToJson)?;

    to_document(&new_doc_json).map_err(EditDocError::JsonToDoc)
}
