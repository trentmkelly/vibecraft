use crate::registry::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockInteractionTriggerKind {
    AnyBlockInteraction,
    DefaultBlockInteraction,
    ItemUsedOnLocation,
}

impl BlockInteractionTriggerKind {
    pub fn trigger_id(self) -> &'static str {
        match self {
            Self::AnyBlockInteraction => "minecraft:any_block_use",
            Self::DefaultBlockInteraction => "minecraft:default_block_use",
            Self::ItemUsedOnLocation => "minecraft:placed_block",
        }
    }

    pub fn loot_context_param_set(self) -> LootContextParamSetModel {
        match self {
            Self::DefaultBlockInteraction => LootContextParamSetModel::BlockUse,
            Self::AnyBlockInteraction | Self::ItemUsedOnLocation => {
                LootContextParamSetModel::AdvancementLocation
            }
        }
    }

    pub fn includes_tool(self) -> bool {
        matches!(self, Self::AnyBlockInteraction | Self::ItemUsedOnLocation)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootContextParamSetModel {
    AdvancementLocation,
    BlockUse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInteractionTriggerInput {
    pub player: String,
    pub block_pos: (i32, i32, i32),
    pub block_state: String,
    pub tool: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInteractionLootContext {
    pub param_set: LootContextParamSetModel,
    pub origin_center: (i32, i32, i32),
    pub this_entity: String,
    pub block_state: String,
    pub tool: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInteractionTriggerInstance {
    pub player_predicate_present: bool,
    pub location: Option<ContextAwarePredicateModel>,
}

impl BlockInteractionTriggerInstance {
    pub fn new(location: Option<ContextAwarePredicateModel>) -> Self {
        Self {
            player_predicate_present: false,
            location,
        }
    }

    pub fn matches(&self, location_context: &BlockInteractionLootContext) -> bool {
        self.location
            .as_ref()
            .is_none_or(|predicate| predicate.matches(location_context))
    }

    pub fn validate(&self, kind: BlockInteractionTriggerKind) -> Vec<String> {
        self.location.as_ref().map_or_else(Vec::new, |predicate| {
            predicate.validate(kind.loot_context_param_set(), "location")
        })
    }

    pub fn placed_block(block: &str) -> CriterionFactoryResult {
        CriterionFactoryResult {
            trigger_id: Identifier::parse("minecraft:placed_block").unwrap(),
            instance: Self::new(Some(ContextAwarePredicateModel::block_state(block))),
        }
    }

    pub fn placed_block_with_property(
        block: &str,
        property: &str,
        property_value: impl ToString,
    ) -> CriterionFactoryResult {
        CriterionFactoryResult {
            trigger_id: Identifier::parse("minecraft:placed_block").unwrap(),
            instance: Self::new(Some(ContextAwarePredicateModel::block_state_property(
                block,
                property,
                &property_value.to_string(),
            ))),
        }
    }

    pub fn item_used_on_block(location: &str, item: &str) -> CriterionFactoryResult {
        CriterionFactoryResult {
            trigger_id: Identifier::parse("minecraft:item_used_on_block").unwrap(),
            instance: Self::new(Some(ContextAwarePredicateModel::location_and_tool(
                location, item,
            ))),
        }
    }

    pub fn allay_drop_item_on_block(location: &str, item: &str) -> CriterionFactoryResult {
        CriterionFactoryResult {
            trigger_id: Identifier::parse("minecraft:allay_drop_item_on_block").unwrap(),
            instance: Self::new(Some(ContextAwarePredicateModel::location_and_tool(
                location, item,
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    required_block_state: Option<String>,
    required_property: Option<(String, String)>,
    required_location: Option<String>,
    required_tool: Option<String>,
    validation_problem: Option<String>,
}

impl ContextAwarePredicateModel {
    pub fn always() -> Self {
        Self {
            required_block_state: None,
            required_property: None,
            required_location: None,
            required_tool: None,
            validation_problem: None,
        }
    }

    pub fn invalid(problem: &str) -> Self {
        Self {
            validation_problem: Some(problem.to_string()),
            ..Self::always()
        }
    }

    pub fn block_state(block: &str) -> Self {
        Self {
            required_block_state: Some(block.to_string()),
            ..Self::always()
        }
    }

    pub fn block_state_property(block: &str, property: &str, value: &str) -> Self {
        Self {
            required_block_state: Some(block.to_string()),
            required_property: Some((property.to_string(), value.to_string())),
            ..Self::always()
        }
    }

    pub fn location_and_tool(location: &str, item: &str) -> Self {
        Self {
            required_location: Some(location.to_string()),
            required_tool: Some(item.to_string()),
            ..Self::always()
        }
    }

    fn matches(&self, context: &BlockInteractionLootContext) -> bool {
        self.required_block_state
            .as_ref()
            .is_none_or(|block| block == &context.block_state)
            && self
                .required_location
                .as_ref()
                .is_none_or(|location| location == &format!("{:?}", context.origin_center))
            && self
                .required_tool
                .as_ref()
                .is_none_or(|tool| context.tool.as_ref() == Some(tool))
    }

    fn validate(&self, param_set: LootContextParamSetModel, field: &str) -> Vec<String> {
        self.validation_problem
            .as_ref()
            .map_or_else(Vec::new, |problem| {
                vec![format!("{param_set:?}.{field}: {problem}")]
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionFactoryResult {
    pub trigger_id: Identifier,
    pub instance: BlockInteractionTriggerInstance,
}

pub fn build_block_interaction_context(
    kind: BlockInteractionTriggerKind,
    input: BlockInteractionTriggerInput,
) -> BlockInteractionLootContext {
    BlockInteractionLootContext {
        param_set: kind.loot_context_param_set(),
        origin_center: input.block_pos,
        this_entity: input.player,
        block_state: input.block_state,
        tool: kind.includes_tool().then_some(input.tool).flatten(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(tool: Option<&str>) -> BlockInteractionTriggerInput {
        BlockInteractionTriggerInput {
            player: "Steve".to_string(),
            block_pos: (1, 64, 2),
            block_state: "minecraft:stone".to_string(),
            tool: tool.map(str::to_string),
        }
    }

    #[test]
    fn any_block_interaction_trigger_context_and_matches_follow_java() {
        let context = build_block_interaction_context(
            BlockInteractionTriggerKind::AnyBlockInteraction,
            input(Some("minecraft:diamond_pickaxe")),
        );
        assert_eq!(
            context.param_set,
            LootContextParamSetModel::AdvancementLocation
        );
        assert_eq!(context.tool, Some("minecraft:diamond_pickaxe".to_string()));
        assert_eq!(
            BlockInteractionTriggerKind::AnyBlockInteraction.trigger_id(),
            "minecraft:any_block_use"
        );
        assert!(BlockInteractionTriggerInstance::new(None).matches(&context));
        assert!(BlockInteractionTriggerInstance::new(Some(
            ContextAwarePredicateModel::block_state("minecraft:stone")
        ))
        .matches(&context));
        assert!(!BlockInteractionTriggerInstance::new(Some(
            ContextAwarePredicateModel::block_state("minecraft:dirt")
        ))
        .matches(&context));
    }

    #[test]
    fn default_block_interaction_uses_block_use_context_without_tool() {
        let context = build_block_interaction_context(
            BlockInteractionTriggerKind::DefaultBlockInteraction,
            input(Some("minecraft:stick")),
        );
        assert_eq!(context.param_set, LootContextParamSetModel::BlockUse);
        assert_eq!(context.tool, None);
        assert_eq!(
            BlockInteractionTriggerKind::DefaultBlockInteraction.trigger_id(),
            "minecraft:default_block_use"
        );
        assert_eq!(
            BlockInteractionTriggerInstance::new(Some(ContextAwarePredicateModel::invalid(
                "bad predicate"
            )))
            .validate(BlockInteractionTriggerKind::DefaultBlockInteraction),
            vec!["BlockUse.location: bad predicate".to_string()]
        );
    }

    #[test]
    fn item_used_on_location_static_factories_match_java_trigger_ids_and_predicates() {
        let placed = BlockInteractionTriggerInstance::placed_block("minecraft:stone");
        assert_eq!(
            placed.trigger_id,
            Identifier::parse("minecraft:placed_block").unwrap()
        );
        assert!(placed.instance.matches(&build_block_interaction_context(
            BlockInteractionTriggerKind::ItemUsedOnLocation,
            input(Some("minecraft:stick")),
        )));

        let property = BlockInteractionTriggerInstance::placed_block_with_property(
            "minecraft:stone",
            "lit",
            true,
        );
        assert_eq!(
            property.trigger_id,
            Identifier::parse("minecraft:placed_block").unwrap()
        );

        let used =
            BlockInteractionTriggerInstance::item_used_on_block("(1, 64, 2)", "minecraft:stick");
        assert_eq!(
            used.trigger_id,
            Identifier::parse("minecraft:item_used_on_block").unwrap()
        );
        assert!(used.instance.matches(&build_block_interaction_context(
            BlockInteractionTriggerKind::ItemUsedOnLocation,
            input(Some("minecraft:stick")),
        )));

        let allay = BlockInteractionTriggerInstance::allay_drop_item_on_block(
            "(1, 64, 2)",
            "minecraft:stick",
        );
        assert_eq!(
            allay.trigger_id,
            Identifier::parse("minecraft:allay_drop_item_on_block").unwrap()
        );
    }
}
