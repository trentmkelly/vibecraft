use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialRecipeKind {
    #[cfg(test)]
    Transmute,
    MapCloning,
    MapExtending,
    BannerDuplicate,
    ShieldDecoration,
    FireworkRocket,
    FireworkStar,
    FireworkStarFade,
    #[cfg(test)]
    SuspiciousStew,
    BookCloning,
    RepairItem,
    DyedItem,
    DecoratedPot,
    #[cfg(test)]
    Imbue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeKind {
    Shaped {
        width: usize,
        height: usize,
        pattern: Vec<Option<IngredientSpec>>,
        result: ItemAmount,
    },
    Shapeless {
        ingredients: Vec<IngredientSpec>,
        result: ItemAmount,
    },
    Cooking {
        kind: CookingKind,
        ingredient: IngredientSpec,
        result: ItemAmount,
        experience_millis: i32,
        cooking_time: Option<i32>,
        /// `AbstractCookingRecipe.category()` — the `CookingBookCategory` from the
        /// recipe JSON, used to pick the recipe-book group.
        category: CookingBookCategory,
    },
    Stonecutting {
        ingredient: IngredientSpec,
        result: ItemAmount,
    },
    Transmute {
        input: IngredientSpec,
        material: IngredientSpec,
        min_material_count: u32,
        max_material_count: u32,
        result: ItemAmount,
        add_material_count_to_result: bool,
    },
    Imbue {
        source: IngredientSpec,
        material: IngredientSpec,
        result: ItemAmount,
    },
    SmithingTransform {
        template: IngredientSpec,
        base: IngredientSpec,
        addition: IngredientSpec,
        result: ItemAmount,
    },
    SmithingTrim {
        template: IngredientSpec,
        base: IngredientSpec,
        addition: IngredientSpec,
        /// The `TrimPattern` id this recipe applies (e.g. `"minecraft:sentry"`),
        /// from the recipe JSON's `pattern` field.
        pattern: &'static str,
    },
    Special {
        kind: SpecialRecipeKind,
        result_hint: Option<ItemAmount>,
    },
}

impl RecipeKind {
    pub fn recipe_type(&self) -> &'static str {
        match self {
            RecipeKind::Shaped { .. }
            | RecipeKind::Shapeless { .. }
            | RecipeKind::Transmute { .. }
            | RecipeKind::Imbue { .. }
            | RecipeKind::Special { .. } => "crafting",
            RecipeKind::Cooking { kind, .. } => match kind {
                CookingKind::Smelting => "smelting",
                CookingKind::Blasting => "blasting",
                CookingKind::Smoking => "smoking",
                CookingKind::CampfireCooking => "campfire_cooking",
            },
            RecipeKind::Stonecutting { .. } => "stonecutting",
            RecipeKind::SmithingTransform { .. } | RecipeKind::SmithingTrim { .. } => "smithing",
        }
    }

    #[cfg(test)]
    pub fn serializer(&self) -> &'static str {
        match self {
            RecipeKind::Shaped { .. } => "crafting_shaped",
            RecipeKind::Shapeless { .. } => "crafting_shapeless",
            RecipeKind::Cooking { kind, .. } => kind.serializer(),
            RecipeKind::Stonecutting { .. } => "stonecutting",
            RecipeKind::Transmute { .. } => "crafting_transmute",
            RecipeKind::Imbue { .. } => "crafting_imbue",
            RecipeKind::SmithingTransform { .. } => "smithing_transform",
            RecipeKind::SmithingTrim { .. } => "smithing_trim",
            RecipeKind::Special { kind, .. } => match kind {
                SpecialRecipeKind::Transmute => "crafting_transmute",
                SpecialRecipeKind::MapCloning => "crafting_special_mapcloning",
                SpecialRecipeKind::MapExtending => "crafting_special_mapextending",
                SpecialRecipeKind::BannerDuplicate => "crafting_special_bannerduplicate",
                SpecialRecipeKind::ShieldDecoration => "crafting_special_shielddecoration",
                SpecialRecipeKind::FireworkRocket => "crafting_special_firework_rocket",
                SpecialRecipeKind::FireworkStar => "crafting_special_firework_star",
                SpecialRecipeKind::FireworkStarFade => "crafting_special_firework_star_fade",
                SpecialRecipeKind::SuspiciousStew => "crafting_special_suspiciousstew",
                SpecialRecipeKind::BookCloning => "crafting_special_bookcloning",
                SpecialRecipeKind::RepairItem => "crafting_special_repairitem",
                SpecialRecipeKind::DyedItem => "crafting_dye",
                SpecialRecipeKind::DecoratedPot => "crafting_decorated_pot",
                SpecialRecipeKind::Imbue => "crafting_imbue",
            },
        }
    }

    #[cfg(test)]
    pub fn is_special(&self) -> bool {
        matches!(self, RecipeKind::Special { .. })
    }

    #[cfg(test)]
    pub fn recipe_book_category(&self) -> &'static str {
        match self {
            RecipeKind::Shaped { .. }
            | RecipeKind::Shapeless { .. }
            | RecipeKind::Transmute { .. }
            | RecipeKind::Imbue { .. } => "crafting_misc",
            // `SmeltingRecipe.recipeBookCategory`: FOOD/BLOCKS/MISC -> the matching
            // furnace book group.
            RecipeKind::Cooking {
                kind: CookingKind::Smelting,
                category,
                ..
            } => match category {
                CookingBookCategory::Food => "furnace_food",
                CookingBookCategory::Blocks => "furnace_blocks",
                CookingBookCategory::Misc => "furnace_misc",
            },
            // `BlastingRecipe.recipeBookCategory`: BLOCKS -> blocks, FOOD/MISC -> misc.
            RecipeKind::Cooking {
                kind: CookingKind::Blasting,
                category,
                ..
            } => match category {
                CookingBookCategory::Blocks => "blast_furnace_blocks",
                CookingBookCategory::Food | CookingBookCategory::Misc => "blast_furnace_misc",
            },
            RecipeKind::Cooking {
                kind: CookingKind::Smoking,
                ..
            } => "smoker_food",
            RecipeKind::Cooking {
                kind: CookingKind::CampfireCooking,
                ..
            } => "campfire",
            RecipeKind::Stonecutting { .. } => "stonecutter",
            RecipeKind::SmithingTransform { .. } | RecipeKind::SmithingTrim { .. } => "smithing",
            RecipeKind::Special { .. } => "crafting_misc",
        }
    }

    #[cfg(test)]
    pub fn show_notification(&self) -> bool {
        !self.is_special()
    }

    #[cfg(test)]
    pub fn single_item_input(&self) -> Option<&IngredientSpec> {
        match self {
            RecipeKind::Cooking { ingredient, .. }
            | RecipeKind::Stonecutting { ingredient, .. } => Some(ingredient),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn single_item_result(&self) -> Option<&ItemAmount> {
        match self {
            RecipeKind::Cooking { result, .. } | RecipeKind::Stonecutting { result, .. } => {
                Some(result)
            }
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn cooking_experience_millis(&self) -> Option<i32> {
        match self {
            RecipeKind::Cooking {
                experience_millis, ..
            } => Some(*experience_millis),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn smithing_placement_info(&self) -> Option<PlacementInfo> {
        match self {
            RecipeKind::SmithingTransform {
                template,
                base,
                addition,
                ..
            }
            | RecipeKind::SmithingTrim {
                template,
                base,
                addition,
                ..
            } => Some(PlacementInfo::create_from_optionals(vec![
                optional_ingredient(template),
                Some(base.clone()),
                optional_ingredient(addition),
            ])),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn smithing_is_incomplete(&self) -> Option<bool> {
        self.smithing_placement_info()
            .map(|placement| placement.is_impossible_to_place())
    }

    pub fn matches(
        &self,
        grid_width: usize,
        grid_height: usize,
        items: &[Option<&'static str>],
    ) -> bool {
        match self {
            RecipeKind::Shaped {
                width,
                height,
                pattern,
                ..
            } => shaped_matches(*width, *height, pattern, grid_width, grid_height, items),
            RecipeKind::Shapeless { ingredients, .. } => shapeless_matches(ingredients, items),
            RecipeKind::Transmute {
                input,
                material,
                min_material_count,
                max_material_count,
                result,
                add_material_count_to_result,
            } => transmute_matches(
                input,
                material,
                *min_material_count,
                *max_material_count,
                result,
                *add_material_count_to_result,
                items,
            ),
            RecipeKind::Imbue {
                source, material, ..
            } => imbue_matches(source, material, grid_width, grid_height, items),
            RecipeKind::Cooking { ingredient, .. }
            | RecipeKind::Stonecutting { ingredient, .. } => {
                let mut present = items.iter().flatten();
                let Some(item) = present.next() else {
                    return false;
                };
                present.next().is_none() && ingredient.matches(item)
            }
            RecipeKind::SmithingTransform {
                template,
                base,
                addition,
                ..
            }
            | RecipeKind::SmithingTrim {
                template,
                base,
                addition,
                ..
            } => {
                let [Some(template_item), Some(base_item), Some(addition_item)] = items else {
                    return false;
                };
                template.matches(template_item)
                    && base.matches(base_item)
                    && addition.matches(addition_item)
            }
            // TODO(recipes-special): `CustomRecipe` subclasses (RepairItemRecipe,
            // BannerDuplicateRecipe, BookCloningRecipe, DecoratedPotRecipe,
            // DyedItemRecipe, FireworkRocket/Star/StarFadeRecipe, MapCloning/
            // ExtendingRecipe, ShieldDecorationRecipe) need *stack-aware* matching —
            // their result depends on input components/damage/counts, not item ids,
            // and this `matches` only receives `&[Option<&'static str>]`. They are
            // matched + assembled live by `special_crafting::special_crafting_result`
            // (consulted via `RecipeMap::special_crafting_result`), so this id-only
            // path always reports `false` for them.
            RecipeKind::Special { .. } => false,
        }
    }

    pub fn assemble(&self) -> Option<ItemAmount> {
        match self {
            RecipeKind::Shaped { result, .. }
            | RecipeKind::Shapeless { result, .. }
            | RecipeKind::Cooking { result, .. }
            | RecipeKind::Stonecutting { result, .. }
            | RecipeKind::Transmute { result, .. }
            | RecipeKind::Imbue { result, .. }
            | RecipeKind::SmithingTransform { result, .. } => Some(result.clone()),
            RecipeKind::SmithingTrim { .. } => None,
            RecipeKind::Special { result_hint, .. } => result_hint.clone(),
        }
    }

    /// Build the result stack for the recipes whose output preserves input data
    /// components — `Transmute` copies the transmuted item's components (e.g. a
    /// recoloured shulker box keeps its contents, a cloned map keeps its id) and
    /// `Imbue` copies the centre source's potion contents (tipped arrows). Returns
    /// `None` for every other kind, whose result is a plain new item (use
    /// [`assemble`](Self::assemble)).
    pub fn component_aware_result(&self, grid: &[ItemStack]) -> Option<ItemStack> {
        match self {
            // `TransmuteRecipe.assemble` = createWithOriginalComponents(result, input,
            // materialCount): the result is the matched input's components retyped to
            // the result item, at `result.count + materialCount` when configured.
            RecipeKind::Transmute {
                input,
                material,
                result,
                add_material_count_to_result,
                ..
            } => {
                let mut input_stack = None;
                let mut material_count = 0u32;
                for stack in grid.iter().filter(|stack| !stack.is_empty()) {
                    if input_stack.is_none() && input.matches(stack.item_id()) {
                        input_stack = Some(stack);
                    } else if material.matches(stack.item_id()) {
                        material_count += 1;
                    }
                }
                let input_stack = input_stack?;
                let count =
                    transmute_result_count(result.count, material_count, *add_material_count_to_result);
                Some(input_stack.transmute_copy(result.item, count as i32))
            }
            // `ImbueRecipe.assemble`: a fresh result carrying the centre slot's
            // potion contents.
            RecipeKind::Imbue { result, .. } => {
                let source = grid.get(4)?;
                let mut stack = ItemStack::new(result.item, result.count as i32);
                if let Some(potion) = source.component("minecraft:potion_contents") {
                    stack.set_component(potion.clone());
                }
                Some(stack)
            }
            _ => None,
        }
    }

    pub fn get_remaining_items(
        &self,
        input: &[Option<CraftingStack>],
    ) -> Vec<Option<CraftingStack>> {
        match self {
            RecipeKind::Shaped { .. }
            | RecipeKind::Shapeless { .. }
            | RecipeKind::Transmute { .. }
            | RecipeKind::Imbue { .. }
            | RecipeKind::Special { .. } => default_crafting_remaining_items(input),
            RecipeKind::Cooking { .. }
            | RecipeKind::Stonecutting { .. }
            | RecipeKind::SmithingTransform { .. }
            | RecipeKind::SmithingTrim { .. } => vec![None; input.len()],
        }
    }

    #[cfg(test)]
    pub fn cooking_time(&self) -> Option<i32> {
        match self {
            RecipeKind::Cooking {
                kind, cooking_time, ..
            } => Some(cooking_time.unwrap_or_else(|| kind.default_cooking_time())),
            _ => None,
        }
    }
}

#[cfg(test)]
fn optional_ingredient(ingredient: &IngredientSpec) -> Option<IngredientSpec> {
    (!ingredient.is_empty()).then(|| ingredient.clone())
}

fn transmute_matches(
    input: &IngredientSpec,
    material: &IngredientSpec,
    min_material_count: u32,
    max_material_count: u32,
    result: &ItemAmount,
    add_material_count_to_result: bool,
    items: &[Option<&'static str>],
) -> bool {
    let ingredient_count = items.iter().filter(|item| item.is_some()).count() as u32;
    if ingredient_count < min_material_count + 1 || ingredient_count > max_material_count + 1 {
        return false;
    }

    let mut found_input = None;
    let mut material_count = 0;
    for item in items.iter().flatten() {
        if input.matches(item) {
            if found_input.replace(*item).is_some() {
                return false;
            }
        } else if material.matches(item) {
            material_count += 1;
            if material_count > max_material_count {
                return false;
            }
        } else {
            return false;
        }
    }

    let Some(input_item) = found_input else {
        return false;
    };
    if material_count < min_material_count || material_count > max_material_count {
        return false;
    }

    let result_count =
        transmute_result_count(result.count, material_count, add_material_count_to_result);
    result_count != 1 || input_item != result.item
}

fn imbue_matches(
    source: &IngredientSpec,
    material: &IngredientSpec,
    grid_width: usize,
    grid_height: usize,
    items: &[Option<&'static str>],
) -> bool {
    if grid_width != 3 || grid_height != 3 || items.len() != 9 || items.iter().any(Option::is_none)
    {
        return false;
    }

    items.iter().enumerate().all(|(index, item)| {
        let Some(item) = item else {
            return false;
        };
        if index == 4 {
            source.matches(item)
        } else {
            material.matches(item)
        }
    })
}

#[cfg(test)]
pub fn transmute_result(
    result: &ItemAmount,
    input: &ComponentCraftingStack,
    material_count: u32,
    add_material_count_to_result: bool,
) -> ComponentCraftingStack {
    let mut output = input.clone();
    output.item = result.item;
    output.count =
        transmute_result_count(result.count, material_count, add_material_count_to_result);
    output
}

#[cfg(test)]
pub fn imbue_result(
    result: &ItemAmount,
    source: &ComponentCraftingStack,
) -> ComponentCraftingStack {
    ComponentCraftingStack {
        item: result.item,
        count: result.count,
        potion_contents: source.potion_contents,
        custom_name: None,
        ..ComponentCraftingStack::one(result.item)
    }
}

fn transmute_result_count(
    base_count: u32,
    material_count: u32,
    add_material_count_to_result: bool,
) -> u32 {
    if add_material_count_to_result {
        base_count + material_count
    } else {
        base_count
    }
}

#[cfg(test)]
pub fn banner_duplicate_result(
    result_item: &'static str,
    input: &[Option<ComponentCraftingStack>],
) -> Option<(ComponentCraftingStack, Vec<Option<ComponentCraftingStack>>)> {
    let present = present_stacks(input);
    if present.len() != 2 {
        return None;
    }

    let mut source = None;
    let mut target = None;
    let mut color = None;
    for stack in present {
        if !is_banner(stack.item) || stack.banner_patterns > 6 {
            return None;
        }
        match color {
            Some(expected) if stack.banner_color != Some(expected) => return None,
            None => color = stack.banner_color,
            _ => {}
        }
        if stack.banner_patterns > 0 {
            if source.replace(stack).is_some() {
                return None;
            }
        } else if target.replace(stack).is_some() {
            return None;
        }
    }

    let source = source?;
    target?;
    let mut result = source.clone();
    result.item = result_item;
    result.count = 1;
    let remaining = input
        .iter()
        .map(|stack| {
            let stack = stack.as_ref()?;
            (stack.banner_patterns > 0).then(|| {
                let mut copy = stack.clone();
                copy.count = 1;
                copy
            })
        })
        .collect();
    Some((result, remaining))
}

#[cfg(test)]
pub fn book_cloning_result(
    result_item: &'static str,
    input: &[Option<ComponentCraftingStack>],
    min_generation: u8,
    max_generation: u8,
) -> Option<(ComponentCraftingStack, Vec<Option<ComponentCraftingStack>>)> {
    let mut source = None;
    let mut material_count = 0;
    for stack in present_stacks(input) {
        if stack.item == "minecraft:written_book" {
            let generation = stack.written_book_generation?;
            if generation < min_generation
                || generation > max_generation
                || source.replace(stack).is_some()
            {
                return None;
            }
        } else if stack.item == "minecraft:writable_book" {
            material_count += 1;
        } else {
            return None;
        }
    }

    let source = source?;
    if material_count == 0 {
        return None;
    }
    let mut result = source.clone();
    result.item = result_item;
    result.count = material_count;
    result.written_book_generation = source
        .written_book_generation
        .map(|generation| generation + 1);
    let mut returned = false;
    let remaining = input
        .iter()
        .map(|stack| {
            let stack = stack.as_ref()?;
            if !returned && stack.written_book_generation.is_some() {
                returned = true;
                let mut copy = stack.clone();
                copy.count = 1;
                Some(copy)
            } else {
                None
            }
        })
        .collect();
    Some((result, remaining))
}

#[cfg(test)]
pub fn decorated_pot_result(
    input: &[Option<ComponentCraftingStack>],
) -> Option<ComponentCraftingStack> {
    if input.len() != 9 || present_stacks(input).len() != 4 {
        return None;
    }
    let back = input[1].as_ref()?;
    let left = input[3].as_ref()?;
    let right = input[5].as_ref()?;
    let front = input[7].as_ref()?;
    if [back, left, right, front]
        .iter()
        .any(|stack| !is_pot_ingredient(stack.item))
    {
        return None;
    }
    let mut result = ComponentCraftingStack::one("minecraft:decorated_pot");
    result.pot_decorations = Some(PotDecorationsModel {
        back: back.item,
        left: left.item,
        right: right.item,
        front: front.item,
    });
    Some(result)
}

#[cfg(test)]
pub fn dye_result(
    result_item: &'static str,
    input: &[Option<ComponentCraftingStack>],
) -> Option<ComponentCraftingStack> {
    if present_stacks(input).len() < 2 {
        return None;
    }
    let mut target = None;
    let mut dyes = Vec::new();
    for stack in present_stacks(input) {
        if let Some(dye) = stack.dye_color {
            dyes.push(dye);
        } else if target.replace(stack).is_some() {
            return None;
        }
    }
    let target = target?;
    if dyes.is_empty() {
        return None;
    }
    let mut result = transmute_result(&ItemAmount::one(result_item), target, 0, false);
    result.dyed_color = Some(blend_dyes(target.dyed_color, &dyes));
    Some(result)
}

#[cfg(test)]
pub fn firework_rocket_result(
    result_item: &'static str,
    result_count: u32,
    input: &[Option<ComponentCraftingStack>],
) -> Option<ComponentCraftingStack> {
    if present_stacks(input).len() < 2 {
        return None;
    }
    let mut shell = false;
    let mut fuel_count = 0;
    let mut explosions = Vec::new();
    for stack in present_stacks(input) {
        if stack.item == "minecraft:paper" {
            if shell {
                return None;
            }
            shell = true;
        } else if stack.item == "minecraft:gunpowder" {
            fuel_count += 1;
            if fuel_count > 3 {
                return None;
            }
        } else if stack.item == "minecraft:firework_star" {
            if let Some(explosion) = &stack.firework_explosion {
                explosions.push(explosion.clone());
            }
        } else {
            return None;
        }
    }
    if !shell || fuel_count == 0 {
        return None;
    }
    let mut result = ComponentCraftingStack::one(result_item);
    result.count = result_count;
    result.fireworks = Some(FireworksModel {
        flight_duration: fuel_count,
        explosions,
    });
    Some(result)
}

#[cfg(test)]
pub fn firework_star_result(
    input: &[Option<ComponentCraftingStack>],
) -> Option<ComponentCraftingStack> {
    if present_stacks(input).len() < 2 {
        return None;
    }
    let mut fuel = false;
    let mut colors = Vec::new();
    let mut shape = "small_ball";
    let mut trail = false;
    let mut twinkle = false;
    for stack in present_stacks(input) {
        match stack.item {
            "minecraft:gunpowder" if !fuel => fuel = true,
            "minecraft:diamond" if !trail => trail = true,
            "minecraft:glowstone_dust" if !twinkle => twinkle = true,
            "minecraft:fire_charge" if shape == "small_ball" => shape = "large_ball",
            "minecraft:feather" if shape == "small_ball" => shape = "burst",
            "minecraft:gold_nugget" if shape == "small_ball" => shape = "star",
            _ if stack.dye_color.is_some() => colors.push(stack.dye_color.unwrap_or(0xFFFFFF)),
            _ => return None,
        }
    }
    if !fuel || colors.is_empty() {
        return None;
    }
    let mut result = ComponentCraftingStack::one("minecraft:firework_star");
    result.firework_explosion = Some(FireworkExplosionModel {
        shape,
        colors,
        fade_colors: Vec::new(),
        trail,
        twinkle,
    });
    Some(result)
}

#[cfg(test)]
pub fn firework_star_fade_result(
    input: &[Option<ComponentCraftingStack>],
) -> Option<ComponentCraftingStack> {
    if present_stacks(input).len() < 2 {
        return None;
    }
    let mut target = None;
    let mut fade_colors = Vec::new();
    for stack in present_stacks(input) {
        if let Some(dye) = stack.dye_color {
            fade_colors.push(dye);
        } else if stack.item == "minecraft:firework_star" && target.replace(stack).is_none() {
        } else {
            return None;
        }
    }
    let target = target?;
    if fade_colors.is_empty() {
        return None;
    }
    let mut result = transmute_result(
        &ItemAmount::one("minecraft:firework_star"),
        target,
        0,
        false,
    );
    let mut explosion = result
        .firework_explosion
        .clone()
        .unwrap_or(FireworkExplosionModel {
            shape: "small_ball",
            colors: Vec::new(),
            fade_colors: Vec::new(),
            trail: false,
            twinkle: false,
        });
    explosion.fade_colors = fade_colors;
    result.firework_explosion = Some(explosion);
    Some(result)
}

#[cfg(test)]
pub fn map_extending_result(
    input: &[Option<ComponentCraftingStack>],
) -> Option<ComponentCraftingStack> {
    if input.len() != 9 {
        return None;
    }
    for (index, stack) in input.iter().enumerate() {
        let stack = stack.as_ref()?;
        if index == 4 {
            if stack.item != "minecraft:filled_map"
                || stack.exploration_map
                || stack.map_scale.is_none_or(|scale| scale >= 4)
            {
                return None;
            }
        } else if stack.item != "minecraft:paper" {
            return None;
        }
    }
    let mut result = transmute_result(
        &ItemAmount::one("minecraft:filled_map"),
        input[4].as_ref()?,
        0,
        false,
    );
    result.map_post_processing_scale = true;
    Some(result)
}

#[cfg(test)]
pub fn repair_item_result(
    input: &[Option<ComponentCraftingStack>],
) -> Option<ComponentCraftingStack> {
    let present = present_stacks(input);
    let [first, second] = present.as_slice() else {
        return None;
    };
    if first.item != second.item || first.count != 1 || second.count != 1 {
        return None;
    }
    let durability = first.max_damage?.max(second.max_damage?);
    let remaining = (first.max_damage? - first.damage?)
        + (second.max_damage? - second.damage?)
        + durability * 5 / 100;
    let mut result = ComponentCraftingStack::one(first.item);
    result.max_damage = Some(durability);
    result.damage = Some(durability.saturating_sub(remaining));
    result.curses = merge_curses(&first.curses, &second.curses);
    Some(result)
}

#[cfg(test)]
pub fn shield_decoration_result(
    input: &[Option<ComponentCraftingStack>],
) -> Option<ComponentCraftingStack> {
    let present = present_stacks(input);
    if present.len() != 2 {
        return None;
    }
    let mut banner = None;
    let mut shield = None;
    for stack in present {
        if is_banner(stack.item) {
            if banner.replace(stack).is_some() {
                return None;
            }
        } else if stack.item == "minecraft:shield" && stack.banner_patterns == 0 {
            if shield.replace(stack).is_some() {
                return None;
            }
        } else {
            return None;
        }
    }
    let banner = banner?;
    let shield = shield?;
    let mut result = transmute_result(&ItemAmount::one("minecraft:shield"), shield, 0, false);
    result.banner_patterns = banner.banner_patterns;
    result.base_color = banner.banner_color;
    Some(result)
}

#[cfg(test)]
fn present_stacks(input: &[Option<ComponentCraftingStack>]) -> Vec<&ComponentCraftingStack> {
    input
        .iter()
        .flatten()
        .filter(|stack| stack.count > 0)
        .collect()
}

#[cfg(test)]
fn is_banner(item: &str) -> bool {
    item.ends_with("_banner")
}

#[cfg(test)]
fn is_pot_ingredient(item: &str) -> bool {
    item == "minecraft:brick" || item.ends_with("_pottery_sherd")
}

#[cfg(test)]
fn blend_dyes(current: Option<u32>, dyes: &[u32]) -> u32 {
    let mut colors = Vec::new();
    if let Some(current) = current {
        colors.push(current);
    }
    colors.extend_from_slice(dyes);
    let len = colors.len() as u32;
    let red = colors.iter().map(|color| (color >> 16) & 0xFF).sum::<u32>() / len;
    let green = colors.iter().map(|color| (color >> 8) & 0xFF).sum::<u32>() / len;
    let blue = colors.iter().map(|color| color & 0xFF).sum::<u32>() / len;
    (red << 16) | (green << 8) | blue
}

#[cfg(test)]
fn merge_curses(
    first: &[EnchantmentComponent],
    second: &[EnchantmentComponent],
) -> Vec<EnchantmentComponent> {
    let mut merged = first.to_vec();
    for enchantment in second {
        if let Some(existing) = merged
            .iter_mut()
            .find(|candidate| candidate.id == enchantment.id)
        {
            existing.level = existing.level.max(enchantment.level);
        } else {
            merged.push(enchantment.clone());
        }
    }
    merged
}

fn shaped_matches(
    recipe_width: usize,
    recipe_height: usize,
    pattern: &[Option<IngredientSpec>],
    grid_width: usize,
    grid_height: usize,
    items: &[Option<&'static str>],
) -> bool {
    let input = ShapedMatchInput {
        recipe_width,
        recipe_height,
        pattern,
        grid_width,
        grid_height,
        items,
    };
    if !input.has_valid_dimensions() {
        return false;
    }

    // `ShapedRecipePattern.matches` tries the pattern both as-authored and (for a
    // non-symmetrical pattern) horizontally mirrored. The offset scan emulates
    // `CraftingInput`'s bounding-box crop. Mirroring a symmetrical pattern just
    // re-tries the identical layout, so always trying both is equivalent to Java's
    // `symmetrical` short-circuit without needing to precompute that flag.
    let mirrored = mirror_pattern(pattern, recipe_width, recipe_height);
    for candidate in [pattern, mirrored.as_slice()] {
        let input = ShapedMatchInput {
            pattern: candidate,
            ..input
        };
        for y_offset in 0..=(grid_height - recipe_height) {
            for x_offset in 0..=(grid_width - recipe_width) {
                if shaped_matches_at(&input, x_offset, y_offset) {
                    return true;
                }
            }
        }
    }

    false
}

/// Horizontally mirror a row-major `recipe_width × recipe_height` pattern.
fn mirror_pattern(
    pattern: &[Option<IngredientSpec>],
    recipe_width: usize,
    recipe_height: usize,
) -> Vec<Option<IngredientSpec>> {
    let mut mirrored = Vec::with_capacity(pattern.len());
    for y in 0..recipe_height {
        for x in 0..recipe_width {
            mirrored.push(pattern[y * recipe_width + (recipe_width - 1 - x)].clone());
        }
    }
    mirrored
}

#[derive(Clone, Copy)]
struct ShapedMatchInput<'a> {
    recipe_width: usize,
    recipe_height: usize,
    pattern: &'a [Option<IngredientSpec>],
    grid_width: usize,
    grid_height: usize,
    items: &'a [Option<&'static str>],
}

impl ShapedMatchInput<'_> {
    fn has_valid_dimensions(&self) -> bool {
        self.recipe_width != 0
            && self.recipe_height != 0
            && self.recipe_width <= self.grid_width
            && self.recipe_height <= self.grid_height
            && self.items.len() == self.grid_width * self.grid_height
            && self.pattern.len() == self.recipe_width * self.recipe_height
    }
}

fn shaped_matches_at(
    input: &ShapedMatchInput<'_>,
    x_offset: usize,
    y_offset: usize,
) -> bool {
    for y in 0..input.grid_height {
        for x in 0..input.grid_width {
            let grid_item = input.items[y * input.grid_width + x];
            let pattern_item = if x >= x_offset
                && x < x_offset + input.recipe_width
                && y >= y_offset
                && y < y_offset + input.recipe_height
            {
                input.pattern[(y - y_offset) * input.recipe_width + (x - x_offset)].as_ref()
            } else {
                None
            };

            match (pattern_item, grid_item) {
                (None, None) => {}
                (Some(ingredient), Some(item)) if ingredient.matches(item) => {}
                _ => return false,
            }
        }
    }

    true
}

fn shapeless_matches(ingredients: &[IngredientSpec], items: &[Option<&'static str>]) -> bool {
    let provided: Vec<&'static str> = items.iter().flatten().copied().collect();
    if provided.len() != ingredients.len() {
        return false;
    }
    // `ShapelessRecipe.matches` delegates to `StackedContents.canCraft`, a full
    // ingredient→item assignment. A greedy first-match is wrong when ingredients
    // overlap (a broad ingredient can claim the only item a narrow one needs), so
    // assign by backtracking instead.
    let mut used = vec![false; provided.len()];
    shapeless_assign(ingredients, &provided, &mut used, 0)
}

/// Backtracking bipartite match: can every ingredient from `index` on be paired
/// with a distinct, as-yet-unused provided item?
fn shapeless_assign(
    ingredients: &[IngredientSpec],
    provided: &[&'static str],
    used: &mut [bool],
    index: usize,
) -> bool {
    let Some(ingredient) = ingredients.get(index) else {
        return true;
    };
    for (item_index, item) in provided.iter().enumerate() {
        if !used[item_index] && ingredient.matches(item) {
            used[item_index] = true;
            if shapeless_assign(ingredients, provided, used, index + 1) {
                return true;
            }
            used[item_index] = false;
        }
    }
    false
}
