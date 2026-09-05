//! Component-preserving crafting parity models.
use super::*;

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentCraftingStack {
    pub item: &'static str,
    pub count: u32,
    pub potion_contents: Option<&'static str>,
    pub custom_name: Option<&'static str>,
    pub banner_color: Option<&'static str>,
    pub banner_patterns: u8,
    pub written_book_generation: Option<u8>,
    pub pot_decorations: Option<PotDecorationsModel>,
    pub map_scale: Option<u8>,
    pub exploration_map: bool,
    pub map_post_processing_scale: bool,
    pub dye_color: Option<u32>,
    pub dyed_color: Option<u32>,
    pub firework_explosion: Option<FireworkExplosionModel>,
    pub fireworks: Option<FireworksModel>,
    pub max_damage: Option<u32>,
    pub damage: Option<u32>,
    pub curses: Vec<EnchantmentComponent>,
    pub base_color: Option<&'static str>,
}

#[cfg(test)]
impl ComponentCraftingStack {
    pub fn one(item: &'static str) -> Self {
        Self {
            item,
            count: 1,
            potion_contents: None,
            custom_name: None,
            banner_color: None,
            banner_patterns: 0,
            written_book_generation: None,
            pot_decorations: None,
            map_scale: None,
            exploration_map: false,
            map_post_processing_scale: false,
            dye_color: None,
            dyed_color: None,
            firework_explosion: None,
            fireworks: None,
            max_damage: None,
            damage: None,
            curses: Vec::new(),
            base_color: None,
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PotDecorationsModel {
    pub back: &'static str,
    pub left: &'static str,
    pub right: &'static str,
    pub front: &'static str,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkExplosionModel {
    pub shape: &'static str,
    pub colors: Vec<u32>,
    pub fade_colors: Vec<u32>,
    pub trail: bool,
    pub twinkle: bool,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworksModel {
    pub flight_duration: u8,
    pub explosions: Vec<FireworkExplosionModel>,
}

pub fn default_crafting_remaining_items(
    input: &[Option<CraftingStack>],
) -> Vec<Option<CraftingStack>> {
    input
        .iter()
        .map(|stack| {
            let item = stack.as_ref()?.item;
            crafting_remainder(item).map(CraftingStack::one)
        })
        .collect()
}

fn crafting_remainder(item: &str) -> Option<&'static str> {
    match item {
        "minecraft:water_bucket" | "minecraft:lava_bucket" | "minecraft:milk_bucket" => {
            Some("minecraft:bucket")
        }
        "minecraft:honey_bottle"
        | "minecraft:potion"
        | "minecraft:splash_potion"
        | "minecraft:lingering_potion" => Some("minecraft:glass_bottle"),
        _ => None,
    }
}

/// `CraftingBookCategory` — the recipe-book group a crafting recipe declares in its
/// JSON `category` field (defaults to `Misc`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CraftingBookCategoryModel {
    Building,
    Equipment,
    Redstone,
    Misc,
}

#[allow(dead_code)]
impl CraftingBookCategoryModel {
    pub fn recipe_book_category(self) -> &'static str {
        match self {
            Self::Building => "crafting_building_blocks",
            Self::Equipment => "crafting_equipment",
            Self::Redstone => "crafting_redstone",
            Self::Misc => "crafting_misc",
        }
    }

    /// Parse the JSON `category` string, defaulting to `Misc` (the `CraftingRecipe`
    /// codec default).
    pub fn from_id(id: Option<&str>) -> Self {
        match id {
            Some("building") => Self::Building,
            Some("equipment") => Self::Equipment,
            Some("redstone") => Self::Redstone,
            _ => Self::Misc,
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalCraftingRecipeModel {
    pub category: CraftingBookCategoryModel,
    pub group: &'static str,
    pub show_notification: bool,
    pub placement_info: PlacementInfo,
}

#[cfg(test)]
impl NormalCraftingRecipeModel {
    pub fn recipe_book_category(&self) -> &'static str {
        self.category.recipe_book_category()
    }

    pub fn is_incomplete(&self) -> bool {
        self.placement_info.is_impossible_to_place()
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantmentComponent {
    pub id: &'static str,
    pub level: u32,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmorTrimComponent {
    pub material: &'static str,
    pub pattern: &'static str,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmithingComponentStack {
    pub item: &'static str,
    pub count: u32,
    pub custom_name: Option<&'static str>,
    pub enchantments: Vec<EnchantmentComponent>,
    pub trim: Option<ArmorTrimComponent>,
}

#[cfg(test)]
impl SmithingComponentStack {
    pub fn one(item: &'static str) -> Self {
        Self {
            item,
            count: 1,
            custom_name: None,
            enchantments: Vec::new(),
            trim: None,
        }
    }
}

#[cfg(test)]
pub fn smithing_transform_result(
    result_item: &'static str,
    base: &SmithingComponentStack,
) -> SmithingComponentStack {
    let mut result = base.clone();
    result.item = result_item;
    result.count = 1;
    result
}

#[cfg(test)]
pub fn smithing_trim_result(
    base: &SmithingComponentStack,
    material_from_addition: Option<&'static str>,
    pattern: &'static str,
) -> Option<SmithingComponentStack> {
    let material = material_from_addition?;
    let trim = ArmorTrimComponent { material, pattern };
    if base.trim.as_ref() == Some(&trim) {
        return None;
    }

    let mut result = base.clone();
    result.count = 1;
    result.trim = Some(trim);
    Some(result)
}


#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleSmithingRecipeModel {
    pub show_notification: bool,
    pub placement_info: PlacementInfo,
}

#[cfg(test)]
impl SimpleSmithingRecipeModel {
    pub fn new(show_notification: bool, placement_info: PlacementInfo) -> Self {
        Self {
            show_notification,
            placement_info,
        }
    }

    pub fn group(&self) -> &'static str {
        ""
    }

    pub fn placement_info(&self) -> &PlacementInfo {
        &self.placement_info
    }
}
