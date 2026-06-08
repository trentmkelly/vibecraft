use std::collections::BTreeMap;

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumeItemTriggerInstance {
    pub player_predicate_present: bool,
    pub item: Option<ItemPredicateModel>,
}

impl ConsumeItemTriggerInstance {
    pub fn new(item: Option<ItemPredicateModel>) -> Self {
        Self {
            player_predicate_present: false,
            item,
        }
    }

    pub fn matches(&self, item_stack: &ItemStackModel) -> bool {
        self.item
            .as_ref()
            .is_none_or(|predicate| predicate.test(item_stack))
    }

    pub fn used_item() -> ConsumeItemCriterion {
        ConsumeItemCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(None),
        }
    }

    pub fn used_item_item(item: Identifier) -> ConsumeItemCriterion {
        Self::used_item_predicate(ItemPredicateModel::item(item))
    }

    pub fn used_item_predicate(predicate: ItemPredicateModel) -> ConsumeItemCriterion {
        ConsumeItemCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(Some(predicate)),
        }
    }
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:consume_item").unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumeItemCriterion {
    pub trigger_id: Identifier,
    pub instance: ConsumeItemTriggerInstance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackModel {
    pub item: Identifier,
    pub count: i32,
    pub components: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateModel {
    pub items: Option<Vec<Identifier>>,
    pub count: IntBoundsModel,
    pub components: DataComponentMatchersModel,
}

impl ItemPredicateModel {
    pub fn any() -> Self {
        Self {
            items: None,
            count: IntBoundsModel::any(),
            components: DataComponentMatchersModel::any(),
        }
    }

    pub fn item(item: Identifier) -> Self {
        Self {
            items: Some(vec![item]),
            ..Self::any()
        }
    }

    pub fn with_count(mut self, count: IntBoundsModel) -> Self {
        self.count = count;
        self
    }

    pub fn with_components(mut self, components: DataComponentMatchersModel) -> Self {
        self.components = components;
        self
    }

    pub fn test(&self, item_stack: &ItemStackModel) -> bool {
        if self
            .items
            .as_ref()
            .is_some_and(|items| !items.contains(&item_stack.item))
        {
            return false;
        }

        self.count.matches(item_stack.count) && self.components.test(&item_stack.components)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataComponentMatchersModel {
    required: BTreeMap<String, String>,
}

impl DataComponentMatchersModel {
    pub fn any() -> Self {
        Self::default()
    }

    pub fn requiring(component: &str, value: &str) -> Self {
        Self {
            required: BTreeMap::from([(component.to_string(), value.to_string())]),
        }
    }

    pub fn test(&self, components: &BTreeMap<String, String>) -> bool {
        self.required
            .iter()
            .all(|(key, value)| components.get(key) == Some(value))
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

    pub fn matches(&self, value: i32) -> bool {
        self.min.is_none_or(|min| value >= min) && self.max.is_none_or(|max| value <= max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn stack(item: &str, count: i32, components: &[(&str, &str)]) -> ItemStackModel {
        ItemStackModel {
            item: id(item),
            count,
            components: components
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        }
    }

    #[test]
    fn consume_item_omitted_item_predicate_matches_any_stack_like_java() {
        let instance = ConsumeItemTriggerInstance::new(None);

        assert!(instance.matches(&stack("minecraft:apple", 1, &[])));
        assert!(instance.matches(&stack("minecraft:potion", 64, &[("custom", "value")])));
    }

    #[test]
    fn consume_item_predicate_checks_items_count_and_components_like_java() {
        let predicate = ItemPredicateModel::item(id("minecraft:golden_apple"))
            .with_count(IntBoundsModel::between(1, 2))
            .with_components(DataComponentMatchersModel::requiring(
                "minecraft:custom_name",
                "Snack",
            ));
        let instance = ConsumeItemTriggerInstance::new(Some(predicate));

        assert!(instance.matches(&stack(
            "minecraft:golden_apple",
            1,
            &[("minecraft:custom_name", "Snack")]
        )));
        assert!(!instance.matches(&stack(
            "minecraft:apple",
            1,
            &[("minecraft:custom_name", "Snack")]
        )));
        assert!(!instance.matches(&stack(
            "minecraft:golden_apple",
            3,
            &[("minecraft:custom_name", "Snack")]
        )));
        assert!(!instance.matches(&stack("minecraft:golden_apple", 1, &[])));
    }

    #[test]
    fn consume_item_item_predicate_defaults_count_and_components_to_any_like_java() {
        let predicate = ItemPredicateModel::item(id("minecraft:honey_bottle"));

        assert!(predicate.test(&stack("minecraft:honey_bottle", 1, &[])));
        assert!(predicate.test(&stack(
            "minecraft:honey_bottle",
            64,
            &[("minecraft:custom_data", "present")]
        )));
        assert!(!predicate.test(&stack("minecraft:milk_bucket", 1, &[])));

        let any_count = ItemPredicateModel::any().with_count(IntBoundsModel::exactly(7));
        assert!(any_count.test(&stack("minecraft:cookie", 7, &[])));
        assert!(!any_count.test(&stack("minecraft:cookie", 6, &[])));
    }

    #[test]
    fn consume_item_factories_use_java_trigger_id_and_fields() {
        let any = ConsumeItemTriggerInstance::used_item();
        assert_eq!(any.trigger_id, id("minecraft:consume_item"));
        assert_eq!(any.instance, ConsumeItemTriggerInstance::new(None));

        let item = ConsumeItemTriggerInstance::used_item_item(id("minecraft:apple"));
        assert_eq!(item.trigger_id, id("minecraft:consume_item"));
        assert_eq!(
            item.instance.item,
            Some(ItemPredicateModel::item(id("minecraft:apple")))
        );

        let predicate =
            ItemPredicateModel::item(id("minecraft:bread")).with_count(IntBoundsModel::exactly(1));
        let predicate_factory = ConsumeItemTriggerInstance::used_item_predicate(predicate.clone());
        assert_eq!(predicate_factory.trigger_id, id("minecraft:consume_item"));
        assert_eq!(predicate_factory.instance.item, Some(predicate));
    }
}
