#![allow(dead_code)]
pub const VANILLA_BLOCK_PACKAGE: &str = "net/minecraft/world/level/block";
pub const VANILLA_TOP_LEVEL_BLOCK_CLASS_COUNT: usize = 322;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockClassEntry {
    pub class_name: &'static str,
    pub package: &'static str,
    pub concrete_block_class: bool,
}

macro_rules! entry {
    ($name:literal, $concrete:expr) => {
        BlockClassEntry {
            class_name: $name,
            package: VANILLA_BLOCK_PACKAGE,
            concrete_block_class: $concrete,
        }
    };
}

pub const TOP_LEVEL_BLOCK_CLASSES: &[BlockClassEntry] = &[
    entry!("AbstractBannerBlock", false),
    entry!("AbstractCandleBlock", false),
    entry!("AbstractCauldronBlock", false),
    entry!("AbstractChestBlock", false),
    entry!("AbstractFurnaceBlock", false),
    entry!("AbstractSkullBlock", false),
    entry!("AirBlock", true),
    entry!("AmethystBlock", true),
    entry!("AmethystClusterBlock", true),
    entry!("AnvilBlock", true),
    entry!("AttachedStemBlock", true),
    entry!("AzaleaBlock", true),
    entry!("BambooSaplingBlock", true),
    entry!("BambooStalkBlock", true),
    entry!("BannerBlock", true),
    entry!("BarrelBlock", true),
    entry!("BarrierBlock", true),
    entry!("BaseCoralFanBlock", true),
    entry!("BaseCoralPlantBlock", true),
    entry!("BaseCoralPlantTypeBlock", true),
    entry!("BaseCoralWallFanBlock", true),
    entry!("BaseEntityBlock", false),
    entry!("BaseFireBlock", false),
    entry!("BasePressurePlateBlock", false),
    entry!("BaseRailBlock", false),
    entry!("BaseTorchBlock", false),
    entry!("BeaconBeamBlock", true),
    entry!("BeaconBlock", true),
    entry!("BedBlock", true),
    entry!("BeehiveBlock", true),
    entry!("BeetrootBlock", true),
    entry!("BellBlock", true),
    entry!("BigDripleafBlock", true),
    entry!("BigDripleafStemBlock", true),
    entry!("BlastFurnaceBlock", true),
    entry!("Block", false),
    entry!("BlockTypes", false),
    entry!("Blocks", false),
    entry!("BonemealableBlock", true),
    entry!("BonemealableFeaturePlacerBlock", true),
    entry!("BrewingStandBlock", true),
    entry!("BrushableBlock", true),
    entry!("BubbleColumnBlock", true),
    entry!("BucketPickup", false),
    entry!("BuddingAmethystBlock", true),
    entry!("BushBlock", true),
    entry!("ButtonBlock", true),
    entry!("CactusBlock", true),
    entry!("CactusFlowerBlock", true),
    entry!("CakeBlock", true),
    entry!("CalibratedSculkSensorBlock", true),
    entry!("CampfireBlock", true),
    entry!("CandleBlock", true),
    entry!("CandleCakeBlock", true),
    entry!("CarpetBlock", true),
    entry!("CarrotBlock", true),
    entry!("CartographyTableBlock", true),
    entry!("CarvedPumpkinBlock", true),
    entry!("CauldronBlock", true),
    entry!("CaveVines", false),
    entry!("CaveVinesBlock", true),
    entry!("CaveVinesPlantBlock", true),
    entry!("CeilingHangingSignBlock", true),
    entry!("ChainBlock", true),
    entry!("ChangeOverTimeBlock", true),
    entry!("ChestBlock", true),
    entry!("ChiseledBookShelfBlock", true),
    entry!("ChorusFlowerBlock", true),
    entry!("ChorusPlantBlock", true),
    entry!("CocoaBlock", true),
    entry!("ColoredFallingBlock", true),
    entry!("CommandBlock", true),
    entry!("ComparatorBlock", true),
    entry!("ComposterBlock", true),
    entry!("ConcretePowderBlock", true),
    entry!("ConduitBlock", true),
    entry!("CopperBulbBlock", true),
    entry!("CopperChestBlock", true),
    entry!("CopperGolemStatueBlock", true),
    entry!("CoralBlock", true),
    entry!("CoralFanBlock", true),
    entry!("CoralPlantBlock", true),
    entry!("CoralWallFanBlock", true),
    entry!("CrafterBlock", true),
    entry!("CraftingTableBlock", true),
    entry!("CreakingHeartBlock", true),
    entry!("CropBlock", true),
    entry!("CrossCollisionBlock", true),
    entry!("CryingObsidianBlock", true),
    entry!("DaylightDetectorBlock", true),
    entry!("DecoratedPotBlock", true),
    entry!("DetectorRailBlock", true),
    entry!("DiodeBlock", true),
    entry!("DirectionalBlock", true),
    entry!("DirtPathBlock", true),
    entry!("DispenserBlock", true),
    entry!("DoorBlock", true),
    entry!("DoubleBlockCombiner", false),
    entry!("DoublePlantBlock", true),
    entry!("DragonEggBlock", true),
    entry!("DriedGhastBlock", true),
    entry!("DropExperienceBlock", true),
    entry!("DropperBlock", true),
    entry!("DryVegetationBlock", true),
    entry!("EnchantingTableBlock", true),
    entry!("EndGatewayBlock", true),
    entry!("EndPortalBlock", true),
    entry!("EndPortalFrameBlock", true),
    entry!("EndRodBlock", true),
    entry!("EnderChestBlock", true),
    entry!("EntityBlock", true),
    entry!("EyeblossomBlock", true),
    entry!("FaceAttachedHorizontalDirectionalBlock", true),
    entry!("Fallable", false),
    entry!("FallingBlock", true),
    entry!("FarmlandBlock", true),
    entry!("FenceBlock", true),
    entry!("FenceGateBlock", true),
    entry!("FireBlock", true),
    entry!("FireflyBushBlock", true),
    entry!("FlowerBedBlock", true),
    entry!("FlowerBlock", true),
    entry!("FlowerPotBlock", true),
    entry!("FrogspawnBlock", true),
    entry!("FrostedIceBlock", true),
    entry!("FurnaceBlock", true),
    entry!("GameMasterBlock", true),
    entry!("GlazedTerracottaBlock", true),
    entry!("GlowLichenBlock", true),
    entry!("GrassBlock", true),
    entry!("GrindstoneBlock", true),
    entry!("GrowingPlantBlock", true),
    entry!("GrowingPlantBodyBlock", true),
    entry!("GrowingPlantHeadBlock", true),
    entry!("HalfTransparentBlock", true),
    entry!("HangingMossBlock", true),
    entry!("HangingRootsBlock", true),
    entry!("HangingSignBlock", true),
    entry!("HayBlock", true),
    entry!("HeavyCoreBlock", true),
    entry!("HoneyBlock", true),
    entry!("HopperBlock", true),
    entry!("HorizontalDirectionalBlock", true),
    entry!("HugeMushroomBlock", true),
    entry!("IceBlock", true),
    entry!("InfestedBlock", true),
    entry!("InfestedRotatedPillarBlock", true),
    entry!("IronBarsBlock", true),
    entry!("JigsawBlock", true),
    entry!("JukeboxBlock", true),
    entry!("KelpBlock", true),
    entry!("KelpPlantBlock", true),
    entry!("LadderBlock", true),
    entry!("LanternBlock", true),
    entry!("LavaCauldronBlock", true),
    entry!("LayeredCauldronBlock", true),
    entry!("LeafLitterBlock", true),
    entry!("LeavesBlock", true),
    entry!("LecternBlock", true),
    entry!("LevelEvent", false),
    entry!("LeverBlock", true),
    entry!("LightBlock", true),
    entry!("LightningRodBlock", true),
    entry!("LilyPadBlock", true),
    entry!("LiquidBlock", true),
    entry!("LiquidBlockContainer", false),
    entry!("LoomBlock", true),
    entry!("MagmaBlock", true),
    entry!("MangroveLeavesBlock", true),
    entry!("MangrovePropaguleBlock", true),
    entry!("MangroveRootsBlock", true),
    entry!("Mirror", false),
    entry!("MossyCarpetBlock", true),
    entry!("MudBlock", true),
    entry!("MultifaceBlock", true),
    entry!("MultifaceSpreadeableBlock", true),
    entry!("MultifaceSpreader", false),
    entry!("MushroomBlock", true),
    entry!("MyceliumBlock", true),
    entry!("NetherFungusBlock", true),
    entry!("NetherPortalBlock", true),
    entry!("NetherRootsBlock", true),
    entry!("NetherSproutsBlock", true),
    entry!("NetherVines", false),
    entry!("NetherWartBlock", true),
    entry!("NetherrackBlock", true),
    entry!("NoteBlock", true),
    entry!("NyliumBlock", true),
    entry!("ObserverBlock", true),
    entry!("PiglinWallSkullBlock", true),
    entry!("PipeBlock", true),
    entry!("PitcherCropBlock", true),
    entry!("PlainSignBlock", true),
    entry!("PlayerHeadBlock", true),
    entry!("PlayerWallHeadBlock", true),
    entry!("PointedDripstoneBlock", true),
    entry!("Portal", false),
    entry!("PotatoBlock", true),
    entry!("PowderSnowBlock", true),
    entry!("PoweredBlock", true),
    entry!("PoweredRailBlock", true),
    entry!("PressurePlateBlock", true),
    entry!("PumpkinBlock", true),
    entry!("RailBlock", true),
    entry!("RailState", false),
    entry!("RedStoneOreBlock", true),
    entry!("RedStoneWireBlock", true),
    entry!("RedstoneLampBlock", true),
    entry!("RedstoneTorchBlock", true),
    entry!("RedstoneWallTorchBlock", true),
    entry!("RenderShape", false),
    entry!("RepeaterBlock", true),
    entry!("RespawnAnchorBlock", true),
    entry!("RodBlock", true),
    entry!("RootedDirtBlock", true),
    entry!("RotatedPillarBlock", true),
    entry!("Rotation", false),
    entry!("SandBlock", true),
    entry!("SaplingBlock", true),
    entry!("ScaffoldingBlock", true),
    entry!("SculkBehaviour", false),
    entry!("SculkBlock", true),
    entry!("SculkCatalystBlock", true),
    entry!("SculkSensorBlock", true),
    entry!("SculkShriekerBlock", true),
    entry!("SculkSpreader", false),
    entry!("SculkVeinBlock", true),
    entry!("SeaPickleBlock", true),
    entry!("SeagrassBlock", true),
    entry!("SegmentableBlock", true),
    entry!("SelectableSlotContainer", false),
    entry!("ShelfBlock", true),
    entry!("ShortDryGrassBlock", true),
    entry!("ShulkerBoxBlock", true),
    entry!("SideChainPartBlock", true),
    entry!("SignBlock", true),
    entry!("SimpleWaterloggedBlock", true),
    entry!("SkullBlock", true),
    entry!("SlabBlock", true),
    entry!("SlimeBlock", true),
    entry!("SmallDripleafBlock", true),
    entry!("SmithingTableBlock", true),
    entry!("SmokerBlock", true),
    entry!("SnifferEggBlock", true),
    entry!("SnowLayerBlock", true),
    entry!("SnowyBlock", true),
    entry!("SoulFireBlock", true),
    entry!("SoulSandBlock", true),
    entry!("SoundType", false),
    entry!("SpawnerBlock", true),
    entry!("SpongeBlock", true),
    entry!("SporeBlossomBlock", true),
    entry!("SpreadingSnowyBlock", true),
    entry!("StainedGlassBlock", true),
    entry!("StainedGlassPaneBlock", true),
    entry!("StairBlock", true),
    entry!("StandingSignBlock", true),
    entry!("StemBlock", true),
    entry!("StonecutterBlock", true),
    entry!("StructureBlock", true),
    entry!("StructureVoidBlock", true),
    entry!("SugarCaneBlock", true),
    entry!("SupportType", false),
    entry!("SuspiciousEffectHolder", false),
    entry!("SweetBerryBushBlock", true),
    entry!("TallDryGrassBlock", true),
    entry!("TallFlowerBlock", true),
    entry!("TallGrassBlock", true),
    entry!("TallSeagrassBlock", true),
    entry!("TargetBlock", true),
    entry!("TestBlock", true),
    entry!("TestInstanceBlock", true),
    entry!("TintedGlassBlock", true),
    entry!("TintedParticleLeavesBlock", true),
    entry!("TntBlock", true),
    entry!("TorchBlock", true),
    entry!("TorchflowerCropBlock", true),
    entry!("TransparentBlock", true),
    entry!("TrapDoorBlock", true),
    entry!("TrappedChestBlock", true),
    entry!("TrialSpawnerBlock", true),
    entry!("TripWireBlock", true),
    entry!("TripWireHookBlock", true),
    entry!("TurtleEggBlock", true),
    entry!("TwistingVinesBlock", true),
    entry!("TwistingVinesPlantBlock", true),
    entry!("UntintedParticleLeavesBlock", true),
    entry!("VaultBlock", true),
    entry!("VegetationBlock", true),
    entry!("VineBlock", true),
    entry!("WallBannerBlock", true),
    entry!("WallBlock", true),
    entry!("WallHangingSignBlock", true),
    entry!("WallSignBlock", true),
    entry!("WallSkullBlock", true),
    entry!("WallTorchBlock", true),
    entry!("WaterloggedTransparentBlock", true),
    entry!("WeatheringCopper", false),
    entry!("WeatheringCopperBarsBlock", true),
    entry!("WeatheringCopperBlocks", false),
    entry!("WeatheringCopperBulbBlock", true),
    entry!("WeatheringCopperChainBlock", true),
    entry!("WeatheringCopperChestBlock", true),
    entry!("WeatheringCopperDoorBlock", true),
    entry!("WeatheringCopperFullBlock", true),
    entry!("WeatheringCopperGolemStatueBlock", true),
    entry!("WeatheringCopperGrateBlock", true),
    entry!("WeatheringCopperSlabBlock", true),
    entry!("WeatheringCopperStairBlock", true),
    entry!("WeatheringCopperTrapDoorBlock", true),
    entry!("WeatheringLanternBlock", true),
    entry!("WeatheringLightningRodBlock", true),
    entry!("WebBlock", true),
    entry!("WeepingVinesBlock", true),
    entry!("WeepingVinesPlantBlock", true),
    entry!("WeightedPressurePlateBlock", true),
    entry!("WetSpongeBlock", true),
    entry!("WitherRoseBlock", true),
    entry!("WitherSkullBlock", true),
    entry!("WitherWallSkullBlock", true),
    entry!("WoolCarpetBlock", true),
    entry!("package-info", false),
];

pub fn block_class_by_name(class_name: &str) -> Option<&'static BlockClassEntry> {
    TOP_LEVEL_BLOCK_CLASSES
        .iter()
        .find(|entry| entry.class_name == class_name)
}

pub fn concrete_block_class_count() -> usize {
    TOP_LEVEL_BLOCK_CLASSES
        .iter()
        .filter(|entry| entry.concrete_block_class)
        .count()
}

#[cfg(test)]
mod tests {
    use super::{
        block_class_by_name, concrete_block_class_count, TOP_LEVEL_BLOCK_CLASSES,
        VANILLA_BLOCK_PACKAGE, VANILLA_TOP_LEVEL_BLOCK_CLASS_COUNT,
    };
    use std::collections::BTreeSet;

    #[test]
    fn top_level_block_class_catalog_matches_decompiled_package_count() {
        assert_eq!(VANILLA_TOP_LEVEL_BLOCK_CLASS_COUNT, 322);
        assert_eq!(
            TOP_LEVEL_BLOCK_CLASSES.len(),
            VANILLA_TOP_LEVEL_BLOCK_CLASS_COUNT
        );
        assert!(TOP_LEVEL_BLOCK_CLASSES
            .iter()
            .all(|entry| entry.package == VANILLA_BLOCK_PACKAGE));
    }

    #[test]
    fn top_level_block_class_catalog_is_sorted_and_unique() {
        let names = TOP_LEVEL_BLOCK_CLASSES
            .iter()
            .map(|entry| entry.class_name)
            .collect::<Vec<_>>();
        let unique = names.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(unique.len(), names.len());
        assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn catalog_covers_representative_vanilla_block_families() {
        for class_name in [
            "AirBlock",
            "BedBlock",
            "ChestBlock",
            "ComparatorBlock",
            "NetherPortalBlock",
            "ObserverBlock",
            "RedStoneWireBlock",
            "SculkSensorBlock",
            "TrialSpawnerBlock",
            "LilyPadBlock",
        ] {
            assert!(
                block_class_by_name(class_name).is_some(),
                "missing {class_name}"
            );
        }
    }

    #[test]
    fn catalog_distinguishes_concrete_block_classes_from_package_support_types() {
        assert_eq!(concrete_block_class_count(), 286);
        assert!(
            block_class_by_name("ChestBlock")
                .unwrap()
                .concrete_block_class
        );
        assert!(!block_class_by_name("Block").unwrap().concrete_block_class);
        assert!(
            !block_class_by_name("AbstractChestBlock")
                .unwrap()
                .concrete_block_class
        );
    }
}
