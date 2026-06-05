//! Item-tag membership resolution for the tags RustCraft needs at runtime.
//!
//! Mirrors `data/minecraft/tags/item/**.json` for the enchantable-item hierarchy
//! (used by `Enchantment.canEnchant` / the anvil + enchanting table) plus the base
//! tool/armour tags it references. Tags resolve recursively: a value beginning with
//! `#` is another tag, otherwise it is a literal item id. The 1:1 source for each tag
//! is the matching JSON file in the 26.1.2 server data.

#![allow(dead_code)]

// --- Base tool/armour tags (`data/minecraft/tags/item/*.json`) -------------------

const SWORDS: &[&str] = &[
    "minecraft:wooden_sword",
    "minecraft:stone_sword",
    "minecraft:golden_sword",
    "minecraft:iron_sword",
    "minecraft:diamond_sword",
    "minecraft:netherite_sword",
    "minecraft:copper_sword",
];
const AXES: &[&str] = &[
    "minecraft:wooden_axe",
    "minecraft:stone_axe",
    "minecraft:golden_axe",
    "minecraft:iron_axe",
    "minecraft:diamond_axe",
    "minecraft:netherite_axe",
    "minecraft:copper_axe",
];
const PICKAXES: &[&str] = &[
    "minecraft:wooden_pickaxe",
    "minecraft:stone_pickaxe",
    "minecraft:golden_pickaxe",
    "minecraft:iron_pickaxe",
    "minecraft:diamond_pickaxe",
    "minecraft:netherite_pickaxe",
    "minecraft:copper_pickaxe",
];
const SHOVELS: &[&str] = &[
    "minecraft:wooden_shovel",
    "minecraft:stone_shovel",
    "minecraft:golden_shovel",
    "minecraft:iron_shovel",
    "minecraft:diamond_shovel",
    "minecraft:netherite_shovel",
    "minecraft:copper_shovel",
];
const HOES: &[&str] = &[
    "minecraft:wooden_hoe",
    "minecraft:stone_hoe",
    "minecraft:golden_hoe",
    "minecraft:iron_hoe",
    "minecraft:diamond_hoe",
    "minecraft:netherite_hoe",
    "minecraft:copper_hoe",
];
const SPEARS: &[&str] = &[
    "minecraft:wooden_spear",
    "minecraft:stone_spear",
    "minecraft:golden_spear",
    "minecraft:iron_spear",
    "minecraft:diamond_spear",
    "minecraft:netherite_spear",
    "minecraft:copper_spear",
];
const FOOT_ARMOR: &[&str] = &[
    "minecraft:leather_boots",
    "minecraft:copper_boots",
    "minecraft:chainmail_boots",
    "minecraft:golden_boots",
    "minecraft:iron_boots",
    "minecraft:diamond_boots",
    "minecraft:netherite_boots",
];
const LEG_ARMOR: &[&str] = &[
    "minecraft:leather_leggings",
    "minecraft:copper_leggings",
    "minecraft:chainmail_leggings",
    "minecraft:golden_leggings",
    "minecraft:iron_leggings",
    "minecraft:diamond_leggings",
    "minecraft:netherite_leggings",
];
const CHEST_ARMOR: &[&str] = &[
    "minecraft:leather_chestplate",
    "minecraft:copper_chestplate",
    "minecraft:chainmail_chestplate",
    "minecraft:golden_chestplate",
    "minecraft:iron_chestplate",
    "minecraft:diamond_chestplate",
    "minecraft:netherite_chestplate",
];
const HEAD_ARMOR: &[&str] = &[
    "minecraft:leather_helmet",
    "minecraft:copper_helmet",
    "minecraft:chainmail_helmet",
    "minecraft:golden_helmet",
    "minecraft:iron_helmet",
    "minecraft:diamond_helmet",
    "minecraft:netherite_helmet",
    "minecraft:turtle_helmet",
];
const SKULLS: &[&str] = &[
    "minecraft:player_head",
    "minecraft:creeper_head",
    "minecraft:zombie_head",
    "minecraft:skeleton_skull",
    "minecraft:wither_skeleton_skull",
    "minecraft:dragon_head",
    "minecraft:piglin_head",
];
const PLANKS: &[&str] = &[
    "minecraft:oak_planks",
    "minecraft:spruce_planks",
    "minecraft:birch_planks",
    "minecraft:jungle_planks",
    "minecraft:acacia_planks",
    "minecraft:dark_oak_planks",
    "minecraft:pale_oak_planks",
    "minecraft:crimson_planks",
    "minecraft:warped_planks",
    "minecraft:mangrove_planks",
    "minecraft:bamboo_planks",
    "minecraft:cherry_planks",
];

// --- Repair-material tags (`ItemStack.isValidRepairItem` via the `Repairable`
// component; `data/minecraft/tags/item/*_tool_materials.json` + `repairs_*_armor.json`)
const DIAMOND_TOOL_MATERIALS: &[&str] = &["minecraft:diamond"];
const IRON_TOOL_MATERIALS: &[&str] = &["minecraft:iron_ingot"];
const GOLD_TOOL_MATERIALS: &[&str] = &["minecraft:gold_ingot"];
const NETHERITE_TOOL_MATERIALS: &[&str] = &["minecraft:netherite_ingot"];
const WOODEN_TOOL_MATERIALS: &[&str] = &["#minecraft:planks"];
const STONE_TOOL_MATERIALS: &[&str] = &[
    "minecraft:cobblestone",
    "minecraft:blackstone",
    "minecraft:cobbled_deepslate",
];
const REPAIRS_LEATHER_ARMOR: &[&str] = &["minecraft:leather"];
const REPAIRS_IRON_ARMOR: &[&str] = &["minecraft:iron_ingot"];
const REPAIRS_GOLD_ARMOR: &[&str] = &["minecraft:gold_ingot"];
const REPAIRS_DIAMOND_ARMOR: &[&str] = &["minecraft:diamond"];
const REPAIRS_NETHERITE_ARMOR: &[&str] = &["minecraft:netherite_ingot"];
const REPAIRS_TURTLE_HELMET: &[&str] = &["minecraft:turtle_scute"];

// --- Enchantable tags (`data/minecraft/tags/item/enchantable/*.json`) -------------
// Values are stored verbatim (tag refs keep their leading `#`) so resolution mirrors
// the JSON. Sub-tag references (incl. nested enchantable tags) resolve recursively.

const E_ARMOR: &[&str] = &[
    "#minecraft:enchantable/foot_armor",
    "#minecraft:enchantable/leg_armor",
    "#minecraft:enchantable/chest_armor",
    "#minecraft:enchantable/head_armor",
];
const E_FOOT_ARMOR: &[&str] = &["#minecraft:foot_armor"];
const E_LEG_ARMOR: &[&str] = &["#minecraft:leg_armor"];
const E_CHEST_ARMOR: &[&str] = &["#minecraft:chest_armor"];
const E_HEAD_ARMOR: &[&str] = &["#minecraft:head_armor"];
const E_BOW: &[&str] = &["minecraft:bow"];
const E_CROSSBOW: &[&str] = &["minecraft:crossbow"];
const E_TRIDENT: &[&str] = &["minecraft:trident"];
const E_FISHING: &[&str] = &["minecraft:fishing_rod"];
const E_MACE: &[&str] = &["minecraft:mace"];
const E_LUNGE: &[&str] = &["#minecraft:spears"];
const E_SWEEPING: &[&str] = &["#minecraft:swords"];
const E_MELEE_WEAPON: &[&str] = &["#minecraft:swords", "#minecraft:spears"];
const E_SHARP_WEAPON: &[&str] = &["#minecraft:enchantable/melee_weapon", "#minecraft:axes"];
const E_WEAPON: &[&str] = &["#minecraft:enchantable/sharp_weapon", "minecraft:mace"];
const E_FIRE_ASPECT: &[&str] = &["#minecraft:enchantable/melee_weapon", "minecraft:mace"];
const E_MINING: &[&str] = &[
    "#minecraft:axes",
    "#minecraft:pickaxes",
    "#minecraft:shovels",
    "#minecraft:hoes",
    "minecraft:shears",
];
const E_MINING_LOOT: &[&str] = &[
    "#minecraft:axes",
    "#minecraft:pickaxes",
    "#minecraft:shovels",
    "#minecraft:hoes",
];
const E_DURABILITY: &[&str] = &[
    "#minecraft:foot_armor",
    "#minecraft:leg_armor",
    "#minecraft:chest_armor",
    "#minecraft:head_armor",
    "minecraft:elytra",
    "minecraft:shield",
    "#minecraft:swords",
    "#minecraft:axes",
    "#minecraft:pickaxes",
    "#minecraft:shovels",
    "#minecraft:hoes",
    "minecraft:bow",
    "minecraft:crossbow",
    "minecraft:trident",
    "minecraft:flint_and_steel",
    "minecraft:shears",
    "minecraft:brush",
    "minecraft:fishing_rod",
    "minecraft:carrot_on_a_stick",
    "minecraft:warped_fungus_on_a_stick",
    "minecraft:mace",
    "#minecraft:spears",
];
const E_EQUIPPABLE: &[&str] = &[
    "#minecraft:foot_armor",
    "#minecraft:leg_armor",
    "#minecraft:chest_armor",
    "#minecraft:head_armor",
    "minecraft:elytra",
    "#minecraft:skulls",
    "minecraft:carved_pumpkin",
];
const E_VANISHING: &[&str] = &[
    "#minecraft:enchantable/durability",
    "minecraft:compass",
    "minecraft:carved_pumpkin",
    "#minecraft:skulls",
];
const TRIMMABLE_ARMOR: &[&str] = &[
    "#minecraft:foot_armor",
    "#minecraft:leg_armor",
    "#minecraft:chest_armor",
    "#minecraft:head_armor",
];

/// Resolve a tag id (without the leading `#`) to its declared values, or `None` if the
/// tag is not modelled here.
fn tag_values(tag: &str) -> Option<&'static [&'static str]> {
    let tag = tag.strip_prefix("minecraft:").unwrap_or(tag);
    Some(match tag {
        // Base tool/armour tags.
        "swords" => SWORDS,
        "axes" => AXES,
        "pickaxes" => PICKAXES,
        "shovels" => SHOVELS,
        "hoes" => HOES,
        "spears" => SPEARS,
        "foot_armor" => FOOT_ARMOR,
        "leg_armor" => LEG_ARMOR,
        "chest_armor" => CHEST_ARMOR,
        "head_armor" => HEAD_ARMOR,
        "skulls" => SKULLS,
        "planks" => PLANKS,
        // Repair-material tags.
        "diamond_tool_materials" => DIAMOND_TOOL_MATERIALS,
        "iron_tool_materials" => IRON_TOOL_MATERIALS,
        "gold_tool_materials" => GOLD_TOOL_MATERIALS,
        "netherite_tool_materials" => NETHERITE_TOOL_MATERIALS,
        "wooden_tool_materials" => WOODEN_TOOL_MATERIALS,
        "stone_tool_materials" => STONE_TOOL_MATERIALS,
        "repairs_leather_armor" => REPAIRS_LEATHER_ARMOR,
        "repairs_iron_armor" => REPAIRS_IRON_ARMOR,
        "repairs_gold_armor" => REPAIRS_GOLD_ARMOR,
        "repairs_diamond_armor" => REPAIRS_DIAMOND_ARMOR,
        "repairs_netherite_armor" => REPAIRS_NETHERITE_ARMOR,
        "repairs_turtle_helmet" => REPAIRS_TURTLE_HELMET,
        // Enchantable tags.
        "enchantable/armor" => E_ARMOR,
        "enchantable/foot_armor" => E_FOOT_ARMOR,
        "enchantable/leg_armor" => E_LEG_ARMOR,
        "enchantable/chest_armor" => E_CHEST_ARMOR,
        "enchantable/head_armor" => E_HEAD_ARMOR,
        "enchantable/bow" => E_BOW,
        "enchantable/crossbow" => E_CROSSBOW,
        "enchantable/trident" => E_TRIDENT,
        "enchantable/fishing" => E_FISHING,
        "enchantable/mace" => E_MACE,
        "enchantable/lunge" => E_LUNGE,
        "enchantable/sweeping" => E_SWEEPING,
        "enchantable/melee_weapon" => E_MELEE_WEAPON,
        "enchantable/sharp_weapon" => E_SHARP_WEAPON,
        "enchantable/weapon" => E_WEAPON,
        "enchantable/fire_aspect" => E_FIRE_ASPECT,
        "enchantable/mining" => E_MINING,
        "enchantable/mining_loot" => E_MINING_LOOT,
        "enchantable/durability" => E_DURABILITY,
        "enchantable/equippable" => E_EQUIPPABLE,
        "enchantable/vanishing" => E_VANISHING,
        "trimmable_armor" => TRIMMABLE_ARMOR,
        _ => return None,
    })
}

/// Whether `item_id` is a member of `tag` (with or without a leading `#`), resolving
/// nested tag references recursively. Unknown tags yield `false`.
pub fn item_in_tag(item_id: &str, tag: &str) -> bool {
    let tag = tag.strip_prefix('#').unwrap_or(tag);
    match tag_values(tag) {
        Some(values) => values.iter().any(|value| match value.strip_prefix('#') {
            Some(sub_tag) => item_in_tag(item_id, sub_tag),
            None => *value == item_id,
        }),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::item_in_tag;

    #[test]
    fn base_tags_resolve_their_items() {
        assert!(item_in_tag("minecraft:diamond_sword", "#minecraft:swords"));
        assert!(item_in_tag("minecraft:copper_sword", "minecraft:swords"));
        assert!(!item_in_tag("minecraft:diamond_axe", "minecraft:swords"));
        assert!(item_in_tag("minecraft:turtle_helmet", "#minecraft:head_armor"));
        assert!(item_in_tag("minecraft:dragon_head", "#minecraft:skulls"));
    }

    #[test]
    fn enchantable_tags_resolve_through_the_hierarchy() {
        // sharp_weapon = melee_weapon (swords + spears) + axes.
        assert!(item_in_tag(
            "minecraft:diamond_sword",
            "#minecraft:enchantable/sharp_weapon"
        ));
        assert!(item_in_tag(
            "minecraft:iron_axe",
            "#minecraft:enchantable/sharp_weapon"
        ));
        assert!(item_in_tag(
            "minecraft:netherite_spear",
            "#minecraft:enchantable/sharp_weapon"
        ));
        // A pickaxe is NOT a sharp weapon.
        assert!(!item_in_tag(
            "minecraft:diamond_pickaxe",
            "#minecraft:enchantable/sharp_weapon"
        ));

        // weapon = sharp_weapon + mace.
        assert!(item_in_tag("minecraft:mace", "#minecraft:enchantable/weapon"));
        assert!(item_in_tag(
            "minecraft:diamond_axe",
            "#minecraft:enchantable/weapon"
        ));

        // armor nests the four armour-slot tags.
        assert!(item_in_tag(
            "minecraft:diamond_chestplate",
            "#minecraft:enchantable/armor"
        ));
        assert!(!item_in_tag(
            "minecraft:diamond_sword",
            "#minecraft:enchantable/armor"
        ));

        // mining = axes/pickaxes/shovels/hoes + shears.
        assert!(item_in_tag(
            "minecraft:netherite_pickaxe",
            "#minecraft:enchantable/mining"
        ));
        assert!(item_in_tag("minecraft:shears", "#minecraft:enchantable/mining"));

        // vanishing nests durability (deep) plus carved_pumpkin/compass/skulls.
        assert!(item_in_tag(
            "minecraft:diamond_sword",
            "#minecraft:enchantable/vanishing"
        ));
        assert!(item_in_tag(
            "minecraft:compass",
            "#minecraft:enchantable/vanishing"
        ));
        assert!(item_in_tag(
            "minecraft:player_head",
            "#minecraft:enchantable/vanishing"
        ));
    }

    #[test]
    fn unknown_tag_or_item_is_not_a_member() {
        assert!(!item_in_tag("minecraft:apple", "#minecraft:enchantable/weapon"));
        assert!(!item_in_tag("minecraft:diamond_sword", "#minecraft:nonexistent"));
    }
}
