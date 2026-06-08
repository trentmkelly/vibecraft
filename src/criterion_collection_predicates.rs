#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionContentsPredicateModel {
    tests: Vec<ValuePredicateModel>,
}

impl CollectionContentsPredicateModel {
    pub fn of(tests: Vec<ValuePredicateModel>) -> Self {
        Self { tests }
    }

    pub fn test(&self, values: &[String]) -> bool {
        let mut tests_to_match = self.tests.clone();
        for value in values {
            tests_to_match.retain(|test| !test.test(value));
            if tests_to_match.is_empty() {
                return true;
            }
        }

        tests_to_match.is_empty()
    }

    pub fn unpack(&self) -> &[ValuePredicateModel] {
        &self.tests
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionCountsPredicateModel {
    entries: Vec<CollectionCountsPredicateEntry>,
}

impl CollectionCountsPredicateModel {
    pub fn of(entries: Vec<CollectionCountsPredicateEntry>) -> Self {
        Self { entries }
    }

    pub fn test(&self, values: &[String]) -> bool {
        self.entries.iter().all(|entry| entry.test(values))
    }

    pub fn unpack(&self) -> &[CollectionCountsPredicateEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionCountsPredicateEntry {
    pub test: ValuePredicateModel,
    pub count: IntBoundsModel,
}

impl CollectionCountsPredicateEntry {
    pub fn new(test: ValuePredicateModel, count: IntBoundsModel) -> Self {
        Self { test, count }
    }

    pub fn test(&self, values: &[String]) -> bool {
        let count = values.iter().filter(|value| self.test.test(value)).count() as i32;
        self.count.matches(count)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionPredicateModel {
    pub contains: Option<CollectionContentsPredicateModel>,
    pub counts: Option<CollectionCountsPredicateModel>,
    pub size: Option<IntBoundsModel>,
}

impl CollectionPredicateModel {
    pub fn new(
        contains: Option<CollectionContentsPredicateModel>,
        counts: Option<CollectionCountsPredicateModel>,
        size: Option<IntBoundsModel>,
    ) -> Self {
        Self {
            contains,
            counts,
            size,
        }
    }

    pub fn test(&self, values: &[String]) -> bool {
        if self
            .contains
            .as_ref()
            .is_some_and(|contains| !contains.test(values))
        {
            return false;
        }

        if self
            .counts
            .as_ref()
            .is_some_and(|counts| !counts.test(values))
        {
            return false;
        }

        self.size
            .as_ref()
            .is_none_or(|size| size.matches(values.len() as i32))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValuePredicateModel {
    Any,
    Exact(String),
    Prefix(String),
    Never,
}

impl ValuePredicateModel {
    pub fn exact(value: &str) -> Self {
        Self::Exact(value.to_string())
    }

    pub fn prefix(prefix: &str) -> Self {
        Self::Prefix(prefix.to_string())
    }

    fn test(&self, value: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(expected) => value == expected,
            Self::Prefix(prefix) => value.starts_with(prefix),
            Self::Never => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntBoundsModel {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

impl IntBoundsModel {
    pub fn any() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    pub fn exactly(value: i32) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    pub fn between(min: i32, max: i32) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    pub fn at_least(value: i32) -> Self {
        Self {
            min: Some(value),
            max: None,
        }
    }

    pub fn matches(&self, value: i32) -> bool {
        self.min.is_none_or(|min| value >= min) && self.max.is_none_or(|max| value <= max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn collection_contents_zero_single_and_multiple_match_java() {
        let zero = CollectionContentsPredicateModel::of(Vec::new());
        assert!(zero.test(&[]));
        assert!(zero.test(&values(&["stone"])));
        assert!(zero.unpack().is_empty());

        let single =
            CollectionContentsPredicateModel::of(vec![ValuePredicateModel::exact("apple")]);
        assert!(single.test(&values(&["stone", "apple"])));
        assert!(!single.test(&values(&["stone", "dirt"])));

        let multiple = CollectionContentsPredicateModel::of(vec![
            ValuePredicateModel::exact("apple"),
            ValuePredicateModel::exact("carrot"),
        ]);
        assert!(multiple.test(&values(&["stone", "apple", "carrot"])));
        assert!(!multiple.test(&values(&["stone", "apple"])));
        assert_eq!(multiple.unpack().len(), 2);
    }

    #[test]
    fn collection_contents_multiple_removes_all_predicates_matched_by_one_value_like_java() {
        let duplicate_tests = CollectionContentsPredicateModel::of(vec![
            ValuePredicateModel::exact("charged"),
            ValuePredicateModel::prefix("charg"),
        ]);

        assert!(duplicate_tests.test(&values(&["charged"])));
        assert!(!duplicate_tests.test(&values(&["charge"])));
    }

    #[test]
    fn collection_counts_entries_count_matching_values_like_java() {
        let values = values(&["apple", "apple", "carrot", "stone"]);

        let zero = CollectionCountsPredicateModel::of(Vec::new());
        assert!(zero.test(&values));
        assert!(zero.unpack().is_empty());

        let apple_count = CollectionCountsPredicateEntry::new(
            ValuePredicateModel::exact("apple"),
            IntBoundsModel::exactly(2),
        );
        assert!(apple_count.test(&values));

        let counts = CollectionCountsPredicateModel::of(vec![
            apple_count,
            CollectionCountsPredicateEntry::new(
                ValuePredicateModel::prefix("c"),
                IntBoundsModel::between(1, 2),
            ),
            CollectionCountsPredicateEntry::new(
                ValuePredicateModel::Any,
                IntBoundsModel::at_least(4),
            ),
        ]);
        assert!(counts.test(&values));
        assert_eq!(counts.unpack().len(), 3);

        let too_many_stones =
            CollectionCountsPredicateModel::of(vec![CollectionCountsPredicateEntry::new(
                ValuePredicateModel::exact("stone"),
                IntBoundsModel::between(2, 4),
            )]);
        assert!(!too_many_stones.test(&values));
    }

    #[test]
    fn collection_predicate_combines_contains_counts_and_size_like_java() {
        let predicate = CollectionPredicateModel::new(
            Some(CollectionContentsPredicateModel::of(vec![
                ValuePredicateModel::exact("apple"),
                ValuePredicateModel::exact("carrot"),
            ])),
            Some(CollectionCountsPredicateModel::of(vec![
                CollectionCountsPredicateEntry::new(
                    ValuePredicateModel::exact("apple"),
                    IntBoundsModel::exactly(2),
                ),
                CollectionCountsPredicateEntry::new(
                    ValuePredicateModel::Never,
                    IntBoundsModel::exactly(0),
                ),
            ])),
            Some(IntBoundsModel::between(3, 4)),
        );

        assert!(predicate.test(&values(&["apple", "apple", "carrot"])));
        assert!(!predicate.test(&values(&["apple", "apple"])));
        assert!(!predicate.test(&values(&["apple", "carrot"])));
        assert!(!predicate.test(&values(&["apple", "apple", "carrot", "stone", "dirt"])));
    }

    #[test]
    fn collection_predicate_omitted_sections_default_to_any_like_java() {
        let predicate = CollectionPredicateModel::new(None, None, Some(IntBoundsModel::any()));

        assert!(predicate.test(&[]));
        assert!(predicate.test(&values(&["anything"])));
    }
}
