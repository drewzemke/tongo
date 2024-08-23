#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use std::fmt::Display;

use mongodb::bson::{oid::ObjectId, Bson, Document};
use ratatui::prelude::*;
use tui_tree_widget::TreeItem;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum MongoKey {
    String(String),
    ObjectId(ObjectId),
    Usize(usize),
}

impl From<MongoKey> for Bson {
    fn from(value: MongoKey) -> Self {
        match value {
            MongoKey::String(s) => Self::String(s),
            MongoKey::ObjectId(oid) => Self::ObjectId(oid),
            MongoKey::Usize(n) => Self::Int64(n as i64),
        }
    }
}

impl From<Bson> for MongoKey {
    fn from(value: Bson) -> Self {
        match value {
            Bson::String(id) => Self::String(id),
            Bson::ObjectId(id) => Self::ObjectId(id),
            Bson::Int32(n) => Self::Usize(n as usize),
            Bson::Int64(n) => Self::Usize(n as usize),
            _ => Self::String(format!("{value:?}")),
        }
    }
}

impl From<&Bson> for MongoKey {
    fn from(value: &Bson) -> Self {
        Self::from(value.clone())
    }
}

impl Default for MongoKey {
    fn default() -> Self {
        Self::ObjectId(ObjectId::default())
    }
}

impl Display for MongoKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ObjectId(id) => format!("ObjectId({id})"),
            Self::String(s) => s.clone(),
            Self::Usize(n) => n.to_string(),
        };
        f.write_str(&s)
    }
}

pub fn top_level_document<'a>(doc: &Document) -> TreeItem<'a, MongoKey> {
    let id = doc
        .get("_id")
        .expect("all mongo documents should have an '_id' field");
    let id = MongoKey::from(id);

    let text = Span::styled(format!("[{id}]"), Style::default().gray());
    TreeItem::new(id, text, doc_children(doc)).expect("document keys are unique")
}

fn doc_children<'a>(doc: &Document) -> Vec<TreeItem<'a, MongoKey>> {
    doc.iter()
        .map(|(key, value)| bson_to_tree_item(value, MongoKey::String(key.clone())))
        .collect()
}

fn document<'a>(doc: &Document, key: MongoKey) -> TreeItem<'a, MongoKey> {
    let text = Line::from(vec![
        Span::styled(key.to_string(), Style::default().white()),
        Span::from(" "),
        // Span::styled("(object)", Style::default().gray()),
    ]);
    TreeItem::new(key, text, doc_children(doc)).expect("document keys are unique")
}

fn array<'a>(arr: &[Bson], key: MongoKey) -> TreeItem<'a, MongoKey> {
    let elements: Vec<TreeItem<'_, _>> = arr
        .iter()
        .enumerate()
        .map(|(index, value)| bson_to_tree_item(value, MongoKey::Usize(index)))
        .collect();
    let text = Line::from(vec![
        Span::styled(key.to_string(), Style::default().white()),
        Span::from(" "),
        Span::styled(
            format!("({} elements)", elements.len()),
            Style::default().gray(),
        ),
    ]);
    TreeItem::new(key, text, elements).expect("document keys are unique")
}

fn bson_to_tree_item<'a>(bson: &Bson, key: MongoKey) -> TreeItem<'a, MongoKey> {
    match bson {
        Bson::Document(doc) => document(doc, key),
        Bson::Array(arr) => array(arr, key),
        bson => {
            let text = Line::from(vec![
                key_to_span(&key),
                Span::styled(": ", Style::default().gray()),
                value_to_span(bson),
            ]);
            TreeItem::new_leaf(key, text)
        }
    }
}

fn key_to_span<'a>(key: &MongoKey) -> Span<'a> {
    let string = match key {
        MongoKey::String(s) => s.clone(),
        MongoKey::ObjectId(id) => format!("ObjectId({id})"),
        MongoKey::Usize(n) => format!("[{n}]"),
    };
    Span::styled(string, Style::default().white())
}

fn value_to_span<'a>(bson: &Bson) -> Span<'a> {
    let string = match bson {
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
    };

    let color = match bson {
        Bson::ObjectId(_d) => Color::White,
        Bson::String(_) => Color::Green,
        Bson::Boolean(_) => Color::Cyan,
        Bson::Double(_) | Bson::Decimal128(_) | Bson::Int32(_) | Bson::Int64(_) => Color::Yellow,
        Bson::Timestamp(_) | Bson::DateTime(_) => Color::Magenta,
        _ => Color::Gray,
    };

    Span::styled(string, color)
}
