use nucleo::{pattern::Pattern, Config as NucleoConfig, Matcher, Utf32Str};

use crate::components::list::InnerListItem;

pub struct ListSearcher {
    matcher: Matcher,
}

impl Default for ListSearcher {
    fn default() -> Self {
        let mut config = NucleoConfig::DEFAULT;
        config.prefer_prefix = true;

        Self {
            matcher: Matcher::new(config),
        }
    }
}

impl Clone for ListSearcher {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl std::fmt::Debug for ListSearcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ListSearcher!")
    }
}

impl ListSearcher {
    /// (original index, score) in original order
    pub fn matches<'a, T: InnerListItem>(
        &mut self,
        items: &'a [T],
        query: &str,
    ) -> impl Iterator<Item = (usize, u32)> + use<'a, '_, T> {
        let pattern = Pattern::parse(
            query,
            nucleo::pattern::CaseMatching::Smart,
            nucleo::pattern::Normalization::Smart,
        );

        let mut buf = Vec::new();

        items.iter().enumerate().filter_map(move |(idx, item)| {
            pattern
                .score(
                    Utf32Str::new(&item.to_search_string(), &mut buf),
                    &mut self.matcher,
                )
                .map(|score| (idx, score))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use ratatui::widgets::ListItem;

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    struct TestItem(&'static str);

    impl InnerListItem for TestItem {
        fn to_list_item(&self, _config: &Config) -> ListItem<'_> {
            ListItem::new(self.0)
        }

        fn to_search_string(&self) -> String {
            self.0.to_string()
        }
    }

    fn indices(matches: &[(usize, u32)]) -> Vec<usize> {
        matches.iter().map(|(idx, _)| *idx).collect()
    }

    #[test]
    fn test_empty_query_matches_everything() {
        let items = [TestItem("orders"), TestItem("users"), TestItem("logs")];
        let matches = ListSearcher::default()
            .matches(&items, "")
            .collect::<Vec<_>>();

        assert_eq!(indices(&matches), vec![0, 1, 2]);
    }

    #[test]
    fn test_filters_out_non_matches() {
        let items = [
            TestItem("orders"),
            TestItem("users"),
            TestItem("old_records"),
        ];
        let matches = ListSearcher::default()
            .matches(&items, "ord")
            .collect::<Vec<_>>();

        assert_eq!(indices(&matches), vec![0, 2]);
    }

    #[test]
    fn test_keeps_original_order_regardless_of_score() {
        // the exact match comes second but scores higher
        let items = [TestItem("a_b_c_d"), TestItem("abc")];
        let matches = ListSearcher::default()
            .matches(&items, "abc")
            .collect::<Vec<_>>();

        assert_eq!(indices(&matches), vec![0, 1]);
        assert!(matches[1].1 > matches[0].1);
    }

    #[test]
    fn test_no_matches() {
        let items = [TestItem("orders"), TestItem("users")];
        let mut searcher = ListSearcher::default();
        let mut matches = searcher.matches(&items, "zzz");

        assert!(matches.next().is_none());
    }

    #[test]
    fn test_empty_items() {
        let items: [TestItem; 0] = [];
        let mut searcher = ListSearcher::default();
        let mut matches = searcher.matches(&items, "abc");

        assert!(matches.next().is_none());
    }

    #[test]
    fn test_smart_case() {
        let items = [TestItem("Orders"), TestItem("orders")];

        // lowercase query ignores case
        let matches = ListSearcher::default()
            .matches(&items, "ord")
            .collect::<Vec<_>>();
        assert_eq!(indices(&matches), vec![0, 1]);

        // uppercase in query makes it case-sensitive
        let matches = ListSearcher::default()
            .matches(&items, "Ord")
            .collect::<Vec<_>>();
        assert_eq!(indices(&matches), vec![0]);
    }

    #[test]
    fn test_reusing_searcher_across_queries() {
        let items = [TestItem("orders"), TestItem("users")];
        let mut searcher = ListSearcher::default();

        assert_eq!(
            indices(&searcher.matches(&items, "ord").collect::<Vec<_>>()),
            vec![0]
        );
        assert_eq!(
            indices(&searcher.matches(&items, "use").collect::<Vec<_>>()),
            vec![1]
        );
        assert_eq!(
            indices(&searcher.matches(&items, "").collect::<Vec<_>>()),
            vec![0, 1]
        );
    }
}
