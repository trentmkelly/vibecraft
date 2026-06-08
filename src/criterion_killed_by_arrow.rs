use std::collections::BTreeSet;

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KilledByArrowTriggerInstanceModel {
    pub player_predicate_present: bool,
    pub victims: Vec<ContextAwareEntityPredicateModel>,
    pub unique_entity_types: IntBoundsModel,
    pub fired_from_weapon: Option<ItemPredicateModel>,
}

impl KilledByArrowTriggerInstanceModel {
    pub fn new(
        victims: Vec<ContextAwareEntityPredicateModel>,
        unique_entity_types: IntBoundsModel,
        fired_from_weapon: Option<ItemPredicateModel>,
    ) -> Self {
        Self {
            player_predicate_present: false,
            victims,
            unique_entity_types,
            fired_from_weapon,
        }
    }

    pub fn crossbow_killed_victims(
        victims: Vec<EntityPredicateModel>,
    ) -> KilledByArrowCriterionModel {
        KilledByArrowCriterionModel {
            trigger_id: trigger_id(),
            instance: Self::new(
                victims
                    .into_iter()
                    .map(ContextAwareEntityPredicateModel::wrap)
                    .collect(),
                IntBoundsModel::ANY,
                Some(ItemPredicateModel::item(id("minecraft:crossbow"))),
            ),
        }
    }

    pub fn crossbow_killed_unique(
        unique_entity_types: IntBoundsModel,
    ) -> KilledByArrowCriterionModel {
        KilledByArrowCriterionModel {
            trigger_id: trigger_id(),
            instance: Self::new(
                Vec::new(),
                unique_entity_types,
                Some(ItemPredicateModel::item(id("minecraft:crossbow"))),
            ),
        }
    }

    pub fn matches(
        &self,
        victims: &[EntityLootContextModel],
        unique_entity_types: i32,
        fired_from_weapon: Option<&ItemStackModel>,
    ) -> bool {
        if self
            .fired_from_weapon
            .as_ref()
            .is_some_and(|predicate| fired_from_weapon.is_none_or(|weapon| !predicate.test(weapon)))
        {
            return false;
        }

        if !self.victims.is_empty() {
            let mut victims_copy = victims.to_vec();
            for predicate in &self.victims {
                let Some(index) = victims_copy
                    .iter()
                    .position(|victim| predicate.matches(victim))
                else {
                    return false;
                };
                victims_copy.remove(index);
            }
        }

        self.unique_entity_types.matches(unique_entity_types)
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KilledByArrowCriterionModel {
    pub trigger_id: Identifier,
    pub instance: KilledByArrowTriggerInstanceModel,
}

fn trigger_id() -> Identifier {
    id("minecraft:killed_by_arrow")
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityLootContextModel {
    entity_type: Identifier,
    name: Option<String>,
}

impl EntityLootContextModel {
    pub fn new(entity_type: Identifier, name: Option<String>) -> Self {
        Self { entity_type, name }
    }
}

pub fn unique_entity_type_count(victims: &[EntityLootContextModel]) -> i32 {
    victims
        .iter()
        .map(|victim| &victim.entity_type)
        .collect::<BTreeSet<_>>()
        .len() as i32
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateModel {
    required_type: Option<Identifier>,
    required_name: Option<String>,
    validation_problem: Option<String>,
}

impl EntityPredicateModel {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwareEntityPredicateModel {
    required_type: Option<Identifier>,
    required_name: Option<String>,
    validation_problem: Option<String>,
}

impl ContextAwareEntityPredicateModel {
    pub fn wrap(predicate: EntityPredicateModel) -> Self {
        Self {
            required_type: predicate.required_type,
            required_name: predicate.required_name,
            validation_problem: predicate.validation_problem,
        }
    }

    pub fn entity_type(entity_type: Identifier) -> Self {
        Self::wrap(EntityPredicateModel::entity_type(entity_type))
    }

    pub fn named(name: &str) -> Self {
        Self::wrap(EntityPredicateModel::named(name))
    }

    pub fn invalid(problem: &str) -> Self {
        Self::wrap(EntityPredicateModel::invalid(problem))
    }

    fn matches(&self, victim: &EntityLootContextModel) -> bool {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateModel {
    item: Identifier,
}

impl ItemPredicateModel {
    pub fn item(item: Identifier) -> Self {
        Self { item }
    }

    pub fn test(&self, stack: &ItemStackModel) -> bool {
        self.item == stack.item
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackModel {
    item: Identifier,
}

impl ItemStackModel {
    pub fn new(item: Identifier) -> Self {
        Self { item }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntBoundsModel {
    min: Option<i32>,
    max: Option<i32>,
}

impl IntBoundsModel {
    pub const ANY: Self = Self {
        min: None,
        max: None,
    };

    pub fn exactly(value: i32) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    pub fn at_least(value: i32) -> Self {
        Self {
            min: Some(value),
            max: None,
        }
    }

    pub fn matches(&self, value: i32) -> bool {
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| max >= value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn victim(entity_type: &str, name: Option<&str>) -> EntityLootContextModel {
        EntityLootContextModel::new(id(entity_type), name.map(str::to_string))
    }

    fn weapon(item: &str) -> ItemStackModel {
        ItemStackModel::new(id(item))
    }

    #[test]
    fn optional_weapon_predicate_rejects_null_or_wrong_weapon_before_victims() {
        let instance = KilledByArrowTriggerInstanceModel::new(
            vec![ContextAwareEntityPredicateModel::entity_type(id(
                "minecraft:zombie",
            ))],
            IntBoundsModel::ANY,
            Some(ItemPredicateModel::item(id("minecraft:crossbow"))),
        );
        let victims = vec![victim("minecraft:zombie", None)];
        let unique = unique_entity_type_count(&victims);

        assert!(!instance.matches(&victims, unique, None));
        assert!(!instance.matches(&victims, unique, Some(&weapon("minecraft:bow"))));
        assert!(instance.matches(&victims, unique, Some(&weapon("minecraft:crossbow"))));
    }

    #[test]
    fn omitted_weapon_predicate_allows_null_weapon() {
        let instance =
            KilledByArrowTriggerInstanceModel::new(Vec::new(), IntBoundsModel::ANY, None);

        assert!(instance.matches(&[], 0, None));
    }

    #[test]
    fn victim_predicates_consume_distinct_victim_contexts_like_java() {
        let instance = KilledByArrowTriggerInstanceModel::new(
            vec![
                ContextAwareEntityPredicateModel::entity_type(id("minecraft:zombie")),
                ContextAwareEntityPredicateModel::named("Same"),
            ],
            IntBoundsModel::ANY,
            None,
        );

        assert!(!instance.matches(&[victim("minecraft:zombie", Some("Same"))], 1, None));
        assert!(instance.matches(
            &[
                victim("minecraft:zombie", Some("Same")),
                victim("minecraft:skeleton", Some("Same")),
            ],
            2,
            None,
        ));
    }

    #[test]
    fn victim_predicates_match_in_order_and_fail_when_missing() {
        let instance = KilledByArrowTriggerInstanceModel::new(
            vec![
                ContextAwareEntityPredicateModel::entity_type(id("minecraft:zombie")),
                ContextAwareEntityPredicateModel::entity_type(id("minecraft:skeleton")),
            ],
            IntBoundsModel::ANY,
            None,
        );

        assert!(instance.matches(
            &[
                victim("minecraft:skeleton", None),
                victim("minecraft:zombie", None),
            ],
            2,
            None,
        ));
        assert!(!instance.matches(&[victim("minecraft:zombie", None)], 1, None));
    }

    #[test]
    fn unique_entity_type_bound_is_checked_after_victims() {
        let instance = KilledByArrowTriggerInstanceModel::new(
            vec![ContextAwareEntityPredicateModel::entity_type(id(
                "minecraft:zombie",
            ))],
            IntBoundsModel::exactly(2),
            None,
        );
        let victims = vec![
            victim("minecraft:zombie", None),
            victim("minecraft:zombie", Some("Other")),
            victim("minecraft:skeleton", None),
        ];

        assert_eq!(unique_entity_type_count(&victims), 2);
        assert!(instance.matches(&victims, unique_entity_type_count(&victims), None));
        assert!(!instance.matches(&victims, 1, None));
    }

    #[test]
    fn crossbow_killed_victim_factory_wraps_victims_and_crossbow_weapon() {
        let criterion = KilledByArrowTriggerInstanceModel::crossbow_killed_victims(vec![
            EntityPredicateModel::entity_type(id("minecraft:pillager")),
        ]);
        let victims = vec![victim("minecraft:pillager", None)];

        assert_eq!(criterion.trigger_id, id("minecraft:killed_by_arrow"));
        assert!(!criterion.instance.player_predicate_present);
        assert_eq!(criterion.instance.unique_entity_types, IntBoundsModel::ANY);
        assert!(criterion.instance.matches(
            &victims,
            unique_entity_type_count(&victims),
            Some(&weapon("minecraft:crossbow")),
        ));
        assert!(!criterion.instance.matches(
            &victims,
            unique_entity_type_count(&victims),
            Some(&weapon("minecraft:bow")),
        ));
    }

    #[test]
    fn crossbow_killed_unique_factory_uses_empty_victims_and_unique_bound() {
        let criterion =
            KilledByArrowTriggerInstanceModel::crossbow_killed_unique(IntBoundsModel::at_least(2));
        let victims = vec![
            victim("minecraft:zombie", None),
            victim("minecraft:skeleton", None),
        ];

        assert!(criterion.instance.victims.is_empty());
        assert!(criterion.instance.matches(
            &victims,
            unique_entity_type_count(&victims),
            Some(&weapon("minecraft:crossbow")),
        ));
        assert!(!criterion.instance.matches(
            &[victim("minecraft:zombie", None)],
            1,
            Some(&weapon("minecraft:crossbow"))
        ));
    }

    #[test]
    fn validate_reports_victim_predicate_paths() {
        let instance = KilledByArrowTriggerInstanceModel::new(
            vec![
                ContextAwareEntityPredicateModel::entity_type(id("minecraft:zombie")),
                ContextAwareEntityPredicateModel::invalid("bad victim"),
            ],
            IntBoundsModel::ANY,
            None,
        );

        assert_eq!(
            instance.validate(),
            vec!["victims[1]: bad victim".to_string()]
        );
    }
}
