use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDurabilityTriggerInstanceModel {
    pub player_predicate_present: bool,
    pub item: Option<ItemPredicateModel>,
    pub durability: IntBoundsModel,
    pub delta: IntBoundsModel,
}

impl ItemDurabilityTriggerInstanceModel {
    pub fn new(
        player_predicate_present: bool,
        item: Option<ItemPredicateModel>,
        durability: IntBoundsModel,
        delta: IntBoundsModel,
    ) -> Self {
        Self {
            player_predicate_present,
            item,
            durability,
            delta,
        }
    }

    pub fn changed_durability(
        item: Option<ItemPredicateModel>,
        durability: IntBoundsModel,
    ) -> ItemDurabilityCriterionModel {
        Self::changed_durability_with_player(false, item, durability)
    }

    pub fn changed_durability_with_player(
        player_predicate_present: bool,
        item: Option<ItemPredicateModel>,
        durability: IntBoundsModel,
    ) -> ItemDurabilityCriterionModel {
        ItemDurabilityCriterionModel {
            trigger_id: trigger_id(),
            instance: Self::new(
                player_predicate_present,
                item,
                durability,
                IntBoundsModel::ANY,
            ),
        }
    }

    pub fn matches(&self, item_stack: &ItemStackModel, new_durability: i32) -> bool {
        if self
            .item
            .as_ref()
            .is_some_and(|predicate| !predicate.test(item_stack))
        {
            return false;
        }

        if !self
            .durability
            .matches(item_stack.max_damage - new_durability)
        {
            return false;
        }

        self.delta.matches(item_stack.damage_value - new_durability)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDurabilityCriterionModel {
    pub trigger_id: Identifier,
    pub instance: ItemDurabilityTriggerInstanceModel,
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:item_durability_changed").unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateModel {
    item: Identifier,
}

impl ItemPredicateModel {
    pub fn item(item: Identifier) -> Self {
        Self { item }
    }

    pub fn test(&self, stack: &ItemStackModel) -> bool {
        self.item == stack.item
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackModel {
    item: Identifier,
    max_damage: i32,
    damage_value: i32,
}

impl ItemStackModel {
    pub fn new(item: Identifier, max_damage: i32, damage_value: i32) -> Self {
        Self {
            item,
            max_damage,
            damage_value,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn stack(item: &str, max_damage: i32, damage_value: i32) -> ItemStackModel {
        ItemStackModel::new(id(item), max_damage, damage_value)
    }

    #[test]
    fn omitted_item_and_any_bounds_match_any_durability_change() {
        let instance = ItemDurabilityTriggerInstanceModel::new(
            false,
            None,
            IntBoundsModel::any(),
            IntBoundsModel::any(),
        );

        assert!(instance.matches(&stack("minecraft:diamond_pickaxe", 1561, 42), 57));
    }

    #[test]
    fn item_predicate_is_checked_before_durability_bounds() {
        let instance = ItemDurabilityTriggerInstanceModel::new(
            false,
            Some(ItemPredicateModel::item(id("minecraft:diamond_pickaxe"))),
            IntBoundsModel::exactly(1551),
            IntBoundsModel::exactly(10),
        );

        assert!(instance.matches(&stack("minecraft:diamond_pickaxe", 1561, 20), 10));
        assert!(!instance.matches(&stack("minecraft:iron_pickaxe", 250, 20), 10));
    }

    #[test]
    fn durability_uses_max_damage_minus_new_durability() {
        let instance = ItemDurabilityTriggerInstanceModel::new(
            false,
            None,
            IntBoundsModel::between(1490, 1500),
            IntBoundsModel::any(),
        );

        assert!(instance.matches(&stack("minecraft:diamond_pickaxe", 1561, 20), 61));
        assert!(!instance.matches(&stack("minecraft:diamond_pickaxe", 1561, 20), 80));
    }

    #[test]
    fn delta_uses_previous_damage_value_minus_new_durability() {
        let instance = ItemDurabilityTriggerInstanceModel::new(
            false,
            None,
            IntBoundsModel::any(),
            IntBoundsModel::exactly(7),
        );

        assert!(instance.matches(&stack("minecraft:diamond_pickaxe", 1561, 20), 13));
        assert!(!instance.matches(&stack("minecraft:diamond_pickaxe", 1561, 20), 12));
    }

    #[test]
    fn changed_durability_factory_preserves_trigger_id_and_uses_any_delta() {
        let criterion = ItemDurabilityTriggerInstanceModel::changed_durability(
            Some(ItemPredicateModel::item(id("minecraft:elytra"))),
            IntBoundsModel::at_most(100),
        );

        assert_eq!(
            criterion.trigger_id,
            id("minecraft:item_durability_changed")
        );
        assert!(!criterion.instance.player_predicate_present);
        assert!(criterion.instance.item.is_some());
        assert_eq!(criterion.instance.durability, IntBoundsModel::at_most(100));
        assert_eq!(criterion.instance.delta, IntBoundsModel::ANY);
    }

    #[test]
    fn changed_durability_player_overload_preserves_player_predicate() {
        let criterion = ItemDurabilityTriggerInstanceModel::changed_durability_with_player(
            true,
            None,
            IntBoundsModel::at_least(1),
        );

        assert!(criterion.instance.player_predicate_present);
        assert!(criterion.instance.item.is_none());
        assert_eq!(criterion.instance.durability, IntBoundsModel::at_least(1));
    }
}
