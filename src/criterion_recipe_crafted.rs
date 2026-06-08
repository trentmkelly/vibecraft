use crate::criterion_item_predicate::{
    IntBoundsModel, ItemPredicateBuilderModel, ItemPredicateModel, ItemStackModel,
};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeCraftedTriggerInstance {
    pub player: Option<ContextAwarePredicateModel>,
    pub recipe_id: Identifier,
    pub ingredients: Vec<ItemPredicateModel>,
}

impl RecipeCraftedTriggerInstance {
    pub fn new(
        player: Option<ContextAwarePredicateModel>,
        recipe_id: Identifier,
        ingredients: Vec<ItemPredicateModel>,
    ) -> Self {
        Self {
            player,
            recipe_id,
            ingredients,
        }
    }

    pub fn codec_field_names() -> [&'static str; 3] {
        ["player", "recipe_id", "ingredients"]
    }

    pub fn codec_default_ingredients() -> Vec<ItemPredicateModel> {
        Vec::new()
    }

    pub fn crafted_item(
        recipe_id: Identifier,
        predicates: Vec<ItemPredicateBuilderModel>,
    ) -> RecipeCraftedCriterion {
        RecipeCraftedCriterion {
            trigger_id: id("minecraft:recipe_crafted"),
            instance: Self::new(
                None,
                recipe_id,
                predicates
                    .into_iter()
                    .map(ItemPredicateBuilderModel::build)
                    .collect(),
            ),
        }
    }

    pub fn crafted_item_without_ingredients(recipe_id: Identifier) -> RecipeCraftedCriterion {
        RecipeCraftedCriterion {
            trigger_id: id("minecraft:recipe_crafted"),
            instance: Self::new(None, recipe_id, Vec::new()),
        }
    }

    pub fn crafter_crafted_item(recipe_id: Identifier) -> RecipeCraftedCriterion {
        RecipeCraftedCriterion {
            trigger_id: id("minecraft:crafter_recipe_crafted"),
            instance: Self::new(None, recipe_id, Vec::new()),
        }
    }

    pub fn matches(&self, recipe_id: &Identifier, used_ingredients: &[ItemStackModel]) -> bool {
        if recipe_id != &self.recipe_id {
            return false;
        }

        let mut remaining = used_ingredients.to_vec();
        for predicate in &self.ingredients {
            let Some(index) = remaining.iter().position(|stack| predicate.test(stack)) else {
                return false;
            };
            remaining.remove(index);
        }

        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeCraftedCriterion {
    pub trigger_id: Identifier,
    pub instance: RecipeCraftedTriggerInstance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: &str, count: i32) -> ItemStackModel {
        ItemStackModel::new(id(item), count, [])
    }

    fn item(item: &str) -> ItemPredicateBuilderModel {
        ItemPredicateModel::builder().of_items([id(item)])
    }

    #[test]
    fn codec_fields_and_default_ingredients_match_java_record_codec() {
        assert_eq!(
            RecipeCraftedTriggerInstance::codec_field_names(),
            ["player", "recipe_id", "ingredients"]
        );
        assert_eq!(
            RecipeCraftedTriggerInstance::codec_default_ingredients(),
            Vec::new()
        );
    }

    #[test]
    fn crafted_item_factories_use_recipe_crafted_trigger_and_empty_player_predicate() {
        let with_predicates = RecipeCraftedTriggerInstance::crafted_item(
            id("minecraft:stick"),
            vec![item("minecraft:planks")],
        );
        let without_predicates =
            RecipeCraftedTriggerInstance::crafted_item_without_ingredients(id("minecraft:torch"));

        assert_eq!(with_predicates.trigger_id, id("minecraft:recipe_crafted"));
        assert!(with_predicates.instance.player.is_none());
        assert_eq!(with_predicates.instance.recipe_id, id("minecraft:stick"));
        assert_eq!(with_predicates.instance.ingredients.len(), 1);

        assert_eq!(
            without_predicates.trigger_id,
            id("minecraft:recipe_crafted")
        );
        assert!(without_predicates.instance.player.is_none());
        assert_eq!(without_predicates.instance.recipe_id, id("minecraft:torch"));
        assert!(without_predicates.instance.ingredients.is_empty());
    }

    #[test]
    fn crafter_factory_uses_separate_java_trigger_id() {
        let criterion =
            RecipeCraftedTriggerInstance::crafter_crafted_item(id("minecraft:dispenser"));

        assert_eq!(criterion.trigger_id, id("minecraft:crafter_recipe_crafted"));
        assert!(criterion.instance.player.is_none());
        assert_eq!(criterion.instance.recipe_id, id("minecraft:dispenser"));
        assert!(criterion.instance.ingredients.is_empty());
    }

    #[test]
    fn recipe_id_must_match_before_ingredient_predicates_are_considered() {
        let criterion = RecipeCraftedTriggerInstance::crafted_item(
            id("minecraft:stick"),
            vec![item("minecraft:planks")],
        );

        assert!(!criterion
            .instance
            .matches(&id("minecraft:torch"), &[stack("minecraft:planks", 1)]));
    }

    #[test]
    fn ingredient_predicates_match_used_stacks_in_any_order() {
        let criterion = RecipeCraftedTriggerInstance::crafted_item(
            id("minecraft:diamond_sword"),
            vec![item("minecraft:diamond"), item("minecraft:stick")],
        );

        assert!(criterion.instance.matches(
            &id("minecraft:diamond_sword"),
            &[stack("minecraft:stick", 1), stack("minecraft:diamond", 1)]
        ));
    }

    #[test]
    fn each_ingredient_predicate_consumes_one_matching_stack() {
        let criterion = RecipeCraftedTriggerInstance::crafted_item(
            id("minecraft:planks"),
            vec![
                item("minecraft:oak_log").with_count(IntBoundsModel::exactly(1)),
                item("minecraft:oak_log").with_count(IntBoundsModel::exactly(1)),
            ],
        );

        assert!(!criterion
            .instance
            .matches(&id("minecraft:planks"), &[stack("minecraft:oak_log", 1)]));
        assert!(criterion.instance.matches(
            &id("minecraft:planks"),
            &[stack("minecraft:oak_log", 1), stack("minecraft:oak_log", 1)]
        ));
    }

    #[test]
    fn unmatched_extra_used_stacks_are_allowed() {
        let criterion = RecipeCraftedTriggerInstance::crafted_item(
            id("minecraft:crafting_table"),
            vec![item("minecraft:planks")],
        );

        assert!(criterion.instance.matches(
            &id("minecraft:crafting_table"),
            &[
                stack("minecraft:planks", 1),
                stack("minecraft:stick", 1),
                stack("minecraft:diamond", 1),
            ]
        ));
    }
}
