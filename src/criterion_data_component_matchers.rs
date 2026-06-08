use std::collections::BTreeMap;

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataComponentMatchersModel {
    pub exact: DataComponentExactPredicateModel,
    pub partial: BTreeMap<Identifier, DataComponentPredicateModel>,
}

impl DataComponentMatchersModel {
    pub fn any() -> Self {
        Self {
            exact: DataComponentExactPredicateModel::empty(),
            partial: BTreeMap::new(),
        }
    }

    pub fn new(
        exact: DataComponentExactPredicateModel,
        partial: BTreeMap<Identifier, DataComponentPredicateModel>,
    ) -> Self {
        Self { exact, partial }
    }

    pub fn builder() -> DataComponentMatchersBuilder {
        DataComponentMatchersBuilder::components()
    }

    pub fn test(&self, values: &DataComponentGetterModel) -> bool {
        if !self.exact.test(values) {
            return false;
        }

        for predicate in self.partial.values() {
            if !predicate.matches(values) {
                return false;
            }
        }

        true
    }

    pub fn is_empty(&self) -> bool {
        self.exact.is_empty() && self.partial.is_empty()
    }
}

impl Default for DataComponentMatchersModel {
    fn default() -> Self {
        Self::any()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataComponentMatchersBuilder {
    exact: DataComponentExactPredicateModel,
    partial: Vec<(Identifier, DataComponentPredicateModel)>,
}

impl DataComponentMatchersBuilder {
    pub fn components() -> Self {
        Self::default()
    }

    pub fn any(mut self, component_type: Identifier) -> Self {
        let predicate_type = DataComponentPredicateTypeModel::any_value(component_type.clone());
        self.partial.push((
            predicate_type.key(),
            DataComponentPredicateModel::any_value(component_type),
        ));
        self
    }

    pub fn partial(
        mut self,
        predicate_type: DataComponentPredicateTypeModel,
        predicate: DataComponentPredicateModel,
    ) -> Self {
        self.partial.push((predicate_type.key(), predicate));
        self
    }

    pub fn exact(mut self, exact: DataComponentExactPredicateModel) -> Self {
        self.exact = exact;
        self
    }

    pub fn build(self) -> Result<DataComponentMatchersModel, String> {
        let mut partial = BTreeMap::new();

        for (predicate_type, predicate) in self.partial {
            if partial.insert(predicate_type.clone(), predicate).is_some() {
                return Err(format!(
                    "Multiple entries with same key: '{}'",
                    predicate_type
                ));
            }
        }

        Ok(DataComponentMatchersModel::new(self.exact, partial))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataComponentExactPredicateModel {
    expected_components: Vec<TypedDataComponentModel>,
}

impl DataComponentExactPredicateModel {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn expect(component_type: Identifier, value: impl Into<String>) -> Self {
        Self {
            expected_components: vec![TypedDataComponentModel::new(component_type, value)],
        }
    }

    pub fn builder() -> DataComponentExactPredicateBuilder {
        DataComponentExactPredicateBuilder::default()
    }

    pub fn is_empty(&self) -> bool {
        self.expected_components.is_empty()
    }

    pub fn test(&self, values: &DataComponentGetterModel) -> bool {
        self.expected_components
            .iter()
            .all(|expected| values.get(&expected.component_type) == Some(expected.value.as_str()))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataComponentExactPredicateBuilder {
    expected_components: Vec<TypedDataComponentModel>,
}

impl DataComponentExactPredicateBuilder {
    pub fn expect(
        mut self,
        component_type: Identifier,
        value: impl Into<String>,
    ) -> Result<Self, String> {
        if self
            .expected_components
            .iter()
            .any(|component| component.component_type == component_type)
        {
            return Err(format!(
                "Predicate already has component of type: '{}'",
                component_type
            ));
        }

        self.expected_components
            .push(TypedDataComponentModel::new(component_type, value));
        Ok(self)
    }

    pub fn build(self) -> DataComponentExactPredicateModel {
        DataComponentExactPredicateModel {
            expected_components: self.expected_components,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedDataComponentModel {
    component_type: Identifier,
    value: String,
}

impl TypedDataComponentModel {
    pub fn new(component_type: Identifier, value: impl Into<String>) -> Self {
        Self {
            component_type,
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataComponentPredicateTypeModel {
    AnyValue { component_type: Identifier },
    Concrete { predicate_type: Identifier },
}

impl DataComponentPredicateTypeModel {
    pub fn any_value(component_type: Identifier) -> Self {
        Self::AnyValue { component_type }
    }

    pub fn concrete(predicate_type: Identifier) -> Self {
        Self::Concrete { predicate_type }
    }

    fn key(&self) -> Identifier {
        match self {
            Self::AnyValue { component_type } => component_type.clone(),
            Self::Concrete { predicate_type } => predicate_type.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataComponentPredicateModel {
    AnyValue {
        component_type: Identifier,
    },
    ExactValue {
        component_type: Identifier,
        expected_value: String,
    },
}

impl DataComponentPredicateModel {
    pub fn any_value(component_type: Identifier) -> Self {
        Self::AnyValue { component_type }
    }

    pub fn exact_value(component_type: Identifier, expected_value: impl Into<String>) -> Self {
        Self::ExactValue {
            component_type,
            expected_value: expected_value.into(),
        }
    }

    pub fn matches(&self, values: &DataComponentGetterModel) -> bool {
        match self {
            Self::AnyValue { component_type } => values.get(component_type).is_some(),
            Self::ExactValue {
                component_type,
                expected_value,
            } => values.get(component_type) == Some(expected_value.as_str()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataComponentGetterModel {
    components: BTreeMap<Identifier, String>,
}

impl DataComponentGetterModel {
    pub fn from_pairs(pairs: Vec<(Identifier, impl Into<String>)>) -> Self {
        Self {
            components: pairs
                .into_iter()
                .map(|(component_type, value)| (component_type, value.into()))
                .collect(),
        }
    }

    pub fn get(&self, component_type: &Identifier) -> Option<&str> {
        self.components.get(component_type).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn getter(pairs: &[(&str, &str)]) -> DataComponentGetterModel {
        DataComponentGetterModel::from_pairs(
            pairs
                .iter()
                .map(|(component_type, value)| (id(component_type), *value))
                .collect(),
        )
    }

    #[test]
    fn any_matcher_is_empty_and_matches_every_component_getter() {
        let matcher = DataComponentMatchersModel::any();

        assert!(matcher.is_empty());
        assert!(matcher.test(&DataComponentGetterModel::default()));
        assert!(matcher.test(&getter(&[("minecraft:custom_name", "Workbench")])));
    }

    #[test]
    fn exact_predicate_requires_all_expected_component_values() {
        let matcher = DataComponentMatchersModel::builder()
            .exact(
                DataComponentExactPredicateModel::builder()
                    .expect(id("minecraft:custom_name"), "Workbench")
                    .unwrap()
                    .expect(id("minecraft:max_damage"), "250")
                    .unwrap()
                    .build(),
            )
            .build()
            .unwrap();

        assert!(matcher.test(&getter(&[
            ("minecraft:custom_name", "Workbench"),
            ("minecraft:max_damage", "250"),
            ("minecraft:damage", "4"),
        ])));
        assert!(!matcher.test(&getter(&[
            ("minecraft:custom_name", "Workbench"),
            ("minecraft:max_damage", "251"),
        ])));
        assert!(!matcher.test(&getter(&[("minecraft:custom_name", "Workbench")])));
    }

    #[test]
    fn exact_predicate_builder_rejects_duplicate_component_types() {
        let duplicate = DataComponentExactPredicateModel::builder()
            .expect(id("minecraft:custom_name"), "First")
            .unwrap()
            .expect(id("minecraft:custom_name"), "Second");

        assert_eq!(
            duplicate,
            Err("Predicate already has component of type: 'minecraft:custom_name'".to_string())
        );
    }

    #[test]
    fn partial_predicates_all_have_to_match_after_exact_predicate() {
        let matcher = DataComponentMatchersModel::builder()
            .exact(DataComponentExactPredicateModel::expect(
                id("minecraft:custom_name"),
                "Workbench",
            ))
            .any(id("minecraft:lore"))
            .partial(
                DataComponentPredicateTypeModel::concrete(id("minecraft:damage")),
                DataComponentPredicateModel::exact_value(id("minecraft:damage"), "4"),
            )
            .build()
            .unwrap();

        assert!(matcher.test(&getter(&[
            ("minecraft:custom_name", "Workbench"),
            ("minecraft:lore", "line one"),
            ("minecraft:damage", "4"),
        ])));
        assert!(!matcher.test(&getter(&[
            ("minecraft:custom_name", "Workbench"),
            ("minecraft:damage", "4"),
        ])));
        assert!(!matcher.test(&getter(&[
            ("minecraft:custom_name", "Workbench"),
            ("minecraft:lore", "line one"),
            ("minecraft:damage", "5"),
        ])));
    }

    #[test]
    fn builder_rejects_duplicate_partial_predicate_type_keys() {
        let duplicate = DataComponentMatchersModel::builder()
            .partial(
                DataComponentPredicateTypeModel::concrete(id("minecraft:damage")),
                DataComponentPredicateModel::exact_value(id("minecraft:damage"), "4"),
            )
            .partial(
                DataComponentPredicateTypeModel::concrete(id("minecraft:damage")),
                DataComponentPredicateModel::exact_value(id("minecraft:damage"), "5"),
            )
            .build();

        assert_eq!(
            duplicate,
            Err("Multiple entries with same key: 'minecraft:damage'".to_string())
        );
    }
}
