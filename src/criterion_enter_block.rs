use std::collections::{BTreeMap, BTreeSet};

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnterBlockTriggerInstance {
    pub player_predicate_present: bool,
    pub block: Option<Identifier>,
    pub state: Option<StatePropertiesPredicateModel>,
}

impl EnterBlockTriggerInstance {
    pub fn new(block: Option<Identifier>, state: Option<StatePropertiesPredicateModel>) -> Self {
        Self {
            player_predicate_present: false,
            block,
            state,
        }
    }

    pub fn validate(&self, block_definition: Option<&BlockDefinitionModel>) -> Result<(), String> {
        let Some(block) = &self.block else {
            return Ok(());
        };
        let Some(state) = &self.state else {
            return Ok(());
        };
        let Some(block_definition) = block_definition else {
            return Ok(());
        };

        if let Some(property) = state.check_state(block_definition) {
            return Err(format!("Block{} has no property {}", block, property));
        }

        Ok(())
    }

    pub fn matches(&self, state: &BlockStateModel) -> bool {
        if self
            .block
            .as_ref()
            .is_some_and(|block| block != &state.block)
        {
            return false;
        }

        self.state
            .as_ref()
            .is_none_or(|predicate| predicate.matches(state))
    }

    pub fn enters_block(block: Identifier) -> EnterBlockCriterion {
        EnterBlockCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(Some(block), None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnterBlockCriterion {
    pub trigger_id: Identifier,
    pub instance: EnterBlockTriggerInstance,
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:enter_block").unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateModel {
    pub block: Identifier,
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDefinitionModel {
    pub block: Identifier,
    pub properties: BTreeSet<String>,
}

impl BlockDefinitionModel {
    pub fn new(block: Identifier, properties: BTreeSet<String>) -> Self {
        Self { block, properties }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatePropertiesPredicateModel {
    properties: BTreeMap<String, String>,
}

impl StatePropertiesPredicateModel {
    pub fn new(properties: BTreeMap<String, String>) -> Self {
        Self { properties }
    }

    pub fn requiring(name: &str, value: &str) -> Self {
        Self::new(BTreeMap::from([(name.to_string(), value.to_string())]))
    }

    pub fn check_state(&self, definition: &BlockDefinitionModel) -> Option<String> {
        self.properties
            .keys()
            .find(|property| !definition.properties.contains(*property))
            .cloned()
    }

    pub fn matches(&self, state: &BlockStateModel) -> bool {
        self.properties
            .iter()
            .all(|(property, expected)| state.properties.get(property) == Some(expected))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn state(block: &str, properties: &[(&str, &str)]) -> BlockStateModel {
        BlockStateModel {
            block: id(block),
            properties: properties
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
        }
    }

    fn definition(block: &str, properties: &[&str]) -> BlockDefinitionModel {
        BlockDefinitionModel::new(
            id(block),
            properties
                .iter()
                .map(|property| property.to_string())
                .collect(),
        )
    }

    #[test]
    fn omitted_block_and_state_match_any_entered_block() {
        let instance = EnterBlockTriggerInstance::new(None, None);

        assert!(instance.matches(&state("minecraft:powder_snow", &[("layers", "1")])));
        assert!(instance.matches(&state("minecraft:water", &[("level", "0")])));
    }

    #[test]
    fn block_filter_is_checked_before_state_predicate_like_java() {
        let instance = EnterBlockTriggerInstance::new(
            Some(id("minecraft:powder_snow")),
            Some(StatePropertiesPredicateModel::requiring("layers", "1")),
        );

        assert!(instance.matches(&state("minecraft:powder_snow", &[("layers", "1")])));
        assert!(!instance.matches(&state("minecraft:water", &[("layers", "1")])));
        assert!(!instance.matches(&state("minecraft:powder_snow", &[("layers", "2")])));
    }

    #[test]
    fn validation_checks_state_properties_only_when_block_is_present() {
        let state_predicate = StatePropertiesPredicateModel::requiring("missing", "true");
        let without_block = EnterBlockTriggerInstance::new(None, Some(state_predicate.clone()));
        let with_block = EnterBlockTriggerInstance::new(
            Some(id("minecraft:powder_snow")),
            Some(state_predicate),
        );
        let powder_snow = definition("minecraft:powder_snow", &["layers"]);

        assert_eq!(without_block.validate(Some(&powder_snow)), Ok(()));
        assert_eq!(
            with_block.validate(Some(&powder_snow)),
            Err("Blockminecraft:powder_snow has no property missing".to_string())
        );
    }

    #[test]
    fn validation_succeeds_when_block_or_state_is_absent() {
        let powder_snow = definition("minecraft:powder_snow", &["layers"]);

        assert_eq!(
            EnterBlockTriggerInstance::new(Some(id("minecraft:powder_snow")), None)
                .validate(Some(&powder_snow)),
            Ok(())
        );
        assert_eq!(
            EnterBlockTriggerInstance::new(None, None).validate(Some(&powder_snow)),
            Ok(())
        );
    }

    #[test]
    fn enters_block_factory_uses_java_trigger_id_and_block_field() {
        let criterion = EnterBlockTriggerInstance::enters_block(id("minecraft:powder_snow"));

        assert_eq!(criterion.trigger_id, id("minecraft:enter_block"));
        assert_eq!(
            criterion.instance,
            EnterBlockTriggerInstance::new(Some(id("minecraft:powder_snow")), None)
        );
        assert!(!criterion.instance.player_predicate_present);
    }
}
