//! `AbstractFurnaceBlockEntity.serverTick` over the NBT-backed furnace, blast
//! furnace and smoker, plus the menu-side `setItem` bookkeeping.

use crate::block_entity::{
    AbstractFurnaceBlockEntity, FurnaceBlockEntityKind, FurnaceCookingRecipe, FurnaceTickResult,
    PotItemStack,
};
use crate::recipe_system::RecipeMap;
use crate::storage::chunk::BlockStateEntry;
use crate::storage::nbt::Tag;

use super::lifecycle::{block_entity_identity, furnace_kind};
use super::{format_block_state, BlockEntityTickEnvironment, TickOutput};

/// NBT keys `AbstractFurnaceBlockEntity.saveAdditional` owns; every other key on
/// the compound (`id`, position, `components`, `CustomName`, `lock`, ...) belongs
/// to `BlockEntity` / `BaseContainerBlockEntity` and is preserved untouched.
const OWNED_KEYS: [&str; 6] = [
    "cooking_time_spent",
    "cooking_total_time",
    "lit_time_remaining",
    "lit_total_time",
    "Items",
    "RecipesUsed",
];

/// `BlockEntityTicker` for the three furnace types. The recipe is resolved like
/// Java's `quickCheck.getRecipeFor(new SingleRecipeInput(ingredient), level)`.
pub fn tick_furnace_family(
    tag: &Tag,
    state: &BlockStateEntry,
    env: &BlockEntityTickEnvironment<'_>,
) -> Option<TickOutput> {
    let (_, ty) = block_entity_identity(tag)?;
    let kind = furnace_kind(ty)?;
    let mut entity = AbstractFurnaceBlockEntity::load_additional(kind, tag);
    let recipe = lookup_recipe(env.recipes, kind, &entity);
    let result = entity.server_tick(env.fuel_values, recipe.as_ref());

    let updated = merge_owned_fields(tag, &entity.save_additional());
    // `AbstractFurnaceBlock.LIT` is rewritten only when the lit state flipped.
    let new_block_state = match result {
        FurnaceTickResult::LitChanged { lit } => {
            let mut lit_state = state.clone();
            lit_state
                .properties
                .insert("lit".to_string(), lit.to_string());
            Some(format_block_state(&lit_state))
        }
        _ => None,
    };
    (&updated != tag || new_block_state.is_some()).then_some(TickOutput {
        tag: updated,
        new_block_state,
    })
}

/// `RecipeManager.getRecipeFor` for the furnace's ingredient slot.
fn lookup_recipe(
    recipes: &RecipeMap,
    kind: FurnaceBlockEntityKind,
    entity: &AbstractFurnaceBlockEntity,
) -> Option<FurnaceCookingRecipe> {
    let ingredient = entity.items[AbstractFurnaceBlockEntity::INGREDIENT_SLOT].as_ref()?;
    FurnaceCookingRecipe::lookup(recipes, kind.recipe_type(), &ingredient.item_id)
}

/// Replaces the furnace-owned fields of `existing` with those of `saved`,
/// keeping all other keys. Item entries keep their `components` when the stack
/// is still the same item in the same slot, since [`PotItemStack`] does not
/// model data components.
fn merge_owned_fields(existing: &Tag, saved: &Tag) -> Tag {
    let (Tag::Compound(existing_fields), Tag::Compound(saved_fields)) = (existing, saved) else {
        return saved.clone();
    };
    let mut fields: Vec<(String, Tag)> = existing_fields
        .iter()
        .filter(|(key, _)| !OWNED_KEYS.contains(&key.as_str()))
        .cloned()
        .collect();
    let old_items = existing_fields
        .iter()
        .find_map(|(key, value)| (key == "Items").then_some(value));
    for (key, value) in saved_fields {
        if key == "Items" {
            fields.push((key.clone(), restore_item_components(value, old_items)));
        } else {
            fields.push((key.clone(), value.clone()));
        }
    }
    Tag::Compound(fields)
}

/// Copies every non-`id`/`count` field (chiefly `components`) of the previous
/// entry in the same slot onto the new entry when the item id is unchanged.
fn restore_item_components(new_items: &Tag, old_items: Option<&Tag>) -> Tag {
    let (Tag::List(items), Some(Tag::List(old_items))) = (new_items, old_items) else {
        return new_items.clone();
    };
    let slot_of = |entries: &[(String, Tag)]| {
        entries.iter().find_map(|(key, value)| match value {
            Tag::Byte(slot) if key == "Slot" => Some(*slot),
            _ => None,
        })
    };
    let id_of = |entries: &[(String, Tag)]| {
        entries.iter().find_map(|(key, value)| match value {
            Tag::String(id) if key == "id" => Some(id.clone()),
            _ => None,
        })
    };
    let restored = items
        .iter()
        .map(|item| {
            let Tag::Compound(entries) = item else {
                return item.clone();
            };
            let previous = old_items.iter().find_map(|old| match old {
                Tag::Compound(old_entries)
                    if slot_of(old_entries) == slot_of(entries)
                        && id_of(old_entries) == id_of(entries) =>
                {
                    Some(old_entries)
                }
                _ => None,
            });
            let mut merged = entries.clone();
            if let Some(previous) = previous {
                merged.extend(
                    previous
                        .iter()
                        .filter(|(key, _)| !matches!(key.as_str(), "Slot" | "id" | "count"))
                        .cloned(),
                );
            }
            Tag::Compound(merged)
        })
        .collect();
    Tag::List(restored)
}

/// Applies the slots a player left in an open furnace menu to the furnace NBT,
/// like Java's menu writing through `Container.setItem`: only changed slots go
/// through `AbstractFurnaceBlockEntity.setItem`, which resets the cook progress
/// and total time when the ingredient slot changes.
pub fn apply_menu_slots(
    kind: FurnaceBlockEntityKind,
    existing: &Tag,
    slots: &[Option<PotItemStack>],
    recipes: &RecipeMap,
) -> Tag {
    let mut entity = AbstractFurnaceBlockEntity::load_additional(kind, existing);
    for (slot, stack) in slots
        .iter()
        .enumerate()
        .take(AbstractFurnaceBlockEntity::SLOT_COUNT)
    {
        if entity.items[slot] == *stack {
            continue;
        }
        let recipe = stack.as_ref().and_then(|stack| {
            FurnaceCookingRecipe::lookup(recipes, kind.recipe_type(), &stack.item_id)
        });
        entity.set_item(slot, stack.clone(), recipe.as_ref());
    }
    merge_owned_fields(existing, &entity.save_additional())
}

/// Writes `updated` (a furnace compound whose `Items` already hold the exact
/// new stacks, components included) after a container-side change and applies
/// `AbstractFurnaceBlockEntity.setItem`'s side effect on the cook counters:
/// changing the ingredient stack resets `cooking_time_spent` and recomputes
/// `cooking_total_time`. `existing` is the compound before the change.
pub fn apply_container_slots(
    kind: FurnaceBlockEntityKind,
    existing: &Tag,
    updated: &Tag,
    slots: &[Option<super::stack::Stack>],
    recipes: &RecipeMap,
) -> Tag {
    let pot_slots: Vec<Option<PotItemStack>> = slots
        .iter()
        .map(|stack| {
            stack.as_ref().map(|stack| PotItemStack {
                item_id: stack.id.clone(),
                count: stack.count,
            })
        })
        .collect();
    let with_counters = apply_menu_slots(kind, existing, &pot_slots, recipes);
    let (Tag::Compound(counters), Tag::Compound(updated_fields)) = (&with_counters, updated)
    else {
        return updated.clone();
    };
    const SETITEM_KEYS: [&str; 2] = ["cooking_time_spent", "cooking_total_time"];
    let mut fields: Vec<(String, Tag)> = updated_fields
        .iter()
        .filter(|(key, _)| !SETITEM_KEYS.contains(&key.as_str()))
        .cloned()
        .collect();
    fields.extend(
        counters
            .iter()
            .filter(|(key, _)| SETITEM_KEYS.contains(&key.as_str()))
            .cloned(),
    );
    Tag::Compound(fields)
}

/// What taking the furnace result awards.
#[derive(Debug, Clone, PartialEq)]
pub struct FurnaceAward {
    /// The furnace compound with `RecipesUsed` cleared.
    pub tag: Tag,
    /// Experience points to pop (`createExperience`, summed over recipes).
    pub experience: i32,
    /// The recipes to add to the player's recipe book (`awardRecipes`).
    pub recipe_ids: Vec<&'static str>,
}

/// `AbstractFurnaceBlockEntity.awardUsedRecipesAndPopExperience`: for every
/// recipe in `RecipesUsed` that still exists, rolls `floor(count * xp)` plus one
/// with probability `frac(count * xp)` (`roll` is `level.getRandom().nextFloat()`),
/// then clears the used recipes.
pub fn award_used_recipes(
    kind: FurnaceBlockEntityKind,
    existing: &Tag,
    recipes: &RecipeMap,
    roll: impl FnMut() -> f32,
) -> FurnaceAward {
    let mut entity = AbstractFurnaceBlockEntity::load_additional(kind, existing);
    let recipe_ids: Vec<&'static str> = entity
        .recipes_used
        .keys()
        .filter_map(|id| recipes.by_key(id).map(|holder| holder.id))
        .collect();
    let experience = entity.xp_to_award_and_clear(
        |id| {
            match &recipes.by_key(id)?.recipe {
                crate::recipe_system::RecipeKind::Cooking {
                    experience_millis, ..
                } => Some(*experience_millis as f32 / 1000.0),
                _ => None,
            }
        },
        roll,
    );
    FurnaceAward {
        tag: merge_owned_fields(existing, &entity.save_additional()),
        experience,
        recipe_ids,
    }
}
