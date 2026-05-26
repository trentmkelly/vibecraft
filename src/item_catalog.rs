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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ItemCatalogEntry {
    key: &'static str,
    static_name: &'static str,
    protocol_id: i32,
}

macro_rules! catalog_item {
    ($key:literal, $protocol_id:expr) => {
        ItemCatalogEntry {
            key: $key,
            static_name: concat!("minecraft:", $key),
            protocol_id: $protocol_id,
        }
    };
}

macro_rules! catalog_alias {
    ($key:literal, $canonical_key:literal, $protocol_id:expr) => {
        ItemCatalogEntry {
            key: $key,
            static_name: concat!("minecraft:", $canonical_key),
            protocol_id: $protocol_id,
        }
    };
}

const ITEM_CATALOG: &[ItemCatalogEntry] = &[
    catalog_item!("air", 0),
    catalog_item!("stone", 1),
    catalog_item!("granite", 2),
    catalog_item!("polished_granite", 3),
    catalog_item!("diorite", 4),
    catalog_item!("polished_diorite", 5),
    catalog_item!("andesite", 6),
    catalog_item!("polished_andesite", 7),
    catalog_item!("deepslate", 8),
    catalog_item!("cobbled_deepslate", 9),
    catalog_item!("grass_block", 27),
    catalog_item!("dirt", 28),
    catalog_item!("coarse_dirt", 29),
    catalog_item!("podzol", 30),
    catalog_item!("rooted_dirt", 31),
    catalog_item!("mud", 32),
    catalog_item!("cobblestone", 35),
    catalog_item!("oak_planks", 36),
    catalog_item!("spruce_planks", 37),
    catalog_item!("birch_planks", 38),
    catalog_item!("jungle_planks", 39),
    catalog_item!("acacia_planks", 40),
    catalog_item!("dark_oak_planks", 42),
    catalog_item!("oak_sapling", 49),
    catalog_item!("spruce_sapling", 50),
    catalog_item!("birch_sapling", 51),
    catalog_item!("jungle_sapling", 52),
    catalog_item!("acacia_sapling", 53),
    catalog_item!("cherry_sapling", 54),
    catalog_item!("dark_oak_sapling", 55),
    catalog_item!("pale_oak_sapling", 56),
    catalog_item!("mangrove_propagule", 57),
    catalog_item!("sand", 59),
    catalog_item!("red_sand", 62),
    catalog_item!("gravel", 63),
    catalog_item!("coal_ore", 64),
    catalog_item!("deepslate_coal_ore", 65),
    catalog_item!("iron_ore", 66),
    catalog_item!("deepslate_iron_ore", 67),
    catalog_item!("copper_ore", 68),
    catalog_item!("deepslate_copper_ore", 69),
    catalog_item!("gold_ore", 70),
    catalog_item!("deepslate_gold_ore", 71),
    catalog_item!("redstone_ore", 72),
    catalog_alias!("lit_redstone_ore", "redstone_ore", 72),
    catalog_item!("deepslate_redstone_ore", 73),
    catalog_alias!("lit_deepslate_redstone_ore", "deepslate_redstone_ore", 73),
    catalog_item!("emerald_ore", 74),
    catalog_item!("deepslate_emerald_ore", 75),
    catalog_item!("lapis_ore", 76),
    catalog_item!("deepslate_lapis_ore", 77),
    catalog_item!("diamond_ore", 78),
    catalog_item!("deepslate_diamond_ore", 79),
    catalog_item!("nether_gold_ore", 80),
    catalog_item!("nether_quartz_ore", 81),
    catalog_item!("oak_log", 134),
    catalog_item!("spruce_log", 135),
    catalog_item!("birch_log", 136),
    catalog_item!("jungle_log", 137),
    catalog_item!("acacia_log", 138),
    catalog_item!("dark_oak_log", 141),
    catalog_item!("stripped_oak_log", 148),
    catalog_item!("stripped_spruce_log", 149),
    catalog_item!("stripped_birch_log", 150),
    catalog_item!("stripped_jungle_log", 151),
    catalog_item!("stripped_acacia_log", 152),
    catalog_item!("stripped_dark_oak_log", 154),
    catalog_item!("stripped_oak_wood", 159),
    catalog_item!("stripped_spruce_wood", 160),
    catalog_item!("stripped_birch_wood", 161),
    catalog_item!("stripped_jungle_wood", 162),
    catalog_item!("stripped_acacia_wood", 163),
    catalog_item!("stripped_dark_oak_wood", 165),
    catalog_item!("oak_wood", 171),
    catalog_item!("spruce_wood", 172),
    catalog_item!("birch_wood", 173),
    catalog_item!("jungle_wood", 174),
    catalog_item!("acacia_wood", 175),
    catalog_item!("dark_oak_wood", 178),
    catalog_item!("oak_leaves", 182),
    catalog_item!("spruce_leaves", 183),
    catalog_item!("birch_leaves", 184),
    catalog_item!("jungle_leaves", 185),
    catalog_item!("acacia_leaves", 186),
    catalog_item!("cherry_leaves", 187),
    catalog_item!("dark_oak_leaves", 188),
    catalog_item!("pale_oak_leaves", 189),
    catalog_item!("mangrove_leaves", 190),
    catalog_item!("azalea_leaves", 191),
    catalog_item!("flowering_azalea_leaves", 192),
    catalog_item!("sandstone", 198),
    catalog_item!("chiseled_sandstone", 199),
    catalog_item!("cut_sandstone", 200),
    catalog_item!("short_grass", 202),
    catalog_item!("fern", 203),
    catalog_item!("azalea", 205),
    catalog_item!("flowering_azalea", 206),
    catalog_item!("dead_bush", 207),
    catalog_item!("firefly_bush", 208),
    catalog_item!("dandelion", 229),
    catalog_item!("golden_dandelion", 230),
    catalog_item!("poppy", 233),
    catalog_item!("blue_orchid", 234),
    catalog_item!("allium", 235),
    catalog_item!("azure_bluet", 236),
    catalog_item!("red_tulip", 237),
    catalog_item!("orange_tulip", 238),
    catalog_item!("white_tulip", 239),
    catalog_item!("pink_tulip", 240),
    catalog_item!("oxeye_daisy", 241),
    catalog_item!("cornflower", 242),
    catalog_item!("lily_of_the_valley", 243),
    catalog_item!("wither_rose", 244),
    catalog_item!("torchflower", 245),
    catalog_item!("brown_mushroom", 248),
    catalog_item!("red_mushroom", 249),
    catalog_item!("sugar_cane", 257),
    catalog_item!("wildflowers", 260),
    catalog_item!("bamboo", 270),
    catalog_item!("smooth_sandstone", 303),
    catalog_item!("bookshelf", 318),
    catalog_item!("snow", 338),
    catalog_item!("snow_block", 340),
    catalog_item!("cactus", 341),
    catalog_item!("clay", 343),
    catalog_item!("pumpkin", 357),
    catalog_item!("glowstone", 368),
    catalog_item!("melon", 410),
    catalog_item!("sunflower", 525),
    catalog_item!("lilac", 526),
    catalog_item!("rose_bush", 527),
    catalog_item!("peony", 528),
    catalog_item!("sea_lantern", 569),
    catalog_item!("redstone", 718),
    catalog_item!("apple", 894),
    catalog_item!("coal", 897),
    catalog_item!("diamond", 899),
    catalog_item!("emerald", 900),
    catalog_item!("lapis_lazuli", 901),
    catalog_item!("quartz", 902),
    catalog_item!("raw_iron", 904),
    catalog_item!("raw_copper", 906),
    catalog_item!("raw_gold", 908),
    catalog_item!("stick", 947),
    catalog_item!("wheat_seeds", 952),
    catalog_item!("bucket", 967),
    catalog_item!("water_bucket", 968),
    catalog_item!("lava_bucket", 969),
    catalog_item!("flint", 983),
    catalog_item!("snowball", 1017),
    catalog_item!("clay_ball", 1027),
    catalog_item!("book", 1030),
    catalog_item!("glowstone_dust", 1057),
    catalog_item!("melon_slice", 1107),
    catalog_item!("gold_nugget", 1119),
    catalog_item!("writable_book", 1221),
    catalog_item!("written_book", 1222),
    catalog_item!("carrot", 1228),
    catalog_item!("potato", 1229),
    catalog_item!("poisonous_potato", 1231),
    catalog_item!("prismarine_crystals", 1249),
    catalog_item!("beetroot", 1288),
    catalog_item!("beetroot_seeds", 1289),
];

fn item_key(registry_id: &str) -> &str {
    registry_id
        .strip_prefix("minecraft:")
        .unwrap_or(registry_id)
}

fn item_catalog_entry(registry_id: &str) -> Option<&'static ItemCatalogEntry> {
    let key = item_key(registry_id);
    ITEM_CATALOG.iter().find(|entry| entry.key == key)
}

/// Maps a Minecraft item registry ID to its canonical `&'static str` registry name.
///
/// Allows dynamic strings produced by the loot system (e.g. `"minecraft:coal"`) to be
/// converted back to compile-time-known names for storage in `DroppedItem` and for passing
/// to `ItemStack::new`.  Aliases (e.g. block-state names like `"lit_redstone_ore"`) resolve
/// to the canonical item name.  Returns `None` for unknown items.
pub fn item_static_name(registry_id: &str) -> Option<&'static str> {
    item_catalog_entry(registry_id).map(|entry| entry.static_name)
}

/// Maps a Minecraft item registry ID to its numeric protocol ID.
///
/// Protocol IDs are sourced from the vanilla data generator reports:
/// `java -DbundlerMainClass=net.minecraft.data.Main -jar server.jar --reports`
/// which produces `generated/reports/registries.json` with authoritative `protocol_id` values.
pub fn item_protocol_id(registry_id: &str) -> Option<i32> {
    item_catalog_entry(registry_id).map(|entry| entry.protocol_id)
}

/// Maps a numeric item protocol ID back to the canonical static item registry name.
///
/// The table mirrors [`item_protocol_id`] for the runtime item surface RustCraft can
/// currently materialize as an [`crate::item_stack::ItemStack`].
pub fn item_static_name_from_protocol_id(protocol_id: i32) -> Option<&'static str> {
    ITEM_CATALOG
        .iter()
        .find(|entry| entry.protocol_id == protocol_id)
        .map(|entry| entry.static_name)
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

    #[test]
    fn item_static_name_returns_qualified_names_matching_protocol_id_coverage() {
        // Every item that has a protocol ID must also have a static name, and vice versa.
        let sample: &[(&str, &str)] = &[
            ("coal", "minecraft:coal"),
            ("minecraft:coal", "minecraft:coal"),
            ("diamond", "minecraft:diamond"),
            ("wheat_seeds", "minecraft:wheat_seeds"),
            ("oak_sapling", "minecraft:oak_sapling"),
            ("stick", "minecraft:stick"),
            ("flint", "minecraft:flint"),
            ("glowstone_dust", "minecraft:glowstone_dust"),
        ];
        for (input, expected) in sample {
            assert_eq!(
                item_static_name(input),
                Some(*expected),
                "item_static_name({input:?}) should return {expected:?}"
            );
        }
    }

    #[test]
    fn item_static_name_aliases_resolve_to_canonical_name() {
        // Block-state alias names (e.g. lit ore states) must resolve to the canonical item name.
        assert_eq!(
            item_static_name("lit_redstone_ore"),
            Some("minecraft:redstone_ore")
        );
        assert_eq!(
            item_static_name("lit_deepslate_redstone_ore"),
            Some("minecraft:deepslate_redstone_ore")
        );
    }

    #[test]
    fn item_static_name_and_protocol_id_have_consistent_coverage() {
        // A item known by item_protocol_id must also be known by item_static_name and vice versa.
        // We verify this for a set of known items.
        let items = &[
            "coal",
            "diamond",
            "emerald",
            "stick",
            "wheat_seeds",
            "coal_ore",
            "oak_log",
            "oak_sapling",
            "flint",
            "clay_ball",
            "glowstone_dust",
        ];
        for item in items {
            assert!(
                item_protocol_id(item).is_some(),
                "item_protocol_id({item:?}) should be Some"
            );
            assert!(
                item_static_name(item).is_some(),
                "item_static_name({item:?}) should be Some"
            );
        }
    }

    #[test]
    fn item_protocol_ids_round_trip_for_runtime_creative_inventory_items() {
        let items = &[
            "minecraft:stone",
            "minecraft:oak_planks",
            "minecraft:stick",
            "minecraft:bucket",
            "minecraft:redstone_ore",
            "minecraft:deepslate_redstone_ore",
        ];
        for item in items {
            let protocol_id = item_protocol_id(item).unwrap();
            assert_eq!(item_static_name_from_protocol_id(protocol_id), Some(*item));
        }
        assert_eq!(item_static_name_from_protocol_id(i32::MAX), None);
    }

    #[test]
    fn item_static_name_returns_none_for_unknown_items() {
        assert_eq!(item_static_name("not_a_real_item"), None);
        assert_eq!(item_static_name("minecraft:not_a_real_item"), None);
    }
}
