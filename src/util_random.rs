#![allow(dead_code)]

use crate::random_source::{random_source_next_i32, RandomSourceKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Weighted<T> {
    value: T,
    weight: i32,
}

impl<T> Weighted<T> {
    pub fn new(value: T, weight: i32) -> Result<Self, String> {
        if weight < 0 {
            return Err("Weight should be >= 0".to_string());
        }
        Ok(Self { value, weight })
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn weight(&self) -> i32 {
        self.weight
    }

    pub fn map<U>(&self, mapper: impl FnOnce(&T) -> U) -> Weighted<U> {
        Weighted {
            value: mapper(&self.value),
            weight: self.weight,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedList<T> {
    total_weight: i32,
    items: Vec<Weighted<T>>,
    selector: Selector<T>,
}

impl<T: Clone> WeightedList<T> {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            total_weight: 0,
            items: Vec::new(),
            selector: Selector::Empty,
        }
    }

    #[allow(dead_code)]
    pub fn single(value: T) -> Self {
        Self {
            total_weight: 1,
            items: vec![Weighted {
                value: value.clone(),
                weight: 1,
            }],
            selector: Selector::Flat(vec![Some(value)]),
        }
    }

    pub fn builder() -> WeightedListBuilder<T> {
        WeightedListBuilder { items: Vec::new() }
    }

    pub fn of(items: Vec<Weighted<T>>) -> Result<Self, String> {
        let total_weight = get_total_weight(&items, Weighted::weight)?;
        let selector = if total_weight == 0 {
            Selector::Empty
        } else if total_weight < 64 {
            Selector::Flat(flatten_entries(&items, total_weight as usize))
        } else {
            Selector::Compact
        };
        Ok(Self {
            total_weight,
            items,
            selector,
        })
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.selector, Selector::Empty)
    }

    pub fn map<U: Clone>(&self, mapper: impl Fn(&T) -> U) -> WeightedList<U> {
        let mapped = self.items.iter().map(|entry| entry.map(&mapper)).collect();
        WeightedList::from_validated(mapped, self.total_weight)
    }

    pub fn get_random(&self, random: &mut RandomSourceKind) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        let selection = random_source_next_i32(random, self.total_weight);
        self.get_selected(selection)
    }

    pub fn get_random_or_throw(&self, random: &mut RandomSourceKind) -> Result<&T, String> {
        self.get_random(random)
            .ok_or_else(|| "Weighted list has no elements".to_string())
    }

    pub fn unwrap(&self) -> &[Weighted<T>] {
        &self.items
    }

    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.items.iter().any(|item| item.value() == value)
    }

    fn get_selected(&self, selection: i32) -> Option<&T> {
        match &self.selector {
            Selector::Empty => None,
            Selector::Flat(entries) => entries.get(selection as usize).and_then(Option::as_ref),
            Selector::Compact => get_weighted_item(&self.items, selection, Weighted::weight)
                .map(Weighted::value),
        }
    }

    fn from_validated(items: Vec<Weighted<T>>, total_weight: i32) -> Self {
        let selector = if total_weight == 0 {
            Selector::Empty
        } else if total_weight < 64 {
            Selector::Flat(flatten_entries(&items, total_weight as usize))
        } else {
            Selector::Compact
        };
        Self {
            total_weight,
            items,
            selector,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Selector<T> {
    Empty,
    Flat(Vec<Option<T>>),
    Compact,
}

pub struct WeightedListBuilder<T> {
    items: Vec<Weighted<T>>,
}

impl<T: Clone> WeightedListBuilder<T> {
    pub fn add(mut self, item: T) -> Self {
        self.items.push(Weighted {
            value: item,
            weight: 1,
        });
        self
    }

    pub fn add_weighted(mut self, item: T, weight: i32) -> Result<Self, String> {
        self.items.push(Weighted::new(item, weight)?);
        Ok(self)
    }

    pub fn build(self) -> Result<WeightedList<T>, String> {
        WeightedList::of(self.items)
    }
}

fn flatten_entries<T: Clone>(entries: &[Weighted<T>], total_weight: usize) -> Vec<Option<T>> {
    let mut result = vec![None; total_weight];
    let mut index = 0;
    for entry in entries {
        for slot in &mut result[index..index + entry.weight() as usize] {
            *slot = Some(entry.value().clone());
        }
        index += entry.weight() as usize;
    }
    result
}

pub fn get_total_weight<T>(
    items: &[T],
    weight_getter: impl Fn(&T) -> i32,
) -> Result<i32, String> {
    let mut total_weight = 0_i64;
    for item in items {
        total_weight += i64::from(weight_getter(item));
    }
    if total_weight > i64::from(i32::MAX) {
        Err("Sum of weights must be <= 2147483647".to_string())
    } else {
        Ok(total_weight as i32)
    }
}

pub fn get_random_item<'a, T>(
    random: &mut RandomSourceKind,
    items: &'a [T],
    total_weight: i32,
    weight_getter: impl Fn(&T) -> i32,
) -> Result<Option<&'a T>, String> {
    if total_weight < 0 {
        return Err("Negative total weight in getRandomItem".to_string());
    }
    if total_weight == 0 {
        return Ok(None);
    }
    let selection = random_source_next_i32(random, total_weight);
    Ok(get_weighted_item(items, selection, weight_getter))
}

#[allow(dead_code)]
pub fn get_random_item_with_total<'a, T>(
    random: &mut RandomSourceKind,
    items: &'a [T],
    weight_getter: impl Fn(&T) -> i32 + Copy,
) -> Result<Option<&'a T>, String> {
    let total_weight = get_total_weight(items, weight_getter)?;
    get_random_item(random, items, total_weight, weight_getter)
}

pub fn get_weighted_item<T>(
    items: &[T],
    mut index: i32,
    weight_getter: impl Fn(&T) -> i32,
) -> Option<&T> {
    for item in items {
        index -= weight_getter(item);
        if index < 0 {
            return Some(item);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random_source::{RandomAlgorithm, RandomSourceKind};
    use std::fmt::Debug;

    fn must_ok<T, E: Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("expected Ok(..), got Err({err:?})"),
        }
    }

    fn must_err<T: Debug>(result: Result<T, String>) -> String {
        match result {
            Ok(value) => panic!("expected Err(..), got Ok({value:?})"),
            Err(err) => err,
        }
    }

    #[test]
    fn weighted_rejects_negative_weight_and_maps_value() {
        let source = vibecraft_java_source!("/net/minecraft/util/random/Weighted.java");
        assert!(source.contains("if (weight < 0)"));
        assert!(source.contains("new IllegalArgumentException(\"Weight should be >= 0\")"));
        assert!(source.contains("public <U> Weighted<U> map"));

        assert_eq!(must_err(Weighted::new("bad", -1)), "Weight should be >= 0");
        let mapped = must_ok(Weighted::new("stone", 3)).map(|value| value.len());
        assert_eq!(*mapped.value(), 5);
        assert_eq!(mapped.weight(), 3);
    }

    #[test]
    fn weighted_random_total_weight_and_selection_match_java_walk() {
        let source = vibecraft_java_source!("/net/minecraft/util/random/WeightedRandom.java");
        assert!(source.contains("long totalWeight = 0L"));
        assert!(source.contains("Sum of weights must be <= 2147483647"));
        assert!(source.contains("Negative total weight in getRandomItem"));
        assert!(source.contains("index -= weightGetter.applyAsInt(item)"));

        let items = [("zero", 0), ("first", 2), ("second", 3)];
        assert_eq!(get_total_weight(&items, |item| item.1), Ok(5));
        assert_eq!(
            get_weighted_item(&items, 0, |item| item.1).map(|item| item.0),
            Some("first")
        );
        assert_eq!(
            get_weighted_item(&items, 1, |item| item.1).map(|item| item.0),
            Some("first")
        );
        assert_eq!(
            get_weighted_item(&items, 2, |item| item.1).map(|item| item.0),
            Some("second")
        );
        assert_eq!(get_weighted_item(&items, 5, |item| item.1), None);

        let overflow = [((), i32::MAX), ((), 1)];
        assert_eq!(
            must_err(get_total_weight(&overflow, |item| item.1)),
            "Sum of weights must be <= 2147483647"
        );
        let mut random = RandomSourceKind::new(1, RandomAlgorithm::Legacy);
        assert_eq!(
            must_err(get_random_item(&mut random, &items, -1, |item| item.1)),
            "Negative total weight in getRandomItem"
        );
        assert!(must_ok(get_random_item(&mut random, &items, 0, |item| item.1)).is_none());
    }

    #[test]
    fn weighted_list_preserves_entries_but_zero_total_is_empty_for_selection() {
        let source = vibecraft_java_source!("/net/minecraft/util/random/WeightedList.java");
        assert!(source.contains("private static final int FLAT_THRESHOLD = 64"));
        assert!(source.contains("if (this.totalWeight == 0)"));
        assert!(source.contains("throw new IllegalStateException(\"Weighted list has no elements\")"));
        assert!(source.contains("Arrays.fill(this.entries, i, i + weight, entry.value())"));

        let list = WeightedList::of(vec![
            must_ok(Weighted::new("kept", 0)),
            must_ok(Weighted::new("also kept", 0)),
        ]);
        let list = must_ok(list);

        assert!(list.is_empty());
        assert_eq!(WeightedList::unwrap(&list).len(), 2);
        assert!(list.contains(&"kept"));
        let mut random = RandomSourceKind::new(2, RandomAlgorithm::Legacy);
        assert_eq!(list.get_random(&mut random), None);
        assert_eq!(
            must_err(list.get_random_or_throw(&mut random).copied()),
            "Weighted list has no elements"
        );
    }

    #[test]
    fn weighted_list_selects_flat_and_compact_ranges_like_java() {
        let flat = WeightedList::of(vec![
            must_ok(Weighted::new("a", 1)),
            must_ok(Weighted::new("b", 2)),
            must_ok(Weighted::new("c", 1)),
        ]);
        let flat = must_ok(flat);
        assert_eq!(flat.get_selected(0), Some(&"a"));
        assert_eq!(flat.get_selected(1), Some(&"b"));
        assert_eq!(flat.get_selected(2), Some(&"b"));
        assert_eq!(flat.get_selected(3), Some(&"c"));
        assert_eq!(WeightedList::unwrap(&flat.map(|value| value.len()))[1].weight(), 2);

        let compact = WeightedList::of(vec![
            must_ok(Weighted::new("small", 1)),
            must_ok(Weighted::new("large", 63)),
        ]);
        let compact = must_ok(compact);
        assert_eq!(compact.get_selected(0), Some(&"small"));
        assert_eq!(compact.get_selected(1), Some(&"large"));
        assert_eq!(compact.get_selected(63), Some(&"large"));

        let builder = must_ok(WeightedList::builder().add("one").add_weighted("two", 2));
        let built = must_ok(builder.build());
        assert_eq!(
            WeightedList::unwrap(&built)
                .iter()
                .map(Weighted::weight)
                .sum::<i32>(),
            3
        );
    }
}
