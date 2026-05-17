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
