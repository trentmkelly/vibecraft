//! Live, stack-aware implementations of the vanilla `CustomRecipe` special
//! crafting recipes (`net.minecraft.world.item.crafting.*Recipe`).
//!
//! Unlike the ordinary `RecipeKind::matches`/`assemble` path — which only sees
//! item ids — these operate on the real 3×3 grid of [`ItemStack`]s so the result
//! carries the correct data components (durability, enchantments, banner
//! patterns, …). Each function mirrors its Java recipe's `matches` + `assemble`
//! (+ `getRemainingItems`) exactly. [`special_crafting_result`] dispatches by the
//! loaded recipe's [`SpecialRecipeKind`]; [`RecipeMap::special_crafting_result`]
//! finds the first registered special recipe that matches the current grid.

use std::collections::BTreeMap;

use super::recipe_results::SpecialRecipeKind;
use super::ItemAmount;
use crate::item_properties::ItemComponent;
use crate::item_stack::ItemStack;

/// The outcome of a successful special craft: the result stack plus the grid's
/// state after one craft is taken. `grid_after[i]` is what input slot `i` becomes
/// once the result is removed — usually empty, but e.g. the patterned banner or
/// written book is left behind by the duplicate/clone recipes.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecialCraftOutcome {
    pub result: ItemStack,
    pub grid_after: Vec<ItemStack>,
}

/// Evaluate one special recipe of the given `kind` against the live grid.
///
/// `result_hint` is the recipe's configured `result` template (item id + count),
/// captured from the recipe JSON — e.g. the colour-specific banner for a banner
/// duplicate, or `firework_rocket × 3`. It is `None` for the recipes whose result
/// is fully dynamic (repair, map cloning).
pub fn special_crafting_result(
    kind: SpecialRecipeKind,
    result_hint: Option<&ItemAmount>,
    grid: &[ItemStack],
) -> Option<SpecialCraftOutcome> {
    match kind {
        SpecialRecipeKind::RepairItem => repair_item(grid),
        SpecialRecipeKind::ShieldDecoration => shield_decoration(result_hint?, grid),
        SpecialRecipeKind::BannerDuplicate => banner_duplicate(result_hint?, grid),
        SpecialRecipeKind::BookCloning => book_cloning(result_hint?, grid),
        SpecialRecipeKind::DecoratedPot => decorated_pot(result_hint?, grid),
        SpecialRecipeKind::DyedItem => dyed_item(result_hint?, grid),
        SpecialRecipeKind::FireworkRocket => firework_rocket(result_hint?, grid),
        SpecialRecipeKind::FireworkStar => firework_star(result_hint?, grid),
        SpecialRecipeKind::FireworkStarFade => firework_star_fade(result_hint?, grid),
        // `MapCloning` is a `crafting_transmute` recipe in 26.1.2 (handled by the
        // ordinary transmute path), so the vestigial `SpecialRecipeKind::MapCloning`
        // is never produced from data.
        //
        // TODO(recipes-mapextending): `MapExtendingRecipe.matches` calls
        // `MapItem.getSavedData(map, level)` to read `data.scale < 4` and
        // `data.isExplorationMap()`, and `assemble` copies the source map's
        // components + `MAP_POST_PROCESSING = SCALE`. This is blocked on the map
        // subsystem: items carry no `minecraft:map_id` component and there is no
        // level-scoped `MapItemSavedData` store reachable from a crafting input, so
        // the scale/exploration checks cannot be evaluated here yet. Implement once
        // `map_id` + map saved-data are exposed on `ItemStack`.
        _ => None,
    }
}

// ----------------------------------------------------------------------------
// Dye colours (DyeColor)
// ----------------------------------------------------------------------------

/// The `texture_diffuse` and `firework` RGB of a `<colour>_dye` `DyeItem`, matching
/// the `DyeColor` table. `texture_diffuse` drives `DyedItemColor.applyDyes`;
/// `firework` drives `DyeColor.getFireworkColor()`. `None` for non-dye items.
fn dye_colors(item_id: &str) -> Option<(u32, u32)> {
    let name = item_id.strip_prefix("minecraft:")?.strip_suffix("_dye")?;
    Some(match name {
        "white" => (16383998, 15790320),
        "orange" => (16351261, 15435844),
        "magenta" => (13061821, 12801229),
        "light_blue" => (3847130, 6719955),
        "yellow" => (16701501, 14602026),
        "lime" => (8439583, 4312372),
        "pink" => (15961002, 14188952),
        "gray" => (4673362, 4408131),
        "light_gray" => (10329495, 11250603),
        "cyan" => (1481884, 2651799),
        "purple" => (8991416, 8073150),
        "blue" => (3949738, 2437522),
        "brown" => (8606770, 5320730),
        "green" => (6192150, 3887386),
        "red" => (11546150, 11743532),
        "black" => (1908001, 1973019),
        _ => return None,
    })
}

/// `DyedItemColor.applyDyes` — the leather-armour colour blend: the intensity-scaled
/// average of the current colour (if any) and the applied dyes' texture colours.
fn apply_dyes(current: Option<u32>, dye_textures: &[u32]) -> u32 {
    let (mut red_total, mut green_total, mut blue_total, mut intensity_total, mut count) =
        (0u32, 0u32, 0u32, 0u32, 0u32);
    let mut accumulate = |rgb: u32| {
        let (red, green, blue) = ((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF);
        intensity_total += red.max(green).max(blue);
        red_total += red;
        green_total += green;
        blue_total += blue;
        count += 1;
    };
    if let Some(rgb) = current {
        accumulate(rgb);
    }
    for rgb in dye_textures {
        accumulate(*rgb);
    }
    if count == 0 {
        return 0;
    }
    let (mut red, mut green, mut blue) =
        (red_total / count, green_total / count, blue_total / count);
    let average_intensity = intensity_total as f32 / count as f32;
    let result_intensity = red.max(green).max(blue) as f32;
    if result_intensity > 0.0 {
        red = (red as f32 * average_intensity / result_intensity) as u32;
        green = (green as f32 * average_intensity / result_intensity) as u32;
        blue = (blue as f32 * average_intensity / result_intensity) as u32;
    }
    (red << 16) | (green << 8) | blue
}

// ----------------------------------------------------------------------------
// DyeRecipe
// ----------------------------------------------------------------------------

/// `DyeRecipe` — a dyeable item (leather/wolf armour) plus one or more dyes blends
/// the dyes (and any existing colour) into the item's `dyed_color` component.
fn dyed_item(result_hint: &ItemAmount, grid: &[ItemStack]) -> Option<SpecialCraftOutcome> {
    let present = present_indexed(grid);
    if present.len() < 2 {
        return None;
    }
    let mut target = None;
    let mut dye_textures = Vec::new();
    for (_, stack) in &present {
        if let Some((texture, _)) = dye_colors(stack.item_id()) {
            dye_textures.push(texture);
        } else if stack.item_id() == result_hint.item {
            // The dyeable target must be the recipe's configured item, and unique.
            if target.replace(*stack).is_some() {
                return None;
            }
        } else {
            return None;
        }
    }
    let target = target?;
    if dye_textures.is_empty() {
        return None;
    }

    let current = match target.component("minecraft:dyed_color") {
        Some(ItemComponent::DyedColor(rgb)) => Some(*rgb),
        _ => None,
    };
    let mut result = target.transmute_copy(result_hint.item, 1);
    result.set_component(ItemComponent::DyedColor(apply_dyes(current, &dye_textures)));
    Some(SpecialCraftOutcome {
        result,
        grid_after: vec![ItemStack::empty(); grid.len()],
    })
}

// ----------------------------------------------------------------------------
// Firework recipes
// ----------------------------------------------------------------------------

/// The `firework_explosion` component carried by a firework star, if any.
fn firework_explosion(stack: &ItemStack) -> Option<crate::item_properties::FireworkExplosion> {
    match stack.component("minecraft:firework_explosion") {
        Some(ItemComponent::FireworkExplosion(explosion)) => Some(explosion.clone()),
        _ => None,
    }
}

/// `FireworkStarRecipe.findShape` — the burst shape contributed by a shape item.
fn firework_shape(item_id: &str) -> Option<&'static str> {
    match item_id {
        "minecraft:feather" => Some("burst"),
        "minecraft:fire_charge" => Some("large_ball"),
        "minecraft:gold_nugget" => Some("star"),
        // `#minecraft:skulls`.
        id if id.ends_with("_skull") || id.ends_with("_head") => Some("creeper"),
        _ => None,
    }
}

/// `FireworkRocketRecipe` — paper + 1–3 gunpowder + optional firework stars yields a
/// rocket whose flight duration is the gunpowder count and which carries the stars'
/// explosions.
fn firework_rocket(result_hint: &ItemAmount, grid: &[ItemStack]) -> Option<SpecialCraftOutcome> {
    let present = present_indexed(grid);
    if present.len() < 2 {
        return None;
    }
    let mut shell = false;
    let mut fuel_count: u8 = 0;
    let mut explosions = Vec::new();
    for (_, stack) in &present {
        match stack.item_id() {
            "minecraft:paper" => {
                if shell {
                    return None;
                }
                shell = true;
            }
            "minecraft:gunpowder" => {
                fuel_count += 1;
                if fuel_count > 3 {
                    return None;
                }
            }
            "minecraft:firework_star" => {
                if let Some(explosion) = firework_explosion(stack) {
                    explosions.push(explosion);
                }
            }
            _ => return None,
        }
    }
    if !shell || fuel_count == 0 {
        return None;
    }
    let mut result = ItemStack::new(result_hint.item, result_hint.count as i32);
    result.set_component(ItemComponent::Fireworks {
        flight_duration: fuel_count,
        explosions,
    });
    Some(SpecialCraftOutcome {
        result,
        grid_after: vec![ItemStack::empty(); grid.len()],
    })
}

/// `FireworkStarRecipe` — gunpowder + ≥1 dye + optional shape/trail/twinkle modifiers
/// yields a firework star carrying the assembled `firework_explosion`.
fn firework_star(result_hint: &ItemAmount, grid: &[ItemStack]) -> Option<SpecialCraftOutcome> {
    let present = present_indexed(grid);
    if present.len() < 2 {
        return None;
    }
    let mut fuel = false;
    let mut shape = "small_ball";
    let mut has_shape = false;
    let mut trail = false;
    let mut twinkle = false;
    let mut colors = Vec::new();
    for (_, stack) in &present {
        let id = stack.item_id();
        if id == "minecraft:glowstone_dust" {
            if twinkle {
                return None;
            }
            twinkle = true;
        } else if id == "minecraft:diamond" {
            if trail {
                return None;
            }
            trail = true;
        } else if id == "minecraft:gunpowder" {
            if fuel {
                return None;
            }
            fuel = true;
        } else if let Some((_, firework)) = dye_colors(id) {
            colors.push(firework);
        } else if let Some(found_shape) = firework_shape(id) {
            if has_shape {
                return None;
            }
            has_shape = true;
            shape = found_shape;
        } else {
            return None;
        }
    }
    if !fuel || colors.is_empty() {
        return None;
    }
    let mut result = ItemStack::new(result_hint.item, result_hint.count as i32);
    result.set_component(ItemComponent::FireworkExplosion(
        crate::item_properties::FireworkExplosion {
            shape,
            colors,
            fade_colors: Vec::new(),
            trail,
            twinkle,
        },
    ));
    Some(SpecialCraftOutcome {
        result,
        grid_after: vec![ItemStack::empty(); grid.len()],
    })
}

/// `FireworkStarFadeRecipe` — a firework star + ≥1 dye stamps the dyes' colours as the
/// star's fade colours.
fn firework_star_fade(
    result_hint: &ItemAmount,
    grid: &[ItemStack],
) -> Option<SpecialCraftOutcome> {
    let present = present_indexed(grid);
    if present.len() < 2 {
        return None;
    }
    let mut target = None;
    let mut fade_colors = Vec::new();
    for (_, stack) in &present {
        if let Some((_, firework)) = dye_colors(stack.item_id()) {
            fade_colors.push(firework);
        } else if stack.item_id() == "minecraft:firework_star" {
            if target.replace(*stack).is_some() {
                return None;
            }
        } else {
            return None;
        }
    }
    let target = target?;
    if fade_colors.is_empty() {
        return None;
    }
    // createWithOriginalComponents(result, target).update(FIREWORK_EXPLOSION, …).
    let mut result = target.transmute_copy(result_hint.item, 1);
    let mut explosion = firework_explosion(&result).unwrap_or(crate::item_properties::FireworkExplosion {
        shape: "small_ball",
        colors: Vec::new(),
        fade_colors: Vec::new(),
        trail: false,
        twinkle: false,
    });
    explosion.fade_colors = fade_colors;
    result.set_component(ItemComponent::FireworkExplosion(explosion));
    Some(SpecialCraftOutcome {
        result,
        grid_after: vec![ItemStack::empty(); grid.len()],
    })
}

/// The non-empty grid stacks paired with their slot index.
fn present_indexed(grid: &[ItemStack]) -> Vec<(usize, &ItemStack)> {
    grid.iter()
        .enumerate()
        .filter(|(_, stack)| !stack.is_empty())
        .collect()
}

/// Whether an item id is one of the 16 `<colour>_banner` `BannerItem`s.
fn is_banner(item_id: &str) -> bool {
    banner_base_color(item_id).is_some()
}

/// `BannerItem.getColor()` — the intrinsic dye colour of a `<colour>_banner`, as the
/// `DyeColor` id (e.g. `"red"`). `None` for non-banner items.
fn banner_base_color(item_id: &str) -> Option<&'static str> {
    const COLORS: &[&str] = &[
        "white",
        "orange",
        "magenta",
        "light_blue",
        "yellow",
        "lime",
        "pink",
        "gray",
        "light_gray",
        "cyan",
        "purple",
        "blue",
        "brown",
        "green",
        "red",
        "black",
    ];
    let name = item_id.strip_prefix("minecraft:")?.strip_suffix("_banner")?;
    COLORS.iter().copied().find(|color| *color == name)
}

/// The `banner_patterns` layers carried by a stack, if any.
fn banner_patterns(stack: &ItemStack) -> Option<Vec<crate::block_entity::BannerPatternLayer>> {
    match stack.component("minecraft:banner_patterns") {
        Some(ItemComponent::BannerPatterns(layers)) => Some(layers.clone()),
        _ => None,
    }
}

/// The number of `banner_patterns` layers on a stack (0 if the component is absent).
fn banner_pattern_count(stack: &ItemStack) -> usize {
    banner_patterns(stack).map_or(0, |layers| layers.len())
}

// ----------------------------------------------------------------------------
// ShieldDecorationRecipe
// ----------------------------------------------------------------------------

/// `ShieldDecorationRecipe` — a banner + a pattern-free shield yields a shield
/// carrying the banner's pattern layers and base colour.
fn shield_decoration(
    result_hint: &ItemAmount,
    grid: &[ItemStack],
) -> Option<SpecialCraftOutcome> {
    let present = present_indexed(grid);
    if present.len() != 2 {
        return None;
    }
    let mut banner = None;
    let mut target = None;
    for (_, stack) in &present {
        if is_banner(stack.item_id()) {
            if banner.replace(*stack).is_some() {
                return None;
            }
        } else if stack.item_id() == "minecraft:shield" && banner_pattern_count(stack) == 0 {
            if target.replace(*stack).is_some() {
                return None;
            }
        } else {
            return None;
        }
    }
    let banner = banner?;
    let target = target?;

    // createWithOriginalComponents(result, target) + BANNER_PATTERNS + BASE_COLOR.
    let mut result = target.transmute_copy(result_hint.item, 1);
    match banner_patterns(banner) {
        Some(layers) => {
            result.set_component(ItemComponent::BannerPatterns(layers));
        }
        // A pattern-free banner clears any patterns and contributes only the colour.
        None => {
            result.remove_component("minecraft:banner_patterns");
        }
    }
    if let Some(color) = banner_base_color(banner.item_id()) {
        result.set_component(ItemComponent::BaseColor(color));
    }
    Some(SpecialCraftOutcome {
        result,
        grid_after: vec![ItemStack::empty(); grid.len()],
    })
}

// ----------------------------------------------------------------------------
// BannerDuplicateRecipe
// ----------------------------------------------------------------------------

/// `BannerDuplicateRecipe` — a patterned banner + a same-colour blank banner copies
/// the patterns onto the blank one; the patterned source banner is left behind.
fn banner_duplicate(
    result_hint: &ItemAmount,
    grid: &[ItemStack],
) -> Option<SpecialCraftOutcome> {
    // This recipe is colour-specific (one per dye), so the inputs must be the same
    // colour as the configured result banner.
    let recipe_color = banner_base_color(result_hint.item)?;
    let present = present_indexed(grid);
    if present.len() != 2 {
        return None;
    }
    let mut source = None;
    let mut target = false;
    for (_, stack) in &present {
        if banner_base_color(stack.item_id()) != Some(recipe_color) {
            return None;
        }
        let count = banner_pattern_count(stack);
        if count > 6 {
            return None;
        }
        if count > 0 {
            if source.replace(*stack).is_some() {
                return None;
            }
        } else if std::mem::replace(&mut target, true) {
            return None;
        }
    }
    let source = source?;
    if !target {
        return None;
    }

    let result = source.transmute_copy(result_hint.item, 1);
    // getRemainingItems: the patterned source banner stays (copyWithCount 1), the
    // blank one is consumed.
    let grid_after = grid
        .iter()
        .map(|stack| {
            if banner_pattern_count(stack) > 0 {
                stack.copy_with_count(1)
            } else {
                ItemStack::empty()
            }
        })
        .collect();
    Some(SpecialCraftOutcome { result, grid_after })
}

// ----------------------------------------------------------------------------
// RepairItemRecipe
// ----------------------------------------------------------------------------

/// `RepairItemRecipe` — combine two damaged items of the same type into one with
/// the summed remaining durability plus a 5%-of-max bonus, carrying over only the
/// items' curse enchantments (at the higher of the two levels).
fn repair_item(grid: &[ItemStack]) -> Option<SpecialCraftOutcome> {
    // `getItemsToCombine`: exactly two non-empty inputs that `canCombine`.
    let present: Vec<(usize, &ItemStack)> = grid
        .iter()
        .enumerate()
        .filter(|(_, stack)| !stack.is_empty())
        .collect();
    let [(first_idx, first), (second_idx, second)] = present.as_slice() else {
        return None;
    };
    if !repair_can_combine(first, second) {
        return None;
    }

    // `assemble`.
    let durability = first.max_damage().max(second.max_damage());
    let remaining = (first.max_damage() - first.damage_value())
        + (second.max_damage() - second.damage_value())
        + durability * 5 / 100;
    let mut result = ItemStack::new(first.item_id(), 1);
    result.set_component(ItemComponent::MaxDamage(durability));
    result.set_damage_value(durability.saturating_sub(remaining));

    // Only `EnchantmentTags.CURSE` enchantments survive (max level of the two).
    let mut curses: BTreeMap<String, i32> = BTreeMap::new();
    for stack in [first, second] {
        for (id, level) in crafting_enchantments(stack) {
            if crate::enchantment_system::is_curse(&id) {
                let entry = curses.entry(id).or_insert(0);
                *entry = (*entry).max(level);
            }
        }
    }
    if !curses.is_empty() {
        result.set_component(ItemComponent::Enchantments(curses));
    }

    // Both single-count inputs are consumed.
    let mut grid_after = grid.to_vec();
    grid_after[*first_idx] = ItemStack::empty();
    grid_after[*second_idx] = ItemStack::empty();
    Some(SpecialCraftOutcome { result, grid_after })
}

/// `RepairItemRecipe.canCombine`: same item, both single, both damageable.
fn repair_can_combine(first: &ItemStack, second: &ItemStack) -> bool {
    second.item_id() == first.item_id()
        && first.count() == 1
        && second.count() == 1
        && first.component("minecraft:max_damage").is_some()
        && second.component("minecraft:max_damage").is_some()
        && first.component("minecraft:damage").is_some()
        && second.component("minecraft:damage").is_some()
}

/// `EnchantmentHelper.getEnchantmentsForCrafting`: the enchantments used for
/// crafting carry-over (the `enchantments` component for ordinary items).
fn crafting_enchantments(stack: &ItemStack) -> BTreeMap<String, i32> {
    match stack.component("minecraft:enchantments") {
        Some(ItemComponent::Enchantments(map)) => map.clone(),
        _ => BTreeMap::new(),
    }
}

// ----------------------------------------------------------------------------
// BookCloningRecipe
// ----------------------------------------------------------------------------

/// `BookCloningRecipe` — a written book (generation 0 or 1) plus one or more
/// writable books yields that many copies at generation + 1; the source written
/// book is left behind.
fn book_cloning(result_hint: &ItemAmount, grid: &[ItemStack]) -> Option<SpecialCraftOutcome> {
    let present = present_indexed(grid);
    if present.len() < 2 {
        return None;
    }
    let mut source = None;
    let mut material_count = 0i32;
    for (_, stack) in &present {
        if stack.item_id() == "minecraft:written_book" {
            // `allowed_generations` defaults to 0..=1.
            let generation = written_book_generation(stack)?;
            if !(0..=1).contains(&generation) || source.replace(*stack).is_some() {
                return None;
            }
        } else if stack.item_id() == "minecraft:writable_book" {
            material_count += 1;
        } else {
            return None;
        }
    }
    let source = source?;
    if material_count == 0 {
        return None;
    }

    // createWithOriginalComponents(result, source, count - 1) -> count copies, then
    // WRITTEN_BOOK_CONTENT = source.craftCopy() (generation + 1).
    let mut result = source.transmute_copy(result_hint.item, material_count);
    bump_written_book_generation(&mut result);

    // getRemainingItems: the (first) written book stays at count 1; the writable
    // books are consumed.
    let mut kept = false;
    let grid_after = grid
        .iter()
        .map(|stack| {
            if !kept && stack.component("minecraft:written_book_content").is_some() {
                kept = true;
                stack.copy_with_count(1)
            } else {
                ItemStack::empty()
            }
        })
        .collect();
    Some(SpecialCraftOutcome { result, grid_after })
}

/// The `generation` of a stack's `written_book_content`, if present.
fn written_book_generation(stack: &ItemStack) -> Option<i32> {
    match stack.component("minecraft:written_book_content") {
        Some(ItemComponent::WrittenBookContent { generation, .. }) => Some(*generation),
        _ => None,
    }
}

/// `WrittenBookContent.craftCopy`: increment the generation by one.
fn bump_written_book_generation(stack: &mut ItemStack) {
    if let Some(ItemComponent::WrittenBookContent {
        title,
        author,
        generation,
        pages,
        resolved,
    }) = stack.component("minecraft:written_book_content")
    {
        let copy = ItemComponent::WrittenBookContent {
            title: title.clone(),
            author: author.clone(),
            generation: generation + 1,
            pages: pages.clone(),
            resolved: *resolved,
        };
        stack.set_component(copy);
    }
}

// ----------------------------------------------------------------------------
// DecoratedPotRecipe
// ----------------------------------------------------------------------------

/// `DecoratedPotRecipe` — four pottery sherds/bricks in the cardinal slots of the
/// 3×3 grid form a decorated pot whose four faces are the placed ingredients.
fn decorated_pot(result_hint: &ItemAmount, grid: &[ItemStack]) -> Option<SpecialCraftOutcome> {
    if grid.len() != 9 {
        return None;
    }
    // The four cardinal slots: back=(1,0), left=(0,1), right=(2,1), front=(1,2).
    let present = present_indexed(grid);
    if present.len() != 4 || present.iter().any(|(index, _)| !matches!(index, 1 | 3 | 5 | 7)) {
        return None;
    }
    let face = |index: usize| -> Option<&'static str> {
        let stack = &grid[index];
        (!stack.is_empty() && is_pot_ingredient(stack.item_id())).then(|| stack.item_id())
    };
    let decorations = ItemComponent::PotDecorations {
        back: face(1)?,
        left: face(3)?,
        right: face(5)?,
        front: face(7)?,
    };
    let mut result = ItemStack::new(result_hint.item, result_hint.count as i32);
    result.set_component(decorations);
    Some(SpecialCraftOutcome {
        result,
        grid_after: vec![ItemStack::empty(); grid.len()],
    })
}

/// `#minecraft:decorated_pot_ingredients` — brick plus every pottery sherd.
fn is_pot_ingredient(item_id: &str) -> bool {
    item_id == "minecraft:brick" || item_id.ends_with("_pottery_sherd")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn damaged(item: &'static str, max: u32, damage: u32) -> ItemStack {
        let mut stack = ItemStack::new(item, 1);
        stack.set_component(ItemComponent::MaxDamage(max));
        stack.set_damage_value(damage);
        stack
    }

    fn grid_with(entries: &[(usize, ItemStack)]) -> Vec<ItemStack> {
        let mut grid = vec![ItemStack::empty(); 9];
        for (index, stack) in entries {
            grid[*index] = stack.clone();
        }
        grid
    }

    #[test]
    fn repair_item_sums_remaining_durability_with_five_percent_bonus() {
        // remaining = (100-80) + (100-90) + 100*5/100 = 20 + 10 + 5 = 35
        // result damage = max(100 - 35, 0) = 65   (matches RepairItemRecipe.assemble)
        let grid = grid_with(&[
            (0, damaged("minecraft:diamond_pickaxe", 100, 80)),
            (4, damaged("minecraft:diamond_pickaxe", 100, 90)),
        ]);
        let outcome = special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid)
            .expect("two damaged matching tools should repair");
        assert_eq!(outcome.result.item_id(), "minecraft:diamond_pickaxe");
        assert_eq!(outcome.result.count(), 1);
        assert_eq!(outcome.result.max_damage(), 100);
        assert_eq!(outcome.result.damage_value(), 65);
        // Both single-count inputs are consumed.
        assert!(outcome.grid_after[0].is_empty());
        assert!(outcome.grid_after[4].is_empty());
    }

    #[test]
    fn repair_item_carries_only_curse_enchantments() {
        let mut a = damaged("minecraft:diamond_pickaxe", 100, 80);
        a.set_component(ItemComponent::Enchantments(BTreeMap::from([
            ("minecraft:binding_curse".to_string(), 1),
            ("minecraft:efficiency".to_string(), 5),
        ])));
        let mut b = damaged("minecraft:diamond_pickaxe", 100, 90);
        b.set_component(ItemComponent::Enchantments(BTreeMap::from([(
            "minecraft:vanishing_curse".to_string(),
            1,
        )])));
        let grid = grid_with(&[(0, a), (1, b)]);
        let outcome = special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).unwrap();
        match outcome.result.component("minecraft:enchantments") {
            Some(ItemComponent::Enchantments(map)) => {
                assert_eq!(map.get("minecraft:binding_curse"), Some(&1));
                assert_eq!(map.get("minecraft:vanishing_curse"), Some(&1));
                assert_eq!(map.get("minecraft:efficiency"), None, "non-curse dropped");
            }
            other => panic!("expected curse enchantments, got {other:?}"),
        }
    }

    fn banner(item: &'static str, pattern_count: usize) -> ItemStack {
        use crate::block_entity::BannerPatternLayer;
        use crate::map_state::DyeColor;
        let mut stack = ItemStack::new(item, 1);
        if pattern_count > 0 {
            let layers = (0..pattern_count)
                .map(|_| BannerPatternLayer {
                    pattern: "minecraft:stripe_top".to_string(),
                    color: DyeColor::White,
                })
                .collect();
            stack.set_component(ItemComponent::BannerPatterns(layers));
        }
        stack
    }

    #[test]
    fn shield_decoration_copies_banner_patterns_and_base_color() {
        let hint = ItemAmount::one("minecraft:shield");
        let grid = grid_with(&[
            (0, banner("minecraft:red_banner", 3)),
            (1, ItemStack::new("minecraft:shield", 1)),
        ]);
        let outcome = special_crafting_result(
            SpecialRecipeKind::ShieldDecoration,
            Some(&hint),
            &grid,
        )
        .expect("banner + clear shield should decorate");
        assert_eq!(outcome.result.item_id(), "minecraft:shield");
        assert_eq!(
            outcome.result.component("minecraft:base_color"),
            Some(&ItemComponent::BaseColor("red"))
        );
        match outcome.result.component("minecraft:banner_patterns") {
            Some(ItemComponent::BannerPatterns(layers)) => assert_eq!(layers.len(), 3),
            other => panic!("expected 3 pattern layers, got {other:?}"),
        }
        // A shield already carrying patterns is rejected (not a clear target).
        let grid = grid_with(&[
            (0, banner("minecraft:red_banner", 3)),
            (1, banner("minecraft:shield", 1)),
        ]);
        assert!(
            special_crafting_result(SpecialRecipeKind::ShieldDecoration, Some(&hint), &grid)
                .is_none()
        );
    }

    #[test]
    fn banner_duplicate_copies_patterns_and_keeps_source() {
        let hint = ItemAmount::one("minecraft:red_banner");
        let grid = grid_with(&[
            (0, banner("minecraft:red_banner", 4)), // source
            (1, banner("minecraft:red_banner", 0)), // blank target
        ]);
        let outcome =
            special_crafting_result(SpecialRecipeKind::BannerDuplicate, Some(&hint), &grid)
                .expect("patterned + blank same-colour banner should duplicate");
        match outcome.result.component("minecraft:banner_patterns") {
            Some(ItemComponent::BannerPatterns(layers)) => assert_eq!(layers.len(), 4),
            other => panic!("expected copied patterns, got {other:?}"),
        }
        // The patterned source banner is left behind; the blank one is consumed.
        assert_eq!(banner_pattern_count(&outcome.grid_after[0]), 4);
        assert!(outcome.grid_after[1].is_empty());

        // A mismatched recipe colour does not match these inputs.
        let other_hint = ItemAmount::one("minecraft:blue_banner");
        assert!(special_crafting_result(
            SpecialRecipeKind::BannerDuplicate,
            Some(&other_hint),
            &grid
        )
        .is_none());
    }

    fn written_book(generation: i32) -> ItemStack {
        let mut stack = ItemStack::new("minecraft:written_book", 1);
        stack.set_component(ItemComponent::WrittenBookContent {
            title: "Tale".to_string(),
            author: "Steve".to_string(),
            generation,
            pages: vec!["page".to_string()],
            resolved: true,
        });
        stack
    }

    #[test]
    fn book_cloning_copies_at_next_generation_and_keeps_source() {
        let hint = ItemAmount::one("minecraft:written_book");
        let grid = grid_with(&[
            (0, written_book(0)),
            (1, ItemStack::new("minecraft:writable_book", 1)),
            (2, ItemStack::new("minecraft:writable_book", 1)),
        ]);
        let outcome = special_crafting_result(SpecialRecipeKind::BookCloning, Some(&hint), &grid)
            .expect("written book + 2 writable books should clone");
        assert_eq!(outcome.result.item_id(), "minecraft:written_book");
        assert_eq!(outcome.result.count(), 2, "one copy per writable book");
        assert_eq!(written_book_generation(&outcome.result), Some(1));
        // Source written book retained, writable books consumed.
        assert_eq!(outcome.grid_after[0].item_id(), "minecraft:written_book");
        assert!(outcome.grid_after[1].is_empty());
        assert!(outcome.grid_after[2].is_empty());

        // A generation-2 source is out of the allowed 0..=1 range.
        let grid = grid_with(&[
            (0, written_book(2)),
            (1, ItemStack::new("minecraft:writable_book", 1)),
        ]);
        assert!(
            special_crafting_result(SpecialRecipeKind::BookCloning, Some(&hint), &grid).is_none()
        );
    }

    #[test]
    fn decorated_pot_records_four_faces() {
        let hint = ItemAmount::one("minecraft:decorated_pot");
        let grid = grid_with(&[
            (1, ItemStack::new("minecraft:brick", 1)),
            (3, ItemStack::new("minecraft:angler_pottery_sherd", 1)),
            (5, ItemStack::new("minecraft:brick", 1)),
            (7, ItemStack::new("minecraft:skull_pottery_sherd", 1)),
        ]);
        let outcome = special_crafting_result(SpecialRecipeKind::DecoratedPot, Some(&hint), &grid)
            .expect("four cardinal pot ingredients should craft a decorated pot");
        assert_eq!(outcome.result.item_id(), "minecraft:decorated_pot");
        assert_eq!(
            outcome.result.component("minecraft:pot_decorations"),
            Some(&ItemComponent::PotDecorations {
                back: "minecraft:brick",
                left: "minecraft:angler_pottery_sherd",
                right: "minecraft:brick",
                front: "minecraft:skull_pottery_sherd",
            })
        );

        // An ingredient off the cardinal slots is rejected.
        let grid = grid_with(&[
            (0, ItemStack::new("minecraft:brick", 1)),
            (3, ItemStack::new("minecraft:brick", 1)),
            (5, ItemStack::new("minecraft:brick", 1)),
            (7, ItemStack::new("minecraft:brick", 1)),
        ]);
        assert!(
            special_crafting_result(SpecialRecipeKind::DecoratedPot, Some(&hint), &grid).is_none()
        );
    }

    #[test]
    fn dyed_item_blends_dye_into_dyed_color() {
        // A single red dye on an uncoloured leather helmet yields exactly the dye's
        // texture-diffuse colour (DyeColor.RED = 11546150).
        let hint = ItemAmount::one("minecraft:leather_helmet");
        let grid = grid_with(&[
            (0, ItemStack::new("minecraft:leather_helmet", 1)),
            (1, ItemStack::new("minecraft:red_dye", 1)),
        ]);
        let outcome = special_crafting_result(SpecialRecipeKind::DyedItem, Some(&hint), &grid)
            .expect("leather + dye should dye");
        assert_eq!(
            outcome.result.component("minecraft:dyed_color"),
            Some(&ItemComponent::DyedColor(11546150))
        );
    }

    #[test]
    fn firework_rocket_flight_duration_is_gunpowder_count() {
        let hint = ItemAmount {
            item: "minecraft:firework_rocket",
            count: 3,
        };
        let grid = grid_with(&[
            (0, ItemStack::new("minecraft:paper", 1)),
            (1, ItemStack::new("minecraft:gunpowder", 1)),
            (2, ItemStack::new("minecraft:gunpowder", 1)),
        ]);
        let outcome = special_crafting_result(SpecialRecipeKind::FireworkRocket, Some(&hint), &grid)
            .expect("paper + gunpowder should craft a rocket");
        assert_eq!(outcome.result.count(), 3);
        assert_eq!(
            outcome.result.component("minecraft:fireworks"),
            Some(&ItemComponent::Fireworks {
                flight_duration: 2,
                explosions: Vec::new(),
            })
        );
        // Four gunpowder exceeds the flight-duration cap.
        let grid = grid_with(&[
            (0, ItemStack::new("minecraft:paper", 1)),
            (1, ItemStack::new("minecraft:gunpowder", 1)),
            (2, ItemStack::new("minecraft:gunpowder", 1)),
            (3, ItemStack::new("minecraft:gunpowder", 1)),
            (4, ItemStack::new("minecraft:gunpowder", 1)),
        ]);
        assert!(
            special_crafting_result(SpecialRecipeKind::FireworkRocket, Some(&hint), &grid).is_none()
        );
    }

    #[test]
    fn firework_star_assembles_shape_color_and_modifiers() {
        use crate::item_properties::FireworkExplosion;
        let hint = ItemAmount::one("minecraft:firework_star");
        let grid = grid_with(&[
            (0, ItemStack::new("minecraft:gunpowder", 1)),
            (1, ItemStack::new("minecraft:red_dye", 1)),
            (2, ItemStack::new("minecraft:fire_charge", 1)), // large_ball
            (3, ItemStack::new("minecraft:glowstone_dust", 1)), // twinkle
        ]);
        let outcome = special_crafting_result(SpecialRecipeKind::FireworkStar, Some(&hint), &grid)
            .expect("gunpowder + dye should craft a star");
        assert_eq!(
            outcome.result.component("minecraft:firework_explosion"),
            Some(&ItemComponent::FireworkExplosion(FireworkExplosion {
                shape: "large_ball",
                colors: vec![11743532], // DyeColor.RED firework colour
                fade_colors: Vec::new(),
                trail: false,
                twinkle: true,
            }))
        );
    }

    #[test]
    fn firework_star_fade_adds_fade_colors() {
        use crate::item_properties::FireworkExplosion;
        let hint = ItemAmount::one("minecraft:firework_star");
        let mut star = ItemStack::new("minecraft:firework_star", 1);
        star.set_component(ItemComponent::FireworkExplosion(FireworkExplosion {
            shape: "small_ball",
            colors: vec![11546150],
            fade_colors: Vec::new(),
            trail: false,
            twinkle: false,
        }));
        let grid = grid_with(&[(0, star), (1, ItemStack::new("minecraft:blue_dye", 1))]);
        let outcome =
            special_crafting_result(SpecialRecipeKind::FireworkStarFade, Some(&hint), &grid)
                .expect("star + dye should add fade colours");
        match outcome.result.component("minecraft:firework_explosion") {
            Some(ItemComponent::FireworkExplosion(explosion)) => {
                assert_eq!(explosion.fade_colors, vec![2437522]); // DyeColor.BLUE firework
                assert_eq!(explosion.colors, vec![11546150], "original colours kept");
            }
            other => panic!("expected explosion, got {other:?}"),
        }
    }

    #[test]
    fn repair_item_rejects_mismatched_or_multi_count_inputs() {
        // Different items.
        let grid = grid_with(&[
            (0, damaged("minecraft:diamond_pickaxe", 100, 80)),
            (1, damaged("minecraft:iron_pickaxe", 100, 80)),
        ]);
        assert!(special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).is_none());

        // A stack of two is rejected.
        let mut pair = damaged("minecraft:diamond_pickaxe", 100, 80);
        pair.set_count(2);
        let grid = grid_with(&[
            (0, pair),
            (1, damaged("minecraft:diamond_pickaxe", 100, 80)),
        ]);
        assert!(special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).is_none());

        // A non-damageable item is rejected.
        let grid = grid_with(&[
            (0, ItemStack::new("minecraft:stone", 1)),
            (1, ItemStack::new("minecraft:stone", 1)),
        ]);
        assert!(special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).is_none());
    }
}
