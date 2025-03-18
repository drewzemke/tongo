use super::mongo_tree::MongoKey;
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

    doc.clone()
        .into_iter()
        .flat_map(|(key, bson)| flatten_bson(key.into(), bson))
        .map(|SearchItem(keys, bson)| prepend_key(id.clone(), keys, bson))
        .collect()
}

fn flatten_bson(key: MongoKey, bson: Bson) -> Vec<SearchItem> {
    match bson {
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

        bson => vec![SearchItem(vec![key], bson)],
    }
}

fn prepend_key(key: MongoKey, keys: Vec<MongoKey>, bson: Bson) -> SearchItem {
    let mut new_keys = vec![key];
    new_keys.extend(keys);
    SearchItem(new_keys, bson)
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
                        // TODO: also put the path into the searchable text
                        // needs a helper function to convert vecs of mongo keys into strings
                        cols[0] = item.1.to_string().into();
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
