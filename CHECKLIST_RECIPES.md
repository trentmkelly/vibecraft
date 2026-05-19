# Recipe System Checklist

## Relevant Java Source Files

- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeManager.java` — loading, map indexing, reload
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeMap.java` — map indexing by type
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/Recipe.java` — core interface
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/CraftingRecipe.java` — crafting sub-interface
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/NormalCraftingRecipe.java` — normal crafting marker
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/CustomRecipe.java` — special-case base
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SingleItemRecipe.java` — stonecutter base
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/AbstractCookingRecipe.java` — cooking base
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SmithingRecipe.java` — smithing interface
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SelectableRecipe.java` — stonecutter selection
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/ShapedRecipe.java` — shaped crafting
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/ShapedRecipePattern.java` — pattern model
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/ShapelessRecipe.java` — shapeless crafting
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/TransmuteRecipe.java` — transmute recipe
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/ImbueRecipe.java` — imbue recipe
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/Ingredient.java` — ingredient matching
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeSerializers.java` — serializer registry
- `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeBookCategories.java` — book category definitions
- `decompiled-server-26.1.2/data/minecraft/recipe/` — vanilla recipe JSON files (hundreds)
- `RustCraft/src/recipe_system.rs` — RustCraft recipe implementation

## Recipe Manager

- [ ] Implement `RecipeManager` loading from datapack `data/*/recipe/*.json` files with `RecipeSerializer` dispatch
- [ ] Implement `RecipeMap` per-type recipe indexing: `getRecipeFor(type, input, level)` and `getAllRecipesFor(type)`
- [ ] Implement `RecipePropertySet` per `RecipeType` for client-side ingredient caching
- [x] Implement `SelectableRecipe` (stonecutter): list of outputs per input item, recipe selection index
- [ ] Implement recipe `PlacementInfo` for client-side ingredient placement hints
- [ ] Implement reload invalidation: clear recipe cache and re-index on `ServerReloadableResources` reload
- [ ] Implement `RecipeAccess` interface for looking up recipe holders by ID
- [ ] Add unit test: recipe manager loads all vanilla JSON files without error, count matches expected recipe count
- [ ] Add unit test: `getRecipeFor` returns correct result for a representative set of shaped, shapeless, smelting, and stonecutting inputs

## Core Recipe Interfaces

- [ ] Implement `Recipe<I extends RecipeInput>`: `matches(input, level)`, `assemble(input, registries)`, `getResultItem(registries)`, `getId()`, `getSerializer()`, `getType()`, `isSpecial()`, `getRemainingItems(input)` (for remainder stacks like buckets/bottles)
- [ ] Implement `CraftingRecipe`: extends `Recipe<CraftingInput>`, `category()` → `CraftingBookCategory`, `isIncomplete()` check
- [ ] Implement `NormalCraftingRecipe`: marker interface for non-special crafting recipes
- [ ] Implement `CustomRecipe`: base for special single-output recipes with `isSpecial() = true`
- [ ] Implement `SingleItemRecipe`: stonecutter/smithing base with single item input, serializer, and result count
- [ ] Implement `AbstractCookingRecipe`: `cookingTime`, `experience`, `category` (CookingBookCategory), ingredient matching
- [ ] Implement `SmithingRecipe`: template + base + addition input matching, isIncomplete check

## Grid Crafting

- [ ] Implement `ShapedRecipe`: row/column pattern with `ShapedRecipePattern`, ingredient-to-key mapping, result ItemStack, `canCraftInDimensions(w, h)`, mirroring and rotation support NOT applied (vanilla does not rotate shaped recipes)
- [ ] Implement `ShapedRecipePattern`: key → `Ingredient` map, raw pattern strings, width/height derivation, `matches(CraftingInput)` with offset scan
- [ ] Implement `ShapelessRecipe`: unordered `List<Ingredient>` matching, `matches()` via subset check with item-count consumption
- [ ] Implement `TransmuteRecipe`: shaped/shapeless recipe that transforms an item to a new type while preserving certain components (e.g., `minecraft:transmute`)
- [ ] Implement `ImbueRecipe`: imbue enchantment or effect onto an item via crafting
- [x] Add unit test: shaped recipe matches correct grid position, rejects wrong orientation
- [x] Add unit test: shapeless recipe accepts any ingredient ordering, respects count requirements
- [ ] Add unit test: transmute recipe preserves expected components on result

## Special Crafting Recipes

- [ ] Implement `BannerDuplicateRecipe`: copy banner patterns from source banner to blank banner
- [ ] Implement `BookCloningRecipe`: duplicate a written book using a blank book, up to `maxBookGeneration - 1` copies
- [ ] Implement `DecoratedPotRecipe`: combine 4 pottery sherds/bricks → decorated pot with correct side assignment
- [ ] Implement `DyeRecipe`: apply dye to wool/glass/beds/candles/shulker boxes/terracotta etc., color blending for leather armor
- [ ] Implement `FireworkRocketRecipe`: combine paper + gunpowder + optional stars → rocket with flight duration 1–3
- [ ] Implement `FireworkStarRecipe`: combine gunpowder + colors + optional shape/trail/twinkle items → star with encoded effects
- [ ] Implement `FireworkStarFadeRecipe`: apply fade colors to existing firework star
- [ ] Implement `MapExtendingRecipe`: surround map with 8 paper → next-zoom-level map
- [ ] Implement `RepairItemRecipe`: combine two damaged tools/armor of the same type → repaired item with summed durability - 5%
- [ ] Implement `ShieldDecorationRecipe`: apply banner pattern to shield
- [ ] Add unit test for each special recipe: verify result components match vanilla for representative inputs

## Cooking Recipes

- [x] Implement `SmeltingRecipe`: recipe type `minecraft:smelting`, base cooking time 200 ticks, `CookingBookCategory`
- [x] Implement `BlastingRecipe`: recipe type `minecraft:blasting`, base cooking time 100 ticks
- [x] Implement `SmokingRecipe`: recipe type `minecraft:smoking`, base cooking time 100 ticks
- [x] Implement `CampfireCookingRecipe`: recipe type `minecraft:campfire_cooking`, base cooking time 100 ticks in 26.1.2, no fuel required
- [ ] Implement `experience` field: XP stored in furnace, released on item extraction
- [ ] Implement `FuelValues` registry: fuel item → burn time mapping matching vanilla defaults
- [ ] Add unit test: cooking recipe cook time, XP per result, and fuel interaction for representative items

## Smithing and Station Recipes

- [ ] Implement `SimpleSmithingRecipe`: placeholder base (template + base + addition → result, no transformation)
- [ ] Implement `SmithingTransformRecipe`: convert base item type to new type using template (e.g., netherite upgrade); preserve applicable components
- [ ] Implement `SmithingTrimRecipe`: apply armor trim from template + material, store `ArmorTrim` component on result
- [x] Implement `StonecutterRecipe`: single input → single output, multiple outputs per input stone type registered separately
- [ ] Add unit test: smithing transform preserves enchantments and custom name
- [ ] Add unit test: smithing trim applies correct material and pattern components
- [x] Add unit test: stonecutter lists all valid outputs for smooth stone input

## Recipe Validation (All Types)

- [ ] For every recipe type: add JSON decode test verifying all fields parse correctly from vanilla data files
- [ ] For every recipe type: add `matches()` test with valid and invalid inputs including edge-case counts and item tags
- [ ] For every recipe type: add `assemble()` result test verifying output item ID, count, and components
- [ ] For every recipe type: add `getRemainingItems()` test verifying remainder stacks (bottles, buckets) are returned
- [ ] For every recipe type: add recipe-book-unlock test verifying `RecipeHolder` is marked unlocked after crafting
- [ ] For every recipe type: add client-recipe-sync test verifying the recipe packet payload matches the format expected by the 26.1.2 client
- [ ] Add Mineflayer crafting/recipe-book test: unlock recipes, craft in 2×2 and 3×3 grids, open a workstation, verify recipe sync and result slots against vanilla

## Migrated From Main Checklist: Source-Derived Granularity Appendix - Recipe Coverage

- [ ] Implement recipe manager loading, recipe map indexing, property sets, selectable recipes, placement info, display metadata, and reload invalidation.
- [ ] Implement core recipe interfaces: `Recipe`, `CraftingRecipe`, `NormalCraftingRecipe`, `CustomRecipe`, `SingleItemRecipe`, `AbstractCookingRecipe`, `SmithingRecipe`, and `SelectableRecipe`.
- [ ] Implement grid crafting: `ShapedRecipe`, `ShapedRecipePattern`, `ShapelessRecipe`, `TransmuteRecipe`, and `ImbueRecipe`.
- [ ] Implement special crafting recipes: `BannerDuplicateRecipe`, `BookCloningRecipe`, `DecoratedPotRecipe`, `DyeRecipe`, `FireworkRocketRecipe`, `FireworkStarRecipe`, `FireworkStarFadeRecipe`, `MapExtendingRecipe`, `RepairItemRecipe`, and `ShieldDecorationRecipe`.
- [ ] Implement cooking recipes: `SmeltingRecipe`, `BlastingRecipe`, `SmokingRecipe`, and `CampfireCookingRecipe`, including cook time, experience, fuel interaction, and recipe book categories.
- [ ] Implement smithing and station recipes: `SimpleSmithingRecipe`, `SmithingTransformRecipe`, `SmithingTrimRecipe`, and `StonecutterRecipe`.
- [ ] Validate every recipe type with JSON decode tests, crafting matrix tests, result component tests, remainder tests, unlock tests, and client recipe sync tests.
