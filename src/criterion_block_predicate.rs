use std::collections::{BTreeMap, BTreeSet};

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionBlockPredicate {
    blocks: Option<BTreeSet<Identifier>>,
    properties: Option<StatePropertiesPredicateModel>,
    nbt: Option<NbtPredicateModel>,
    components: DataComponentMatchersModel,
}

impl CriterionBlockPredicate {
    pub fn new(
        blocks: Option<BTreeSet<Identifier>>,
        properties: Option<StatePropertiesPredicateModel>,
        nbt: Option<NbtPredicateModel>,
        components: DataComponentMatchersModel,
    ) -> Self {
        Self {
            blocks,
            properties,
            nbt,
            components,
        }
    }

    pub fn builder() -> CriterionBlockPredicateBuilder {
        CriterionBlockPredicateBuilder::default()
    }

    pub fn matches_level(&self, level: &BlockPredicateLevelModel, pos: (i32, i32, i32)) -> bool {
        if !level.loaded_positions.contains(&pos) {
            return false;
        }

        let Some(state) = level.block_states.get(&pos) else {
            return false;
        };
        if !self.matches_state(state) {
            return false;
        }

        if self.nbt.is_some() || !self.components.is_empty() {
            let block_entity = level.block_entities.get(&pos);
            if self
                .nbt
                .as_ref()
                .is_some_and(|nbt| !matches_block_entity(block_entity, nbt))
            {
                return false;
            }

            if !self.components.is_empty()
                && !block_entity.is_some_and(|entity| self.components.test(&entity.components))
            {
                return false;
            }
        }

        true
    }

    pub fn matches_block_in_world(&self, block_in_world: &BlockInWorldModel) -> bool {
        if !self.matches_state(&block_in_world.state) {
            return false;
        }

        self.nbt
            .as_ref()
            .is_none_or(|nbt| matches_block_entity(block_in_world.entity.as_ref(), nbt))
    }

    fn matches_state(&self, state: &BlockStateModel) -> bool {
        if self
            .blocks
            .as_ref()
            .is_some_and(|blocks| !blocks.contains(&state.block))
        {
            return false;
        }

        self.properties
            .as_ref()
            .is_none_or(|properties| properties.matches(state))
    }

    pub fn requires_nbt(&self) -> bool {
        self.nbt.is_some()
    }
}

fn matches_block_entity(entity: Option<&BlockEntityModel>, nbt: &NbtPredicateModel) -> bool {
    entity.is_some_and(|entity| nbt.matches(&entity.nbt))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CriterionBlockPredicateBuilder {
    blocks: Option<BTreeSet<Identifier>>,
    properties: Option<StatePropertiesPredicateModel>,
    nbt: Option<NbtPredicateModel>,
    components: DataComponentMatchersModel,
}

impl CriterionBlockPredicateBuilder {
    pub fn of_blocks(mut self, blocks: impl IntoIterator<Item = Identifier>) -> Self {
        self.blocks = Some(blocks.into_iter().collect());
        self
    }

    pub fn has_nbt(mut self, nbt: BTreeMap<String, String>) -> Self {
        self.nbt = Some(NbtPredicateModel { tag: nbt });
        self
    }

    pub fn set_properties(mut self, properties: StatePropertiesPredicateModel) -> Self {
        self.properties = Some(properties);
        self
    }

    pub fn components(mut self, components: DataComponentMatchersModel) -> Self {
        self.components = components;
        self
    }

    pub fn build(self) -> CriterionBlockPredicate {
        CriterionBlockPredicate::new(self.blocks, self.properties, self.nbt, self.components)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPredicateLevelModel {
    pub loaded_positions: BTreeSet<(i32, i32, i32)>,
    pub block_states: BTreeMap<(i32, i32, i32), BlockStateModel>,
    pub block_entities: BTreeMap<(i32, i32, i32), BlockEntityModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInWorldModel {
    pub state: BlockStateModel,
    pub entity: Option<BlockEntityModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateModel {
    pub block: Identifier,
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityModel {
    pub nbt: BTreeMap<String, String>,
    pub components: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatePropertiesPredicateModel {
    properties: Vec<PropertyMatcherModel>,
}

impl StatePropertiesPredicateModel {
    pub fn new(properties: Vec<PropertyMatcherModel>) -> Self {
        Self { properties }
    }

    pub fn matches(&self, state: &BlockStateModel) -> bool {
        self.properties
            .iter()
            .all(|matcher| matcher.matches(&state.properties))
    }

    pub fn check_state(&self, known_properties: &BTreeSet<String>) -> Option<String> {
        self.properties.iter().find_map(|matcher| {
            (!known_properties.contains(&matcher.name)).then(|| matcher.name.clone())
        })
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtPredicateModel {
    tag: BTreeMap<String, String>,
}

impl NbtPredicateModel {
    pub fn matches(&self, tag: &BTreeMap<String, String>) -> bool {
        self.tag
            .iter()
            .all(|(key, value)| tag.get(key) == Some(value))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataComponentMatchersModel {
    required: BTreeMap<String, String>,
}

impl DataComponentMatchersModel {
    pub fn any() -> Self {
        Self::default()
    }

    pub fn requiring(component: &str, value: &str) -> Self {
        Self {
            required: BTreeMap::from([(component.to_string(), value.to_string())]),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.required.is_empty()
    }

    pub fn test(&self, components: &BTreeMap<String, String>) -> bool {
        self.required
            .iter()
            .all(|(key, value)| components.get(key) == Some(value))
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
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        }
    }

    fn entity(nbt: &[(&str, &str)], components: &[(&str, &str)]) -> BlockEntityModel {
        BlockEntityModel {
            nbt: nbt
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            components: components
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        }
    }

    fn level() -> BlockPredicateLevelModel {
        BlockPredicateLevelModel {
            loaded_positions: BTreeSet::from([(1, 64, 2)]),
            block_states: BTreeMap::from([(
                (1, 64, 2),
                state("minecraft:chest", &[("facing", "north"), ("level", "3")]),
            )]),
            block_entities: BTreeMap::from([(
                (1, 64, 2),
                entity(
                    &[("LootTable", "minecraft:chests/simple_dungeon")],
                    &[("minecraft:custom_name", "Treasure")],
                ),
            )]),
        }
    }

    #[test]
    fn block_predicate_level_match_requires_loaded_position_and_matching_state() {
        let predicate = CriterionBlockPredicate::builder()
            .of_blocks([id("minecraft:chest")])
            .set_properties(StatePropertiesPredicateModel::new(vec![
                PropertyMatcherModel::exact("facing", "north"),
                PropertyMatcherModel::ranged("level", Some("2"), Some("4")),
            ]))
            .build();

        assert!(predicate.matches_level(&level(), (1, 64, 2)));
        assert!(!predicate.matches_level(&level(), (2, 64, 2)));
        assert!(!CriterionBlockPredicate::builder()
            .of_blocks([id("minecraft:barrel")])
            .build()
            .matches_level(&level(), (1, 64, 2)));
        assert!(!CriterionBlockPredicate::builder()
            .set_properties(StatePropertiesPredicateModel::new(vec![
                PropertyMatcherModel::exact("facing", "south",)
            ]))
            .build()
            .matches_level(&level(), (1, 64, 2)));
    }

    #[test]
    fn block_predicate_level_match_checks_nbt_and_components_only_when_needed() {
        let predicate = CriterionBlockPredicate::builder()
            .has_nbt(BTreeMap::from([(
                "LootTable".to_string(),
                "minecraft:chests/simple_dungeon".to_string(),
            )]))
            .components(DataComponentMatchersModel::requiring(
                "minecraft:custom_name",
                "Treasure",
            ))
            .build();
        assert!(predicate.requires_nbt());
        assert!(predicate.matches_level(&level(), (1, 64, 2)));

        let mut no_entity = level();
        no_entity.block_entities.clear();
        assert!(!predicate.matches_level(&no_entity, (1, 64, 2)));

        let components_only = CriterionBlockPredicate::builder()
            .components(DataComponentMatchersModel::requiring(
                "minecraft:custom_name",
                "Treasure",
            ))
            .build();
        assert!(!components_only.requires_nbt());
        assert!(components_only.matches_level(&level(), (1, 64, 2)));
        assert!(!components_only.matches_level(&no_entity, (1, 64, 2)));
        assert!(DataComponentMatchersModel::any().is_empty());
    }

    #[test]
    fn block_predicate_block_in_world_matches_state_and_nbt_but_not_components() {
        let block_in_world = BlockInWorldModel {
            state: state("minecraft:chest", &[("facing", "north")]),
            entity: Some(entity(&[("id", "minecraft:chest")], &[])),
        };
        let predicate = CriterionBlockPredicate::builder()
            .of_blocks([id("minecraft:chest")])
            .has_nbt(BTreeMap::from([(
                "id".to_string(),
                "minecraft:chest".to_string(),
            )]))
            .components(DataComponentMatchersModel::requiring(
                "ignored",
                "by_block_in_world",
            ))
            .build();

        assert!(predicate.matches_block_in_world(&block_in_world));
        assert!(!CriterionBlockPredicate::builder()
            .has_nbt(BTreeMap::from([(
                "id".to_string(),
                "minecraft:barrel".to_string()
            )]))
            .build()
            .matches_block_in_world(&block_in_world));
    }

    #[test]
    fn state_properties_predicate_check_state_returns_first_unknown_property() {
        let predicate = StatePropertiesPredicateModel::new(vec![
            PropertyMatcherModel::exact("facing", "north"),
            PropertyMatcherModel::exact("missing", "value"),
        ]);
        assert_eq!(
            predicate.check_state(&BTreeSet::from(["facing".to_string()])),
            Some("missing".to_string())
        );
        assert_eq!(
            predicate.check_state(&BTreeSet::from([
                "facing".to_string(),
                "missing".to_string()
            ])),
            None
        );
    }

    #[test]
    fn state_properties_predicate_matches_exact_range_and_invalid_typed_bounds_like_java() {
        let matching_state = state("minecraft:chest", &[("level", "10"), ("facing", "north")]);
        let predicate = StatePropertiesPredicateModel::new(vec![
            PropertyMatcherModel::exact("facing", "north"),
            PropertyMatcherModel::ranged("level", Some("2"), Some("12")),
        ]);
        assert!(predicate.matches(&matching_state));

        let too_low = StatePropertiesPredicateModel::new(vec![PropertyMatcherModel::ranged(
            "level",
            Some("11"),
            None,
        )]);
        assert!(!too_low.matches(&matching_state));

        let invalid_bound = StatePropertiesPredicateModel::new(vec![PropertyMatcherModel::ranged(
            "level",
            Some("not_a_level"),
            None,
        )]);
        assert!(!invalid_bound.matches(&matching_state));
    }
}
