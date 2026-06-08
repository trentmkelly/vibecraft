use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CuredEntityLootContext {
    pub entity_type: Identifier,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CuredZombieVillagerTriggerInstance {
    pub player_predicate_present: bool,
    pub zombie: Option<ContextAwareEntityPredicateModel>,
    pub villager: Option<ContextAwareEntityPredicateModel>,
}

impl CuredZombieVillagerTriggerInstance {
    pub fn new(
        zombie: Option<ContextAwareEntityPredicateModel>,
        villager: Option<ContextAwareEntityPredicateModel>,
    ) -> Self {
        Self {
            player_predicate_present: false,
            zombie,
            villager,
        }
    }

    pub fn matches(
        &self,
        zombie: &CuredEntityLootContext,
        villager: &CuredEntityLootContext,
    ) -> bool {
        if self
            .zombie
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(zombie))
        {
            return false;
        }

        self.villager
            .as_ref()
            .is_none_or(|predicate| predicate.matches(villager))
    }

    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        collect_validation_problem(&mut problems, "zombie", &self.zombie);
        collect_validation_problem(&mut problems, "villager", &self.villager);
        problems
    }

    pub fn cured_zombie_villager() -> CuredZombieVillagerCriterion {
        CuredZombieVillagerCriterion {
            trigger_id: Identifier::parse("minecraft:cured_zombie_villager").unwrap(),
            instance: Self::new(None, None),
        }
    }
}

fn collect_validation_problem(
    problems: &mut Vec<String>,
    field: &str,
    predicate: &Option<ContextAwareEntityPredicateModel>,
) {
    if let Some(problem) = predicate
        .as_ref()
        .and_then(ContextAwareEntityPredicateModel::validation_problem)
    {
        problems.push(format!("{field}: {problem}"));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CuredZombieVillagerCriterion {
    pub trigger_id: Identifier,
    pub instance: CuredZombieVillagerTriggerInstance,
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

    fn matches(&self, context: &CuredEntityLootContext) -> bool {
        self.required_type
            .as_ref()
            .is_none_or(|entity_type| entity_type == &context.entity_type)
            && self.required_name.as_ref().is_none_or(|name| {
                context
                    .name
                    .as_ref()
                    .is_some_and(|context_name| context_name == name)
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

    fn entity(entity_type: &str, name: Option<&str>) -> CuredEntityLootContext {
        CuredEntityLootContext {
            entity_type: id(entity_type),
            name: name.map(str::to_string),
        }
    }

    #[test]
    fn cured_zombie_villager_omitted_predicates_match_any_contexts_like_java() {
        let instance = CuredZombieVillagerTriggerInstance::new(None, None);

        assert!(instance.matches(
            &entity("minecraft:zombie_villager", Some("Patient")),
            &entity("minecraft:villager", Some("Cured")),
        ));
    }

    #[test]
    fn cured_zombie_villager_checks_zombie_then_villager_like_java() {
        let instance = CuredZombieVillagerTriggerInstance::new(
            Some(ContextAwareEntityPredicateModel::entity_type(id(
                "minecraft:zombie_villager",
            ))),
            Some(ContextAwareEntityPredicateModel::named("Cured")),
        );

        assert!(instance.matches(
            &entity("minecraft:zombie_villager", None),
            &entity("minecraft:villager", Some("Cured")),
        ));
        assert!(!instance.matches(
            &entity("minecraft:zombie", None),
            &entity("minecraft:villager", Some("Cured")),
        ));
        assert!(!instance.matches(
            &entity("minecraft:zombie_villager", None),
            &entity("minecraft:villager", Some("Other")),
        ));
    }

    #[test]
    fn cured_zombie_villager_factory_uses_java_trigger_id_and_empty_predicates() {
        let criterion = CuredZombieVillagerTriggerInstance::cured_zombie_villager();

        assert_eq!(criterion.trigger_id, id("minecraft:cured_zombie_villager"));
        assert_eq!(
            criterion.instance,
            CuredZombieVillagerTriggerInstance::new(None, None)
        );
    }

    #[test]
    fn cured_zombie_villager_validate_reports_zombie_and_villager_predicates() {
        let instance = CuredZombieVillagerTriggerInstance::new(
            Some(ContextAwareEntityPredicateModel::invalid("bad zombie")),
            Some(ContextAwareEntityPredicateModel::invalid("bad villager")),
        );

        assert_eq!(
            instance.validate(),
            vec![
                "zombie: bad zombie".to_string(),
                "villager: bad villager".to_string(),
            ]
        );
    }
}
