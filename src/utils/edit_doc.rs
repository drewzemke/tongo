use anyhow::{Context, Result};
use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use edit::{edit_with_builder, Builder};
use mongodb::bson::{from_document, to_document, Document};
use serde_json::{from_str, Value};
use std::io::stdout;

// TODO: better error handling
/// Edit document in external editor
pub fn edit_doc(doc: Document) -> Result<Document> {
    let doc_string = from_document::<Value>(doc)
        .context("converting doc to json")
        .and_then(|json| {
            serde_json::to_string_pretty(&json).context("converting json to string")
        })?;

    stdout().execute(LeaveAlternateScreen)?;
    let updated_string =
        edit_with_builder(doc_string, Builder::new().suffix(".json")).context("editing string")?;
    stdout().execute(EnterAlternateScreen)?;

    let new_doc_json = from_str::<Value>(&updated_string).context("converting string to json")?;
    to_document(&new_doc_json).context("converting json to doc")
}
