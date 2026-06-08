use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FishingRodHookedTriggerInstance {
    pub player_predicate_present: bool,
    pub rod: Option<ItemPredicateModel>,
    pub entity: Option<ContextAwarePredicateModel>,
    pub item: Option<ItemPredicateModel>,
}

impl FishingRodHookedTriggerInstance {
    pub fn new(
        rod: Option<ItemPredicateModel>,
        entity: Option<ContextAwarePredicateModel>,
        item: Option<ItemPredicateModel>,
    ) -> Self {
        Self {
            player_predicate_present: false,
            rod,
            entity,
            item,
        }
    }

    pub fn fished_item(
        rod: Option<ItemPredicateModel>,
        entity: Option<EntityPredicateModel>,
        item: Option<ItemPredicateModel>,
    ) -> FishingRodHookedCriterion {
        FishingRodHookedCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(rod, entity.map(ContextAwarePredicateModel::wrap), item),
        }
    }

    pub fn context_entity_for_hook(
        hook: HookedEntityModel,
        hooked_in: Option<HookedEntityModel>,
    ) -> LootContextModel {
        LootContextModel::new(Some(hooked_in.unwrap_or(hook)))
    }

    pub fn matches(
        &self,
        rod: &ItemStackModel,
        hooked_in: &LootContextModel,
        items: &[ItemStackModel],
    ) -> bool {
        if self
            .rod
            .as_ref()
            .is_some_and(|predicate| !predicate.test(rod))
        {
            return false;
        }

        if self
            .entity
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(hooked_in))
        {
            return false;
        }

        let Some(item_predicate) = &self.item else {
            return true;
        };

        if hooked_in
            .this_entity
            .as_ref()
            .and_then(HookedEntityModel::item_stack)
            .is_some_and(|stack| item_predicate.test(stack))
        {
            return true;
        }

        items.iter().any(|item| item_predicate.test(item))
    }

    pub fn validate(&self) -> Vec<String> {
        if self.entity.is_some() {
            vec!["entity".to_string()]
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FishingRodHookedCriterion {
    pub trigger_id: Identifier,
    pub instance: FishingRodHookedTriggerInstance,
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:fishing_rod_hooked").unwrap()
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateModel {
    entity_type: Identifier,
}

impl EntityPredicateModel {
    pub fn entity_type(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    entity_type: Identifier,
}

impl ContextAwarePredicateModel {
    pub fn wrap(predicate: EntityPredicateModel) -> Self {
        Self {
            entity_type: predicate.entity_type,
        }
    }

    pub fn matches(&self, context: &LootContextModel) -> bool {
        context
            .this_entity
            .as_ref()
            .is_some_and(|entity| entity.entity_type() == &self.entity_type)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextModel {
    this_entity: Option<HookedEntityModel>,
}

impl LootContextModel {
    pub fn new(this_entity: Option<HookedEntityModel>) -> Self {
        Self { this_entity }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookedEntityModel {
    Entity {
        entity_type: Identifier,
    },
    ItemEntity {
        item_entity_type: Identifier,
        item: ItemStackModel,
    },
}

impl HookedEntityModel {
    pub fn entity_type(&self) -> &Identifier {
        match self {
            Self::Entity { entity_type } => entity_type,
            Self::ItemEntity {
                item_entity_type, ..
            } => item_entity_type,
        }
    }

    fn item_stack(&self) -> Option<&ItemStackModel> {
        match self {
            Self::Entity { .. } => None,
            Self::ItemEntity { item, .. } => Some(item),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn stack(item: &str) -> ItemStackModel {
        ItemStackModel::new(id(item))
    }

    fn item_predicate(item: &str) -> ItemPredicateModel {
        ItemPredicateModel::item(id(item))
    }

    fn entity_context(entity_type: &str) -> LootContextModel {
        LootContextModel::new(Some(HookedEntityModel::Entity {
            entity_type: id(entity_type),
        }))
    }

    fn item_entity_context(item: &str) -> LootContextModel {
        LootContextModel::new(Some(HookedEntityModel::ItemEntity {
            item_entity_type: id("minecraft:item"),
            item: stack(item),
        }))
    }

    #[test]
    fn trigger_context_prefers_hooked_entity_and_falls_back_to_hook() {
        let hook = HookedEntityModel::Entity {
            entity_type: id("minecraft:fishing_bobber"),
        };
        let hooked = HookedEntityModel::Entity {
            entity_type: id("minecraft:cod"),
        };

        let fallback = FishingRodHookedTriggerInstance::context_entity_for_hook(hook.clone(), None);
        let preferred =
            FishingRodHookedTriggerInstance::context_entity_for_hook(hook, Some(hooked));

        assert!(
            ContextAwarePredicateModel::wrap(EntityPredicateModel::entity_type(id(
                "minecraft:fishing_bobber"
            )))
            .matches(&fallback)
        );
        assert!(
            ContextAwarePredicateModel::wrap(EntityPredicateModel::entity_type(id(
                "minecraft:cod"
            )))
            .matches(&preferred)
        );
    }

    #[test]
    fn omitted_predicates_match_any_hook_event() {
        let instance = FishingRodHookedTriggerInstance::new(None, None, None);

        assert!(instance.matches(
            &stack("minecraft:fishing_rod"),
            &entity_context("minecraft:cod"),
            &[stack("minecraft:cod")]
        ));
    }

    #[test]
    fn rod_and_entity_predicates_are_checked_before_items() {
        let instance = FishingRodHookedTriggerInstance::new(
            Some(item_predicate("minecraft:fishing_rod")),
            Some(ContextAwarePredicateModel::wrap(
                EntityPredicateModel::entity_type(id("minecraft:cod")),
            )),
            Some(item_predicate("minecraft:cod")),
        );

        assert!(instance.matches(
            &stack("minecraft:fishing_rod"),
            &entity_context("minecraft:cod"),
            &[stack("minecraft:cod")]
        ));
        assert!(!instance.matches(
            &stack("minecraft:carrot_on_a_stick"),
            &entity_context("minecraft:cod"),
            &[stack("minecraft:cod")]
        ));
        assert!(!instance.matches(
            &stack("minecraft:fishing_rod"),
            &entity_context("minecraft:salmon"),
            &[stack("minecraft:cod")]
        ));
    }

    #[test]
    fn item_predicate_checks_hooked_item_entity_before_collected_items() {
        let instance = FishingRodHookedTriggerInstance::new(
            None,
            None,
            Some(item_predicate("minecraft:enchanted_book")),
        );

        assert!(instance.matches(
            &stack("minecraft:fishing_rod"),
            &item_entity_context("minecraft:enchanted_book"),
            &[]
        ));
        assert!(instance.matches(
            &stack("minecraft:fishing_rod"),
            &entity_context("minecraft:cod"),
            &[stack("minecraft:stick"), stack("minecraft:enchanted_book")]
        ));
        assert!(!instance.matches(
            &stack("minecraft:fishing_rod"),
            &entity_context("minecraft:cod"),
            &[stack("minecraft:stick")]
        ));
    }

    #[test]
    fn factory_wraps_optional_entity_predicate_and_preserves_fields() {
        let criterion = FishingRodHookedTriggerInstance::fished_item(
            Some(item_predicate("minecraft:fishing_rod")),
            Some(EntityPredicateModel::entity_type(id("minecraft:item"))),
            Some(item_predicate("minecraft:cod")),
        );

        assert_eq!(criterion.trigger_id, id("minecraft:fishing_rod_hooked"));
        assert!(!criterion.instance.player_predicate_present);
        assert!(criterion.instance.rod.is_some());
        assert!(criterion.instance.entity.is_some());
        assert!(criterion.instance.item.is_some());
    }

    #[test]
    fn validation_reports_entity_label_only_when_entity_predicate_exists() {
        let without_entity = FishingRodHookedTriggerInstance::new(None, None, None);
        let with_entity = FishingRodHookedTriggerInstance::new(
            None,
            Some(ContextAwarePredicateModel::wrap(
                EntityPredicateModel::entity_type(id("minecraft:cod")),
            )),
            None,
        );

        assert!(without_entity.validate().is_empty());
        assert_eq!(with_entity.validate(), vec!["entity".to_string()]);
    }
}
