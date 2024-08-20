use anyhow::Result;
use itertools::Itertools;
use std::str::FromStr;
use syntect::{
    easy::ScopeRegionIterator,
    highlighting::ScopeSelectors,
    parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum JsonLabel {
    Punctuation,
    Number,
    Key,
    Value,
    Constant,
    Whitespace,
    Error,
}

#[derive(Debug)]
pub struct JsonLabeler {
    syntax_set: SyntaxSet,
    syntax: SyntaxReference,
    selectors: Vec<(JsonLabel, ScopeSelectors)>,
}

const SELECTOR_TEMPLATE: [(JsonLabel, &str); 6] = [
    (JsonLabel::Error, "invalid"),
    (
        JsonLabel::Punctuation,
        "punctuation.definition.string, punctuation.separator, punctuation.section",
    ),
    (JsonLabel::Number, "constant.numeric"),
    (JsonLabel::Key, "meta.structure.dictionary.key"),
    (JsonLabel::Constant, "constant.language.json"),
    (JsonLabel::Value, "meta.structure.dictionary.value"),
];

impl Default for JsonLabeler {
    fn default() -> Self {
        Self::new()
    }
}

pub type JsonLabels = Vec<(String, JsonLabel)>;

impl JsonLabeler {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let syntax = syntax_set
            .find_syntax_by_extension("json")
            .expect("json should be an included syntax")
            .clone();

        let selectors = SELECTOR_TEMPLATE
            .into_iter()
            .map(|(name, def)| {
                (
                    name,
                    ScopeSelectors::from_str(def).expect("valid scope defn"),
                )
            })
            .collect();

        Self {
            syntax_set,
            syntax,
            selectors,
        }
    }

    pub fn label_line(&self, line: &str) -> Result<JsonLabels> {
        let mut parse_state = ParseState::new(&self.syntax);
        let ops = parse_state.parse_line(line, &self.syntax_set)?;
        let mut stack = ScopeStack::new();

        let mut labeled_substrings = vec![];

        for (s, op) in ScopeRegionIterator::new(&ops, line) {
            stack.apply(op)?;
            if s.is_empty() {
                continue;
            }

            if s.chars().all(char::is_whitespace) {
                labeled_substrings.push((s.to_string(), JsonLabel::Whitespace));
                continue;
            }

            let matching_selector = self
                .selectors
                .iter()
                .find(|(_, selector)| selector.does_match(stack.as_slice()).is_some());

            if let Some((label, _)) = matching_selector {
                labeled_substrings.push((s.to_string(), label.clone()));
            }
        }

        let grouped_substrings = labeled_substrings
            .into_iter()
            .chunk_by(|(_, label)| label.clone())
            .into_iter()
            .map(|(label, chunk)| {
                let joined_str: String = chunk.into_iter().map(|(s, _)| s).collect();
                (joined_str, label)
            })
            .collect();

        Ok(grouped_substrings)
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_object() {
        let parser = JsonLabeler::new();
        let json = "{ }";
        let parsed = parser.label_line(json).unwrap();

        assert_eq!(
            parsed,
            vec![
                ("{".to_string(), JsonLabel::Punctuation),
                (" ".to_string(), JsonLabel::Whitespace),
                ("}".to_string(), JsonLabel::Punctuation),
            ]
        );
    }

    #[test]
    fn parse_key_val() {
        let parser = JsonLabeler::new();
        let json = "{\"key\":\"value\"}";
        let parsed = parser.label_line(json).unwrap();

        assert_eq!(
            parsed,
            vec![
                ("{\"".to_string(), JsonLabel::Punctuation),
                ("key".to_string(), JsonLabel::Key),
                ("\":\"".to_string(), JsonLabel::Punctuation),
                ("value".to_string(), JsonLabel::Value),
                ("\"}".to_string(), JsonLabel::Punctuation),
            ]
        );
    }

    #[test]
    fn parse_array_with_spaces() {
        let parser = JsonLabeler::new();
        let json = "{ \"key\" :   [1, 2,  3]}";
        let parsed = parser.label_line(json).unwrap();

        assert_eq!(
            parsed,
            vec![
                ("{".to_string(), JsonLabel::Punctuation),
                (" ".to_string(), JsonLabel::Whitespace),
                ("\"".to_string(), JsonLabel::Punctuation),
                ("key".to_string(), JsonLabel::Key),
                ("\"".to_string(), JsonLabel::Punctuation),
                (" ".to_string(), JsonLabel::Whitespace),
                (":".to_string(), JsonLabel::Punctuation),
                ("   ".to_string(), JsonLabel::Whitespace),
                ("[".to_string(), JsonLabel::Punctuation),
                ("1".to_string(), JsonLabel::Number),
                (",".to_string(), JsonLabel::Punctuation),
                (" ".to_string(), JsonLabel::Whitespace),
                ("2".to_string(), JsonLabel::Number),
                (",".to_string(), JsonLabel::Punctuation),
                ("  ".to_string(), JsonLabel::Whitespace),
                ("3".to_string(), JsonLabel::Number),
                ("]}".to_string(), JsonLabel::Punctuation),
            ]
        );
    }

    #[test]
    fn parse_bad_json() {
        let parser = JsonLabeler::new();
        let json = "{\"key\" xxx :3}";
        let parsed = parser.label_line(json).unwrap();

        assert_eq!(
            parsed,
            vec![
                ("{\"".to_string(), JsonLabel::Punctuation),
                ("key".to_string(), JsonLabel::Key),
                ("\"".to_string(), JsonLabel::Punctuation),
                (" ".to_string(), JsonLabel::Whitespace),
                ("xxx".to_string(), JsonLabel::Error),
                (" ".to_string(), JsonLabel::Whitespace),
                (":".to_string(), JsonLabel::Punctuation),
                ("3".to_string(), JsonLabel::Number),
                ("}".to_string(), JsonLabel::Punctuation),
            ]
        );
    }
}
