const RECIPE_UNLOCK_ADVANCEMENT_BUILDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeUnlockAdvancementBuilder.java"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecipeUnlockCategory {
    BuildingBlocks,
    Tools,
    Misc,
}

impl RecipeUnlockCategory {
    fn folder_name(self) -> &'static str {
        match self {
            Self::BuildingBlocks => "building_blocks",
            Self::Tools => "tools",
            Self::Misc => "misc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecipeUnlockAdvancementModel {
    criteria: Vec<(String, String)>,
}

impl RecipeUnlockAdvancementModel {
    fn new() -> Self {
        Self {
            criteria: Vec::new(),
        }
    }

    fn unlocked_by(&mut self, name: &str, criterion: &str) {
        if let Some((_, existing)) = self
            .criteria
            .iter_mut()
            .find(|(existing_name, _)| existing_name == name)
        {
            *existing = criterion.to_string();
        } else {
            self.criteria
                .push((name.to_string(), criterion.to_string()));
        }
    }

    fn build(
        &self,
        recipe_id: &str,
        category: RecipeUnlockCategory,
    ) -> Result<AdvancementBuildModel, String> {
        if self.criteria.is_empty() {
            return Err(format!(
                "No way of obtaining recipe {}",
                identifier_path(recipe_id)
            ));
        }

        let mut criteria = vec![(
            "has_the_recipe".to_string(),
            format!("recipe_unlocked:{recipe_id}"),
        )];
        criteria.extend(self.criteria.iter().cloned());
        Ok(AdvancementBuildModel {
            parent: "recipes/root".to_string(),
            rewards: format!("recipe:{recipe_id}"),
            requirements_strategy: "OR".to_string(),
            criteria,
            id: format!(
                "recipes/{}/{}",
                category.folder_name(),
                identifier_path(recipe_id)
            ),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdvancementBuildModel {
    parent: String,
    rewards: String,
    requirements_strategy: String,
    criteria: Vec<(String, String)>,
    id: String,
}

fn identifier_path(id: &str) -> &str {
    id.split_once(':').map_or(id, |(_, path)| path)
}

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, expected: &[&str]) {
    for sentinel in expected {
        assert!(
            source.contains(sentinel),
            "missing Java sentinel: {sentinel}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_unlock_advancement_rejects_empty_criteria_like_java() {
        let builder = RecipeUnlockAdvancementModel::new();
        assert_eq!(
            builder
                .build("minecraft:diamond_pickaxe", RecipeUnlockCategory::Tools)
                .unwrap_err(),
            "No way of obtaining recipe diamond_pickaxe"
        );
    }

    #[test]
    fn recipe_unlock_advancement_builds_recipe_parent_reward_or_requirements_and_path() {
        let mut builder = RecipeUnlockAdvancementModel::new();
        builder.unlocked_by("has_diamonds", "inventory_changed:diamond");
        builder.unlocked_by("has_sticks", "inventory_changed:stick");

        let advancement = builder
            .build("minecraft:diamond_pickaxe", RecipeUnlockCategory::Tools)
            .unwrap();
        assert_eq!(advancement.parent, "recipes/root");
        assert_eq!(advancement.rewards, "recipe:minecraft:diamond_pickaxe");
        assert_eq!(advancement.requirements_strategy, "OR");
        assert_eq!(advancement.id, "recipes/tools/diamond_pickaxe");
        assert_eq!(
            advancement.criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:diamond_pickaxe".to_string(),
                ),
                (
                    "has_diamonds".to_string(),
                    "inventory_changed:diamond".to_string(),
                ),
                (
                    "has_sticks".to_string(),
                    "inventory_changed:stick".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn recipe_unlock_advancement_uses_linked_hash_map_replacement_semantics() {
        let mut builder = RecipeUnlockAdvancementModel::new();
        builder.unlocked_by("has_base", "first");
        builder.unlocked_by("has_addition", "second");
        builder.unlocked_by("has_base", "replacement");

        let advancement = builder
            .build("minecraft:decorated_pot", RecipeUnlockCategory::Misc)
            .unwrap();
        assert_eq!(
            advancement.criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:decorated_pot".to_string(),
                ),
                ("has_base".to_string(), "replacement".to_string()),
                ("has_addition".to_string(), "second".to_string()),
            ]
        );
    }

    #[test]
    fn recipe_unlock_advancement_category_folder_prefix_matches_recipe_category() {
        let mut builder = RecipeUnlockAdvancementModel::new();
        builder.unlocked_by("has_oak_log", "inventory_changed:oak_log");
        assert_eq!(
            builder
                .build("minecraft:oak_planks", RecipeUnlockCategory::BuildingBlocks)
                .unwrap()
                .id,
            "recipes/building_blocks/oak_planks"
        );
    }

    #[test]
    fn recipe_unlock_advancement_builder_source_counts_match_authoritative_java() {
        assert_eq!(
            count_occurrences(RECIPE_UNLOCK_ADVANCEMENT_BUILDER_JAVA, "unlockedBy"),
            1
        );
        assert_eq!(
            count_occurrences(RECIPE_UNLOCK_ADVANCEMENT_BUILDER_JAVA, "build("),
            2
        );
        assert_eq!(
            count_occurrences(RECIPE_UNLOCK_ADVANCEMENT_BUILDER_JAVA, "criteria"),
            4
        );
        assert_eq!(
            count_occurrences(RECIPE_UNLOCK_ADVANCEMENT_BUILDER_JAVA, "LinkedHashMap"),
            2
        );
    }

    #[test]
    fn recipe_unlock_advancement_builder_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(
            RECIPE_UNLOCK_ADVANCEMENT_BUILDER_JAVA,
            &[
                "private final Map<String, Criterion<?>> criteria = new LinkedHashMap<>();",
                "public void unlockedBy(final String name, final Criterion<?> criterion)",
                "this.criteria.put(name, criterion);",
                "if (this.criteria.isEmpty())",
                "throw new IllegalStateException(\"No way of obtaining recipe \" + id.identifier());",
                "output.advancement()",
                ".addCriterion(\"has_the_recipe\", RecipeUnlockedTrigger.unlocked(id))",
                ".rewards(AdvancementRewards.Builder.recipe(id))",
                ".requirements(AdvancementRequirements.Strategy.OR);",
                "this.criteria.forEach(advancement::addCriterion);",
                "return advancement.build(id.identifier().withPrefix(\"recipes/\" + category.getFolderName() + \"/\"));",
            ],
        );
    }
}
