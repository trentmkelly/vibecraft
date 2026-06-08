use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedUpItemTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub item: Option<ItemPredicateModel>,
    pub entity: Option<ContextAwarePredicateModel>,
}

impl PickedUpItemTriggerInstanceModel {
    pub fn new(
        player: Option<ContextAwarePredicateModel>,
        item: Option<ItemPredicateModel>,
        entity: Option<ContextAwarePredicateModel>,
    ) -> Self {
        Self {
            player,
            item,
            entity,
        }
    }

    pub fn thrown_item_picked_up_by_entity(
        player: ContextAwarePredicateModel,
        item: Option<ItemPredicateModel>,
        entity: Option<ContextAwarePredicateModel>,
    ) -> PickedUpItemCriterionModel {
        PickedUpItemCriterionModel {
            trigger_id: id("minecraft:thrown_item_picked_up_by_entity"),
            instance: Self::new(Some(player), item, entity),
        }
    }

    pub fn thrown_item_picked_up_by_player(
        player: Option<ContextAwarePredicateModel>,
        item: Option<ItemPredicateModel>,
        entity: Option<ContextAwarePredicateModel>,
    ) -> PickedUpItemCriterionModel {
        PickedUpItemCriterionModel {
            trigger_id: id("minecraft:thrown_item_picked_up_by_player"),
            instance: Self::new(player, item, entity),
        }
    }

    pub fn matches(
        &self,
        _player: &ServerPlayerModel,
        item_stack: &ItemStackModel,
        picked_up_by: &LootContextModel,
    ) -> bool {
        if self
            .item
            .as_ref()
            .is_some_and(|item| !item.test(item_stack))
        {
            return false;
        }

        self.entity
            .as_ref()
            .is_none_or(|entity| entity.matches(picked_up_by))
    }

    pub fn validate(&self) -> Vec<String> {
        self.entity
            .as_ref()
            .and_then(ContextAwarePredicateModel::validation_problem)
            .map(|problem| vec![format!("entity: {problem}")])
            .unwrap_or_default()
    }

    pub fn codec_field_names() -> [&'static str; 3] {
        ["player", "item", "entity"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedUpItemCriterionModel {
    pub trigger_id: Identifier,
    pub instance: PickedUpItemTriggerInstanceModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedUpItemTriggerModel {
    listeners: Vec<PickedUpItemTriggerInstanceModel>,
}

impl PickedUpItemTriggerModel {
    pub fn new(listeners: Vec<PickedUpItemTriggerInstanceModel>) -> Self {
        Self { listeners }
    }

    pub fn trigger(
        &self,
        player: &ServerPlayerModel,
        item_stack: &ItemStackModel,
        entity: Option<EntityContextModel>,
    ) -> Vec<&PickedUpItemTriggerInstanceModel> {
        let picked_up_by = EntityPredicateModel::create_context(player, entity);
        self.listeners
            .iter()
            .filter(|listener| listener.matches(player, item_stack, &picked_up_by))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPlayerModel {
    entity_type: Identifier,
}

impl ServerPlayerModel {
    pub fn new(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackModel {
    item: Identifier,
    count: i32,
}

impl ItemStackModel {
    pub fn new(item: Identifier, count: i32) -> Self {
        Self { item, count }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateModel {
    item: Option<Identifier>,
    min_count: Option<i32>,
}

impl ItemPredicateModel {
    pub fn any() -> Self {
        Self {
            item: None,
            min_count: None,
        }
    }

    pub fn item(item: Identifier) -> Self {
        Self {
            item: Some(item),
            min_count: None,
        }
    }

    pub fn with_min_count(mut self, count: i32) -> Self {
        self.min_count = Some(count);
        self
    }

    fn test(&self, item_stack: &ItemStackModel) -> bool {
        self.item
            .as_ref()
            .is_none_or(|item| item == &item_stack.item)
            && self
                .min_count
                .is_none_or(|min_count| min_count <= item_stack.count)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityContextModel {
    entity_type: Identifier,
}

impl EntityContextModel {
    pub fn new(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextModel {
    entity_type: Option<Identifier>,
}

impl LootContextModel {
    fn from_entity(entity: Option<EntityContextModel>) -> Self {
        Self {
            entity_type: entity.map(|entity| entity.entity_type),
        }
    }
}

pub struct EntityPredicateModel;

impl EntityPredicateModel {
    fn create_context(
        _player: &ServerPlayerModel,
        entity: Option<EntityContextModel>,
    ) -> LootContextModel {
        LootContextModel::from_entity(entity)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    required_entity_type: Option<Identifier>,
    validation_problem: Option<String>,
}

impl ContextAwarePredicateModel {
    pub fn any() -> Self {
        Self {
            required_entity_type: None,
            validation_problem: None,
        }
    }

    pub fn entity_type(entity_type: Identifier) -> Self {
        Self {
            required_entity_type: Some(entity_type),
            validation_problem: None,
        }
    }

    pub fn invalid(problem: &str) -> Self {
        Self {
            required_entity_type: None,
            validation_problem: Some(problem.to_string()),
        }
    }

    fn matches(&self, context: &LootContextModel) -> bool {
        self.required_entity_type.as_ref().is_none_or(|required| {
            context
                .entity_type
                .as_ref()
                .is_some_and(|entity_type| entity_type == required)
        })
    }

    fn validation_problem(&self) -> Option<&str> {
        self.validation_problem.as_deref()
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player() -> ServerPlayerModel {
        ServerPlayerModel::new(id("minecraft:player"))
    }

    fn stack(item: &str, count: i32) -> ItemStackModel {
        ItemStackModel::new(id(item), count)
    }

    fn entity(entity_type: &str) -> EntityContextModel {
        EntityContextModel::new(id(entity_type))
    }

    #[test]
    fn codec_field_names_match_java_record_codec() {
        assert_eq!(
            PickedUpItemTriggerInstanceModel::codec_field_names(),
            ["player", "item", "entity"]
        );
    }

    #[test]
    fn omitted_item_and_entity_predicates_match_any_pickup_event() {
        let instance = PickedUpItemTriggerInstanceModel::new(None, None, None);

        assert!(instance.matches(
            &player(),
            &stack("minecraft:stone", 1),
            &LootContextModel::from_entity(None)
        ));
        assert!(instance.matches(
            &player(),
            &stack("minecraft:diamond", 64),
            &LootContextModel::from_entity(Some(entity("minecraft:zombie")))
        ));
        assert!(ItemPredicateModel::any().test(&stack("minecraft:stone", 1)));
    }

    #[test]
    fn item_predicate_is_checked_before_entity_predicate() {
        let instance = PickedUpItemTriggerInstanceModel::new(
            None,
            Some(ItemPredicateModel::item(id("minecraft:diamond")).with_min_count(2)),
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:zombie",
            ))),
        );

        assert!(instance.matches(
            &player(),
            &stack("minecraft:diamond", 2),
            &LootContextModel::from_entity(Some(entity("minecraft:zombie")))
        ));
        assert!(!instance.matches(
            &player(),
            &stack("minecraft:emerald", 2),
            &LootContextModel::from_entity(Some(entity("minecraft:zombie")))
        ));
        assert!(!instance.matches(
            &player(),
            &stack("minecraft:diamond", 1),
            &LootContextModel::from_entity(Some(entity("minecraft:zombie")))
        ));
    }

    #[test]
    fn entity_predicate_matches_created_loot_context_and_rejects_null_entity_when_specific() {
        let instance = PickedUpItemTriggerInstanceModel::new(
            None,
            None,
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:villager",
            ))),
        );
        let trigger = PickedUpItemTriggerModel::new(vec![instance.clone()]);

        assert_eq!(
            trigger.trigger(
                &player(),
                &stack("minecraft:bread", 1),
                Some(entity("minecraft:villager"))
            ),
            vec![&instance]
        );
        assert!(trigger
            .trigger(&player(), &stack("minecraft:bread", 1), None)
            .is_empty());
        assert!(trigger
            .trigger(
                &player(),
                &stack("minecraft:bread", 1),
                Some(entity("minecraft:zombie"))
            )
            .is_empty());
    }

    #[test]
    fn thrown_item_picked_up_by_entity_factory_requires_player_predicate_and_trigger_id() {
        let criterion = PickedUpItemTriggerInstanceModel::thrown_item_picked_up_by_entity(
            ContextAwarePredicateModel::any(),
            Some(ItemPredicateModel::item(id("minecraft:arrow"))),
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:skeleton",
            ))),
        );

        assert_eq!(
            criterion.trigger_id,
            id("minecraft:thrown_item_picked_up_by_entity")
        );
        assert!(criterion.instance.player.is_some());
        assert!(criterion.instance.item.is_some());
        assert!(criterion.instance.entity.is_some());
    }

    #[test]
    fn thrown_item_picked_up_by_player_factory_preserves_optional_player_predicate() {
        let with_player = PickedUpItemTriggerInstanceModel::thrown_item_picked_up_by_player(
            Some(ContextAwarePredicateModel::any()),
            None,
            None,
        );
        let without_player =
            PickedUpItemTriggerInstanceModel::thrown_item_picked_up_by_player(None, None, None);

        assert_eq!(
            with_player.trigger_id,
            id("minecraft:thrown_item_picked_up_by_player")
        );
        assert!(with_player.instance.player.is_some());
        assert!(without_player.instance.player.is_none());
    }

    #[test]
    fn validation_reports_entity_predicate_label_only() {
        let instance = PickedUpItemTriggerInstanceModel::new(
            Some(ContextAwarePredicateModel::invalid("bad player")),
            None,
            Some(ContextAwarePredicateModel::invalid("bad entity")),
        );

        assert_eq!(instance.validate(), vec!["entity: bad entity".to_string()]);
    }
}
