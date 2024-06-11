use mongodb::bson::{Bson, Document};
use ratatui::prelude::*;
use tui_tree_widget::TreeItem;

pub fn top_level_document<'a>(doc: &Document) -> TreeItem<'a, String> {
    let id = doc
        .get("_id")
        .expect("all mongo documents should have an '_id' field");
    let id = bson_to_string(id);

    TreeItem::new(
        id.clone(),
        Span::styled(format!("[{id}]"), Style::default().gray()),
        doc_children(doc),
    )
    .expect("document keys are unique")
}

fn doc_children<'a>(doc: &Document) -> Vec<TreeItem<'a, String>> {
    doc.iter()
        .map(|(key, value)| bson_to_tree_item(value, key))
        .collect()
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
            // Span::styled("(object)", Style::default().gray()),
        ]),
        doc_children(doc),
    )
    .expect("document keys are unique")
}

fn array<'a, 'b>(arr: &'a [Bson], key: &'a str) -> TreeItem<'b, String> {
    let elements: Vec<TreeItem<'_, String>> = arr
        .iter()
        .enumerate()
        .map(|(index, value)| bson_to_tree_item(value, &format!("[{index}]")))
        .collect();
    TreeItem::new(
        key.to_string(),
        Line::from(vec![
            Span::styled(key.to_string(), Style::default().white()),
            Span::from(" "),
            Span::styled(
                format!("({} elements)", elements.len()),
                Style::default().gray(),
            ),
        ]),
        elements,
    )
    .expect("document keys are unique")
}

fn bson_to_tree_item<'a, 'b>(bson: &'a Bson, key: &'a str) -> TreeItem<'b, String> {
    match bson {
        Bson::Document(doc) => document(doc, key),
        Bson::Array(arr) => array(arr, key),
        bson => key_val_leaf(key, &bson_to_string(bson), bson_color(bson)),
    }
}

fn bson_to_string(bson: &Bson) -> String {
    match bson {
        Bson::ObjectId(id) => format!("ObjectId({id})"),
        Bson::String(s) => format!("\"{s}\""),
        Bson::Boolean(b) => b.to_string(),

        Bson::Double(n) => n.to_string(),
        Bson::Decimal128(n) => n.to_string(),
        Bson::Int32(n) => n.to_string(),
        Bson::Int64(n) => n.to_string(),

        Bson::Null => "null".to_string(),
        Bson::Undefined => "undefined".to_string(),

        Bson::Timestamp(t) => t.to_string(),
        Bson::DateTime(d) => d.to_string(),

        other => format!("{other:?}"),
    }
}

const fn bson_color(bson: &Bson) -> Color {
    match bson {
        Bson::ObjectId(_d) => Color::White,
        Bson::String(_) => Color::Green,
        Bson::Boolean(_) => Color::Cyan,
        Bson::Double(_) | Bson::Decimal128(_) | Bson::Int32(_) | Bson::Int64(_) => Color::Yellow,
        Bson::Timestamp(_) | Bson::DateTime(_) => Color::Magenta,
        _ => Color::Gray,
    }
}
