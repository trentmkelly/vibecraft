use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeUnlockedTriggerInstance {
    pub player: Option<ContextAwarePredicateModel>,
    pub recipe: Identifier,
}

impl RecipeUnlockedTriggerInstance {
    pub fn new(player: Option<ContextAwarePredicateModel>, recipe: Identifier) -> Self {
        Self { player, recipe }
    }

    pub fn codec_field_names() -> [&'static str; 2] {
        ["player", "recipe"]
    }

    pub fn unlocked(recipe: Identifier) -> RecipeUnlockedCriterion {
        RecipeUnlockedCriterion {
            trigger_id: id("minecraft:recipe_unlocked"),
            instance: Self::new(None, recipe),
        }
    }

    pub fn matches(&self, recipe: &RecipeHolderModel) -> bool {
        self.recipe == recipe.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeUnlockedCriterion {
    pub trigger_id: Identifier,
    pub instance: RecipeUnlockedTriggerInstance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeHolderModel {
    id: Identifier,
}

impl RecipeHolderModel {
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_fields_match_java_record_codec() {
        assert_eq!(
            RecipeUnlockedTriggerInstance::codec_field_names(),
            ["player", "recipe"]
        );
    }

    #[test]
    fn unlocked_factory_uses_recipe_unlocked_trigger_and_empty_player_predicate() {
        let criterion = RecipeUnlockedTriggerInstance::unlocked(id("minecraft:stick"));

        assert_eq!(criterion.trigger_id, id("minecraft:recipe_unlocked"));
        assert!(criterion.instance.player.is_none());
        assert_eq!(criterion.instance.recipe, id("minecraft:stick"));
    }

    #[test]
    fn matches_recipe_holder_id_by_exact_recipe_key_identity() {
        let instance = RecipeUnlockedTriggerInstance::new(None, id("minecraft:diamond_sword"));

        assert!(instance.matches(&RecipeHolderModel::new(id("minecraft:diamond_sword"))));
        assert!(!instance.matches(&RecipeHolderModel::new(id("minecraft:iron_sword"))));
    }
}
