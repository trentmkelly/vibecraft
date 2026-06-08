use crate::criterion_item_predicate::{
    IntBoundsModel, ItemPredicateBuilderModel, ItemPredicateModel, ItemStackModel,
};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsedTotemTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub item: Option<ItemPredicateModel>,
}

impl UsedTotemTriggerInstanceModel {
    pub fn new(
        player: Option<ContextAwarePredicateModel>,
        item: Option<ItemPredicateModel>,
    ) -> Self {
        Self { player, item }
    }

    pub fn codec_fields() -> [CodecFieldModel; 2] {
        [
            CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC"),
            CodecFieldModel::optional("item", "ItemPredicate.CODEC"),
        ]
    }

    pub fn used_totem(item: ItemPredicateModel) -> UsedTotemCriterionModel {
        UsedTotemCriterionModel {
            trigger_id: id("minecraft:used_totem"),
            instance: Self::new(None, Some(item)),
        }
    }

    pub fn used_totem_itemlike(itemlike: Identifier) -> UsedTotemCriterionModel {
        UsedTotemCriterionModel {
            trigger_id: id("minecraft:used_totem"),
            instance: Self::new(
                None,
                Some(
                    ItemPredicateBuilderModel::item()
                        .of_items([itemlike])
                        .build(),
                ),
            ),
        }
    }

    pub fn matches(&self, item_stack: &ItemStackModel) -> bool {
        self.item.as_ref().is_none_or(|item| item.test(item_stack))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsedTotemTriggerModel {
    listeners: Vec<UsedTotemTriggerInstanceModel>,
}

impl UsedTotemTriggerModel {
    pub fn new(listeners: impl IntoIterator<Item = UsedTotemTriggerInstanceModel>) -> Self {
        Self {
            listeners: listeners.into_iter().collect(),
        }
    }

    pub fn codec_type() -> &'static str {
        "UsedTotemTrigger.TriggerInstance.CODEC"
    }

    pub fn trigger(
        &self,
        _player: &ServerPlayerModel,
        item_stack: &ItemStackModel,
    ) -> Vec<&UsedTotemTriggerInstanceModel> {
        self.listeners
            .iter()
            .filter(|listener| listener.matches(item_stack))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsedTotemCriterionModel {
    pub trigger_id: Identifier,
    pub instance: UsedTotemTriggerInstanceModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodecFieldModel {
    pub name: &'static str,
    pub codec: &'static str,
    pub optional: bool,
}

impl CodecFieldModel {
    pub fn optional(name: &'static str, codec: &'static str) -> Self {
        Self {
            name,
            codec,
            optional: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPlayerModel;

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
    fn used_totem_trigger_uses_trigger_instance_codec() {
        assert_eq!(
            UsedTotemTriggerModel::codec_type(),
            "UsedTotemTrigger.TriggerInstance.CODEC"
        );
    }

    #[test]
    fn used_totem_codec_fields_match_java_record_codec() {
        assert_eq!(
            UsedTotemTriggerInstanceModel::codec_fields(),
            [
                CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC"),
                CodecFieldModel::optional("item", "ItemPredicate.CODEC"),
            ]
        );
    }

    #[test]
    fn used_totem_predicate_factory_uses_empty_player_and_used_totem_trigger() {
        let predicate = ItemPredicateBuilderModel::item()
            .of_items([id("minecraft:totem_of_undying")])
            .with_count(IntBoundsModel::exactly(1))
            .build();
        let criterion = UsedTotemTriggerInstanceModel::used_totem(predicate.clone());

        assert_eq!(criterion.trigger_id, id("minecraft:used_totem"));
        assert!(criterion.instance.player.is_none());
        assert_eq!(criterion.instance.item, Some(predicate));
    }

    #[test]
    fn used_totem_itemlike_factory_builds_single_item_predicate_under_same_trigger_id() {
        let criterion =
            UsedTotemTriggerInstanceModel::used_totem_itemlike(id("minecraft:totem_of_undying"));

        assert_eq!(criterion.trigger_id, id("minecraft:used_totem"));
        assert!(criterion.instance.player.is_none());
        assert!(criterion
            .instance
            .matches(&stack("minecraft:totem_of_undying", 1)));
        assert!(!criterion.instance.matches(&stack("minecraft:apple", 1)));
    }

    #[test]
    fn omitted_item_predicate_matches_any_used_stack() {
        let instance = UsedTotemTriggerInstanceModel::new(None, None);

        assert!(instance.matches(&stack("minecraft:totem_of_undying", 1)));
        assert!(instance.matches(&stack("minecraft:stone", 64)));
    }

    #[test]
    fn present_item_predicate_delegates_to_item_predicate_test() {
        let instance = UsedTotemTriggerInstanceModel::new(
            None,
            Some(
                ItemPredicateBuilderModel::item()
                    .of_items([id("minecraft:totem_of_undying")])
                    .with_count(IntBoundsModel::exactly(1))
                    .build(),
            ),
        );

        assert!(instance.matches(&stack("minecraft:totem_of_undying", 1)));
        assert!(!instance.matches(&stack("minecraft:totem_of_undying", 2)));
        assert!(!instance.matches(&stack("minecraft:apple", 1)));
    }

    #[test]
    fn trigger_forwards_used_stack_to_instance_matches() {
        let matching = UsedTotemTriggerInstanceModel::new(
            None,
            Some(
                ItemPredicateBuilderModel::item()
                    .of_items([id("minecraft:totem_of_undying")])
                    .build(),
            ),
        );
        let non_matching = UsedTotemTriggerInstanceModel::new(
            None,
            Some(
                ItemPredicateBuilderModel::item()
                    .of_items([id("minecraft:apple")])
                    .build(),
            ),
        );
        let trigger = UsedTotemTriggerModel::new([matching.clone(), non_matching]);

        assert_eq!(
            trigger.trigger(&ServerPlayerModel, &stack("minecraft:totem_of_undying", 1)),
            vec![&matching]
        );
    }
}
