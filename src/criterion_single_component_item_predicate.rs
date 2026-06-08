use crate::criterion_data_component_matchers::DataComponentGetterModel;
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleComponentItemPredicateModel {
    component_type: Identifier,
    value_predicate: ComponentValuePredicateModel,
}

impl SingleComponentItemPredicateModel {
    pub fn new(component_type: Identifier, value_predicate: ComponentValuePredicateModel) -> Self {
        Self {
            component_type,
            value_predicate,
        }
    }

    pub fn component_type(&self) -> &Identifier {
        &self.component_type
    }

    pub fn matches(&self, components: &DataComponentGetterModel) -> bool {
        let Some(value) = components.get(&self.component_type) else {
            return false;
        };

        self.value_predicate.matches(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentValuePredicateModel {
    Any,
    Exact(String),
    Never,
    PanicIfEvaluated,
}

impl ComponentValuePredicateModel {
    pub fn exact(value: impl Into<String>) -> Self {
        Self::Exact(value.into())
    }

    fn matches(&self, value: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(expected) => value == expected,
            Self::Never => false,
            Self::PanicIfEvaluated => panic!("value predicate should not have been evaluated"),
        }
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn getter(pairs: &[(&str, &str)]) -> DataComponentGetterModel {
        DataComponentGetterModel::from_pairs(
            pairs
                .iter()
                .map(|(component, value)| (id(component), *value))
                .collect(),
        )
    }

    #[test]
    fn component_type_method_returns_declared_component_type() {
        let predicate = SingleComponentItemPredicateModel::new(
            id("minecraft:custom_name"),
            ComponentValuePredicateModel::Any,
        );

        assert_eq!(predicate.component_type(), &id("minecraft:custom_name"));
    }

    #[test]
    fn missing_component_fails_without_evaluating_value_predicate() {
        let predicate = SingleComponentItemPredicateModel::new(
            id("minecraft:damage"),
            ComponentValuePredicateModel::PanicIfEvaluated,
        );

        assert!(!predicate.matches(&getter(&[("minecraft:custom_name", "Sword")])));
        assert!(!predicate.matches(&DataComponentGetterModel::default()));
    }

    #[test]
    fn present_component_delegates_to_value_predicate() {
        let predicate = SingleComponentItemPredicateModel::new(
            id("minecraft:custom_name"),
            ComponentValuePredicateModel::exact("Sword"),
        );

        assert!(predicate.matches(&getter(&[("minecraft:custom_name", "Sword")])));
        assert!(!predicate.matches(&getter(&[("minecraft:custom_name", "Pickaxe")])));
    }

    #[test]
    fn present_component_can_still_fail_value_match() {
        let predicate = SingleComponentItemPredicateModel::new(
            id("minecraft:damage"),
            ComponentValuePredicateModel::Never,
        );

        assert!(!predicate.matches(&getter(&[("minecraft:damage", "4")])));
    }
}
