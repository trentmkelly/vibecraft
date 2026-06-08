use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeDimensionTriggerInstance {
    pub player_predicate_present: bool,
    pub from: Option<Identifier>,
    pub to: Option<Identifier>,
}

impl ChangeDimensionTriggerInstance {
    pub fn new(from: Option<Identifier>, to: Option<Identifier>) -> Self {
        Self {
            player_predicate_present: false,
            from,
            to,
        }
    }

    pub fn matches(&self, from: &Identifier, to: &Identifier) -> bool {
        if self
            .from
            .as_ref()
            .is_some_and(|expected_from| expected_from != from)
        {
            return false;
        }

        self.to.as_ref().is_none_or(|expected_to| expected_to == to)
    }

    pub fn changed_dimension() -> ChangeDimensionCriterion {
        ChangeDimensionCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(None, None),
        }
    }

    pub fn changed_dimension_between(from: Identifier, to: Identifier) -> ChangeDimensionCriterion {
        ChangeDimensionCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(Some(from), Some(to)),
        }
    }

    pub fn changed_dimension_to(to: Identifier) -> ChangeDimensionCriterion {
        ChangeDimensionCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(None, Some(to)),
        }
    }

    pub fn changed_dimension_from(from: Identifier) -> ChangeDimensionCriterion {
        ChangeDimensionCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(Some(from), None),
        }
    }
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:changed_dimension").unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeDimensionCriterion {
    pub trigger_id: Identifier,
    pub instance: ChangeDimensionTriggerInstance,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    #[test]
    fn changed_dimension_omitted_from_and_to_match_any_transition_like_java() {
        let instance = ChangeDimensionTriggerInstance::new(None, None);

        assert!(instance.matches(&id("minecraft:overworld"), &id("minecraft:the_nether")));
        assert!(instance.matches(&id("minecraft:the_end"), &id("minecraft:overworld")));
    }

    #[test]
    fn changed_dimension_checks_from_before_to_like_java() {
        let instance = ChangeDimensionTriggerInstance::new(
            Some(id("minecraft:overworld")),
            Some(id("minecraft:the_nether")),
        );

        assert!(instance.matches(&id("minecraft:overworld"), &id("minecraft:the_nether")));
        assert!(!instance.matches(&id("minecraft:the_end"), &id("minecraft:the_nether")));
        assert!(!instance.matches(&id("minecraft:overworld"), &id("minecraft:the_end")));
    }

    #[test]
    fn changed_dimension_single_sided_predicates_match_java() {
        let from_only = ChangeDimensionTriggerInstance::new(Some(id("minecraft:the_nether")), None);
        assert!(from_only.matches(&id("minecraft:the_nether"), &id("minecraft:overworld")));
        assert!(!from_only.matches(&id("minecraft:overworld"), &id("minecraft:the_nether")));

        let to_only = ChangeDimensionTriggerInstance::new(None, Some(id("minecraft:the_end")));
        assert!(to_only.matches(&id("minecraft:overworld"), &id("minecraft:the_end")));
        assert!(!to_only.matches(&id("minecraft:the_nether"), &id("minecraft:overworld")));
    }

    #[test]
    fn changed_dimension_factories_use_java_trigger_id_and_fields() {
        let any = ChangeDimensionTriggerInstance::changed_dimension();
        assert_eq!(any.trigger_id, id("minecraft:changed_dimension"));
        assert_eq!(
            any.instance,
            ChangeDimensionTriggerInstance::new(None, None)
        );

        let between = ChangeDimensionTriggerInstance::changed_dimension_between(
            id("minecraft:overworld"),
            id("minecraft:the_nether"),
        );
        assert_eq!(between.trigger_id, id("minecraft:changed_dimension"));
        assert_eq!(between.instance.from, Some(id("minecraft:overworld")));
        assert_eq!(between.instance.to, Some(id("minecraft:the_nether")));

        let to = ChangeDimensionTriggerInstance::changed_dimension_to(id("minecraft:the_end"));
        assert_eq!(to.instance.from, None);
        assert_eq!(to.instance.to, Some(id("minecraft:the_end")));

        let from =
            ChangeDimensionTriggerInstance::changed_dimension_from(id("minecraft:the_nether"));
        assert_eq!(from.instance.from, Some(id("minecraft:the_nether")));
        assert_eq!(from.instance.to, None);
    }
}
