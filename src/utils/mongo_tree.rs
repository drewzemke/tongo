use mongodb::bson::{oid::ObjectId, Bson, Document};
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tui_tree_widget::TreeItem;

use crate::config::{color_map::ColorKey, Config};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
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
            #[expect(clippy::cast_possible_wrap)]
            MongoKey::Usize(n) => Self::Int64(n as i64),
        }
    }
}

impl From<Bson> for MongoKey {
    fn from(value: Bson) -> Self {
        match value {
            Bson::String(id) => Self::String(id),
            Bson::ObjectId(id) => Self::ObjectId(id),
            #[expect(clippy::cast_sign_loss)]
            Bson::Int32(n) => Self::Usize(n as usize),
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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

impl From<String> for MongoKey {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<usize> for MongoKey {
    fn from(value: usize) -> Self {
        Self::Usize(value)
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

#[derive(Debug, Default, Clone)]
pub struct MongoTreeBuilder<'a> {
    config: Config,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> MongoTreeBuilder<'a> {
    #[must_use]
    pub const fn new(config: Config) -> Self {
        Self {
            config,
            phantom: std::marker::PhantomData,
        }
    }

    /// # Panics
    /// If the passed-in document does not uphold the Mongo invariant of every doc
    /// having an `_id` field.
    #[must_use]
    pub fn build_tree_item(&self, doc: &Document) -> TreeItem<'a, MongoKey> {
        let id = doc
            .get("_id")
            .expect("all mongo documents should have an '_id' field");
        let id = MongoKey::from(id);

        let text = Span::styled(
            format!("[{id}]"),
            Style::default().fg(self.config.color_map.get(&ColorKey::DocumentsObjectId)),
        );
        TreeItem::new(id, text, self.build_doc_children(doc)).expect("document keys are unique")
    }

    fn build_doc_children(&self, doc: &Document) -> Vec<TreeItem<'a, MongoKey>> {
        doc.iter()
            .map(|(key, value)| self.bson_to_tree_item(value, MongoKey::String(key.clone())))
            .collect()
    }

    fn build_document(&self, doc: &Document, key: MongoKey) -> TreeItem<'a, MongoKey> {
        let text = Line::from(vec![
            Span::styled(
                key.to_string(),
                Style::default().fg(self.config.color_map.get(&ColorKey::DocumentsKey)),
            ),
            Span::from(" "),
        ]);
        TreeItem::new(key, text, self.build_doc_children(doc)).expect("document keys are unique")
    }

    fn build_array(&self, arr: &[Bson], key: MongoKey) -> TreeItem<'a, MongoKey> {
        let elements: Vec<TreeItem<'_, _>> = arr
            .iter()
            .enumerate()
            .map(|(index, value)| self.bson_to_tree_item(value, MongoKey::Usize(index)))
            .collect();

        let text = Line::from(vec![
            Span::styled(
                key.to_string(),
                Style::default().fg(self.config.color_map.get(&ColorKey::DocumentsKey)),
            ),
            Span::from(" "),
            Span::styled(
                format!("({} elements)", elements.len()),
                Style::default().fg(self.config.color_map.get(&ColorKey::DocumentsNote)),
            ),
        ]);
        TreeItem::new(key, text, elements).expect("document keys are unique")
    }

    fn bson_to_tree_item(&self, bson: &Bson, key: MongoKey) -> TreeItem<'a, MongoKey> {
        match bson {
            Bson::Document(doc) => self.build_document(doc, key),
            Bson::Array(arr) => self.build_array(arr, key),
            bson => {
                let text = Line::from(vec![
                    self.key_to_span(&key),
                    Span::styled(
                        ": ",
                        Style::default().fg(self.config.color_map.get(&ColorKey::DocumentsNote)),
                    ),
                    self.value_to_span(bson),
                ]);
                TreeItem::new_leaf(key, text)
            }
        }
    }

    fn key_to_span<'b>(&self, key: &MongoKey) -> Span<'b> {
        let string = match key {
            MongoKey::String(s) => s.clone(),
            MongoKey::ObjectId(id) => format!("ObjectId({id})"),
            MongoKey::Usize(n) => format!("{n}"),
        };
        Span::styled(
            string,
            Style::default().fg(self.config.color_map.get(&ColorKey::DocumentsKey)),
        )
    }

    fn value_to_span<'b>(&self, bson: &Bson) -> Span<'b> {
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
            Bson::ObjectId(_) => self.config.color_map.get(&ColorKey::DocumentsObjectId),
            Bson::String(_) => self.config.color_map.get(&ColorKey::DocumentsString),
            Bson::Boolean(_) => self.config.color_map.get(&ColorKey::DocumentsBoolean),
            Bson::Double(_) | Bson::Decimal128(_) | Bson::Int32(_) | Bson::Int64(_) => {
                self.config.color_map.get(&ColorKey::DocumentsNumber)
            }
            Bson::Timestamp(_) | Bson::DateTime(_) => {
                self.config.color_map.get(&ColorKey::DocumentsDate)
            }
            _ => self.config.color_map.get(&ColorKey::DocumentsNote),
        };

        Span::styled(string, Style::default().fg(color))
    }
}
