use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatePropertiesPredicateModel {
    properties: Vec<PropertyMatcherModel>,
}

impl StatePropertiesPredicateModel {
    pub fn new(properties: impl IntoIterator<Item = PropertyMatcherModel>) -> Self {
        Self {
            properties: properties.into_iter().collect(),
        }
    }

    pub fn codec_shape() -> CodecShapeModel {
        CodecShapeModel {
            properties_codec: "Codec.unboundedMap(Codec.STRING, ValueMatcher.CODEC)",
            codec_mapping:
                "xmap(StatePropertiesPredicate::new, StatePropertiesPredicate::properties)",
            stream_codec: "PropertyMatcher.STREAM_CODEC.apply(ByteBufCodecs.list())",
        }
    }

    pub fn properties(&self) -> &[PropertyMatcherModel] {
        &self.properties
    }

    pub fn matches(&self, definition: &StateDefinitionModel, state: &StateHolderModel) -> bool {
        self.properties
            .iter()
            .all(|matcher| matcher.matches(definition, state))
    }

    pub fn matches_block_state(&self, state: &BlockStateModel) -> bool {
        self.matches(state.definition(), state.state())
    }

    pub fn matches_fluid_state(&self, state: &FluidStateModel) -> bool {
        self.matches(state.definition(), state.state())
    }

    pub fn check_state(&self, states: &StateDefinitionModel) -> Option<String> {
        self.properties
            .iter()
            .find_map(|matcher| matcher.check_state(states))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodecShapeModel {
    pub properties_codec: &'static str,
    pub codec_mapping: &'static str,
    pub stream_codec: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatePropertiesPredicateBuilderModel {
    matchers: Vec<PropertyMatcherModel>,
}

impl StatePropertiesPredicateBuilderModel {
    pub fn properties() -> Self {
        Self::default()
    }

    pub fn has_property_string(mut self, property: &PropertyDefinitionModel, value: &str) -> Self {
        self.matchers
            .push(PropertyMatcherModel::exact(property.name(), value));
        self
    }

    pub fn has_property_int(mut self, property: &PropertyDefinitionModel, value: i32) -> Self {
        self.matchers.push(PropertyMatcherModel::exact(
            property.name(),
            &value.to_string(),
        ));
        self
    }

    pub fn has_property_bool(mut self, property: &PropertyDefinitionModel, value: bool) -> Self {
        self.matchers.push(PropertyMatcherModel::exact(
            property.name(),
            if value { "true" } else { "false" },
        ));
        self
    }

    pub fn has_property_representable(
        mut self,
        property: &PropertyDefinitionModel,
        value: &StringRepresentableValueModel,
    ) -> Self {
        self.matchers.push(PropertyMatcherModel::exact(
            property.name(),
            value.serialized_name(),
        ));
        self
    }

    pub fn build(self) -> Option<StatePropertiesPredicateModel> {
        Some(StatePropertiesPredicateModel::new(self.matchers))
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

    pub fn stream_codec_shape() -> &'static str {
        "StreamCodec.composite(ByteBufCodecs.STRING_UTF8, ValueMatcher.STREAM_CODEC)"
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value_matcher(&self) -> &ValueMatcherModel {
        &self.value_matcher
    }

    fn matches(&self, definition: &StateDefinitionModel, state: &StateHolderModel) -> bool {
        let Some(property) = definition.get_property(&self.name) else {
            return false;
        };

        self.value_matcher.match_value(state, property)
    }

    fn check_state(&self, states: &StateDefinitionModel) -> Option<String> {
        states
            .get_property(&self.name)
            .is_none()
            .then(|| self.name.clone())
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
    pub fn codec_shape() -> &'static str {
        "Codec.either(ExactMatcher.CODEC, RangedMatcher.CODEC)"
    }

    pub fn stream_codec_shape() -> &'static str {
        "ByteBufCodecs.either(ExactMatcher.STREAM_CODEC, RangedMatcher.STREAM_CODEC)"
    }

    fn match_value(&self, state: &StateHolderModel, property: &PropertyDefinitionModel) -> bool {
        let Some(actual) = state.get_value(property.name()) else {
            return false;
        };

        let Some(actual_rank) = property.typed_value(actual) else {
            return false;
        };

        match self {
            Self::Exact(expected) => property
                .typed_value(expected)
                .is_some_and(|expected_rank| actual_rank == expected_rank),
            Self::Ranged { min, max } => {
                if let Some(min) = min {
                    let Some(min_rank) = property.typed_value(min) else {
                        return false;
                    };
                    if actual_rank < min_rank {
                        return false;
                    }
                }

                if let Some(max) = max {
                    let Some(max_rank) = property.typed_value(max) else {
                        return false;
                    };
                    if actual_rank > max_rank {
                        return false;
                    }
                }

                true
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDefinitionModel {
    properties: BTreeMap<String, PropertyDefinitionModel>,
}

impl StateDefinitionModel {
    pub fn new(properties: impl IntoIterator<Item = PropertyDefinitionModel>) -> Self {
        Self {
            properties: properties
                .into_iter()
                .map(|property| (property.name.clone(), property))
                .collect(),
        }
    }

    fn get_property(&self, name: &str) -> Option<&PropertyDefinitionModel> {
        self.properties.get(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyDefinitionModel {
    name: String,
    values: BTreeMap<String, i32>,
}

impl PropertyDefinitionModel {
    pub fn ordered(name: &str, values: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            name: name.to_string(),
            values: values
                .into_iter()
                .enumerate()
                .map(|(rank, value)| (value.to_string(), rank as i32))
                .collect(),
        }
    }

    pub fn integer(name: &str, min: i32, max: i32) -> Self {
        Self {
            name: name.to_string(),
            values: (min..=max)
                .map(|value| (value.to_string(), value))
                .collect(),
        }
    }

    pub fn boolean(name: &str) -> Self {
        Self::ordered(name, ["false", "true"])
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn typed_value(&self, value: &str) -> Option<i32> {
        self.values.get(value).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateHolderModel {
    values: BTreeMap<String, String>,
}

impl StateHolderModel {
    pub fn new(values: impl IntoIterator<Item = (&'static str, &'static str)>) -> Self {
        Self {
            values: values
                .into_iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
        }
    }

    fn get_value(&self, property: &str) -> Option<&str> {
        self.values.get(property).map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateModel {
    definition: StateDefinitionModel,
    state: StateHolderModel,
}

impl BlockStateModel {
    pub fn new(definition: StateDefinitionModel, state: StateHolderModel) -> Self {
        Self { definition, state }
    }

    fn definition(&self) -> &StateDefinitionModel {
        &self.definition
    }

    fn state(&self) -> &StateHolderModel {
        &self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FluidStateModel {
    definition: StateDefinitionModel,
    state: StateHolderModel,
}

impl FluidStateModel {
    pub fn new(definition: StateDefinitionModel, state: StateHolderModel) -> Self {
        Self { definition, state }
    }

    fn definition(&self) -> &StateDefinitionModel {
        &self.definition
    }

    fn state(&self) -> &StateHolderModel {
        &self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringRepresentableValueModel {
    serialized_name: String,
}

impl StringRepresentableValueModel {
    pub fn new(serialized_name: &str) -> Self {
        Self {
            serialized_name: serialized_name.to_string(),
        }
    }

    fn serialized_name(&self) -> &str {
        &self.serialized_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_definition() -> StateDefinitionModel {
        StateDefinitionModel::new([
            PropertyDefinitionModel::ordered("facing", ["north", "south", "east", "west"]),
            PropertyDefinitionModel::integer("level", 0, 15),
            PropertyDefinitionModel::boolean("powered"),
        ])
    }

    fn block_state(properties: &[(&'static str, &'static str)]) -> BlockStateModel {
        BlockStateModel::new(
            block_definition(),
            StateHolderModel::new(properties.iter().copied()),
        )
    }

    fn fluid_state(properties: &[(&'static str, &'static str)]) -> FluidStateModel {
        FluidStateModel::new(
            StateDefinitionModel::new([
                PropertyDefinitionModel::integer("level", 0, 8),
                PropertyDefinitionModel::boolean("falling"),
            ]),
            StateHolderModel::new(properties.iter().copied()),
        )
    }

    #[test]
    fn state_properties_codec_and_stream_codec_shapes_match_java() {
        assert_eq!(
            StatePropertiesPredicateModel::codec_shape(),
            CodecShapeModel {
                properties_codec: "Codec.unboundedMap(Codec.STRING, ValueMatcher.CODEC)",
                codec_mapping:
                    "xmap(StatePropertiesPredicate::new, StatePropertiesPredicate::properties)",
                stream_codec: "PropertyMatcher.STREAM_CODEC.apply(ByteBufCodecs.list())",
            }
        );
        assert_eq!(
            ValueMatcherModel::codec_shape(),
            "Codec.either(ExactMatcher.CODEC, RangedMatcher.CODEC)"
        );
        assert_eq!(
            ValueMatcherModel::stream_codec_shape(),
            "ByteBufCodecs.either(ExactMatcher.STREAM_CODEC, RangedMatcher.STREAM_CODEC)"
        );
        assert_eq!(
            PropertyMatcherModel::stream_codec_shape(),
            "StreamCodec.composite(ByteBufCodecs.STRING_UTF8, ValueMatcher.STREAM_CODEC)"
        );
    }

    #[test]
    fn empty_predicate_matches_any_state_and_builder_still_returns_some() {
        let predicate = StatePropertiesPredicateModel::new([]);
        let built = StatePropertiesPredicateBuilderModel::properties().build();

        assert!(predicate.matches_block_state(&block_state(&[])));
        assert!(predicate.matches_fluid_state(&fluid_state(&[])));
        assert_eq!(built, Some(StatePropertiesPredicateModel::new([])));
    }

    #[test]
    fn all_property_matchers_must_match() {
        let predicate = StatePropertiesPredicateModel::new([
            PropertyMatcherModel::exact("facing", "north"),
            PropertyMatcherModel::ranged("level", Some("2"), Some("4")),
            PropertyMatcherModel::exact("powered", "true"),
        ]);

        assert!(predicate.matches_block_state(&block_state(&[
            ("facing", "north"),
            ("level", "3"),
            ("powered", "true"),
        ])));
        assert!(!predicate.matches_block_state(&block_state(&[
            ("facing", "north"),
            ("level", "5"),
            ("powered", "true"),
        ])));
    }

    #[test]
    fn exact_matcher_parses_expected_value_through_property() {
        let predicate = StatePropertiesPredicateModel::new([
            PropertyMatcherModel::exact("level", "3"),
            PropertyMatcherModel::exact("powered", "true"),
        ]);
        let invalid_expected =
            StatePropertiesPredicateModel::new([PropertyMatcherModel::exact("level", "three")]);

        assert!(
            predicate.matches_block_state(&block_state(&[("level", "3"), ("powered", "true"),]))
        );
        assert!(
            !predicate.matches_block_state(&block_state(&[("level", "4"), ("powered", "true"),]))
        );
        assert!(!invalid_expected.matches_block_state(&block_state(&[("level", "3")])));
    }

    #[test]
    fn ranged_matcher_parses_bounds_and_honors_open_ends() {
        let between = StatePropertiesPredicateModel::new([PropertyMatcherModel::ranged(
            "level",
            Some("2"),
            Some("4"),
        )]);
        let min_only = StatePropertiesPredicateModel::new([PropertyMatcherModel::ranged(
            "level",
            Some("2"),
            None,
        )]);
        let max_only = StatePropertiesPredicateModel::new([PropertyMatcherModel::ranged(
            "level",
            None,
            Some("4"),
        )]);

        assert!(between.matches_block_state(&block_state(&[("level", "3")])));
        assert!(!between.matches_block_state(&block_state(&[("level", "1")])));
        assert!(!between.matches_block_state(&block_state(&[("level", "5")])));
        assert!(min_only.matches_block_state(&block_state(&[("level", "15")])));
        assert!(!min_only.matches_block_state(&block_state(&[("level", "1")])));
        assert!(max_only.matches_block_state(&block_state(&[("level", "0")])));
        assert!(!max_only.matches_block_state(&block_state(&[("level", "5")])));
    }

    #[test]
    fn invalid_range_bounds_or_state_values_fail_match() {
        let invalid_min = StatePropertiesPredicateModel::new([PropertyMatcherModel::ranged(
            "level",
            Some("low"),
            None,
        )]);
        let invalid_max = StatePropertiesPredicateModel::new([PropertyMatcherModel::ranged(
            "level",
            None,
            Some("high"),
        )]);
        let valid_range = StatePropertiesPredicateModel::new([PropertyMatcherModel::ranged(
            "level",
            Some("2"),
            Some("4"),
        )]);

        assert!(!invalid_min.matches_block_state(&block_state(&[("level", "3")])));
        assert!(!invalid_max.matches_block_state(&block_state(&[("level", "3")])));
        assert!(!valid_range.matches_block_state(&block_state(&[("level", "unknown")])));
    }

    #[test]
    fn missing_property_fails_match_and_check_state_returns_first_unknown_name() {
        let predicate = StatePropertiesPredicateModel::new([
            PropertyMatcherModel::exact("facing", "north"),
            PropertyMatcherModel::exact("missing", "value"),
            PropertyMatcherModel::exact("also_missing", "value"),
        ]);

        assert!(!predicate.matches_block_state(&block_state(&[("facing", "north")])));
        assert_eq!(
            predicate.check_state(&block_definition()),
            Some("missing".to_string())
        );
    }

    #[test]
    fn block_and_fluid_state_overloads_use_their_own_state_definitions() {
        let fluid_predicate = StatePropertiesPredicateModel::new([
            PropertyMatcherModel::exact("falling", "false"),
            PropertyMatcherModel::ranged("level", Some("1"), Some("2")),
        ]);

        assert!(fluid_predicate
            .matches_fluid_state(&fluid_state(&[("falling", "false"), ("level", "2"),])));
        assert!(!fluid_predicate
            .matches_fluid_state(&fluid_state(&[("falling", "false"), ("level", "3"),])));
    }

    #[test]
    fn builder_has_property_overloads_store_exact_serialized_values() {
        let definition = block_definition();
        let facing = definition.get_property("facing").unwrap();
        let level = definition.get_property("level").unwrap();
        let powered = definition.get_property("powered").unwrap();

        let predicate = StatePropertiesPredicateBuilderModel::properties()
            .has_property_string(facing, "south")
            .has_property_int(level, 2)
            .has_property_bool(powered, true)
            .has_property_representable(facing, &StringRepresentableValueModel::new("east"))
            .build()
            .unwrap();

        assert_eq!(predicate.properties().len(), 4);
        assert_eq!(predicate.properties()[0].name(), "facing");
        assert_eq!(
            predicate.properties()[0].value_matcher(),
            &ValueMatcherModel::Exact("south".to_string())
        );
        assert_eq!(
            predicate.properties()[1].value_matcher(),
            &ValueMatcherModel::Exact("2".to_string())
        );
        assert_eq!(
            predicate.properties()[2].value_matcher(),
            &ValueMatcherModel::Exact("true".to_string())
        );
        assert_eq!(
            predicate.properties()[3].value_matcher(),
            &ValueMatcherModel::Exact("east".to_string())
        );
    }
}
