use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningVictimLootContext {
    pub entity_type: Identifier,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChanneledLightningTriggerInstance {
    pub player_predicate_present: bool,
    pub victims: Vec<ContextAwareEntityPredicateModel>,
}

impl ChanneledLightningTriggerInstance {
    pub fn new(victims: Vec<ContextAwareEntityPredicateModel>) -> Self {
        Self {
            player_predicate_present: false,
            victims,
        }
    }

    pub fn matches(&self, victims: &[LightningVictimLootContext]) -> bool {
        self.victims
            .iter()
            .all(|predicate| victims.iter().any(|victim| predicate.matches(victim)))
    }

    pub fn validate(&self) -> Vec<String> {
        self.victims
            .iter()
            .enumerate()
            .filter_map(|(index, predicate)| {
                predicate
                    .validation_problem()
                    .map(|problem| format!("victims[{index}]: {problem}"))
            })
            .collect()
    }

    pub fn channeled_lightning(
        victims: Vec<ContextAwareEntityPredicateModel>,
    ) -> ChanneledLightningCriterion {
        ChanneledLightningCriterion {
            trigger_id: Identifier::parse("minecraft:channeled_lightning").unwrap(),
            instance: Self::new(victims),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChanneledLightningCriterion {
    pub trigger_id: Identifier,
    pub instance: ChanneledLightningTriggerInstance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwareEntityPredicateModel {
    required_type: Option<Identifier>,
    required_name: Option<String>,
    validation_problem: Option<String>,
}

impl ContextAwareEntityPredicateModel {
    pub fn any() -> Self {
        Self {
            required_type: None,
            required_name: None,
            validation_problem: None,
        }
    }

    pub fn entity_type(entity_type: Identifier) -> Self {
        Self {
            required_type: Some(entity_type),
            ..Self::any()
        }
    }

    pub fn named(name: &str) -> Self {
        Self {
            required_name: Some(name.to_string()),
            ..Self::any()
        }
    }

    pub fn invalid(problem: &str) -> Self {
        Self {
            validation_problem: Some(problem.to_string()),
            ..Self::any()
        }
    }

    fn matches(&self, victim: &LightningVictimLootContext) -> bool {
        self.required_type
            .as_ref()
            .is_none_or(|entity_type| entity_type == &victim.entity_type)
            && self.required_name.as_ref().is_none_or(|name| {
                victim
                    .name
                    .as_ref()
                    .is_some_and(|victim_name| victim_name == name)
            })
    }

    fn validation_problem(&self) -> Option<&str> {
        self.validation_problem.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn victim(entity_type: &str, name: Option<&str>) -> LightningVictimLootContext {
        LightningVictimLootContext {
            entity_type: id(entity_type),
            name: name.map(str::to_string),
        }
    }

    #[test]
    fn channeled_lightning_empty_victim_predicates_match_any_collection_like_java() {
        let instance = ChanneledLightningTriggerInstance::new(Vec::new());

        assert!(instance.matches(&[]));
        assert!(instance.matches(&[victim("minecraft:pig", None)]));
    }

    #[test]
    fn channeled_lightning_requires_each_predicate_to_match_some_victim_like_java() {
        let instance = ChanneledLightningTriggerInstance::new(vec![
            ContextAwareEntityPredicateModel::entity_type(id("minecraft:pig")),
            ContextAwareEntityPredicateModel::entity_type(id("minecraft:creeper")),
        ]);
        let victims = vec![
            victim("minecraft:pig", None),
            victim("minecraft:creeper", None),
            victim("minecraft:villager", None),
        ];

        assert!(instance.matches(&victims));
        assert!(!instance.matches(&[victim("minecraft:pig", None)]));
        assert!(!instance.matches(&[]));
    }

    #[test]
    fn channeled_lightning_allows_one_victim_to_satisfy_multiple_predicates_like_java() {
        let instance = ChanneledLightningTriggerInstance::new(vec![
            ContextAwareEntityPredicateModel::entity_type(id("minecraft:pig")),
            ContextAwareEntityPredicateModel::named("Charged"),
        ]);

        assert!(instance.matches(&[victim("minecraft:pig", Some("Charged"))]));
        assert!(!instance.matches(&[victim("minecraft:pig", Some("Other"))]));
    }

    #[test]
    fn channeled_lightning_factory_uses_java_trigger_id_and_ordered_victims() {
        let victims = vec![
            ContextAwareEntityPredicateModel::entity_type(id("minecraft:pig")),
            ContextAwareEntityPredicateModel::entity_type(id("minecraft:villager")),
        ];
        let criterion = ChanneledLightningTriggerInstance::channeled_lightning(victims.clone());

        assert_eq!(criterion.trigger_id, id("minecraft:channeled_lightning"));
        assert_eq!(
            criterion.instance,
            ChanneledLightningTriggerInstance::new(victims)
        );
    }

    #[test]
    fn channeled_lightning_validate_reports_victim_list_predicates() {
        let instance = ChanneledLightningTriggerInstance::new(vec![
            ContextAwareEntityPredicateModel::any(),
            ContextAwareEntityPredicateModel::invalid("bad victim"),
        ]);

        assert_eq!(
            instance.validate(),
            vec!["victims[1]: bad victim".to_string()]
        );
    }
}
