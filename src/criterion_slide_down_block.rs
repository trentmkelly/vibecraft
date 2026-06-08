use std::collections::{BTreeMap, BTreeSet};

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideDownBlockTriggerInstance {
    pub player: Option<ContextAwarePredicateModel>,
    pub block: Option<Identifier>,
    pub state: Option<StatePropertiesPredicateModel>,
}

impl SlideDownBlockTriggerInstance {
    pub fn new(
        player: Option<ContextAwarePredicateModel>,
        block: Option<Identifier>,
        state: Option<StatePropertiesPredicateModel>,
    ) -> Self {
        Self {
            player,
            block,
            state,
        }
    }

    pub fn codec_field_names() -> [&'static str; 3] {
        ["player", "block", "state"]
    }

    pub fn validate(&self, registry: &BlockRegistryModel) -> Result<(), String> {
        let Some(block) = &self.block else {
            return Ok(());
        };
        let Some(state) = &self.state else {
            return Ok(());
        };
        let Some(definition) = registry.state_definition(block) else {
            return Ok(());
        };

        if let Some(property) = state.check_state(definition) {
            return Err(format!("Block{} has no property {}", block, property));
        }

        Ok(())
    }

    pub fn slides_down_block(block: Identifier) -> SlideDownBlockCriterion {
        SlideDownBlockCriterion {
            trigger_id: id("minecraft:slide_down_block"),
            instance: Self::new(None, Some(block), None),
        }
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideDownBlockCriterion {
    pub trigger_id: Identifier,
    pub instance: SlideDownBlockTriggerInstance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateModel {
    block: Identifier,
    properties: BTreeMap<String, String>,
}

impl BlockStateModel {
    pub fn new(
        block: Identifier,
        properties: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> Self {
        Self {
            block,
            properties: properties
                .into_iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlockRegistryModel {
    state_definitions: BTreeMap<Identifier, BTreeSet<String>>,
}

impl BlockRegistryModel {
    pub fn new(
        state_definitions: impl IntoIterator<Item = (Identifier, Vec<&'static str>)>,
    ) -> Self {
        Self {
            state_definitions: state_definitions
                .into_iter()
                .map(|(block, properties)| {
                    (block, properties.into_iter().map(str::to_string).collect())
                })
                .collect(),
        }
    }

    fn state_definition(&self, block: &Identifier) -> Option<&BTreeSet<String>> {
        self.state_definitions.get(block)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatePropertiesPredicateModel {
    properties: Vec<PropertyMatcherModel>,
}

impl StatePropertiesPredicateModel {
    pub fn new(properties: Vec<PropertyMatcherModel>) -> Self {
        Self { properties }
    }

    pub fn exact(name: &str, value: &str) -> Self {
        Self::new(vec![PropertyMatcherModel::exact(name, value)])
    }

    pub fn check_state(&self, known_properties: &BTreeSet<String>) -> Option<String> {
        self.properties.iter().find_map(|matcher| {
            (!known_properties.contains(&matcher.name)).then(|| matcher.name.clone())
        })
    }

    pub fn matches(&self, state: &BlockStateModel) -> bool {
        self.properties
            .iter()
            .all(|matcher| matcher.matches(&state.properties))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyMatcherModel {
    name: String,
    value_matcher: ValueMatcherModel,
}

impl PropertyMatcherModel {
    pub fn exact(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value_matcher: ValueMatcherModel::Exact(value.to_string()),
        }
    }

    pub fn ranged(name: &str, min: Option<&str>, max: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            value_matcher: ValueMatcherModel::Ranged {
                min: min.map(str::to_string),
                max: max.map(str::to_string),
            },
        }
    }

    fn matches(&self, properties: &BTreeMap<String, String>) -> bool {
        properties
            .get(&self.name)
            .is_some_and(|value| self.value_matcher.matches(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueMatcherModel {
    Exact(String),
    Ranged {
        min: Option<String>,
        max: Option<String>,
    },
}

impl ValueMatcherModel {
    fn matches(&self, value: &str) -> bool {
        match self {
            Self::Exact(expected) => value == expected,
            Self::Ranged { min, max } => {
                min.as_ref()
                    .is_none_or(|min| property_value_gte(value, min))
                    && max
                        .as_ref()
                        .is_none_or(|max| property_value_lte(value, max))
            }
        }
    }
}

fn property_value_gte(value: &str, min: &str) -> bool {
    match (value.parse::<i64>(), min.parse::<i64>()) {
        (Ok(value), Ok(min)) => value >= min,
        (Err(_), Err(_)) => value >= min,
        _ => false,
    }
}

fn property_value_lte(value: &str, max: &str) -> bool {
    match (value.parse::<i64>(), max.parse::<i64>()) {
        (Ok(value), Ok(max)) => value <= max,
        (Err(_), Err(_)) => value <= max,
        _ => false,
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(block: &str, properties: &[(&'static str, &'static str)]) -> BlockStateModel {
        BlockStateModel::new(id(block), properties.iter().copied())
    }

    fn registry() -> BlockRegistryModel {
        BlockRegistryModel::new([
            (id("minecraft:honey_block"), vec!["slippery", "level"]),
            (id("minecraft:slime_block"), vec!["slippery"]),
        ])
    }

    #[test]
    fn codec_fields_match_java_record_codec() {
        assert_eq!(
            SlideDownBlockTriggerInstance::codec_field_names(),
            ["player", "block", "state"]
        );
    }

    #[test]
    fn validation_checks_state_properties_only_when_block_is_present() {
        let predicate = StatePropertiesPredicateModel::exact("missing", "true");
        let without_block = SlideDownBlockTriggerInstance::new(None, None, Some(predicate.clone()));
        let with_block = SlideDownBlockTriggerInstance::new(
            None,
            Some(id("minecraft:honey_block")),
            Some(predicate),
        );

        assert_eq!(without_block.validate(&registry()), Ok(()));
        assert_eq!(
            with_block.validate(&registry()),
            Err("Blockminecraft:honey_block has no property missing".to_string())
        );
    }

    #[test]
    fn slides_down_block_factory_uses_honey_block_slide_trigger_and_empty_player_state() {
        let criterion =
            SlideDownBlockTriggerInstance::slides_down_block(id("minecraft:honey_block"));

        assert_eq!(criterion.trigger_id, id("minecraft:slide_down_block"));
        assert!(criterion.instance.player.is_none());
        assert_eq!(criterion.instance.block, Some(id("minecraft:honey_block")));
        assert!(criterion.instance.state.is_none());
    }

    #[test]
    fn omitted_block_and_state_match_any_slid_block_state() {
        let instance = SlideDownBlockTriggerInstance::new(None, None, None);

        assert!(instance.matches(&state("minecraft:honey_block", &[("slippery", "true")])));
        assert!(instance.matches(&state("minecraft:stone", &[("facing", "north")])));
    }

    #[test]
    fn block_filter_is_checked_before_state_predicate() {
        let instance = SlideDownBlockTriggerInstance::new(
            None,
            Some(id("minecraft:honey_block")),
            Some(StatePropertiesPredicateModel::exact("slippery", "true")),
        );

        assert!(instance.matches(&state("minecraft:honey_block", &[("slippery", "true")])));
        assert!(!instance.matches(&state("minecraft:slime_block", &[("slippery", "true")])));
        assert!(!instance.matches(&state("minecraft:honey_block", &[("slippery", "false")])));
    }

    #[test]
    fn state_predicate_can_match_without_block_filter() {
        let instance = SlideDownBlockTriggerInstance::new(
            None,
            None,
            Some(StatePropertiesPredicateModel::new(vec![
                PropertyMatcherModel::ranged("level", Some("2"), Some("4")),
            ])),
        );

        assert!(instance.matches(&state("minecraft:honey_block", &[("level", "3")])));
        assert!(!instance.matches(&state("minecraft:honey_block", &[("level", "5")])));
        assert!(!instance.matches(&state("minecraft:honey_block", &[("slippery", "true")])));
    }
}
