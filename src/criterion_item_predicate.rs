use std::collections::{BTreeMap, BTreeSet};

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateModel {
    pub items: Option<BTreeSet<Identifier>>,
    pub count: IntBoundsModel,
    pub components: DataComponentMatchersModel,
}

impl ItemPredicateModel {
    pub fn new(
        items: Option<BTreeSet<Identifier>>,
        count: IntBoundsModel,
        components: DataComponentMatchersModel,
    ) -> Self {
        Self {
            items,
            count,
            components,
        }
    }

    pub fn builder() -> ItemPredicateBuilderModel {
        ItemPredicateBuilderModel::item()
    }

    pub fn test(&self, item_stack: &ItemStackModel) -> bool {
        if self
            .items
            .as_ref()
            .is_some_and(|items| !items.contains(&item_stack.item))
        {
            return false;
        }

        if !self.count.matches(item_stack.count) {
            return false;
        }

        self.components.test(item_stack)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateBuilderModel {
    items: Option<BTreeSet<Identifier>>,
    count: IntBoundsModel,
    components: DataComponentMatchersModel,
}

impl ItemPredicateBuilderModel {
    pub fn item() -> Self {
        Self {
            items: None,
            count: IntBoundsModel::ANY,
            components: DataComponentMatchersModel::any(),
        }
    }

    pub fn of_items(mut self, items: impl IntoIterator<Item = Identifier>) -> Self {
        self.items = Some(items.into_iter().collect());
        self
    }

    pub fn of_tag(mut self, lookup: &TagLookupModel, tag: &str) -> Result<Self, String> {
        self.items = Some(lookup.get_or_throw(tag)?);
        Ok(self)
    }

    pub fn with_count(mut self, count: IntBoundsModel) -> Self {
        self.count = count;
        self
    }

    pub fn with_components(mut self, components: DataComponentMatchersModel) -> Self {
        self.components = components;
        self
    }

    pub fn build(self) -> ItemPredicateModel {
        ItemPredicateModel::new(self.items, self.count, self.components)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagLookupModel {
    tags: BTreeMap<String, BTreeSet<Identifier>>,
}

impl TagLookupModel {
    pub fn new(tags: impl IntoIterator<Item = (&'static str, Vec<Identifier>)>) -> Self {
        Self {
            tags: tags
                .into_iter()
                .map(|(tag, items)| (tag.to_string(), items.into_iter().collect()))
                .collect(),
        }
    }

    fn get_or_throw(&self, tag: &str) -> Result<BTreeSet<Identifier>, String> {
        self.tags
            .get(tag)
            .cloned()
            .ok_or_else(|| format!("missing tag {tag}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackModel {
    item: Identifier,
    count: i32,
    components: BTreeMap<String, String>,
}

impl ItemStackModel {
    pub fn new(
        item: Identifier,
        count: i32,
        components: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> Self {
        Self {
            item,
            count,
            components: components
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntBoundsModel {
    min: Option<i32>,
    max: Option<i32>,
}

impl IntBoundsModel {
    pub const ANY: Self = Self {
        min: None,
        max: None,
    };

    pub fn any() -> Self {
        Self::ANY
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

    pub fn at_most(value: i32) -> Self {
        Self {
            min: None,
            max: Some(value),
        }
    }

    pub fn matches(&self, value: i32) -> bool {
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| max >= value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataComponentMatchersModel {
    required: BTreeMap<String, String>,
}

impl DataComponentMatchersModel {
    pub const ANY: Self = Self {
        required: BTreeMap::new(),
    };

    pub fn any() -> Self {
        Self::ANY
    }

    pub fn requiring(component: &str, value: &str) -> Self {
        Self {
            required: BTreeMap::from([(component.to_string(), value.to_string())]),
        }
    }

    pub fn test(&self, item_stack: &ItemStackModel) -> bool {
        self.required
            .iter()
            .all(|(key, value)| item_stack.components.get(key) == Some(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn stack(
        item: &str,
        count: i32,
        components: &[(&'static str, &'static str)],
    ) -> ItemStackModel {
        ItemStackModel::new(id(item), count, components.iter().copied())
    }

    #[test]
    fn default_builder_matches_any_item_count_and_components() {
        let predicate = ItemPredicateModel::builder().build();

        assert_eq!(predicate.items, None);
        assert_eq!(predicate.count, IntBoundsModel::ANY);
        assert_eq!(predicate.components, DataComponentMatchersModel::ANY);
        assert!(predicate.test(&stack("minecraft:stone", 1, &[])));
        assert!(predicate.test(&stack(
            "minecraft:diamond",
            64,
            &[("minecraft:custom_name", "Gem")]
        )));
    }

    #[test]
    fn item_set_is_checked_before_count_and_components() {
        let predicate = ItemPredicateModel::builder()
            .of_items([id("minecraft:apple"), id("minecraft:bread")])
            .with_count(IntBoundsModel::exactly(2))
            .with_components(DataComponentMatchersModel::requiring(
                "minecraft:custom_name",
                "Snack",
            ))
            .build();

        assert!(predicate.test(&stack(
            "minecraft:apple",
            2,
            &[("minecraft:custom_name", "Snack")]
        )));
        assert!(!predicate.test(&stack(
            "minecraft:carrot",
            2,
            &[("minecraft:custom_name", "Snack")]
        )));
        assert!(!predicate.test(&stack(
            "minecraft:apple",
            3,
            &[("minecraft:custom_name", "Snack")]
        )));
        assert!(!predicate.test(&stack("minecraft:apple", 2, &[])));
    }

    #[test]
    fn count_bounds_support_any_exact_range_lower_and_upper_shapes() {
        assert!(IntBoundsModel::any().matches(i32::MAX));
        assert!(IntBoundsModel::exactly(4).matches(4));
        assert!(!IntBoundsModel::exactly(4).matches(5));
        assert!(IntBoundsModel::between(2, 5).matches(3));
        assert!(!IntBoundsModel::between(2, 5).matches(6));
        assert!(IntBoundsModel::at_least(2).matches(99));
        assert!(!IntBoundsModel::at_least(2).matches(1));
        assert!(IntBoundsModel::at_most(2).matches(1));
        assert!(!IntBoundsModel::at_most(2).matches(3));
    }

    #[test]
    fn direct_item_builder_uses_homogeneous_holder_set_membership() {
        let predicate = ItemPredicateModel::builder()
            .of_items([id("minecraft:oak_log"), id("minecraft:spruce_log")])
            .build();

        assert!(predicate.test(&stack("minecraft:oak_log", 1, &[])));
        assert!(predicate.test(&stack("minecraft:spruce_log", 1, &[])));
        assert!(!predicate.test(&stack("minecraft:birch_log", 1, &[])));
    }

    #[test]
    fn tag_builder_uses_lookup_get_or_throw_behavior() {
        let lookup = TagLookupModel::new([(
            "minecraft:logs",
            vec![id("minecraft:oak_log"), id("minecraft:spruce_log")],
        )]);
        let predicate = ItemPredicateModel::builder()
            .of_tag(&lookup, "minecraft:logs")
            .unwrap()
            .build();

        assert!(predicate.test(&stack("minecraft:oak_log", 1, &[])));
        assert!(!predicate.test(&stack("minecraft:stone", 1, &[])));
        assert_eq!(
            ItemPredicateModel::builder()
                .of_tag(&lookup, "minecraft:missing")
                .unwrap_err(),
            "missing tag minecraft:missing"
        );
    }

    #[test]
    fn component_matchers_are_delegated_after_item_and_count_match() {
        let predicate = ItemPredicateModel::builder()
            .with_count(IntBoundsModel::between(1, 3))
            .with_components(DataComponentMatchersModel::requiring(
                "minecraft:custom_data",
                "present",
            ))
            .build();

        assert!(predicate.test(&stack(
            "minecraft:bundle",
            1,
            &[("minecraft:custom_data", "present")]
        )));
        assert!(!predicate.test(&stack("minecraft:bundle", 1, &[])));
    }
}
