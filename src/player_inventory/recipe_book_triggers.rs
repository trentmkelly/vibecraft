//! The recipe-book hooks advancements need on [`InventoryMenu`].

use super::InventoryMenu;

impl InventoryMenu {
    /// Recipes newly added to the book since the last call, for
    /// `RecipeUnlockedTrigger.trigger(player, recipe)` (`ServerRecipeBook.addRecipes`).
    /// Independent of [`Self::drain_recipe_unlock_events`], which feeds the client packet.
    pub fn drain_recipe_trigger_events(&mut self) -> Vec<&'static str> {
        std::mem::take(&mut self.recipe_trigger_events)
    }

    /// `ServerPlayer.awardRecipesByKey` for one recipe id: unlocks it when the recipe
    /// exists. Returns whether it was newly added.
    pub fn unlock_recipe_by_key(&mut self, recipe_key: &str) -> bool {
        let Some(holder_id) = self.recipes.by_key(recipe_key).map(|holder| holder.id) else {
            return false;
        };
        self.unlock_recipe(holder_id)
    }
}
