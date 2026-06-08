use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerInteractTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub item: Option<ItemPredicateModel>,
    pub entity: Option<ContextAwarePredicateModel>,
}

impl PlayerInteractTriggerInstanceModel {
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

    pub fn item_used_on_entity(
        player: Option<ContextAwarePredicateModel>,
        item: ItemPredicateBuilderModel,
        entity: Option<ContextAwarePredicateModel>,
    ) -> PlayerInteractCriterionModel {
        PlayerInteractCriterionModel {
            trigger_id: id("minecraft:player_interacted_with_entity"),
            instance: Self::new(player, Some(item.build()), entity),
        }
    }

    pub fn item_used_on_entity_without_player(
        item: ItemPredicateBuilderModel,
        entity: Option<ContextAwarePredicateModel>,
    ) -> PlayerInteractCriterionModel {
        Self::item_used_on_entity(None, item, entity)
    }

    pub fn equipment_sheared(
        player: Option<ContextAwarePredicateModel>,
        item: ItemPredicateBuilderModel,
        entity: Option<ContextAwarePredicateModel>,
    ) -> PlayerInteractCriterionModel {
        PlayerInteractCriterionModel {
            trigger_id: id("minecraft:player_sheared_equipment"),
            instance: Self::new(player, Some(item.build()), entity),
        }
    }

    pub fn equipment_sheared_without_player(
        item: ItemPredicateBuilderModel,
        entity: Option<ContextAwarePredicateModel>,
    ) -> PlayerInteractCriterionModel {
        Self::equipment_sheared(None, item, entity)
    }

    pub fn matches(&self, item_stack: &ItemStackModel, interacted_with: &LootContextModel) -> bool {
        if self
            .item
            .as_ref()
            .is_some_and(|item| !item.test(item_stack))
        {
            return false;
        }

        self.entity
            .as_ref()
            .is_none_or(|entity| entity.matches(interacted_with))
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
pub struct PlayerInteractCriterionModel {
    pub trigger_id: Identifier,
    pub instance: PlayerInteractTriggerInstanceModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerInteractTriggerModel {
    listeners: Vec<PlayerInteractTriggerInstanceModel>,
}

impl PlayerInteractTriggerModel {
    pub fn new(listeners: Vec<PlayerInteractTriggerInstanceModel>) -> Self {
        Self { listeners }
    }

    pub fn trigger(
        &self,
        player: &ServerPlayerModel,
        item_stack: &ItemStackModel,
        interacted_with: EntityContextModel,
    ) -> Vec<&PlayerInteractTriggerInstanceModel> {
        let context = EntityPredicateModel::create_context(player, interacted_with);
        self.listeners
            .iter()
            .filter(|listener| listener.matches(item_stack, &context))
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

    fn test(&self, item_stack: &ItemStackModel) -> bool {
        self.item
            .as_ref()
            .is_none_or(|item| item == &item_stack.item)
            && self
                .min_count
                .is_none_or(|min_count| min_count <= item_stack.count)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemPredicateBuilderModel {
    item: Option<Identifier>,
    min_count: Option<i32>,
}

impl ItemPredicateBuilderModel {
    pub fn item() -> Self {
        Self::default()
    }

    pub fn of(mut self, item: Identifier) -> Self {
        self.item = Some(item);
        self
    }

    pub fn with_min_count(mut self, min_count: i32) -> Self {
        self.min_count = Some(min_count);
        self
    }

    fn build(self) -> ItemPredicateModel {
        ItemPredicateModel {
            item: self.item,
            min_count: self.min_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityContextModel {
    entity_type: Identifier,
    name: Option<String>,
}

impl EntityContextModel {
    pub fn new(entity_type: Identifier, name: Option<&str>) -> Self {
        Self {
            entity_type,
            name: name.map(str::to_string),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextModel {
    entity_type: Identifier,
    name: Option<String>,
}

pub struct EntityPredicateModel;

impl EntityPredicateModel {
    fn create_context(_player: &ServerPlayerModel, entity: EntityContextModel) -> LootContextModel {
        LootContextModel {
            entity_type: entity.entity_type,
            name: entity.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    required_type: Option<Identifier>,
    required_name: Option<String>,
    validation_problem: Option<String>,
}

impl ContextAwarePredicateModel {
    pub fn any() -> Self {
        Self {
            required_type: None,
            required_name: None,
            validation_problem: None,
        }
    }

    pub fn entity_type(entity_type: Identifier) -> Self {
        Self {
            required_type: Some(entity_type),
            required_name: None,
            validation_problem: None,
        }
    }

    pub fn named(name: &str) -> Self {
        Self {
            required_type: None,
            required_name: Some(name.to_string()),
            validation_problem: None,
        }
    }

    pub fn invalid(problem: &str) -> Self {
        Self {
            required_type: None,
            required_name: None,
            validation_problem: Some(problem.to_string()),
        }
    }

    fn matches(&self, context: &LootContextModel) -> bool {
        self.required_type
            .as_ref()
            .is_none_or(|entity_type| entity_type == &context.entity_type)
            && self.required_name.as_ref().is_none_or(|name| {
                context
                    .name
                    .as_ref()
                    .is_some_and(|entity_name| entity_name == name)
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

    fn entity(entity_type: &str, name: Option<&str>) -> EntityContextModel {
        EntityContextModel::new(id(entity_type), name)
    }

    fn item_builder(item: &str) -> ItemPredicateBuilderModel {
        ItemPredicateBuilderModel::item().of(id(item))
    }

    #[test]
    fn codec_field_names_match_java_record_codec() {
        assert_eq!(
            PlayerInteractTriggerInstanceModel::codec_field_names(),
            ["player", "item", "entity"]
        );
    }

    #[test]
    fn omitted_item_and_entity_predicates_match_any_interaction() {
        let instance = PlayerInteractTriggerInstanceModel::new(None, None, None);

        assert!(instance.matches(
            &stack("minecraft:stick", 1),
            &EntityPredicateModel::create_context(&player(), entity("minecraft:villager", None))
        ));
        assert!(ItemPredicateModel::any().test(&stack("minecraft:stick", 1)));
    }

    #[test]
    fn item_predicate_is_checked_before_entity_predicate() {
        let instance = PlayerInteractTriggerInstanceModel::new(
            None,
            Some(
                ItemPredicateBuilderModel::item()
                    .of(id("minecraft:shears"))
                    .with_min_count(1)
                    .build(),
            ),
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:sheep",
            ))),
        );
        let sheep =
            EntityPredicateModel::create_context(&player(), entity("minecraft:sheep", None));

        assert!(instance.matches(&stack("minecraft:shears", 1), &sheep));
        assert!(!instance.matches(&stack("minecraft:stick", 1), &sheep));
        assert!(!instance.matches(&stack("minecraft:shears", 0), &sheep));
    }

    #[test]
    fn trigger_creates_interacted_entity_loot_context() {
        let listener = PlayerInteractTriggerInstanceModel::new(
            None,
            None,
            Some(ContextAwarePredicateModel::named("Trader")),
        );
        let trigger = PlayerInteractTriggerModel::new(vec![listener.clone()]);

        assert_eq!(
            trigger.trigger(
                &player(),
                &stack("minecraft:emerald", 1),
                entity("minecraft:villager", Some("Trader")),
            ),
            vec![&listener]
        );
        assert!(trigger
            .trigger(
                &player(),
                &stack("minecraft:emerald", 1),
                entity("minecraft:villager", Some("Other")),
            )
            .is_empty());
    }

    #[test]
    fn item_used_on_entity_factories_build_item_and_set_trigger_id() {
        let with_player = PlayerInteractTriggerInstanceModel::item_used_on_entity(
            Some(ContextAwarePredicateModel::any()),
            item_builder("minecraft:carrot"),
            Some(ContextAwarePredicateModel::entity_type(id("minecraft:pig"))),
        );
        let without_player = PlayerInteractTriggerInstanceModel::item_used_on_entity_without_player(
            item_builder("minecraft:wheat"),
            None,
        );

        assert_eq!(
            with_player.trigger_id,
            id("minecraft:player_interacted_with_entity")
        );
        assert!(with_player.instance.player.is_some());
        assert!(with_player.instance.item.is_some());
        assert!(with_player.instance.entity.is_some());
        assert_eq!(
            without_player.trigger_id,
            id("minecraft:player_interacted_with_entity")
        );
        assert!(without_player.instance.player.is_none());
        assert!(without_player.instance.item.is_some());
    }

    #[test]
    fn equipment_sheared_factories_build_item_and_set_trigger_id() {
        let with_player = PlayerInteractTriggerInstanceModel::equipment_sheared(
            Some(ContextAwarePredicateModel::any()),
            item_builder("minecraft:shears"),
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:sheep",
            ))),
        );
        let without_player = PlayerInteractTriggerInstanceModel::equipment_sheared_without_player(
            item_builder("minecraft:shears"),
            None,
        );

        assert_eq!(
            with_player.trigger_id,
            id("minecraft:player_sheared_equipment")
        );
        assert!(with_player.instance.player.is_some());
        assert!(with_player.instance.item.is_some());
        assert!(with_player.instance.entity.is_some());
        assert_eq!(
            without_player.trigger_id,
            id("minecraft:player_sheared_equipment")
        );
        assert!(without_player.instance.player.is_none());
        assert!(without_player.instance.item.is_some());
    }

    #[test]
    fn validation_reports_entity_predicate_label_only() {
        let instance = PlayerInteractTriggerInstanceModel::new(
            Some(ContextAwarePredicateModel::invalid("bad player")),
            Some(item_builder("minecraft:stick").build()),
            Some(ContextAwarePredicateModel::invalid("bad entity")),
        );

        assert_eq!(instance.validate(), vec!["entity: bad entity".to_string()]);
    }
}
