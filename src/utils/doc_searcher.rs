use super::mongo_tree::MongoKey;
use itertools::Itertools;
use mongodb::bson::Bson;
use nucleo::Nucleo;
use std::sync::Arc;

struct SearchItem(Vec<MongoKey>, Bson);

fn flatten_doc(doc: &Bson) -> Vec<SearchItem> {
    let doc = doc.as_document().expect("should only accept documents");
    let id = doc
        .get("_id")
        .expect("all mongo documents should have an '_id' field");
    let id = MongoKey::from(id);

    let mut flattened_docs: Vec<_> = doc
        .clone()
        .into_iter()
        .flat_map(|(key, bson)| flatten_bson(key.into(), bson))
        .map(|SearchItem(keys, bson)| prepend_key(id.clone(), keys, bson))
        .collect();
    flattened_docs.push(SearchItem(vec![id], Bson::String(String::default())));

    flattened_docs
}

fn flatten_bson(key: MongoKey, bson: Bson) -> Vec<SearchItem> {
    let mut flattened_docs: Vec<_> = match bson {
        Bson::Document(doc) => doc
            .into_iter()
            .flat_map(|(key, bson)| flatten_bson(key.into(), bson))
            .map(|SearchItem(keys, bson)| prepend_key(key.clone(), keys, bson))
            .collect(),

        // TODO:
        Bson::Array(arr) => arr
            .into_iter()
            .enumerate()
            .flat_map(|(idx, bson)| flatten_bson(idx.into(), bson))
            .map(|SearchItem(keys, bson)| prepend_key(key.clone(), keys, bson))
            .collect(),

        bson => vec![SearchItem(vec![key.clone()], bson)],
    };

    // include just the key with an empty bson so that search can target the key too
    flattened_docs.push(SearchItem(vec![key], Bson::String(String::default())));

    flattened_docs
}

fn prepend_key(key: MongoKey, keys: Vec<MongoKey>, bson: Bson) -> SearchItem {
    let mut new_keys = vec![key];
    new_keys.extend(keys);
    SearchItem(new_keys, bson)
}

fn mongo_key_path_to_str(path: &[MongoKey]) -> String {
    path.iter().map(MongoKey::to_string).join(".")
}

pub struct DocSearcher {
    nucleo: Nucleo<SearchItem>,
}

impl Default for DocSearcher {
    fn default() -> Self {
        Self {
            nucleo: Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1),
        }
    }
}

impl std::fmt::Debug for DocSearcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DocSearcher!")
    }
}

impl DocSearcher {
    pub fn load_docs(&mut self, docs: &[Bson]) {
        let docs = docs.to_vec();
        let injector = self.nucleo.injector();

        tokio::spawn(async move {
            for doc in docs {
                for item in flatten_doc(&doc) {
                    injector.push(item, |item, cols| {
                        cols[0] = format!("{}:{}", mongo_key_path_to_str(&item.0), item.1).into();
                    });
                }
            }
        });
    }

    pub fn update_pattern(&mut self, pat: &str) {
        self.nucleo.pattern.reparse(
            0,
            pat,
            nucleo::pattern::CaseMatching::Smart,
            nucleo::pattern::Normalization::Smart,
            false,
        );

        self.nucleo.tick(10);
    }

    #[must_use]
    pub fn num_matches(&self) -> u32 {
        self.nucleo.snapshot().matched_item_count()
    }

    #[must_use]
    pub fn nth_match(&self, n: u32) -> Option<&Vec<MongoKey>> {
        self.nucleo
            .snapshot()
            .get_matched_item(n)
            .map(|item| &item.data.0)
    }
}
