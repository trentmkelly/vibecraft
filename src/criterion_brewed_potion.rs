use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewedPotionTriggerInstance {
    pub player_predicate_present: bool,
    pub potion: Option<Identifier>,
}

impl BrewedPotionTriggerInstance {
    pub fn new(potion: Option<Identifier>) -> Self {
        Self {
            player_predicate_present: false,
            potion,
        }
    }

    pub fn matches(&self, potion: &Identifier) -> bool {
        self.potion
            .as_ref()
            .is_none_or(|expected| expected == potion)
    }

    pub fn brewed_potion() -> BrewedPotionCriterion {
        BrewedPotionCriterion {
            trigger_id: Identifier::parse("minecraft:brewed_potion").unwrap(),
            instance: Self::new(None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewedPotionCriterion {
    pub trigger_id: Identifier,
    pub instance: BrewedPotionTriggerInstance,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    #[test]
    fn brewed_potion_omitted_potion_matches_any_holder_like_java() {
        let instance = BrewedPotionTriggerInstance::new(None);

        assert!(instance.matches(&id("minecraft:water")));
        assert!(instance.matches(&id("minecraft:long_night_vision")));
    }

    #[test]
    fn brewed_potion_configured_potion_requires_holder_equality_like_java() {
        let instance = BrewedPotionTriggerInstance::new(Some(id("minecraft:awkward")));

        assert!(instance.matches(&id("minecraft:awkward")));
        assert!(!instance.matches(&id("minecraft:mundane")));
        assert!(!instance.matches(&id("minecraft:water")));
    }

    #[test]
    fn brewed_potion_factory_uses_java_trigger_id_and_empty_potion() {
        let criterion = BrewedPotionTriggerInstance::brewed_potion();

        assert_eq!(criterion.trigger_id, id("minecraft:brewed_potion"));
        assert_eq!(criterion.instance, BrewedPotionTriggerInstance::new(None));
        assert!(criterion.instance.matches(&id("minecraft:strength")));
    }
}
