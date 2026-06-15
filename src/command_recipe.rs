#[cfg(test)]
mod tests {
    #[cfg(vibecraft_has_decompiled_sources)]
    const RECIPE_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/RecipeCommand.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn recipe_command_source_matches_java_26_1_2() {
        assert!(RECIPE_COMMAND_JAVA.contains("Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)"));
        assert!(RECIPE_COMMAND_JAVA.contains("ResourceKeyArgument.key(Registries.RECIPE)"));
        assert!(RECIPE_COMMAND_JAVA.contains("ResourceKeyArgument.getRecipe(c, \"recipe\")"));
        assert!(RECIPE_COMMAND_JAVA.contains("getRecipeManager().getRecipes()"));
        assert!(RECIPE_COMMAND_JAVA.contains("success += player.awardRecipes(recipes);"));
        assert!(RECIPE_COMMAND_JAVA.contains("success += player.resetRecipes(recipes);"));
        assert!(RECIPE_COMMAND_JAVA.contains("commands.recipe.give.failed"));
        assert!(RECIPE_COMMAND_JAVA.contains("commands.recipe.take.failed"));
        assert!(RECIPE_COMMAND_JAVA.contains("commands.recipe.give.success.single"));
        assert!(RECIPE_COMMAND_JAVA.contains("commands.recipe.give.success.multiple"));
        assert!(RECIPE_COMMAND_JAVA.contains("commands.recipe.take.success.single"));
        assert!(RECIPE_COMMAND_JAVA.contains("commands.recipe.take.success.multiple"));
    }
}
