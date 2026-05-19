#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSourceKind {
    Class,
    Interface,
    Enum,
    Record,
    PackageInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemRole {
    Core,
    Stack,
    BlockPlacement,
    Tool,
    Weapon,
    Projectile,
    Consumable,
    Container,
    EntityPlacement,
    Creative,
    Data,
    Utility,
    Metadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemSourceInfo {
    pub java_name: &'static str,
    pub source_kind: ItemSourceKind,
    pub role: ItemRole,
}

pub const ITEM_SOURCE_SURFACE_COUNT_26_1_2: usize = 101;

pub const ITEM_SOURCE_SURFACE: &[ItemSourceInfo] = &[
    info(
        "AdventureModePredicate",
        ItemSourceKind::Record,
        ItemRole::Data,
    ),
    info("AirItem", ItemSourceKind::Class, ItemRole::Core),
    info(
        "ArmorStandItem",
        ItemSourceKind::Class,
        ItemRole::EntityPlacement,
    ),
    info("ArrowItem", ItemSourceKind::Class, ItemRole::Projectile),
    info("AxeItem", ItemSourceKind::Class, ItemRole::Tool),
    info(
        "BannerItem",
        ItemSourceKind::Class,
        ItemRole::BlockPlacement,
    ),
    info("BedItem", ItemSourceKind::Class, ItemRole::BlockPlacement),
    info("BlockItem", ItemSourceKind::Class, ItemRole::BlockPlacement),
    info("BoatItem", ItemSourceKind::Class, ItemRole::EntityPlacement),
    info("BoneMealItem", ItemSourceKind::Class, ItemRole::Consumable),
    info("BottleItem", ItemSourceKind::Class, ItemRole::Container),
    info("BowItem", ItemSourceKind::Class, ItemRole::Weapon),
    info("BrushItem", ItemSourceKind::Class, ItemRole::Tool),
    info("BucketItem", ItemSourceKind::Class, ItemRole::Container),
    info("BundleItem", ItemSourceKind::Class, ItemRole::Container),
    info("CompassItem", ItemSourceKind::Class, ItemRole::Utility),
    info("CreativeModeTab", ItemSourceKind::Class, ItemRole::Creative),
    info(
        "CreativeModeTabs",
        ItemSourceKind::Class,
        ItemRole::Creative,
    ),
    info("CrossbowItem", ItemSourceKind::Class, ItemRole::Weapon),
    info("DebugStickItem", ItemSourceKind::Class, ItemRole::Tool),
    info("DiscFragmentItem", ItemSourceKind::Class, ItemRole::Utility),
    info(
        "DispensibleContainerItem",
        ItemSourceKind::Interface,
        ItemRole::Container,
    ),
    info(
        "DoubleHighBlockItem",
        ItemSourceKind::Class,
        ItemRole::BlockPlacement,
    ),
    info("DyeColor", ItemSourceKind::Enum, ItemRole::Data),
    info("DyeItem", ItemSourceKind::Class, ItemRole::Consumable),
    info("EggItem", ItemSourceKind::Class, ItemRole::Projectile),
    info("EmptyMapItem", ItemSourceKind::Class, ItemRole::Utility),
    info(
        "EndCrystalItem",
        ItemSourceKind::Class,
        ItemRole::EntityPlacement,
    ),
    info("EnderEyeItem", ItemSourceKind::Class, ItemRole::Consumable),
    info(
        "EnderpearlItem",
        ItemSourceKind::Class,
        ItemRole::Projectile,
    ),
    info(
        "ExperienceBottleItem",
        ItemSourceKind::Class,
        ItemRole::Projectile,
    ),
    info(
        "FireChargeItem",
        ItemSourceKind::Class,
        ItemRole::Consumable,
    ),
    info(
        "FireworkRocketItem",
        ItemSourceKind::Class,
        ItemRole::Projectile,
    ),
    info("FishingRodItem", ItemSourceKind::Class, ItemRole::Tool),
    info("FlintAndSteelItem", ItemSourceKind::Class, ItemRole::Tool),
    info("FoodOnAStickItem", ItemSourceKind::Class, ItemRole::Tool),
    info(
        "GameMasterBlockItem",
        ItemSourceKind::Class,
        ItemRole::BlockPlacement,
    ),
    info(
        "GlowInkSacItem",
        ItemSourceKind::Class,
        ItemRole::Consumable,
    ),
    info(
        "HangingEntityItem",
        ItemSourceKind::Class,
        ItemRole::EntityPlacement,
    ),
    info(
        "HangingSignItem",
        ItemSourceKind::Class,
        ItemRole::BlockPlacement,
    ),
    info("HoeItem", ItemSourceKind::Class, ItemRole::Tool),
    info("HoneycombItem", ItemSourceKind::Class, ItemRole::Consumable),
    info("InkSacItem", ItemSourceKind::Class, ItemRole::Consumable),
    info("Instrument", ItemSourceKind::Record, ItemRole::Data),
    info("InstrumentItem", ItemSourceKind::Class, ItemRole::Utility),
    info("Instruments", ItemSourceKind::Interface, ItemRole::Data),
    info("Item", ItemSourceKind::Class, ItemRole::Core),
    info("ItemCooldowns", ItemSourceKind::Class, ItemRole::Utility),
    info("ItemDisplayContext", ItemSourceKind::Enum, ItemRole::Data),
    info(
        "ItemFrameItem",
        ItemSourceKind::Class,
        ItemRole::EntityPlacement,
    ),
    info("ItemInstance", ItemSourceKind::Interface, ItemRole::Data),
    info("ItemStack", ItemSourceKind::Class, ItemRole::Stack),
    info("ItemStackLinkedSet", ItemSourceKind::Class, ItemRole::Stack),
    info("ItemStackTemplate", ItemSourceKind::Record, ItemRole::Stack),
    info("ItemUseAnimation", ItemSourceKind::Enum, ItemRole::Data),
    info("ItemUtils", ItemSourceKind::Class, ItemRole::Utility),
    info("Items", ItemSourceKind::Class, ItemRole::Core),
    info("JukeboxPlayable", ItemSourceKind::Record, ItemRole::Data),
    info("JukeboxSong", ItemSourceKind::Record, ItemRole::Data),
    info(
        "JukeboxSongPlayer",
        ItemSourceKind::Class,
        ItemRole::Utility,
    ),
    info("JukeboxSongs", ItemSourceKind::Interface, ItemRole::Data),
    info(
        "KnowledgeBookItem",
        ItemSourceKind::Class,
        ItemRole::Consumable,
    ),
    info("LeadItem", ItemSourceKind::Class, ItemRole::Tool),
    info(
        "LingeringPotionItem",
        ItemSourceKind::Class,
        ItemRole::Projectile,
    ),
    info("MaceItem", ItemSourceKind::Class, ItemRole::Weapon),
    info("MapItem", ItemSourceKind::Class, ItemRole::Utility),
    info(
        "MinecartItem",
        ItemSourceKind::Class,
        ItemRole::EntityPlacement,
    ),
    info("MobBucketItem", ItemSourceKind::Class, ItemRole::Container),
    info("NameTagItem", ItemSourceKind::Class, ItemRole::Tool),
    info(
        "PlaceOnWaterBlockItem",
        ItemSourceKind::Class,
        ItemRole::BlockPlacement,
    ),
    info(
        "PlayerHeadItem",
        ItemSourceKind::Class,
        ItemRole::BlockPlacement,
    ),
    info("PotionItem", ItemSourceKind::Class, ItemRole::Consumable),
    info(
        "ProjectileItem",
        ItemSourceKind::Interface,
        ItemRole::Projectile,
    ),
    info(
        "ProjectileWeaponItem",
        ItemSourceKind::Class,
        ItemRole::Weapon,
    ),
    info("Rarity", ItemSourceKind::Enum, ItemRole::Data),
    info(
        "ScaffoldingBlockItem",
        ItemSourceKind::Class,
        ItemRole::BlockPlacement,
    ),
    info(
        "ServerItemCooldowns",
        ItemSourceKind::Class,
        ItemRole::Utility,
    ),
    info("ShearsItem", ItemSourceKind::Class, ItemRole::Tool),
    info("ShieldItem", ItemSourceKind::Class, ItemRole::Weapon),
    info("ShovelItem", ItemSourceKind::Class, ItemRole::Tool),
    info(
        "SignApplicator",
        ItemSourceKind::Interface,
        ItemRole::BlockPlacement,
    ),
    info("SignItem", ItemSourceKind::Class, ItemRole::BlockPlacement),
    info(
        "SmithingTemplateItem",
        ItemSourceKind::Class,
        ItemRole::Utility,
    ),
    info("SnowballItem", ItemSourceKind::Class, ItemRole::Projectile),
    info(
        "SolidBucketItem",
        ItemSourceKind::Class,
        ItemRole::Container,
    ),
    info(
        "SpawnEggItem",
        ItemSourceKind::Class,
        ItemRole::EntityPlacement,
    ),
    info(
        "SpectralArrowItem",
        ItemSourceKind::Class,
        ItemRole::Projectile,
    ),
    info(
        "SplashPotionItem",
        ItemSourceKind::Class,
        ItemRole::Projectile,
    ),
    info("SpyglassItem", ItemSourceKind::Class, ItemRole::Utility),
    info(
        "StandingAndWallBlockItem",
        ItemSourceKind::Class,
        ItemRole::BlockPlacement,
    ),
    info("SwingAnimationType", ItemSourceKind::Enum, ItemRole::Data),
    info(
        "ThrowablePotionItem",
        ItemSourceKind::Class,
        ItemRole::Projectile,
    ),
    info(
        "TippedArrowItem",
        ItemSourceKind::Class,
        ItemRole::Projectile,
    ),
    info("ToolMaterial", ItemSourceKind::Record, ItemRole::Data),
    info("TooltipFlag", ItemSourceKind::Interface, ItemRole::Data),
    info("TridentItem", ItemSourceKind::Class, ItemRole::Weapon),
    info(
        "WeatheringCopperItems",
        ItemSourceKind::Record,
        ItemRole::Utility,
    ),
    info(
        "WindChargeItem",
        ItemSourceKind::Class,
        ItemRole::Projectile,
    ),
    info("WritableBookItem", ItemSourceKind::Class, ItemRole::Utility),
    info("WrittenBookItem", ItemSourceKind::Class, ItemRole::Utility),
    info(
        "package-info",
        ItemSourceKind::PackageInfo,
        ItemRole::Metadata,
    ),
];

pub const fn info(
    java_name: &'static str,
    source_kind: ItemSourceKind,
    role: ItemRole,
) -> ItemSourceInfo {
    ItemSourceInfo {
        java_name,
        source_kind,
        role,
    }
}

pub fn item_source(java_name: &str) -> Option<&'static ItemSourceInfo> {
    ITEM_SOURCE_SURFACE
        .iter()
        .find(|info| info.java_name == java_name)
}

pub fn item_sources_by_role(role: ItemRole) -> impl Iterator<Item = &'static ItemSourceInfo> {
    ITEM_SOURCE_SURFACE
        .iter()
        .filter(move |info| info.role == role)
}

pub fn concrete_item_class_count() -> usize {
    ITEM_SOURCE_SURFACE
        .iter()
        .filter(|info| info.source_kind == ItemSourceKind::Class)
        .count()
}

/// Maps a Minecraft item registry ID to its numeric protocol ID.
/// Maps a Minecraft item registry ID to its numeric protocol ID.
///
/// Protocol IDs are sourced from the vanilla data generator reports:
/// `java -DbundlerMainClass=net.minecraft.data.Main -jar server.jar --reports`
/// which produces `generated/reports/registries.json` with authoritative `protocol_id` values.
pub fn item_protocol_id(registry_id: &str) -> Option<i32> {
    let key = registry_id
        .strip_prefix("minecraft:")
        .unwrap_or(registry_id);
    Some(match key {
        // Stone variants
        "air" => 0,
        "stone" => 1,
        "granite" => 2,
        "polished_granite" => 3,
        "diorite" => 4,
        "polished_diorite" => 5,
        "andesite" => 6,
        "polished_andesite" => 7,
        "deepslate" => 8,
        "cobbled_deepslate" => 9,
        // Dirt/soil
        "grass_block" => 27,
        "dirt" => 28,
        "coarse_dirt" => 29,
        "podzol" => 30,
        "rooted_dirt" => 31,
        "mud" => 32,
        "clay" => 343,
        "cobblestone" => 35,
        // Saplings
        "oak_sapling" => 49,
        "spruce_sapling" => 50,
        "birch_sapling" => 51,
        "jungle_sapling" => 52,
        "acacia_sapling" => 53,
        "cherry_sapling" => 54,
        "dark_oak_sapling" => 55,
        "pale_oak_sapling" => 56,
        "mangrove_propagule" => 57,
        // Wood — planks
        "oak_planks" => 36,
        "spruce_planks" => 37,
        "birch_planks" => 38,
        "jungle_planks" => 39,
        "acacia_planks" => 40,
        "dark_oak_planks" => 42,
        // Terrain
        "sand" => 59,
        "red_sand" => 62,
        "gravel" => 63,
        "flint" => 983,
        // Ore blocks (for silk touch — not yet implemented, here for completeness)
        "coal_ore" => 64,
        "deepslate_coal_ore" => 65,
        "iron_ore" => 66,
        "deepslate_iron_ore" => 67,
        "copper_ore" => 68,
        "deepslate_copper_ore" => 69,
        "gold_ore" => 70,
        "deepslate_gold_ore" => 71,
        "redstone_ore" | "lit_redstone_ore" => 72,
        "deepslate_redstone_ore" | "lit_deepslate_redstone_ore" => 73,
        "emerald_ore" => 74,
        "deepslate_emerald_ore" => 75,
        "lapis_ore" => 76,
        "deepslate_lapis_ore" => 77,
        "diamond_ore" => 78,
        "deepslate_diamond_ore" => 79,
        "nether_gold_ore" => 80,
        "nether_quartz_ore" => 81,
        // Ore drops
        "coal" => 897,
        "raw_iron" => 904,
        "raw_gold" => 908,
        "raw_copper" => 906,
        "redstone" => 718,
        "diamond" => 899,
        "emerald" => 900,
        "lapis_lazuli" => 901,
        "quartz" => 902,
        "gold_nugget" => 1119,
        // Wood — logs (each variant has a distinct protocol ID)
        "oak_log" => 134,
        "spruce_log" => 135,
        "birch_log" => 136,
        "jungle_log" => 137,
        "acacia_log" => 138,
        "dark_oak_log" => 141,
        "stripped_oak_log" => 148,
        "stripped_spruce_log" => 149,
        "stripped_birch_log" => 150,
        "stripped_jungle_log" => 151,
        "stripped_acacia_log" => 152,
        "stripped_dark_oak_log" => 154,
        "oak_wood" => 171,
        "spruce_wood" => 172,
        "birch_wood" => 173,
        "jungle_wood" => 174,
        "acacia_wood" => 175,
        "dark_oak_wood" => 178,
        "stripped_oak_wood" => 159,
        "stripped_spruce_wood" => 160,
        "stripped_birch_wood" => 161,
        "stripped_jungle_wood" => 162,
        "stripped_acacia_wood" => 163,
        "stripped_dark_oak_wood" => 165,
        // Leaves (for shears self-drop)
        "oak_leaves" => 182,
        "spruce_leaves" => 183,
        "birch_leaves" => 184,
        "jungle_leaves" => 185,
        "acacia_leaves" => 186,
        "cherry_leaves" => 187,
        "dark_oak_leaves" => 188,
        "pale_oak_leaves" => 189,
        "mangrove_leaves" => 190,
        "azalea_leaves" => 191,
        "flowering_azalea_leaves" => 192,
        // Sandstone
        "sandstone" => 198,
        "chiseled_sandstone" => 199,
        "cut_sandstone" => 200,
        "smooth_sandstone" => 303,
        // Plants — grass-type
        "short_grass" => 202,
        "fern" => 203,
        "dead_bush" => 207,
        "firefly_bush" => 208,
        // Azalea bushes (drop from azalea/flowering_azalea leaves)
        "azalea" => 205,
        "flowering_azalea" => 206,
        // Flowers — all self-drop
        "dandelion" => 229,
        "golden_dandelion" => 230,
        "poppy" => 233,
        "blue_orchid" => 234,
        "allium" => 235,
        "azure_bluet" => 236,
        "red_tulip" => 237,
        "orange_tulip" => 238,
        "white_tulip" => 239,
        "pink_tulip" => 240,
        "oxeye_daisy" => 241,
        "cornflower" => 242,
        "lily_of_the_valley" => 243,
        "wither_rose" => 244,
        "torchflower" => 245,
        "brown_mushroom" => 248,
        "red_mushroom" => 249,
        "wildflowers" => 260,
        // Double-tall flowers (drop self from lower half)
        "sunflower" => 525,
        "lilac" => 526,
        "rose_bush" => 527,
        "peony" => 528,
        // Crops and food
        "apple" => 894,
        "carrot" => 1228,
        "potato" => 1229,
        "poisonous_potato" => 1231,
        "beetroot" => 1288,
        "beetroot_seeds" => 1289,
        "melon_slice" => 1107,
        // Natural blocks
        "sugar_cane" => 257,
        "pumpkin" => 357,
        "melon" => 410,
        "cactus" => 341,
        "bamboo" => 270,
        "snow" => 338,
        "snow_block" => 340,
        "glowstone" => 368,
        "sea_lantern" => 569,
        "bookshelf" => 318,
        // Natural block fragment drops
        "clay_ball" => 1027,
        "glowstone_dust" => 1057,
        "prismarine_crystals" => 1249,
        "snowball" => 1017,
        "book" => 1030,
        // Crafted items
        "stick" => 947,
        "wheat_seeds" => 952,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn item_source_surface_matches_decompiled_top_level_files() {
        assert_eq!(ITEM_SOURCE_SURFACE.len(), ITEM_SOURCE_SURFACE_COUNT_26_1_2);
        assert_eq!(concrete_item_class_count(), 81);
        assert_eq!(
            item_source("package-info").unwrap().source_kind,
            ItemSourceKind::PackageInfo
        );
    }

    #[test]
    fn item_source_surface_is_sorted_and_unique() {
        let names: Vec<&str> = ITEM_SOURCE_SURFACE
            .iter()
            .map(|info| info.java_name)
            .collect();
        let sorted = {
            let mut copy = names.clone();
            copy.sort_unstable();
            copy
        };
        assert_eq!(names, sorted);
        assert_eq!(
            names.iter().copied().collect::<BTreeSet<_>>().len(),
            names.len()
        );
    }

    #[test]
    fn item_catalog_covers_core_runtime_and_stack_sources() {
        assert_eq!(item_source("Item").unwrap().role, ItemRole::Core);
        assert_eq!(item_source("Items").unwrap().role, ItemRole::Core);
        assert_eq!(item_source("ItemStack").unwrap().role, ItemRole::Stack);
        assert_eq!(
            item_source("ItemStackTemplate").unwrap().source_kind,
            ItemSourceKind::Record
        );
    }

    #[test]
    fn item_catalog_groups_behavior_families_needed_by_later_tasks() {
        let block_placement = item_sources_by_role(ItemRole::BlockPlacement).count();
        let projectiles = item_sources_by_role(ItemRole::Projectile).count();
        let containers = item_sources_by_role(ItemRole::Container).count();
        let tools = item_sources_by_role(ItemRole::Tool).count();
        assert!(block_placement >= 10);
        assert!(projectiles >= 10);
        assert!(containers >= 5);
        assert!(tools >= 8);
    }

    #[test]
    fn item_catalog_records_non_class_support_types() {
        assert_eq!(
            item_source("DyeColor").unwrap().source_kind,
            ItemSourceKind::Enum
        );
        assert_eq!(
            item_source("DispensibleContainerItem").unwrap().source_kind,
            ItemSourceKind::Interface
        );
        assert_eq!(
            item_source("JukeboxSong").unwrap().source_kind,
            ItemSourceKind::Record
        );
    }
}
