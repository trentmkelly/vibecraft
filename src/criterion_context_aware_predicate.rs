use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    conditions: Vec<LootConditionModel>,
}

impl ContextAwarePredicateModel {
    pub fn new(conditions: Vec<LootConditionModel>) -> Self {
        Self { conditions }
    }

    pub fn create(conditions: Vec<LootConditionModel>) -> Self {
        Self::new(conditions)
    }

    pub fn matches(&self, context: &LootContextModel) -> bool {
        self.conditions
            .iter()
            .all(|condition| condition.matches(context))
    }

    pub fn validate(&self, context: &ValidationContextModel) -> Vec<String> {
        self.conditions
            .iter()
            .enumerate()
            .flat_map(|(index, condition)| {
                condition
                    .validate(context)
                    .into_iter()
                    .map(move |problem| format!("[{index}]: {problem}"))
            })
            .collect()
    }

    pub fn conditions(&self) -> &[LootConditionModel] {
        &self.conditions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextModel {
    pub facts: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationContextModel {
    pub allowed_params: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootConditionModel {
    Always,
    Never,
    FactEquals { key: String, value: String },
    RequiresParam(String),
    Invalid(String),
    PanicIfEvaluated,
}

impl LootConditionModel {
    pub fn fact_equals(key: &str, value: &str) -> Self {
        Self::FactEquals {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    pub fn requires_param(param: &str) -> Self {
        Self::RequiresParam(param.to_string())
    }

    pub fn invalid(problem: &str) -> Self {
        Self::Invalid(problem.to_string())
    }

    fn matches(&self, context: &LootContextModel) -> bool {
        match self {
            Self::Always | Self::RequiresParam(_) | Self::Invalid(_) => true,
            Self::Never => false,
            Self::FactEquals { key, value } => context.facts.get(key) == Some(value),
            Self::PanicIfEvaluated => panic!("condition should not have been evaluated"),
        }
    }

    fn validate(&self, context: &ValidationContextModel) -> Vec<String> {
        match self {
            Self::RequiresParam(param) if !context.allowed_params.contains(param) => {
                vec![format!("parameter not provided: {param}")]
            }
            Self::Invalid(problem) => vec![problem.clone()],
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(facts: &[(&str, &str)]) -> LootContextModel {
        LootContextModel {
            facts: facts
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        }
    }

    fn validation_context(allowed_params: &[&str]) -> ValidationContextModel {
        ValidationContextModel {
            allowed_params: allowed_params
                .iter()
                .map(|param| param.to_string())
                .collect(),
        }
    }

    #[test]
    fn context_aware_predicate_empty_conditions_match_true_like_util_all_of() {
        let predicate = ContextAwarePredicateModel::new(Vec::new());

        assert!(predicate.matches(&context(&[])));
        assert!(predicate.validate(&validation_context(&[])).is_empty());
        assert!(predicate.conditions().is_empty());
    }

    #[test]
    fn context_aware_predicate_matches_all_conditions_like_java() {
        let predicate = ContextAwarePredicateModel::create(vec![
            LootConditionModel::Always,
            LootConditionModel::fact_equals("entity", "minecraft:pig"),
            LootConditionModel::requires_param("this_entity"),
        ]);

        assert!(predicate.matches(&context(&[("entity", "minecraft:pig")])));
        assert!(!predicate.matches(&context(&[("entity", "minecraft:cow")])));
        assert_eq!(predicate.conditions().len(), 3);
    }

    #[test]
    fn context_aware_predicate_short_circuits_failed_conditions_like_util_all_of() {
        let predicate = ContextAwarePredicateModel::new(vec![
            LootConditionModel::Never,
            LootConditionModel::PanicIfEvaluated,
        ]);

        assert!(!predicate.matches(&context(&[])));
    }

    #[test]
    fn context_aware_predicate_validate_indexes_condition_list_like_java() {
        let predicate = ContextAwarePredicateModel::new(vec![
            LootConditionModel::requires_param("this_entity"),
            LootConditionModel::invalid("bad condition"),
            LootConditionModel::requires_param("origin"),
        ]);

        assert_eq!(
            predicate.validate(&validation_context(&["this_entity"])),
            vec![
                "[1]: bad condition".to_string(),
                "[2]: parameter not provided: origin".to_string(),
            ]
        );
    }
}
