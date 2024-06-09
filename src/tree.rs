use mongodb::bson::{Bson, Document};
use ratatui::prelude::*;
use tui_tree_widget::TreeItem;

pub fn top_level_document<'a>(doc: &Document) -> TreeItem<'a, String> {
    let id = doc
        .get("_id")
        .and_then(Bson::as_object_id)
        .map_or("(id missing)".to_string(), |id| id.to_string());

    TreeItem::new(
        id.clone(),
        Span::styled(format!("[{id}]"), Style::default().gray()),
        doc_children(doc),
    )
    .expect("document keys are unique")
}

fn doc_children<'a>(doc: &Document) -> Vec<TreeItem<'a, String>> {
    doc.iter().map(|(key, value)| bson(value, key)).collect()
}

fn key_val_leaf<'a, 'b>(key: &'a str, value: &'a str, color: Color) -> TreeItem<'b, String> {
    let key = key.to_string();
    TreeItem::new_leaf(
        key.clone(),
        Line::from(vec![
            Span::styled(key, Style::default().white()),
            Span::styled(": ", Style::default().gray()),
            Span::styled(value.to_string(), color),
        ]),
    )
}

fn document<'a, 'b>(doc: &'a Document, key: &'a str) -> TreeItem<'b, String> {
    TreeItem::new(
        key.to_string(),
        Line::from(vec![
            Span::styled(key.to_string(), Style::default().white()),
            Span::from(" "),
            Span::styled("(object)", Style::default().gray()),
        ]),
        doc_children(doc),
    )
    .expect("document keys are unique")
}

fn array<'a, 'b>(arr: &'a [Bson], key: &'a str) -> TreeItem<'b, String> {
    let elements: Vec<TreeItem<'_, String>> = arr
        .iter()
        .enumerate()
        .map(|(index, value)| bson(value, &format!("[{index}]")))
        .collect();
    TreeItem::new(
        key.to_string(),
        Line::from(vec![
            Span::styled(key.to_string(), Style::default().white()),
            Span::from(" "),
            Span::styled("(array)", Style::default().gray()),
        ]),
        elements,
    )
    .expect("document keys are unique")
}

fn bson<'a, 'b>(bson: &'a Bson, key: &'a str) -> TreeItem<'b, String> {
    match bson {
        Bson::ObjectId(id) => key_val_leaf(key, &format!("ObjectId({id})"), Color::White),
        Bson::String(s) => key_val_leaf(key, &format!("\"{s}\""), Color::Green),
        Bson::Boolean(b) => key_val_leaf(key, &b.to_string(), Color::Cyan),

        Bson::Double(n) => key_val_leaf(key, &n.to_string(), Color::Yellow),
        Bson::Decimal128(n) => key_val_leaf(key, &n.to_string(), Color::Yellow),
        Bson::Int32(n) => key_val_leaf(key, &n.to_string(), Color::Yellow),
        Bson::Int64(n) => key_val_leaf(key, &n.to_string(), Color::Yellow),

        Bson::Null => key_val_leaf(key, "null", Color::Gray),
        Bson::Undefined => key_val_leaf(key, "undefined", Color::Gray),

        Bson::Document(doc) => document(doc, key),
        Bson::Array(arr) => array(arr, key),

        Bson::Timestamp(t) => key_val_leaf(key, &t.to_string(), Color::Magenta),
        Bson::DateTime(d) => key_val_leaf(key, &d.to_string(), Color::Magenta),

        Bson::Binary(_) => key_val_leaf(key, "(binary)", Color::Gray),

        _ => key_val_leaf(key, &format!("{bson:?}"), Color::Gray),
    }
}
