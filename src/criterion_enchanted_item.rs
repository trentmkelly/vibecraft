use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantedItemTriggerInstance {
    pub player_predicate_present: bool,
    pub item: Option<ItemPredicateModel>,
    pub levels: IntBoundsModel,
}

impl EnchantedItemTriggerInstance {
    pub fn new(item: Option<ItemPredicateModel>, levels: IntBoundsModel) -> Self {
        Self {
            player_predicate_present: false,
            item,
            levels,
        }
    }

    pub fn matches(&self, item_stack: &ItemStackModel, levels: i32) -> bool {
        if self
            .item
            .as_ref()
            .is_some_and(|item| !item.test(item_stack))
        {
            return false;
        }

        self.levels.matches(levels)
    }

    pub fn enchanted_item() -> EnchantedItemCriterion {
        EnchantedItemCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(None, IntBoundsModel::any()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantedItemCriterion {
    pub trigger_id: Identifier,
    pub instance: EnchantedItemTriggerInstance,
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:enchanted_item").unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackModel {
    pub item: Identifier,
}

impl ItemStackModel {
    pub fn new(item: Identifier) -> Self {
        Self { item }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateModel {
    items: Option<Vec<Identifier>>,
}

impl ItemPredicateModel {
    pub fn any() -> Self {
        Self { items: None }
    }

    pub fn item(item: Identifier) -> Self {
        Self {
            items: Some(vec![item]),
        }
    }

    pub fn test(&self, item_stack: &ItemStackModel) -> bool {
        self.items
            .as_ref()
            .is_none_or(|items| items.contains(&item_stack.item))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IntBoundsModel {
    min: Option<i32>,
    max: Option<i32>,
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
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| max >= value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn stack(item: &str) -> ItemStackModel {
        ItemStackModel::new(id(item))
    }

    #[test]
    fn omitted_item_and_levels_match_any_enchanted_item_event() {
        let instance = EnchantedItemTriggerInstance::new(None, IntBoundsModel::any());

        assert!(instance.matches(&stack("minecraft:diamond_sword"), 30));
        assert!(instance.matches(&stack("minecraft:book"), 1));
    }

    #[test]
    fn item_predicate_is_checked_before_level_bounds_like_java() {
        let instance = EnchantedItemTriggerInstance::new(
            Some(ItemPredicateModel::item(id("minecraft:diamond_sword"))),
            IntBoundsModel::between(10, 20),
        );

        assert!(instance.matches(&stack("minecraft:diamond_sword"), 15));
        assert!(!instance.matches(&stack("minecraft:book"), 15));
        assert!(!instance.matches(&stack("minecraft:diamond_sword"), 9));
        assert!(!instance.matches(&stack("minecraft:diamond_sword"), 21));
    }

    #[test]
    fn exact_level_bounds_match_single_enchantment_cost() {
        let instance = EnchantedItemTriggerInstance::new(None, IntBoundsModel::exactly(3));

        assert!(instance.matches(&stack("minecraft:iron_pickaxe"), 3));
        assert!(!instance.matches(&stack("minecraft:iron_pickaxe"), 2));
    }

    #[test]
    fn any_item_predicate_does_not_filter_item_stack() {
        let instance = EnchantedItemTriggerInstance::new(
            Some(ItemPredicateModel::any()),
            IntBoundsModel::any(),
        );

        assert!(instance.matches(&stack("minecraft:stick"), 1));
    }

    #[test]
    fn enchanted_item_factory_uses_java_trigger_id_and_defaults() {
        let criterion = EnchantedItemTriggerInstance::enchanted_item();

        assert_eq!(criterion.trigger_id, id("minecraft:enchanted_item"));
        assert_eq!(
            criterion.instance,
            EnchantedItemTriggerInstance::new(None, IntBoundsModel::any())
        );
        assert!(!criterion.instance.player_predicate_present);
    }
}
