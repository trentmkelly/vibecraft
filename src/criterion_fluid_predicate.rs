use std::collections::{BTreeMap, BTreeSet};

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FluidPredicateModel {
    fluids: Option<BTreeSet<Identifier>>,
    properties: Option<StatePropertiesPredicateModel>,
}

impl FluidPredicateModel {
    pub fn new(
        fluids: Option<BTreeSet<Identifier>>,
        properties: Option<StatePropertiesPredicateModel>,
    ) -> Self {
        Self { fluids, properties }
    }

    pub fn builder() -> FluidPredicateBuilderModel {
        FluidPredicateBuilderModel::default()
    }

    pub fn matches(&self, level: &FluidPredicateLevelModel, pos: (i32, i32, i32)) -> bool {
        if !level.loaded_positions.contains(&pos) {
            return false;
        }

        let state = level.fluid_states.get(&pos).unwrap_or(&level.empty_state);
        if self
            .fluids
            .as_ref()
            .is_some_and(|fluids| !fluids.contains(&state.fluid))
        {
            return false;
        }

        self.properties
            .as_ref()
            .is_none_or(|properties| properties.matches(state))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FluidPredicateBuilderModel {
    fluids: Option<BTreeSet<Identifier>>,
    properties: Option<StatePropertiesPredicateModel>,
}

impl FluidPredicateBuilderModel {
    pub fn fluid() -> Self {
        Self::default()
    }

    pub fn of_fluid(mut self, fluid: Identifier) -> Self {
        self.fluids = Some(BTreeSet::from([fluid]));
        self
    }

    pub fn of_fluids(mut self, fluids: impl IntoIterator<Item = Identifier>) -> Self {
        self.fluids = Some(fluids.into_iter().collect());
        self
    }

    pub fn set_properties(mut self, properties: StatePropertiesPredicateModel) -> Self {
        self.properties = Some(properties);
        self
    }

    pub fn build(self) -> FluidPredicateModel {
        FluidPredicateModel::new(self.fluids, self.properties)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FluidPredicateLevelModel {
    pub loaded_positions: BTreeSet<(i32, i32, i32)>,
    pub fluid_states: BTreeMap<(i32, i32, i32), FluidStateModel>,
    pub empty_state: FluidStateModel,
}

impl FluidPredicateLevelModel {
    pub fn new(
        loaded_positions: BTreeSet<(i32, i32, i32)>,
        fluid_states: BTreeMap<(i32, i32, i32), FluidStateModel>,
    ) -> Self {
        Self {
            loaded_positions,
            fluid_states,
            empty_state: FluidStateModel::new(Identifier::parse("minecraft:empty").unwrap(), []),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FluidStateModel {
    fluid: Identifier,
    properties: BTreeMap<String, String>,
}

impl FluidStateModel {
    pub fn new(
        fluid: Identifier,
        properties: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> Self {
        Self {
            fluid,
            properties: properties
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        }
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

    pub fn matches(&self, state: &FluidStateModel) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn state(fluid: &str, properties: &[(&'static str, &'static str)]) -> FluidStateModel {
        FluidStateModel::new(id(fluid), properties.iter().copied())
    }

    fn level() -> FluidPredicateLevelModel {
        FluidPredicateLevelModel::new(
            BTreeSet::from([(1, 64, 2), (2, 64, 2)]),
            BTreeMap::from([(
                (1, 64, 2),
                state("minecraft:water", &[("level", "0"), ("falling", "false")]),
            )]),
        )
    }

    #[test]
    fn loaded_position_is_required_before_reading_fluid_state() {
        let predicate = FluidPredicateModel::builder()
            .of_fluid(id("minecraft:water"))
            .build();

        assert!(predicate.matches(&level(), (1, 64, 2)));
        assert!(!predicate.matches(&level(), (8, 64, 2)));
    }

    #[test]
    fn omitted_predicates_match_any_loaded_fluid_state() {
        let predicate = FluidPredicateModel::builder().build();

        assert!(predicate.matches(&level(), (1, 64, 2)));
        assert!(predicate.matches(&level(), (2, 64, 2)));
    }

    #[test]
    fn optional_fluid_holder_set_filters_state_type() {
        let water_or_lava = FluidPredicateModel::builder()
            .of_fluids([id("minecraft:water"), id("minecraft:lava")])
            .build();
        let lava_only = FluidPredicateModel::builder()
            .of_fluid(id("minecraft:lava"))
            .build();

        assert!(water_or_lava.matches(&level(), (1, 64, 2)));
        assert!(!lava_only.matches(&level(), (1, 64, 2)));
    }

    #[test]
    fn state_properties_match_exact_range_and_missing_properties() {
        let predicate = FluidPredicateModel::builder()
            .set_properties(StatePropertiesPredicateModel::new(vec![
                PropertyMatcherModel::exact("falling", "false"),
                PropertyMatcherModel::ranged("level", Some("0"), Some("2")),
            ]))
            .build();
        let wrong_value = FluidPredicateModel::builder()
            .set_properties(StatePropertiesPredicateModel::new(vec![
                PropertyMatcherModel::exact("falling", "true"),
            ]))
            .build();
        let missing_property = FluidPredicateModel::builder()
            .set_properties(StatePropertiesPredicateModel::new(vec![
                PropertyMatcherModel::exact("unknown", "value"),
            ]))
            .build();

        assert!(predicate.matches(&level(), (1, 64, 2)));
        assert!(!wrong_value.matches(&level(), (1, 64, 2)));
        assert!(!missing_property.matches(&level(), (1, 64, 2)));
    }

    #[test]
    fn invalid_typed_range_bound_fails_like_java_property_lookup() {
        let predicate = FluidPredicateModel::builder()
            .set_properties(StatePropertiesPredicateModel::new(vec![
                PropertyMatcherModel::ranged("level", Some("not_a_level"), None),
            ]))
            .build();

        assert!(!predicate.matches(&level(), (1, 64, 2)));
    }

    #[test]
    fn builder_preserves_direct_fluid_set_and_properties() {
        let properties =
            StatePropertiesPredicateModel::new(vec![PropertyMatcherModel::exact("level", "0")]);
        let predicate = FluidPredicateBuilderModel::fluid()
            .of_fluid(id("minecraft:water"))
            .set_properties(properties.clone())
            .build();

        assert_eq!(
            predicate.fluids,
            Some(BTreeSet::from([id("minecraft:water")]))
        );
        assert_eq!(predicate.properties, Some(properties));
    }
}
