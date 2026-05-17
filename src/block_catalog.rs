#![allow(dead_code)]
pub const VANILLA_BLOCK_PACKAGE: &str = "net/minecraft/world/level/block";
pub const VANILLA_TOP_LEVEL_BLOCK_CLASS_COUNT: usize = 322;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockClassEntry {
    pub class_name: &'static str,
    pub package: &'static str,
    pub concrete_block_class: bool,
}

pub const TOP_LEVEL_BLOCK_CLASSES: &[BlockClassEntry] = &[
    BlockClassEntry {
        class_name: "AbstractBannerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "AbstractCandleBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "AbstractCauldronBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "AbstractChestBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "AbstractFurnaceBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "AbstractSkullBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "AirBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "AmethystBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "AmethystClusterBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "AnvilBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "AttachedStemBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "AzaleaBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BambooSaplingBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BambooStalkBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BannerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BarrelBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BarrierBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BaseCoralFanBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BaseCoralPlantBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BaseCoralPlantTypeBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BaseCoralWallFanBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BaseEntityBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "BaseFireBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "BasePressurePlateBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "BaseRailBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "BaseTorchBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "BeaconBeamBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BeaconBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BedBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BeehiveBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BeetrootBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BellBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BigDripleafBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BigDripleafStemBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BlastFurnaceBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "Block",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "BlockTypes",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "Blocks",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "BonemealableBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BonemealableFeaturePlacerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BrewingStandBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BrushableBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BubbleColumnBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BucketPickup",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "BuddingAmethystBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "BushBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ButtonBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CactusBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CactusFlowerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CakeBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CalibratedSculkSensorBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CampfireBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CandleBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CandleCakeBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CarpetBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CarrotBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CartographyTableBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CarvedPumpkinBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CauldronBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CaveVines",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "CaveVinesBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CaveVinesPlantBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CeilingHangingSignBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ChainBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ChangeOverTimeBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ChestBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ChiseledBookShelfBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ChorusFlowerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ChorusPlantBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CocoaBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ColoredFallingBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CommandBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ComparatorBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ComposterBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ConcretePowderBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ConduitBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CopperBulbBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CopperChestBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CopperGolemStatueBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CoralBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CoralFanBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CoralPlantBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CoralWallFanBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CrafterBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CraftingTableBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CreakingHeartBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CropBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CrossCollisionBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "CryingObsidianBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DaylightDetectorBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DecoratedPotBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DetectorRailBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DiodeBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DirectionalBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DirtPathBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DispenserBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DoorBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DoubleBlockCombiner",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "DoublePlantBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DragonEggBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DriedGhastBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DropExperienceBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DropperBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "DryVegetationBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "EnchantingTableBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "EndGatewayBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "EndPortalBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "EndPortalFrameBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "EndRodBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "EnderChestBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "EntityBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "EyeblossomBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FaceAttachedHorizontalDirectionalBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "Fallable",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "FallingBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FarmlandBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FenceBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FenceGateBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FireBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FireflyBushBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FlowerBedBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FlowerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FlowerPotBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FrogspawnBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FrostedIceBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "FurnaceBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "GameMasterBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "GlazedTerracottaBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "GlowLichenBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "GrassBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "GrindstoneBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "GrowingPlantBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "GrowingPlantBodyBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "GrowingPlantHeadBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "HalfTransparentBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "HangingMossBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "HangingRootsBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "HangingSignBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "HayBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "HeavyCoreBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "HoneyBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "HopperBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "HorizontalDirectionalBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "HugeMushroomBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "IceBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "InfestedBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "InfestedRotatedPillarBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "IronBarsBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "JigsawBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "JukeboxBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "KelpBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "KelpPlantBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LadderBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LanternBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LavaCauldronBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LayeredCauldronBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LeafLitterBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LeavesBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LecternBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LevelEvent",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "LeverBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LightBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LightningRodBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LilyPadBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LiquidBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "LiquidBlockContainer",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "LoomBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "MagmaBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "MangroveLeavesBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "MangrovePropaguleBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "MangroveRootsBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "Mirror",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "MossyCarpetBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "MudBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "MultifaceBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "MultifaceSpreadeableBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "MultifaceSpreader",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "MushroomBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "MyceliumBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "NetherFungusBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "NetherPortalBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "NetherRootsBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "NetherSproutsBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "NetherVines",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "NetherWartBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "NetherrackBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "NoteBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "NyliumBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ObserverBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PiglinWallSkullBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PipeBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PitcherCropBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PlainSignBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PlayerHeadBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PlayerWallHeadBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PointedDripstoneBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "Portal",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "PotatoBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PowderSnowBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PoweredBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PoweredRailBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PressurePlateBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "PumpkinBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "RailBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "RailState",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "RedStoneOreBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "RedStoneWireBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "RedstoneLampBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "RedstoneTorchBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "RedstoneWallTorchBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "RenderShape",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "RepeaterBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "RespawnAnchorBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "RodBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "RootedDirtBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "RotatedPillarBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "Rotation",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "SandBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SaplingBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ScaffoldingBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SculkBehaviour",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "SculkBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SculkCatalystBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SculkSensorBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SculkShriekerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SculkSpreader",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "SculkVeinBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SeaPickleBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SeagrassBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SegmentableBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SelectableSlotContainer",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "ShelfBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ShortDryGrassBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "ShulkerBoxBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SideChainPartBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SignBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SimpleWaterloggedBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SkullBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SlabBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SlimeBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SmallDripleafBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SmithingTableBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SmokerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SnifferEggBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SnowLayerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SnowyBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SoulFireBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SoulSandBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SoundType",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "SpawnerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SpongeBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SporeBlossomBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SpreadingSnowyBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "StainedGlassBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "StainedGlassPaneBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "StairBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "StandingSignBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "StemBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "StonecutterBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "StructureBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "StructureVoidBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SugarCaneBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "SupportType",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "SuspiciousEffectHolder",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "SweetBerryBushBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TallDryGrassBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TallFlowerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TallGrassBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TallSeagrassBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TargetBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TestBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TestInstanceBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TintedGlassBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TintedParticleLeavesBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TntBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TorchBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TorchflowerCropBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TransparentBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TrapDoorBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TrappedChestBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TrialSpawnerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TripWireBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TripWireHookBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TurtleEggBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TwistingVinesBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "TwistingVinesPlantBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "UntintedParticleLeavesBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "VaultBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "VegetationBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "VineBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WallBannerBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WallBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WallHangingSignBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WallSignBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WallSkullBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WallTorchBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WaterloggedTransparentBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringCopper",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperBarsBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperBlocks",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperBulbBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperChainBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperChestBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperDoorBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperFullBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperGolemStatueBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperGrateBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperSlabBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperStairBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringCopperTrapDoorBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringLanternBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeatheringLightningRodBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WebBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeepingVinesBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeepingVinesPlantBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WeightedPressurePlateBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WetSpongeBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WitherRoseBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WitherSkullBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WitherWallSkullBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "WoolCarpetBlock",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: true,
    },
    BlockClassEntry {
        class_name: "package-info",
        package: VANILLA_BLOCK_PACKAGE,
        concrete_block_class: false,
    },
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
