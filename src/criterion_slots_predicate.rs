use std::collections::BTreeMap;

use crate::criterion_item_predicate::{IntBoundsModel, ItemPredicateModel, ItemStackModel};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotsPredicateModel {
    slots: Vec<(SlotRangeModel, SlotItemPredicateModel)>,
}

impl SlotsPredicateModel {
    pub fn new(slots: impl IntoIterator<Item = (SlotRangeModel, SlotItemPredicateModel)>) -> Self {
        Self {
            slots: slots.into_iter().collect(),
        }
    }

    pub fn codec_key_value_types() -> (&'static str, &'static str) {
        ("SlotRanges.CODEC", "ItemPredicate.CODEC")
    }

    pub fn slots(&self) -> &[(SlotRangeModel, SlotItemPredicateModel)] {
        &self.slots
    }

    pub fn matches(&self, slot_provider: &SlotProviderModel) -> bool {
        self.slots
            .iter()
            .all(|(range, predicate)| Self::match_slots(slot_provider, predicate, range.slots()))
    }

    fn match_slots(
        slot_provider: &SlotProviderModel,
        predicate: &SlotItemPredicateModel,
        slots: &[i32],
    ) -> bool {
        for slot_id in slots {
            if let Some(slot) = slot_provider.get_slot(*slot_id) {
                if predicate.test(slot) {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRangeModel {
    name: String,
    slots: Vec<i32>,
}

impl SlotRangeModel {
    pub fn new(name: impl Into<String>, slots: impl IntoIterator<Item = i32>) -> Self {
        Self {
            name: name.into(),
            slots: slots.into_iter().collect(),
        }
    }

    pub fn slots(&self) -> &[i32] {
        &self.slots
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotItemPredicateModel {
    Item(ItemPredicateModel),
    PanicIfEvaluated,
}

impl SlotItemPredicateModel {
    pub fn test(&self, item_stack: &ItemStackModel) -> bool {
        match self {
            Self::Item(predicate) => predicate.test(item_stack),
            Self::PanicIfEvaluated => panic!("predicate should not be evaluated for null slots"),
        }
    }
}

impl From<ItemPredicateModel> for SlotItemPredicateModel {
    fn from(value: ItemPredicateModel) -> Self {
        Self::Item(value)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SlotProviderModel {
    slots: BTreeMap<i32, ItemStackModel>,
}

impl SlotProviderModel {
    pub fn new(slots: impl IntoIterator<Item = (i32, ItemStackModel)>) -> Self {
        Self {
            slots: slots.into_iter().collect(),
        }
    }

    fn get_slot(&self, slot_id: i32) -> Option<&ItemStackModel> {
        self.slots.get(&slot_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn stack(item: &str, count: i32) -> ItemStackModel {
        ItemStackModel::new(id(item), count, [])
    }

    fn item_predicate(item: &str) -> SlotItemPredicateModel {
        ItemPredicateModel::builder()
            .of_items([id(item)])
            .build()
            .into()
    }

    fn counted_item_predicate(item: &str, count: i32) -> SlotItemPredicateModel {
        ItemPredicateModel::builder()
            .of_items([id(item)])
            .with_count(IntBoundsModel::exactly(count))
            .build()
            .into()
    }

    #[test]
    fn slots_predicate_codec_is_unbounded_slot_range_to_item_predicate_map() {
        assert_eq!(
            SlotsPredicateModel::codec_key_value_types(),
            ("SlotRanges.CODEC", "ItemPredicate.CODEC")
        );

        let predicate = SlotsPredicateModel::new([
            (
                SlotRangeModel::new("weapon.mainhand", [98]),
                item_predicate("minecraft:diamond_sword"),
            ),
            (
                SlotRangeModel::new("armor.feet", [100]),
                item_predicate("minecraft:diamond_boots"),
            ),
        ]);

        assert_eq!(predicate.slots().len(), 2);
        assert_eq!(predicate.slots()[0].0.name(), "weapon.mainhand");
        assert_eq!(predicate.slots()[0].0.slots(), &[98]);
        assert_eq!(predicate.slots()[1].0.name(), "armor.feet");
        assert_eq!(predicate.slots()[1].0.slots(), &[100]);
    }

    #[test]
    fn slots_predicate_empty_slot_map_matches_any_provider() {
        let predicate = SlotsPredicateModel::new([]);
        let provider = SlotProviderModel::new([(0, stack("minecraft:dirt", 64))]);

        assert!(predicate.matches(&provider));
        assert!(predicate.matches(&SlotProviderModel::default()));
    }

    #[test]
    fn slots_predicate_requires_every_slot_range_entry_to_match() {
        let predicate = SlotsPredicateModel::new([
            (
                SlotRangeModel::new("weapon.mainhand", [98]),
                item_predicate("minecraft:diamond_sword"),
            ),
            (
                SlotRangeModel::new("armor.feet", [100]),
                item_predicate("minecraft:diamond_boots"),
            ),
        ]);
        let matching_provider = SlotProviderModel::new([
            (98, stack("minecraft:diamond_sword", 1)),
            (100, stack("minecraft:diamond_boots", 1)),
        ]);
        let missing_boots = SlotProviderModel::new([(98, stack("minecraft:diamond_sword", 1))]);

        assert!(predicate.matches(&matching_provider));
        assert!(!predicate.matches(&missing_boots));
    }

    #[test]
    fn slots_predicate_matches_any_slot_inside_a_range() {
        let predicate = SlotsPredicateModel::new([(
            SlotRangeModel::new("container.*", [0, 1, 2]),
            item_predicate("minecraft:diamond"),
        )]);

        assert!(predicate.matches(&SlotProviderModel::new([
            (0, stack("minecraft:cobblestone", 64)),
            (1, stack("minecraft:dirt", 64)),
            (2, stack("minecraft:diamond", 1)),
        ])));
        assert!(!predicate.matches(&SlotProviderModel::new([
            (0, stack("minecraft:cobblestone", 64)),
            (1, stack("minecraft:dirt", 64)),
            (2, stack("minecraft:emerald", 1)),
        ])));
    }

    #[test]
    fn slots_predicate_skips_null_slot_access_without_testing_item_predicate() {
        let only_null_slot = SlotsPredicateModel::new([(
            SlotRangeModel::new("unknown", [10]),
            SlotItemPredicateModel::PanicIfEvaluated,
        )]);
        assert!(!only_null_slot.matches(&SlotProviderModel::default()));

        let null_then_match = SlotsPredicateModel::new([(
            SlotRangeModel::new("mixed", [10, 11]),
            item_predicate("minecraft:diamond"),
        )]);

        assert!(null_then_match.matches(&SlotProviderModel::new([(
            11,
            stack("minecraft:diamond", 1),
        )])));
    }

    #[test]
    fn slots_predicate_delegates_to_item_predicate_with_slot_stack() {
        let predicate = SlotsPredicateModel::new([(
            SlotRangeModel::new("inventory.0", [0]),
            counted_item_predicate("minecraft:diamond", 3),
        )]);

        assert!(predicate.matches(&SlotProviderModel::new([(
            0,
            stack("minecraft:diamond", 3),
        )])));
        assert!(!predicate.matches(&SlotProviderModel::new([(
            0,
            stack("minecraft:diamond", 2),
        )])));
        assert!(!predicate.matches(&SlotProviderModel::new([(
            0,
            stack("minecraft:emerald", 3),
        )])));
    }
}
