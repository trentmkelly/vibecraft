#[cfg(test)]
mod tests {
    #[cfg(vibecraft_has_decompiled_sources)]
    const RECIPE_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/RecipeCommand.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn recipe_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"recipe\")",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "Commands.literal(\"give\")",
            "Commands.literal(\"take\")",
            "EntityArgument.players()",
            "Commands.argument(\"recipe\", ResourceKeyArgument.key(Registries.RECIPE))",
            "ResourceKeyArgument.getRecipe(c, \"recipe\")",
            "Commands.literal(\"*\")",
            "getRecipeManager().getRecipes()",
            "Collections.singleton(ResourceKeyArgument.getRecipe(c, \"recipe\"))",
            "success += player.awardRecipes(recipes);",
            "success += player.resetRecipes(recipes);",
            "if (success == 0)",
            "throw ERROR_GIVE_FAILED.create();",
            "throw ERROR_TAKE_FAILED.create();",
            "players.size() == 1",
            "commands.recipe.give.failed",
            "commands.recipe.take.failed",
            "commands.recipe.give.success.single",
            "commands.recipe.give.success.multiple",
            "commands.recipe.take.success.single",
            "commands.recipe.take.success.multiple",
            "return success;",
        ] {
            assert!(
                RECIPE_COMMAND_JAVA.contains(sentinel),
                "RecipeCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
