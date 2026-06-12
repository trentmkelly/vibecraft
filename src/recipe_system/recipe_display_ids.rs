use super::*;

impl RecipeMap {
    /// Java `RecipeManager.unpackRecipeInfo`: recipe display ids are compact
    /// indexes over the server's synchronized display table, not raw positions in
    /// the recipe map. VibeCraft currently synchronizes the ordinary display
    /// variants below; omitted display kinds must not consume ids.
    pub fn display_index_for_recipe(&self, recipe_id: &str) -> Option<i32> {
        self.values()
            .iter()
            .filter(|holder| recipe_has_synchronized_display(&holder.recipe))
            .position(|holder| holder.id == recipe_id)
            .map(|index| index as i32)
    }

    /// Resolve a client `RecipeDisplayId` back to the parent recipe holder, like
    /// Java `RecipeManager.getRecipeFromDisplay(id).parent()`.
    pub fn holder_for_display_index(&self, display_index: i32) -> Option<&RecipeHolder> {
        if display_index < 0 {
            return None;
        }
        self.values()
            .iter()
            .filter(|holder| recipe_has_synchronized_display(&holder.recipe))
            .nth(display_index as usize)
    }
}

fn recipe_has_synchronized_display(recipe: &RecipeKind) -> bool {
    matches!(
        recipe,
        RecipeKind::Shaped { .. }
            | RecipeKind::Shapeless { .. }
            | RecipeKind::Cooking { .. }
            | RecipeKind::Stonecutting { .. }
            | RecipeKind::SmithingTransform { .. }
            | RecipeKind::SmithingTrim { .. }
    )
}
