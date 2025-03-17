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
        .flat_map(|(key, bson)| flatten_bson(key, bson))
        .map(|SearchItem(keys, bson)| {
            let mut new_keys = vec![id.clone()];
            new_keys.extend(keys);
            SearchItem(new_keys, bson)
        })
        .collect()
}

fn flatten_bson(key: String, bson: Bson) -> Vec<SearchItem> {
    let key = MongoKey::String(key);
    match bson {
        Bson::Document(doc) => doc
            .into_iter()
            .flat_map(|(key, bson)| flatten_bson(key, bson))
            .map(|SearchItem(keys, bson)| {
                let mut new_keys = vec![key.clone()];
                new_keys.extend(keys);
                SearchItem(new_keys, bson)
            })
            .collect(),

        // TODO:
        Bson::Array(_arr) => vec![],

        bson => vec![SearchItem(vec![key], bson)],
    }
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
        for doc in docs {
            for item in flatten_doc(doc) {
                self.nucleo.injector().push(item, |item, cols| {
                    // TODO: also put the path into the searchable text
                    // needs a helper function to convert vecs of mongo keys into strings
                    cols[0] = item.1.to_string().into();
                });
            }
        }
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
