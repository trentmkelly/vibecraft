use std::collections::BTreeSet;

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryChangeTriggerInstanceModel {
    pub player_predicate_present: bool,
    pub slots: InventorySlotsModel,
    pub items: Vec<ItemPredicateModel>,
}

impl InventoryChangeTriggerInstanceModel {
    pub fn new(slots: InventorySlotsModel, items: Vec<ItemPredicateModel>) -> Self {
        Self {
            player_predicate_present: false,
            slots,
            items,
        }
    }

    pub fn has_items(predicates: Vec<ItemPredicateModel>) -> InventoryChangeCriterionModel {
        InventoryChangeCriterionModel {
            trigger_id: trigger_id(),
            instance: Self::new(InventorySlotsModel::ANY, predicates),
        }
    }

    pub fn has_item_likes(
        items: impl IntoIterator<Item = Identifier>,
    ) -> InventoryChangeCriterionModel {
        Self::has_items(items.into_iter().map(ItemPredicateModel::item).collect())
    }

    pub fn matches(
        &self,
        inventory: &InventoryModel,
        changed_item: &ItemStackModel,
        slots_full: i32,
        slots_empty: i32,
        slots_occupied: i32,
    ) -> bool {
        if !self.slots.matches(slots_full, slots_empty, slots_occupied) {
            return false;
        }

        if self.items.is_empty() {
            return true;
        }

        if self.items.len() == 1 {
            return !changed_item.is_empty() && self.items[0].test(changed_item);
        }

        let mut predicates = self.items.clone();
        for stack in &inventory.slots {
            if predicates.is_empty() {
                return true;
            }

            if !stack.is_empty() {
                predicates.retain(|predicate| !predicate.test(stack));
            }
        }

        predicates.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryChangeCriterionModel {
    pub trigger_id: Identifier,
    pub instance: InventoryChangeTriggerInstanceModel,
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:inventory_changed").unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InventorySlotsModel {
    occupied: IntBoundsModel,
    full: IntBoundsModel,
    empty: IntBoundsModel,
}

impl InventorySlotsModel {
    pub const ANY: Self = Self {
        occupied: IntBoundsModel::ANY,
        full: IntBoundsModel::ANY,
        empty: IntBoundsModel::ANY,
    };

    pub fn new(occupied: IntBoundsModel, full: IntBoundsModel, empty: IntBoundsModel) -> Self {
        Self {
            occupied,
            full,
            empty,
        }
    }

    pub fn matches(&self, slots_full: i32, slots_empty: i32, slots_occupied: i32) -> bool {
        if !self.full.matches(slots_full) {
            return false;
        }

        if !self.empty.matches(slots_empty) {
            return false;
        }

        self.occupied.matches(slots_occupied)
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
pub struct ItemPredicateModel {
    items: Option<BTreeSet<Identifier>>,
    count: IntBoundsModel,
}

impl ItemPredicateModel {
    pub fn any() -> Self {
        Self {
            items: None,
            count: IntBoundsModel::ANY,
        }
    }

    pub fn item(item: Identifier) -> Self {
        Self {
            items: Some(BTreeSet::from([item])),
            count: IntBoundsModel::ANY,
        }
    }

    pub fn with_count(mut self, count: IntBoundsModel) -> Self {
        self.count = count;
        self
    }

    pub fn test(&self, stack: &ItemStackModel) -> bool {
        if stack.is_empty() {
            return false;
        }

        if self
            .items
            .as_ref()
            .is_some_and(|items| !items.contains(&stack.item))
        {
            return false;
        }

        self.count.matches(stack.count)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryModel {
    slots: Vec<ItemStackModel>,
}

impl InventoryModel {
    pub fn new(slots: Vec<ItemStackModel>) -> Self {
        Self { slots }
    }

    pub fn slot_counts(&self) -> InventorySlotCountsModel {
        let mut full = 0;
        let mut empty = 0;
        let mut occupied = 0;

        for stack in &self.slots {
            if stack.is_empty() {
                empty += 1;
            } else {
                occupied += 1;
                if stack.count >= stack.max_stack_size {
                    full += 1;
                }
            }
        }

        InventorySlotCountsModel {
            full,
            empty,
            occupied,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InventorySlotCountsModel {
    pub full: i32,
    pub empty: i32,
    pub occupied: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackModel {
    item: Identifier,
    count: i32,
    max_stack_size: i32,
}

impl ItemStackModel {
    pub fn new(item: Identifier, count: i32, max_stack_size: i32) -> Self {
        Self {
            item,
            count,
            max_stack_size,
        }
    }

    pub fn empty() -> Self {
        Self::new(Identifier::parse("minecraft:air").unwrap(), 0, 64)
    }

    pub fn is_empty(&self) -> bool {
        self.count <= 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn stack(item: &str, count: i32, max_stack_size: i32) -> ItemStackModel {
        ItemStackModel::new(id(item), count, max_stack_size)
    }

    fn inventory() -> InventoryModel {
        InventoryModel::new(vec![
            stack("minecraft:stone", 64, 64),
            stack("minecraft:apple", 3, 64),
            ItemStackModel::empty(),
            stack("minecraft:diamond", 1, 64),
        ])
    }

    #[test]
    fn trigger_counts_empty_occupied_and_full_slots_like_java() {
        let counts = inventory().slot_counts();

        assert_eq!(
            counts,
            InventorySlotCountsModel {
                full: 1,
                empty: 1,
                occupied: 3,
            }
        );
    }

    #[test]
    fn slots_match_full_then_empty_then_occupied_bounds() {
        let slots = InventorySlotsModel::new(
            IntBoundsModel::exactly(3),
            IntBoundsModel::exactly(1),
            IntBoundsModel::exactly(1),
        );
        let counts = inventory().slot_counts();

        assert!(slots.matches(counts.full, counts.empty, counts.occupied));
        assert!(!slots.matches(0, counts.empty, counts.occupied));
        assert!(!slots.matches(counts.full, 0, counts.occupied));
        assert!(!slots.matches(counts.full, counts.empty, 0));
    }

    #[test]
    fn empty_item_predicate_list_matches_after_slots_match() {
        let instance = InventoryChangeTriggerInstanceModel::new(InventorySlotsModel::ANY, vec![]);
        let counts = inventory().slot_counts();

        assert!(instance.matches(
            &inventory(),
            &ItemStackModel::empty(),
            counts.full,
            counts.empty,
            counts.occupied,
        ));

        let impossible_slots = InventorySlotsModel::new(
            IntBoundsModel::exactly(99),
            IntBoundsModel::ANY,
            IntBoundsModel::ANY,
        );
        let instance = InventoryChangeTriggerInstanceModel::new(impossible_slots, vec![]);
        assert!(!instance.matches(
            &inventory(),
            &stack("minecraft:stone", 64, 64),
            counts.full,
            counts.empty,
            counts.occupied,
        ));
    }

    #[test]
    fn single_item_predicate_tests_changed_item_only() {
        let instance = InventoryChangeTriggerInstanceModel::new(
            InventorySlotsModel::ANY,
            vec![ItemPredicateModel::item(id("minecraft:emerald"))],
        );
        let counts = inventory().slot_counts();

        assert!(instance.matches(
            &inventory(),
            &stack("minecraft:emerald", 1, 64),
            counts.full,
            counts.empty,
            counts.occupied,
        ));
        assert!(!instance.matches(
            &inventory(),
            &ItemStackModel::empty(),
            counts.full,
            counts.empty,
            counts.occupied,
        ));
        assert!(!instance.matches(
            &InventoryModel::new(vec![stack("minecraft:emerald", 1, 64)]),
            &stack("minecraft:apple", 1, 64),
            counts.full,
            counts.empty,
            counts.occupied,
        ));
    }

    #[test]
    fn multiple_item_predicates_scan_inventory_and_ignore_changed_item() {
        let instance = InventoryChangeTriggerInstanceModel::new(
            InventorySlotsModel::ANY,
            vec![
                ItemPredicateModel::item(id("minecraft:stone")),
                ItemPredicateModel::item(id("minecraft:diamond")),
            ],
        );
        let counts = inventory().slot_counts();

        assert!(instance.matches(
            &inventory(),
            &stack("minecraft:stick", 1, 64),
            counts.full,
            counts.empty,
            counts.occupied,
        ));
        assert!(!instance.matches(
            &InventoryModel::new(vec![stack("minecraft:stone", 64, 64)]),
            &stack("minecraft:diamond", 1, 64),
            counts.full,
            counts.empty,
            counts.occupied,
        ));
    }

    #[test]
    fn one_inventory_stack_can_remove_multiple_matching_predicates() {
        let instance = InventoryChangeTriggerInstanceModel::new(
            InventorySlotsModel::ANY,
            vec![
                ItemPredicateModel::item(id("minecraft:stone")),
                ItemPredicateModel::any().with_count(IntBoundsModel::at_least(32)),
            ],
        );
        let one_stack_inventory = InventoryModel::new(vec![stack("minecraft:stone", 64, 64)]);
        let counts = one_stack_inventory.slot_counts();

        assert!(instance.matches(
            &one_stack_inventory,
            &stack("minecraft:stick", 1, 64),
            counts.full,
            counts.empty,
            counts.occupied,
        ));
    }

    #[test]
    fn has_item_likes_factory_uses_inventory_changed_trigger_and_direct_item_predicates() {
        let criterion = InventoryChangeTriggerInstanceModel::has_item_likes([
            id("minecraft:apple"),
            id("minecraft:diamond"),
        ]);
        let counts = inventory().slot_counts();

        assert_eq!(criterion.trigger_id, id("minecraft:inventory_changed"));
        assert!(!criterion.instance.player_predicate_present);
        assert_eq!(criterion.instance.slots, InventorySlotsModel::ANY);
        assert!(criterion.instance.matches(
            &inventory(),
            &stack("minecraft:stick", 1, 64),
            counts.full,
            counts.empty,
            counts.occupied,
        ));
    }

    #[test]
    fn item_predicate_count_bounds_are_applied_to_non_empty_stacks() {
        let predicate = ItemPredicateModel::item(id("minecraft:apple"))
            .with_count(IntBoundsModel::between(2, 4));

        assert!(predicate.test(&stack("minecraft:apple", 3, 64)));
        assert!(!predicate.test(&stack("minecraft:apple", 1, 64)));
        assert!(!predicate.test(&ItemStackModel::empty()));
        assert!(IntBoundsModel::at_most(4).matches(3));
    }
}
