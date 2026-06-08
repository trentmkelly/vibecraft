use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BredAnimalLootContext {
    pub entity_type: Identifier,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BredAnimalsTriggerInstance {
    pub player_predicate_present: bool,
    pub parent: Option<ContextAwareEntityPredicateModel>,
    pub partner: Option<ContextAwareEntityPredicateModel>,
    pub child: Option<ContextAwareEntityPredicateModel>,
}

impl BredAnimalsTriggerInstance {
    pub fn new(
        parent: Option<ContextAwareEntityPredicateModel>,
        partner: Option<ContextAwareEntityPredicateModel>,
        child: Option<ContextAwareEntityPredicateModel>,
    ) -> Self {
        Self {
            player_predicate_present: false,
            parent,
            partner,
            child,
        }
    }

    pub fn matches(
        &self,
        parent: &BredAnimalLootContext,
        partner: &BredAnimalLootContext,
        child: Option<&BredAnimalLootContext>,
    ) -> bool {
        if self
            .child
            .as_ref()
            .is_some_and(|predicate| !child.is_some_and(|child| predicate.matches(child)))
        {
            return false;
        }

        optional_predicate_matches(&self.parent, parent)
            && optional_predicate_matches(&self.partner, partner)
            || optional_predicate_matches(&self.parent, partner)
                && optional_predicate_matches(&self.partner, parent)
    }

    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        collect_validation_problem(&mut problems, "parent", &self.parent);
        collect_validation_problem(&mut problems, "partner", &self.partner);
        collect_validation_problem(&mut problems, "child", &self.child);
        problems
    }

    pub fn bred_animals() -> BredAnimalsCriterion {
        BredAnimalsCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(None, None, None),
        }
    }

    pub fn bred_animals_child(child: ContextAwareEntityPredicateModel) -> BredAnimalsCriterion {
        BredAnimalsCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(None, None, Some(child)),
        }
    }

    pub fn bred_animals_with_predicates(
        parent1: Option<ContextAwareEntityPredicateModel>,
        parent2: Option<ContextAwareEntityPredicateModel>,
        child: Option<ContextAwareEntityPredicateModel>,
    ) -> BredAnimalsCriterion {
        BredAnimalsCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(parent1, parent2, child),
        }
    }
}

fn optional_predicate_matches(
    predicate: &Option<ContextAwareEntityPredicateModel>,
    context: &BredAnimalLootContext,
) -> bool {
    predicate
        .as_ref()
        .is_none_or(|predicate| predicate.matches(context))
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

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:bred_animals").unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BredAnimalsCriterion {
    pub trigger_id: Identifier,
    pub instance: BredAnimalsTriggerInstance,
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

    fn matches(&self, context: &BredAnimalLootContext) -> bool {
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

    fn entity(entity_type: &str, name: Option<&str>) -> BredAnimalLootContext {
        BredAnimalLootContext {
            entity_type: id(entity_type),
            name: name.map(str::to_string),
        }
    }

    #[test]
    fn bred_animals_matches_parent_and_partner_in_either_order_like_java() {
        let instance = BredAnimalsTriggerInstance::new(
            Some(ContextAwareEntityPredicateModel::entity_type(id(
                "minecraft:cow",
            ))),
            Some(ContextAwareEntityPredicateModel::entity_type(id(
                "minecraft:sheep",
            ))),
            None,
        );
        let cow = entity("minecraft:cow", None);
        let sheep = entity("minecraft:sheep", None);
        let pig = entity("minecraft:pig", None);

        assert!(instance.matches(&cow, &sheep, None));
        assert!(instance.matches(&sheep, &cow, None));
        assert!(!instance.matches(&cow, &pig, None));
        assert!(!instance.matches(&pig, &sheep, None));
    }

    #[test]
    fn bred_animals_checks_child_predicate_before_parent_pair_like_java() {
        let instance = BredAnimalsTriggerInstance::new(
            Some(ContextAwareEntityPredicateModel::entity_type(id(
                "minecraft:cow",
            ))),
            Some(ContextAwareEntityPredicateModel::entity_type(id(
                "minecraft:cow",
            ))),
            Some(ContextAwareEntityPredicateModel::entity_type(id(
                "minecraft:mooshroom",
            ))),
        );
        let cow = entity("minecraft:cow", None);
        let mooshroom = entity("minecraft:mooshroom", None);
        let calf = entity("minecraft:cow", None);

        assert!(instance.matches(&cow, &cow, Some(&mooshroom)));
        assert!(!instance.matches(&cow, &cow, None));
        assert!(!instance.matches(&cow, &cow, Some(&calf)));
    }

    #[test]
    fn bred_animals_omitted_predicates_match_any_parent_partner_and_absent_child() {
        let instance = BredAnimalsTriggerInstance::new(None, None, None);

        assert!(instance.matches(
            &entity("minecraft:panda", Some("one")),
            &entity("minecraft:turtle", Some("two")),
            None,
        ));
        assert!(instance.matches(
            &entity("minecraft:panda", None),
            &entity("minecraft:turtle", None),
            Some(&entity("minecraft:panda", None)),
        ));
    }

    #[test]
    fn bred_animals_factory_methods_use_java_trigger_id_and_fields() {
        let no_predicates = BredAnimalsTriggerInstance::bred_animals();
        assert_eq!(no_predicates.trigger_id, id("minecraft:bred_animals"));
        assert_eq!(
            no_predicates.instance,
            BredAnimalsTriggerInstance::new(None, None, None)
        );

        let child = ContextAwareEntityPredicateModel::named("Junior");
        let child_factory = BredAnimalsTriggerInstance::bred_animals_child(child.clone());
        assert_eq!(child_factory.trigger_id, id("minecraft:bred_animals"));
        assert_eq!(child_factory.instance.child, Some(child));

        let predicate_factory = BredAnimalsTriggerInstance::bred_animals_with_predicates(
            Some(ContextAwareEntityPredicateModel::entity_type(id(
                "minecraft:cow",
            ))),
            None,
            None,
        );
        assert_eq!(predicate_factory.trigger_id, id("minecraft:bred_animals"));
        assert!(predicate_factory.instance.partner.is_none());
    }

    #[test]
    fn bred_animals_validate_reports_parent_partner_and_child_predicates() {
        let instance = BredAnimalsTriggerInstance::new(
            Some(ContextAwareEntityPredicateModel::invalid("bad parent")),
            Some(ContextAwareEntityPredicateModel::invalid("bad partner")),
            Some(ContextAwareEntityPredicateModel::invalid("bad child")),
        );

        assert_eq!(
            instance.validate(),
            vec![
                "parent: bad parent".to_string(),
                "partner: bad partner".to_string(),
                "child: bad child".to_string(),
            ]
        );
    }
}
