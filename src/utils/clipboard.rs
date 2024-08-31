use anyhow::{Context, Result};
use arboard::Clipboard;
use mongodb::bson::{from_bson, Bson};
use serde_json::Value;

pub fn send_bson_to_clipboard(bson: &Bson) -> Result<()> {
    // if the Bson is a doc or array, turn it to json;
    // otherwise, .to_string it
    let string = match bson {
        Bson::Array(_) | Bson::Document(_) => from_bson::<Value>(bson.clone())
            .context("converting doc to json")
            .and_then(|json| {
                serde_json::to_string_pretty(&json).context("converting json to string")
            })?,
        Bson::ObjectId(v) => v.to_string(),
        Bson::String(v) | Bson::Symbol(v) | Bson::JavaScriptCode(v) => v.to_string(),
        Bson::Boolean(v) => v.to_string(),
        Bson::Int32(v) => v.to_string(),
        Bson::Int64(v) => v.to_string(),
        Bson::Double(v) => v.to_string(),
        Bson::Decimal128(v) => v.to_string(),
        Bson::Timestamp(v) => v.to_string(),
        Bson::DateTime(v) => v.to_string(),
        Bson::Binary(v) => v.to_string(),
        Bson::Null => "null".to_string(),
        Bson::Undefined => "undefined".to_string(),
        Bson::RegularExpression(v) => v.to_string(),
        Bson::JavaScriptCodeWithScope(v) => v.to_string(),
        _ => return Ok(()),
    };

    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(string)?;

    Ok(())
}
