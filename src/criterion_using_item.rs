use crate::criterion_item_predicate::{
    IntBoundsModel, ItemPredicateBuilderModel, ItemPredicateModel, ItemStackModel,
};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsingItemTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub item: Option<ItemPredicateModel>,
}

impl UsingItemTriggerInstanceModel {
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

    pub fn looking_at(
        player: EntityPredicateBuilderModel,
        with: ItemPredicateBuilderModel,
    ) -> UsingItemCriterionModel {
        UsingItemCriterionModel {
            trigger_id: id("minecraft:using_item"),
            instance: Self::new(
                Some(ContextAwarePredicateModel::wrap_builder(player)),
                Some(with.build()),
            ),
        }
    }

    pub fn matches(&self, item: &ItemStackModel) -> bool {
        self.item
            .as_ref()
            .is_none_or(|predicate| predicate.test(item))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsingItemTriggerModel {
    listeners: Vec<UsingItemTriggerInstanceModel>,
}

impl UsingItemTriggerModel {
    pub fn new(listeners: impl IntoIterator<Item = UsingItemTriggerInstanceModel>) -> Self {
        Self {
            listeners: listeners.into_iter().collect(),
        }
    }

    pub fn codec_type() -> &'static str {
        "UsingItemTrigger.TriggerInstance.CODEC"
    }

    pub fn trigger(
        &self,
        _player: &ServerPlayerModel,
        item: &ItemStackModel,
    ) -> Vec<&UsingItemTriggerInstanceModel> {
        self.listeners
            .iter()
            .filter(|listener| listener.matches(item))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsingItemCriterionModel {
    pub trigger_id: Identifier,
    pub instance: UsingItemTriggerInstanceModel,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityPredicateBuilderModel {
    entity_type: Option<Identifier>,
}

impl EntityPredicateBuilderModel {
    pub fn entity() -> Self {
        Self::default()
    }

    pub fn of(mut self, entity_type: Identifier) -> Self {
        self.entity_type = Some(entity_type);
        self
    }

    fn build(self) -> EntityPredicateDataModel {
        EntityPredicateDataModel {
            entity_type: self.entity_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateDataModel {
    entity_type: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    required_entity_type: Option<Identifier>,
}

impl ContextAwarePredicateModel {
    fn wrap_builder(builder: EntityPredicateBuilderModel) -> Self {
        Self {
            required_entity_type: builder.build().entity_type,
        }
    }

    pub fn entity_type(&self) -> Option<&Identifier> {
        self.required_entity_type.as_ref()
    }
}

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
    fn using_item_trigger_uses_trigger_instance_codec() {
        assert_eq!(
            UsingItemTriggerModel::codec_type(),
            "UsingItemTrigger.TriggerInstance.CODEC"
        );
    }

    #[test]
    fn using_item_codec_fields_match_java_record_codec() {
        assert_eq!(
            UsingItemTriggerInstanceModel::codec_fields(),
            [
                CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC"),
                CodecFieldModel::optional("item", "ItemPredicate.CODEC"),
            ]
        );
    }

    #[test]
    fn looking_at_factory_wraps_player_and_builds_item_predicate() {
        let criterion = UsingItemTriggerInstanceModel::looking_at(
            EntityPredicateBuilderModel::entity().of(id("minecraft:player")),
            ItemPredicateBuilderModel::item()
                .of_items([id("minecraft:spyglass")])
                .with_count(IntBoundsModel::exactly(1)),
        );

        assert_eq!(criterion.trigger_id, id("minecraft:using_item"));
        assert_eq!(
            criterion
                .instance
                .player
                .as_ref()
                .and_then(ContextAwarePredicateModel::entity_type),
            Some(&id("minecraft:player"))
        );
        assert!(criterion.instance.item.is_some());
        assert!(criterion.instance.matches(&stack("minecraft:spyglass", 1)));
        assert!(!criterion.instance.matches(&stack("minecraft:spyglass", 2)));
    }

    #[test]
    fn omitted_item_predicate_matches_any_using_stack() {
        let instance = UsingItemTriggerInstanceModel::new(None, None);

        assert!(instance.matches(&stack("minecraft:spyglass", 1)));
        assert!(instance.matches(&stack("minecraft:shield", 1)));
    }

    #[test]
    fn present_item_predicate_delegates_to_item_predicate_test() {
        let instance = UsingItemTriggerInstanceModel::new(
            None,
            Some(
                ItemPredicateBuilderModel::item()
                    .of_items([id("minecraft:shield")])
                    .with_count(IntBoundsModel::exactly(1))
                    .build(),
            ),
        );

        assert!(instance.matches(&stack("minecraft:shield", 1)));
        assert!(!instance.matches(&stack("minecraft:shield", 2)));
        assert!(!instance.matches(&stack("minecraft:spyglass", 1)));
    }

    #[test]
    fn trigger_forwards_currently_used_item_to_instance_matches() {
        let matching = UsingItemTriggerInstanceModel::new(
            None,
            Some(
                ItemPredicateBuilderModel::item()
                    .of_items([id("minecraft:spyglass")])
                    .build(),
            ),
        );
        let non_matching = UsingItemTriggerInstanceModel::new(
            None,
            Some(
                ItemPredicateBuilderModel::item()
                    .of_items([id("minecraft:shield")])
                    .build(),
            ),
        );
        let trigger = UsingItemTriggerModel::new([matching.clone(), non_matching]);

        assert_eq!(
            trigger.trigger(&ServerPlayerModel, &stack("minecraft:spyglass", 1)),
            vec![&matching]
        );
    }
}
