use std::collections::BTreeSet;

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementRequirementsModel {
    requirements: Vec<Vec<String>>,
}

impl AdvancementRequirementsModel {
    pub fn new(requirements: Vec<Vec<String>>) -> Self {
        Self { requirements }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn all_of(criteria: impl IntoIterator<Item = String>) -> Self {
        Self::new(
            criteria
                .into_iter()
                .map(|criterion| vec![criterion])
                .collect(),
        )
    }

    pub fn any_of(criteria: impl IntoIterator<Item = String>) -> Self {
        Self::new(vec![criteria.into_iter().collect()])
    }

    pub fn requirements(&self) -> &[Vec<String>] {
        &self.requirements
    }

    pub fn size(&self) -> usize {
        self.requirements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.requirements.is_empty()
    }

    pub fn test(&self, mut predicate: impl FnMut(&str) -> bool) -> bool {
        !self.requirements.is_empty()
            && self
                .requirements
                .iter()
                .all(|group| group.iter().any(|criterion| predicate(criterion)))
    }

    pub fn count(&self, mut predicate: impl FnMut(&str) -> bool) -> usize {
        self.requirements
            .iter()
            .filter(|group| group.iter().any(|criterion| predicate(criterion)))
            .count()
    }

    pub fn validate(&self, expected_criteria: &BTreeSet<String>) -> Result<(), String> {
        let mut referenced_criteria = BTreeSet::new();
        for group in &self.requirements {
            if group.is_empty() && expected_criteria.is_empty() {
                return Err("Requirement entry cannot be empty".to_string());
            }
            referenced_criteria.extend(group.iter().cloned());
        }

        if referenced_criteria == *expected_criteria {
            return Ok(());
        }

        let missing = expected_criteria
            .difference(&referenced_criteria)
            .cloned()
            .collect::<Vec<_>>();
        let unknown = referenced_criteria
            .difference(expected_criteria)
            .cloned()
            .collect::<Vec<_>>();
        Err(format!(
            "Advancement completion requirements did not exactly match specified criteria. Missing: {}. Unknown: {}",
            java_set_display(&missing),
            java_set_display(&unknown)
        ))
    }

    pub fn names(&self) -> BTreeSet<String> {
        self.requirements
            .iter()
            .flat_map(|group| group.iter().cloned())
            .collect()
    }

    pub fn java_to_string(&self) -> String {
        let groups = self
            .requirements
            .iter()
            .map(|group| java_set_display(group))
            .collect::<Vec<_>>();
        java_set_display(&groups)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementRequirementStrategy {
    And,
    Or,
}

impl AdvancementRequirementStrategy {
    pub fn create(
        self,
        criteria: impl IntoIterator<Item = String>,
    ) -> AdvancementRequirementsModel {
        match self {
            Self::And => AdvancementRequirementsModel::all_of(criteria),
            Self::Or => AdvancementRequirementsModel::any_of(criteria),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionModel<T> {
    pub trigger: CriterionTriggerModel,
    pub trigger_instance: T,
}

impl<T> CriterionModel<T> {
    pub fn new(trigger: CriterionTriggerModel, trigger_instance: T) -> Self {
        Self {
            trigger,
            trigger_instance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionTriggerModel {
    id: Identifier,
}

impl CriterionTriggerModel {
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn create_criterion<T>(&self, instance: T) -> CriterionModel<T> {
        CriterionModel::new(self.clone(), instance)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionListenerModel<T> {
    pub trigger: T,
    pub advancement: Identifier,
    pub criterion: String,
}

impl<T> CriterionListenerModel<T> {
    pub fn run<R>(&self, mut award: impl FnMut(&Identifier, &str) -> R) -> R {
        award(&self.advancement, &self.criterion)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CriterionValidationContext {
    problems: Vec<String>,
}

impl CriterionValidationContext {
    pub fn report(&mut self, problem: impl Into<String>) {
        self.problems.push(problem.into());
    }

    pub fn problems(&self) -> &[String] {
        &self.problems
    }
}

pub trait CriterionTriggerInstanceModel {
    fn validate(&self, validator: &mut CriterionValidationContext);
}

fn java_set_display(values: &[String]) -> String {
    format!("[{}]", values.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn advancement_requirements_match_java_and_or_test_count_and_names() {
        let all = AdvancementRequirementStrategy::And
            .create(["root", "mine_stone"].into_iter().map(str::to_string));
        assert_eq!(
            all.requirements(),
            &[vec!["root".to_string()], vec!["mine_stone".to_string()]]
        );
        assert_eq!(all.size(), 2);
        assert!(!all.is_empty());
        assert!(!AdvancementRequirementsModel::empty().test(|_| true));
        assert!(!all.test(|criterion| criterion == "root"));
        assert!(all.test(|criterion| criterion == "root" || criterion == "mine_stone"));
        assert_eq!(all.count(|criterion| criterion == "root"), 1);
        assert_eq!(all.names(), set(&["root", "mine_stone"]));
        assert_eq!(all.java_to_string(), "[[root], [mine_stone]]");

        let any = AdvancementRequirementStrategy::Or
            .create(["root", "mine_stone"].into_iter().map(str::to_string));
        assert_eq!(
            any.requirements(),
            &[vec!["root".to_string(), "mine_stone".to_string()]]
        );
        assert!(any.test(|criterion| criterion == "mine_stone"));
        assert_eq!(any.count(|criterion| criterion == "mine_stone"), 1);
    }

    #[test]
    fn advancement_requirements_validation_matches_java_exact_criteria_contract() {
        let requirements = AdvancementRequirementsModel::new(vec![
            vec!["root".to_string()],
            vec!["mine_stone".to_string(), "smelt_iron".to_string()],
        ]);
        assert!(requirements
            .validate(&set(&["root", "mine_stone", "smelt_iron"]))
            .is_ok());
        assert_eq!(
            requirements.validate(&set(&["root"])).unwrap_err(),
            "Advancement completion requirements did not exactly match specified criteria. Missing: []. Unknown: [mine_stone, smelt_iron]"
        );
        assert_eq!(
            AdvancementRequirementsModel::new(vec![Vec::new()])
                .validate(&BTreeSet::new())
                .unwrap_err(),
            "Requirement entry cannot be empty"
        );
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RecordingInstance {
        problem: Option<String>,
    }

    impl CriterionTriggerInstanceModel for RecordingInstance {
        fn validate(&self, validator: &mut CriterionValidationContext) {
            if let Some(problem) = &self.problem {
                validator.report(problem);
            }
        }
    }

    #[test]
    fn criterion_trigger_models_create_criterion_listener_run_and_validate_instance() {
        let trigger = CriterionTriggerModel::new(Identifier::parse("minecraft:tick").unwrap());
        let instance = RecordingInstance {
            problem: Some("bad condition".to_string()),
        };
        let criterion = trigger.create_criterion(instance.clone());
        assert_eq!(
            criterion.trigger.id(),
            &Identifier::parse("minecraft:tick").unwrap()
        );
        assert_eq!(criterion.trigger_instance, instance);

        let listener = CriterionListenerModel {
            trigger: criterion.trigger_instance,
            advancement: Identifier::parse("minecraft:story/root").unwrap(),
            criterion: "tick".to_string(),
        };
        let awarded = listener.run(|advancement, criterion| format!("{advancement}#{criterion}"));
        assert_eq!(awarded, "minecraft:story/root#tick");

        let mut validator = CriterionValidationContext::default();
        listener.trigger.validate(&mut validator);
        assert_eq!(validator.problems(), &["bad condition".to_string()]);
    }
}
