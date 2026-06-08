use crate::criterion_item_predicate::{IntBoundsModel, ItemPredicateModel, ItemStackModel};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub villager: Option<ContextAwarePredicateModel>,
    pub item: Option<TradeItemPredicateModel>,
}

impl TradeTriggerInstanceModel {
    pub fn new(
        player: Option<ContextAwarePredicateModel>,
        villager: Option<ContextAwarePredicateModel>,
        item: Option<TradeItemPredicateModel>,
    ) -> Self {
        Self {
            player,
            villager,
            item,
        }
    }

    pub fn codec_fields() -> [CodecFieldModel; 3] {
        [
            CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC"),
            CodecFieldModel::optional("villager", "EntityPredicate.ADVANCEMENT_CODEC"),
            CodecFieldModel::optional("item", "ItemPredicate.CODEC"),
        ]
    }

    pub fn traded_with_villager() -> TradeCriterionModel {
        TradeCriterionModel {
            trigger_id: id("minecraft:villager_trade"),
            instance: Self::new(None, None, None),
        }
    }

    pub fn traded_with_villager_player(player: EntityPredicateBuilderModel) -> TradeCriterionModel {
        TradeCriterionModel {
            trigger_id: id("minecraft:villager_trade"),
            instance: Self::new(
                Some(ContextAwarePredicateModel::wrap_builder(player)),
                None,
                None,
            ),
        }
    }

    pub fn matches(
        &self,
        villager_context: &LootContextModel,
        item_stack: &ItemStackModel,
    ) -> bool {
        if self
            .villager
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(villager_context))
        {
            return false;
        }

        self.item.as_ref().is_none_or(|item| item.test(item_stack))
    }

    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if let Some(problem) = self
            .player
            .as_ref()
            .and_then(ContextAwarePredicateModel::validation_problem)
        {
            problems.push(format!("player: {problem}"));
        }
        if let Some(problem) = self
            .villager
            .as_ref()
            .and_then(ContextAwarePredicateModel::validation_problem)
        {
            problems.push(format!("villager: {problem}"));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeTriggerModel {
    listeners: Vec<TradeTriggerInstanceModel>,
}

impl TradeTriggerModel {
    pub fn new(listeners: impl IntoIterator<Item = TradeTriggerInstanceModel>) -> Self {
        Self {
            listeners: listeners.into_iter().collect(),
        }
    }

    pub fn codec_type() -> &'static str {
        "TradeTrigger.TriggerInstance.CODEC"
    }

    pub fn trigger(
        &self,
        player: &ServerPlayerModel,
        villager: AbstractVillagerModel,
        item_stack: &ItemStackModel,
    ) -> Vec<&TradeTriggerInstanceModel> {
        let villager_context = EntityPredicateModel::create_context(player, villager);
        self.listeners
            .iter()
            .filter(|listener| listener.matches(&villager_context, item_stack))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeCriterionModel {
    pub trigger_id: Identifier,
    pub instance: TradeTriggerInstanceModel,
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
pub enum TradeItemPredicateModel {
    Item(ItemPredicateModel),
    PanicIfEvaluated,
}

impl TradeItemPredicateModel {
    pub fn test(&self, item_stack: &ItemStackModel) -> bool {
        match self {
            Self::Item(item) => item.test(item_stack),
            Self::PanicIfEvaluated => panic!("item predicate should not be evaluated"),
        }
    }
}

impl From<ItemPredicateModel> for TradeItemPredicateModel {
    fn from(value: ItemPredicateModel) -> Self {
        Self::Item(value)
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
pub struct AbstractVillagerModel {
    entity_type: Identifier,
}

impl AbstractVillagerModel {
    pub fn new(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextModel {
    player_type: Identifier,
    villager_type: Identifier,
}

pub struct EntityPredicateModel;

impl EntityPredicateModel {
    fn create_context(
        player: &ServerPlayerModel,
        villager: AbstractVillagerModel,
    ) -> LootContextModel {
        LootContextModel {
            player_type: player.entity_type.clone(),
            villager_type: villager.entity_type,
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
    validation_problem: Option<String>,
}

impl ContextAwarePredicateModel {
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

    fn wrap_builder(builder: EntityPredicateBuilderModel) -> Self {
        let predicate = builder.build();
        Self {
            required_entity_type: predicate.entity_type,
            validation_problem: None,
        }
    }

    fn matches(&self, context: &LootContextModel) -> bool {
        self.required_entity_type
            .as_ref()
            .is_none_or(|required| &context.villager_type == required)
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

    fn villager(entity_type: &str) -> AbstractVillagerModel {
        AbstractVillagerModel::new(id(entity_type))
    }

    fn stack(item: &str, count: i32) -> ItemStackModel {
        ItemStackModel::new(id(item), count, [])
    }

    fn item_predicate(item: &str, count: i32) -> TradeItemPredicateModel {
        ItemPredicateModel::builder()
            .of_items([id(item)])
            .with_count(IntBoundsModel::exactly(count))
            .build()
            .into()
    }

    #[test]
    fn trade_trigger_uses_trigger_instance_codec() {
        assert_eq!(
            TradeTriggerModel::codec_type(),
            "TradeTrigger.TriggerInstance.CODEC"
        );
    }

    #[test]
    fn trade_codec_fields_match_java_record_codec() {
        assert_eq!(
            TradeTriggerInstanceModel::codec_fields(),
            [
                CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC"),
                CodecFieldModel::optional("villager", "EntityPredicate.ADVANCEMENT_CODEC"),
                CodecFieldModel::optional("item", "ItemPredicate.CODEC"),
            ]
        );
    }

    #[test]
    fn traded_with_villager_no_arg_factory_uses_empty_instance() {
        let criterion = TradeTriggerInstanceModel::traded_with_villager();

        assert_eq!(criterion.trigger_id, id("minecraft:villager_trade"));
        assert_eq!(
            criterion.instance,
            TradeTriggerInstanceModel::new(None, None, None)
        );
    }

    #[test]
    fn traded_with_villager_player_factory_wraps_only_player_predicate() {
        let criterion = TradeTriggerInstanceModel::traded_with_villager_player(
            EntityPredicateBuilderModel::entity().of(id("minecraft:player")),
        );

        assert_eq!(criterion.trigger_id, id("minecraft:villager_trade"));
        assert_eq!(
            criterion.instance.player,
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:player"
            )))
        );
        assert!(criterion.instance.villager.is_none());
        assert!(criterion.instance.item.is_none());
    }

    #[test]
    fn omitted_villager_and_item_predicates_match_any_trade() {
        let instance = TradeTriggerInstanceModel::new(None, None, None);
        let context =
            EntityPredicateModel::create_context(&player(), villager("minecraft:villager"));

        assert!(instance.matches(&context, &stack("minecraft:emerald", 1)));
        assert!(instance.matches(&context, &stack("minecraft:diamond", 64)));
    }

    #[test]
    fn villager_predicate_is_checked_before_item_predicate() {
        let instance = TradeTriggerInstanceModel::new(
            None,
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:villager",
            ))),
            Some(TradeItemPredicateModel::PanicIfEvaluated),
        );
        let context =
            EntityPredicateModel::create_context(&player(), villager("minecraft:wandering_trader"));

        assert!(!instance.matches(&context, &stack("minecraft:emerald", 1)));
    }

    #[test]
    fn item_predicate_delegates_after_villager_match() {
        let instance = TradeTriggerInstanceModel::new(
            None,
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:villager",
            ))),
            Some(item_predicate("minecraft:diamond_sword", 1)),
        );
        let trigger = TradeTriggerModel::new([instance.clone()]);

        assert_eq!(
            trigger.trigger(
                &player(),
                villager("minecraft:villager"),
                &stack("minecraft:diamond_sword", 1)
            ),
            vec![&instance]
        );
        assert!(trigger
            .trigger(
                &player(),
                villager("minecraft:villager"),
                &stack("minecraft:diamond_sword", 2)
            )
            .is_empty());
        assert!(trigger
            .trigger(
                &player(),
                villager("minecraft:villager"),
                &stack("minecraft:iron_sword", 1)
            )
            .is_empty());
    }

    #[test]
    fn trigger_context_is_created_from_player_and_villager() {
        let context =
            EntityPredicateModel::create_context(&player(), villager("minecraft:villager"));

        assert_eq!(context.player_type, id("minecraft:player"));
        assert_eq!(context.villager_type, id("minecraft:villager"));
    }

    #[test]
    fn validation_runs_inherited_player_label_and_villager_label() {
        let instance = TradeTriggerInstanceModel::new(
            Some(ContextAwarePredicateModel::invalid("bad player")),
            Some(ContextAwarePredicateModel::invalid("bad villager")),
            Some(item_predicate("minecraft:emerald", 1)),
        );

        assert_eq!(
            instance.validate(),
            vec![
                "player: bad player".to_string(),
                "villager: bad villager".to_string(),
            ]
        );
    }
}
