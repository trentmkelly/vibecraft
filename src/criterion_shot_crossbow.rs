use crate::criterion_item_predicate::{
    IntBoundsModel, ItemPredicateBuilderModel, ItemPredicateModel, ItemStackModel,
};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShotCrossbowTriggerInstance {
    pub player: Option<ContextAwarePredicateModel>,
    pub item: Option<ItemPredicateModel>,
}

impl ShotCrossbowTriggerInstance {
    pub fn new(
        player: Option<ContextAwarePredicateModel>,
        item: Option<ItemPredicateModel>,
    ) -> Self {
        Self { player, item }
    }

    pub fn codec_field_names() -> [&'static str; 2] {
        ["player", "item"]
    }

    pub fn shot_crossbow(item: Option<ItemPredicateModel>) -> ShotCrossbowCriterion {
        ShotCrossbowCriterion {
            trigger_id: id("minecraft:shot_crossbow"),
            instance: Self::new(None, item),
        }
    }

    pub fn shot_crossbow_item(item: Identifier) -> ShotCrossbowCriterion {
        ShotCrossbowCriterion {
            trigger_id: id("minecraft:shot_crossbow"),
            instance: Self::new(
                None,
                Some(ItemPredicateBuilderModel::item().of_items([item]).build()),
            ),
        }
    }

    pub fn matches(&self, item_stack: &ItemStackModel) -> bool {
        self.item
            .as_ref()
            .is_none_or(|predicate| predicate.test(item_stack))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShotCrossbowCriterion {
    pub trigger_id: Identifier,
    pub instance: ShotCrossbowTriggerInstance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: &str, count: i32) -> ItemStackModel {
        ItemStackModel::new(id(item), count, [])
    }

    #[test]
    fn codec_fields_match_java_record_codec() {
        assert_eq!(
            ShotCrossbowTriggerInstance::codec_field_names(),
            ["player", "item"]
        );
    }

    #[test]
    fn optional_item_factory_uses_shot_crossbow_trigger_and_empty_player_predicate() {
        let any = ShotCrossbowTriggerInstance::shot_crossbow(None);
        let crossbow = ShotCrossbowTriggerInstance::shot_crossbow(Some(
            ItemPredicateBuilderModel::item()
                .of_items([id("minecraft:crossbow")])
                .with_count(IntBoundsModel::exactly(1))
                .build(),
        ));

        assert_eq!(any.trigger_id, id("minecraft:shot_crossbow"));
        assert!(any.instance.player.is_none());
        assert!(any.instance.item.is_none());

        assert_eq!(crossbow.trigger_id, id("minecraft:shot_crossbow"));
        assert!(crossbow.instance.player.is_none());
        assert!(crossbow.instance.item.is_some());
        assert!(crossbow.instance.matches(&stack("minecraft:crossbow", 1)));
        assert!(!crossbow.instance.matches(&stack("minecraft:crossbow", 2)));
    }

    #[test]
    fn itemlike_factory_builds_single_item_predicate_under_same_trigger_id() {
        let criterion = ShotCrossbowTriggerInstance::shot_crossbow_item(id("minecraft:crossbow"));

        assert_eq!(criterion.trigger_id, id("minecraft:shot_crossbow"));
        assert!(criterion.instance.player.is_none());
        assert!(criterion.instance.matches(&stack("minecraft:crossbow", 1)));
        assert!(!criterion.instance.matches(&stack("minecraft:bow", 1)));
    }

    #[test]
    fn missing_item_predicate_matches_any_shot_stack() {
        let instance = ShotCrossbowTriggerInstance::new(None, None);

        assert!(instance.matches(&stack("minecraft:crossbow", 1)));
        assert!(instance.matches(&stack("minecraft:bow", 1)));
    }

    #[test]
    fn present_item_predicate_delegates_to_item_predicate_test() {
        let instance = ShotCrossbowTriggerInstance::new(
            None,
            Some(
                ItemPredicateBuilderModel::item()
                    .of_items([id("minecraft:crossbow")])
                    .with_count(IntBoundsModel::exactly(1))
                    .build(),
            ),
        );

        assert!(instance.matches(&stack("minecraft:crossbow", 1)));
        assert!(!instance.matches(&stack("minecraft:crossbow", 2)));
        assert!(!instance.matches(&stack("minecraft:bow", 1)));
    }
}
