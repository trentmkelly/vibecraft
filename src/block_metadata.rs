#![allow(dead_code)]

use std::collections::BTreeMap;

pub const VANILLA_BLOCK_REGISTRY_COUNT: usize = 1144;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRegistryEntry {
    pub numeric_id: usize,
    pub field_name: &'static str,
    pub registry_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Empty,
    FullCube,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockPhysicalProperties {
    pub map_color: &'static str,
    pub sound_type: &'static str,
    pub destroy_time: f32,
    pub explosion_resistance: f32,
    pub has_collision: bool,
    pub occludes: bool,
    pub light_emission: u8,
    pub pathfind_land: bool,
    pub pathfind_air: bool,
    pub can_survive_without_support: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPropertyDefinition {
    pub name: &'static str,
    pub values: &'static [&'static str],
    pub default_value: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStateDefinition {
    pub registry_id: &'static str,
    pub properties: &'static [BlockPropertyDefinition],
    pub physical: BlockPhysicalProperties,
    pub collision_shape: ShapeKind,
    pub occlusion_shape: ShapeKind,
}

pub const BLOCK_REGISTRY: &[BlockRegistryEntry] = &[
    BlockRegistryEntry {
        numeric_id: 0,
        field_name: "AIR",
        registry_id: "minecraft:air",
    },
    BlockRegistryEntry {
        numeric_id: 1,
        field_name: "STONE",
        registry_id: "minecraft:stone",
    },
    BlockRegistryEntry {
        numeric_id: 2,
        field_name: "GRANITE",
        registry_id: "minecraft:granite",
    },
    BlockRegistryEntry {
        numeric_id: 3,
        field_name: "POLISHED_GRANITE",
        registry_id: "minecraft:polished_granite",
    },
    BlockRegistryEntry {
        numeric_id: 4,
        field_name: "DIORITE",
        registry_id: "minecraft:diorite",
    },
    BlockRegistryEntry {
        numeric_id: 5,
        field_name: "POLISHED_DIORITE",
        registry_id: "minecraft:polished_diorite",
    },
    BlockRegistryEntry {
        numeric_id: 6,
        field_name: "ANDESITE",
        registry_id: "minecraft:andesite",
    },
    BlockRegistryEntry {
        numeric_id: 7,
        field_name: "POLISHED_ANDESITE",
        registry_id: "minecraft:polished_andesite",
    },
    BlockRegistryEntry {
        numeric_id: 8,
        field_name: "GRASS_BLOCK",
        registry_id: "minecraft:grass_block",
    },
    BlockRegistryEntry {
        numeric_id: 9,
        field_name: "DIRT",
        registry_id: "minecraft:dirt",
    },
    BlockRegistryEntry {
        numeric_id: 10,
        field_name: "COARSE_DIRT",
        registry_id: "minecraft:coarse_dirt",
    },
    BlockRegistryEntry {
        numeric_id: 11,
        field_name: "PODZOL",
        registry_id: "minecraft:podzol",
    },
    BlockRegistryEntry {
        numeric_id: 12,
        field_name: "COBBLESTONE",
        registry_id: "minecraft:cobblestone",
    },
    BlockRegistryEntry {
        numeric_id: 13,
        field_name: "OAK_PLANKS",
        registry_id: "minecraft:oak_planks",
    },
    BlockRegistryEntry {
        numeric_id: 14,
        field_name: "SPRUCE_PLANKS",
        registry_id: "minecraft:spruce_planks",
    },
    BlockRegistryEntry {
        numeric_id: 15,
        field_name: "BIRCH_PLANKS",
        registry_id: "minecraft:birch_planks",
    },
    BlockRegistryEntry {
        numeric_id: 16,
        field_name: "JUNGLE_PLANKS",
        registry_id: "minecraft:jungle_planks",
    },
    BlockRegistryEntry {
        numeric_id: 17,
        field_name: "ACACIA_PLANKS",
        registry_id: "minecraft:acacia_planks",
    },
    BlockRegistryEntry {
        numeric_id: 18,
        field_name: "CHERRY_PLANKS",
        registry_id: "minecraft:cherry_planks",
    },
    BlockRegistryEntry {
        numeric_id: 19,
        field_name: "DARK_OAK_PLANKS",
        registry_id: "minecraft:dark_oak_planks",
    },
    BlockRegistryEntry {
        numeric_id: 20,
        field_name: "PALE_OAK_WOOD",
        registry_id: "minecraft:pale_oak_wood",
    },
    BlockRegistryEntry {
        numeric_id: 21,
        field_name: "PALE_OAK_PLANKS",
        registry_id: "minecraft:pale_oak_planks",
    },
    BlockRegistryEntry {
        numeric_id: 22,
        field_name: "MANGROVE_PLANKS",
        registry_id: "minecraft:mangrove_planks",
    },
    BlockRegistryEntry {
        numeric_id: 23,
        field_name: "BAMBOO_PLANKS",
        registry_id: "minecraft:bamboo_planks",
    },
    BlockRegistryEntry {
        numeric_id: 24,
        field_name: "BAMBOO_MOSAIC",
        registry_id: "minecraft:bamboo_mosaic",
    },
    BlockRegistryEntry {
        numeric_id: 25,
        field_name: "OAK_SAPLING",
        registry_id: "minecraft:oak_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 26,
        field_name: "SPRUCE_SAPLING",
        registry_id: "minecraft:spruce_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 27,
        field_name: "BIRCH_SAPLING",
        registry_id: "minecraft:birch_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 28,
        field_name: "JUNGLE_SAPLING",
        registry_id: "minecraft:jungle_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 29,
        field_name: "ACACIA_SAPLING",
        registry_id: "minecraft:acacia_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 30,
        field_name: "CHERRY_SAPLING",
        registry_id: "minecraft:cherry_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 31,
        field_name: "DARK_OAK_SAPLING",
        registry_id: "minecraft:dark_oak_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 32,
        field_name: "PALE_OAK_SAPLING",
        registry_id: "minecraft:pale_oak_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 33,
        field_name: "MANGROVE_PROPAGULE",
        registry_id: "minecraft:mangrove_propagule",
    },
    BlockRegistryEntry {
        numeric_id: 34,
        field_name: "BEDROCK",
        registry_id: "minecraft:bedrock",
    },
    BlockRegistryEntry {
        numeric_id: 35,
        field_name: "WATER",
        registry_id: "minecraft:water",
    },
    BlockRegistryEntry {
        numeric_id: 36,
        field_name: "LAVA",
        registry_id: "minecraft:lava",
    },
    BlockRegistryEntry {
        numeric_id: 37,
        field_name: "SAND",
        registry_id: "minecraft:sand",
    },
    BlockRegistryEntry {
        numeric_id: 38,
        field_name: "SUSPICIOUS_SAND",
        registry_id: "minecraft:suspicious_sand",
    },
    BlockRegistryEntry {
        numeric_id: 39,
        field_name: "RED_SAND",
        registry_id: "minecraft:red_sand",
    },
    BlockRegistryEntry {
        numeric_id: 40,
        field_name: "GRAVEL",
        registry_id: "minecraft:gravel",
    },
    BlockRegistryEntry {
        numeric_id: 41,
        field_name: "SUSPICIOUS_GRAVEL",
        registry_id: "minecraft:suspicious_gravel",
    },
    BlockRegistryEntry {
        numeric_id: 42,
        field_name: "GOLD_ORE",
        registry_id: "minecraft:gold_ore",
    },
    BlockRegistryEntry {
        numeric_id: 43,
        field_name: "DEEPSLATE_GOLD_ORE",
        registry_id: "minecraft:deepslate_gold_ore",
    },
    BlockRegistryEntry {
        numeric_id: 44,
        field_name: "IRON_ORE",
        registry_id: "minecraft:iron_ore",
    },
    BlockRegistryEntry {
        numeric_id: 45,
        field_name: "DEEPSLATE_IRON_ORE",
        registry_id: "minecraft:deepslate_iron_ore",
    },
    BlockRegistryEntry {
        numeric_id: 46,
        field_name: "COAL_ORE",
        registry_id: "minecraft:coal_ore",
    },
    BlockRegistryEntry {
        numeric_id: 47,
        field_name: "DEEPSLATE_COAL_ORE",
        registry_id: "minecraft:deepslate_coal_ore",
    },
    BlockRegistryEntry {
        numeric_id: 48,
        field_name: "NETHER_GOLD_ORE",
        registry_id: "minecraft:nether_gold_ore",
    },
    BlockRegistryEntry {
        numeric_id: 49,
        field_name: "OAK_LOG",
        registry_id: "minecraft:oak_log",
    },
    BlockRegistryEntry {
        numeric_id: 50,
        field_name: "SPRUCE_LOG",
        registry_id: "minecraft:spruce_log",
    },
    BlockRegistryEntry {
        numeric_id: 51,
        field_name: "BIRCH_LOG",
        registry_id: "minecraft:birch_log",
    },
    BlockRegistryEntry {
        numeric_id: 52,
        field_name: "JUNGLE_LOG",
        registry_id: "minecraft:jungle_log",
    },
    BlockRegistryEntry {
        numeric_id: 53,
        field_name: "ACACIA_LOG",
        registry_id: "minecraft:acacia_log",
    },
    BlockRegistryEntry {
        numeric_id: 54,
        field_name: "CHERRY_LOG",
        registry_id: "minecraft:cherry_log",
    },
    BlockRegistryEntry {
        numeric_id: 55,
        field_name: "DARK_OAK_LOG",
        registry_id: "minecraft:dark_oak_log",
    },
    BlockRegistryEntry {
        numeric_id: 56,
        field_name: "PALE_OAK_LOG",
        registry_id: "minecraft:pale_oak_log",
    },
    BlockRegistryEntry {
        numeric_id: 57,
        field_name: "MANGROVE_LOG",
        registry_id: "minecraft:mangrove_log",
    },
    BlockRegistryEntry {
        numeric_id: 58,
        field_name: "MANGROVE_ROOTS",
        registry_id: "minecraft:mangrove_roots",
    },
    BlockRegistryEntry {
        numeric_id: 59,
        field_name: "MUDDY_MANGROVE_ROOTS",
        registry_id: "minecraft:muddy_mangrove_roots",
    },
    BlockRegistryEntry {
        numeric_id: 60,
        field_name: "BAMBOO_BLOCK",
        registry_id: "minecraft:bamboo_block",
    },
    BlockRegistryEntry {
        numeric_id: 61,
        field_name: "STRIPPED_SPRUCE_LOG",
        registry_id: "minecraft:stripped_spruce_log",
    },
    BlockRegistryEntry {
        numeric_id: 62,
        field_name: "STRIPPED_BIRCH_LOG",
        registry_id: "minecraft:stripped_birch_log",
    },
    BlockRegistryEntry {
        numeric_id: 63,
        field_name: "STRIPPED_JUNGLE_LOG",
        registry_id: "minecraft:stripped_jungle_log",
    },
    BlockRegistryEntry {
        numeric_id: 64,
        field_name: "STRIPPED_ACACIA_LOG",
        registry_id: "minecraft:stripped_acacia_log",
    },
    BlockRegistryEntry {
        numeric_id: 65,
        field_name: "STRIPPED_CHERRY_LOG",
        registry_id: "minecraft:stripped_cherry_log",
    },
    BlockRegistryEntry {
        numeric_id: 66,
        field_name: "STRIPPED_DARK_OAK_LOG",
        registry_id: "minecraft:stripped_dark_oak_log",
    },
    BlockRegistryEntry {
        numeric_id: 67,
        field_name: "STRIPPED_PALE_OAK_LOG",
        registry_id: "minecraft:stripped_pale_oak_log",
    },
    BlockRegistryEntry {
        numeric_id: 68,
        field_name: "STRIPPED_OAK_LOG",
        registry_id: "minecraft:stripped_oak_log",
    },
    BlockRegistryEntry {
        numeric_id: 69,
        field_name: "STRIPPED_MANGROVE_LOG",
        registry_id: "minecraft:stripped_mangrove_log",
    },
    BlockRegistryEntry {
        numeric_id: 70,
        field_name: "STRIPPED_BAMBOO_BLOCK",
        registry_id: "minecraft:stripped_bamboo_block",
    },
    BlockRegistryEntry {
        numeric_id: 71,
        field_name: "OAK_WOOD",
        registry_id: "minecraft:oak_wood",
    },
    BlockRegistryEntry {
        numeric_id: 72,
        field_name: "SPRUCE_WOOD",
        registry_id: "minecraft:spruce_wood",
    },
    BlockRegistryEntry {
        numeric_id: 73,
        field_name: "BIRCH_WOOD",
        registry_id: "minecraft:birch_wood",
    },
    BlockRegistryEntry {
        numeric_id: 74,
        field_name: "JUNGLE_WOOD",
        registry_id: "minecraft:jungle_wood",
    },
    BlockRegistryEntry {
        numeric_id: 75,
        field_name: "ACACIA_WOOD",
        registry_id: "minecraft:acacia_wood",
    },
    BlockRegistryEntry {
        numeric_id: 76,
        field_name: "CHERRY_WOOD",
        registry_id: "minecraft:cherry_wood",
    },
    BlockRegistryEntry {
        numeric_id: 77,
        field_name: "DARK_OAK_WOOD",
        registry_id: "minecraft:dark_oak_wood",
    },
    BlockRegistryEntry {
        numeric_id: 78,
        field_name: "MANGROVE_WOOD",
        registry_id: "minecraft:mangrove_wood",
    },
    BlockRegistryEntry {
        numeric_id: 79,
        field_name: "STRIPPED_OAK_WOOD",
        registry_id: "minecraft:stripped_oak_wood",
    },
    BlockRegistryEntry {
        numeric_id: 80,
        field_name: "STRIPPED_SPRUCE_WOOD",
        registry_id: "minecraft:stripped_spruce_wood",
    },
    BlockRegistryEntry {
        numeric_id: 81,
        field_name: "STRIPPED_BIRCH_WOOD",
        registry_id: "minecraft:stripped_birch_wood",
    },
    BlockRegistryEntry {
        numeric_id: 82,
        field_name: "STRIPPED_JUNGLE_WOOD",
        registry_id: "minecraft:stripped_jungle_wood",
    },
    BlockRegistryEntry {
        numeric_id: 83,
        field_name: "STRIPPED_ACACIA_WOOD",
        registry_id: "minecraft:stripped_acacia_wood",
    },
    BlockRegistryEntry {
        numeric_id: 84,
        field_name: "STRIPPED_CHERRY_WOOD",
        registry_id: "minecraft:stripped_cherry_wood",
    },
    BlockRegistryEntry {
        numeric_id: 85,
        field_name: "STRIPPED_DARK_OAK_WOOD",
        registry_id: "minecraft:stripped_dark_oak_wood",
    },
    BlockRegistryEntry {
        numeric_id: 86,
        field_name: "STRIPPED_PALE_OAK_WOOD",
        registry_id: "minecraft:stripped_pale_oak_wood",
    },
    BlockRegistryEntry {
        numeric_id: 87,
        field_name: "STRIPPED_MANGROVE_WOOD",
        registry_id: "minecraft:stripped_mangrove_wood",
    },
    BlockRegistryEntry {
        numeric_id: 88,
        field_name: "OAK_LEAVES",
        registry_id: "minecraft:oak_leaves",
    },
    BlockRegistryEntry {
        numeric_id: 89,
        field_name: "SPRUCE_LEAVES",
        registry_id: "minecraft:spruce_leaves",
    },
    BlockRegistryEntry {
        numeric_id: 90,
        field_name: "BIRCH_LEAVES",
        registry_id: "minecraft:birch_leaves",
    },
    BlockRegistryEntry {
        numeric_id: 91,
        field_name: "JUNGLE_LEAVES",
        registry_id: "minecraft:jungle_leaves",
    },
    BlockRegistryEntry {
        numeric_id: 92,
        field_name: "ACACIA_LEAVES",
        registry_id: "minecraft:acacia_leaves",
    },
    BlockRegistryEntry {
        numeric_id: 93,
        field_name: "CHERRY_LEAVES",
        registry_id: "minecraft:cherry_leaves",
    },
    BlockRegistryEntry {
        numeric_id: 94,
        field_name: "DARK_OAK_LEAVES",
        registry_id: "minecraft:dark_oak_leaves",
    },
    BlockRegistryEntry {
        numeric_id: 95,
        field_name: "PALE_OAK_LEAVES",
        registry_id: "minecraft:pale_oak_leaves",
    },
    BlockRegistryEntry {
        numeric_id: 96,
        field_name: "MANGROVE_LEAVES",
        registry_id: "minecraft:mangrove_leaves",
    },
    BlockRegistryEntry {
        numeric_id: 97,
        field_name: "AZALEA_LEAVES",
        registry_id: "minecraft:azalea_leaves",
    },
    BlockRegistryEntry {
        numeric_id: 98,
        field_name: "FLOWERING_AZALEA_LEAVES",
        registry_id: "minecraft:flowering_azalea_leaves",
    },
    BlockRegistryEntry {
        numeric_id: 99,
        field_name: "SPONGE",
        registry_id: "minecraft:sponge",
    },
    BlockRegistryEntry {
        numeric_id: 100,
        field_name: "WET_SPONGE",
        registry_id: "minecraft:wet_sponge",
    },
    BlockRegistryEntry {
        numeric_id: 101,
        field_name: "GLASS",
        registry_id: "minecraft:glass",
    },
    BlockRegistryEntry {
        numeric_id: 102,
        field_name: "LAPIS_ORE",
        registry_id: "minecraft:lapis_ore",
    },
    BlockRegistryEntry {
        numeric_id: 103,
        field_name: "DEEPSLATE_LAPIS_ORE",
        registry_id: "minecraft:deepslate_lapis_ore",
    },
    BlockRegistryEntry {
        numeric_id: 104,
        field_name: "LAPIS_BLOCK",
        registry_id: "minecraft:lapis_block",
    },
    BlockRegistryEntry {
        numeric_id: 105,
        field_name: "DISPENSER",
        registry_id: "minecraft:dispenser",
    },
    BlockRegistryEntry {
        numeric_id: 106,
        field_name: "SANDSTONE",
        registry_id: "minecraft:sandstone",
    },
    BlockRegistryEntry {
        numeric_id: 107,
        field_name: "CHISELED_SANDSTONE",
        registry_id: "minecraft:chiseled_sandstone",
    },
    BlockRegistryEntry {
        numeric_id: 108,
        field_name: "CUT_SANDSTONE",
        registry_id: "minecraft:cut_sandstone",
    },
    BlockRegistryEntry {
        numeric_id: 109,
        field_name: "NOTE_BLOCK",
        registry_id: "minecraft:note_block",
    },
    BlockRegistryEntry {
        numeric_id: 110,
        field_name: "WHITE_BED",
        registry_id: "minecraft:white_bed",
    },
    BlockRegistryEntry {
        numeric_id: 111,
        field_name: "ORANGE_BED",
        registry_id: "minecraft:orange_bed",
    },
    BlockRegistryEntry {
        numeric_id: 112,
        field_name: "MAGENTA_BED",
        registry_id: "minecraft:magenta_bed",
    },
    BlockRegistryEntry {
        numeric_id: 113,
        field_name: "LIGHT_BLUE_BED",
        registry_id: "minecraft:light_blue_bed",
    },
    BlockRegistryEntry {
        numeric_id: 114,
        field_name: "YELLOW_BED",
        registry_id: "minecraft:yellow_bed",
    },
    BlockRegistryEntry {
        numeric_id: 115,
        field_name: "LIME_BED",
        registry_id: "minecraft:lime_bed",
    },
    BlockRegistryEntry {
        numeric_id: 116,
        field_name: "PINK_BED",
        registry_id: "minecraft:pink_bed",
    },
    BlockRegistryEntry {
        numeric_id: 117,
        field_name: "GRAY_BED",
        registry_id: "minecraft:gray_bed",
    },
    BlockRegistryEntry {
        numeric_id: 118,
        field_name: "LIGHT_GRAY_BED",
        registry_id: "minecraft:light_gray_bed",
    },
    BlockRegistryEntry {
        numeric_id: 119,
        field_name: "CYAN_BED",
        registry_id: "minecraft:cyan_bed",
    },
    BlockRegistryEntry {
        numeric_id: 120,
        field_name: "PURPLE_BED",
        registry_id: "minecraft:purple_bed",
    },
    BlockRegistryEntry {
        numeric_id: 121,
        field_name: "BLUE_BED",
        registry_id: "minecraft:blue_bed",
    },
    BlockRegistryEntry {
        numeric_id: 122,
        field_name: "BROWN_BED",
        registry_id: "minecraft:brown_bed",
    },
    BlockRegistryEntry {
        numeric_id: 123,
        field_name: "GREEN_BED",
        registry_id: "minecraft:green_bed",
    },
    BlockRegistryEntry {
        numeric_id: 124,
        field_name: "RED_BED",
        registry_id: "minecraft:red_bed",
    },
    BlockRegistryEntry {
        numeric_id: 125,
        field_name: "BLACK_BED",
        registry_id: "minecraft:black_bed",
    },
    BlockRegistryEntry {
        numeric_id: 126,
        field_name: "POWERED_RAIL",
        registry_id: "minecraft:powered_rail",
    },
    BlockRegistryEntry {
        numeric_id: 127,
        field_name: "DETECTOR_RAIL",
        registry_id: "minecraft:detector_rail",
    },
    BlockRegistryEntry {
        numeric_id: 128,
        field_name: "STICKY_PISTON",
        registry_id: "minecraft:sticky_piston",
    },
    BlockRegistryEntry {
        numeric_id: 129,
        field_name: "COBWEB",
        registry_id: "minecraft:cobweb",
    },
    BlockRegistryEntry {
        numeric_id: 130,
        field_name: "SHORT_GRASS",
        registry_id: "minecraft:short_grass",
    },
    BlockRegistryEntry {
        numeric_id: 131,
        field_name: "FERN",
        registry_id: "minecraft:fern",
    },
    BlockRegistryEntry {
        numeric_id: 132,
        field_name: "DEAD_BUSH",
        registry_id: "minecraft:dead_bush",
    },
    BlockRegistryEntry {
        numeric_id: 133,
        field_name: "BUSH",
        registry_id: "minecraft:bush",
    },
    BlockRegistryEntry {
        numeric_id: 134,
        field_name: "SHORT_DRY_GRASS",
        registry_id: "minecraft:short_dry_grass",
    },
    BlockRegistryEntry {
        numeric_id: 135,
        field_name: "TALL_DRY_GRASS",
        registry_id: "minecraft:tall_dry_grass",
    },
    BlockRegistryEntry {
        numeric_id: 136,
        field_name: "SEAGRASS",
        registry_id: "minecraft:seagrass",
    },
    BlockRegistryEntry {
        numeric_id: 137,
        field_name: "TALL_SEAGRASS",
        registry_id: "minecraft:tall_seagrass",
    },
    BlockRegistryEntry {
        numeric_id: 138,
        field_name: "PISTON",
        registry_id: "minecraft:piston",
    },
    BlockRegistryEntry {
        numeric_id: 139,
        field_name: "PISTON_HEAD",
        registry_id: "minecraft:piston_head",
    },
    BlockRegistryEntry {
        numeric_id: 140,
        field_name: "WHITE_WOOL",
        registry_id: "minecraft:white_wool",
    },
    BlockRegistryEntry {
        numeric_id: 141,
        field_name: "ORANGE_WOOL",
        registry_id: "minecraft:orange_wool",
    },
    BlockRegistryEntry {
        numeric_id: 142,
        field_name: "MAGENTA_WOOL",
        registry_id: "minecraft:magenta_wool",
    },
    BlockRegistryEntry {
        numeric_id: 143,
        field_name: "LIGHT_BLUE_WOOL",
        registry_id: "minecraft:light_blue_wool",
    },
    BlockRegistryEntry {
        numeric_id: 144,
        field_name: "YELLOW_WOOL",
        registry_id: "minecraft:yellow_wool",
    },
    BlockRegistryEntry {
        numeric_id: 145,
        field_name: "LIME_WOOL",
        registry_id: "minecraft:lime_wool",
    },
    BlockRegistryEntry {
        numeric_id: 146,
        field_name: "PINK_WOOL",
        registry_id: "minecraft:pink_wool",
    },
    BlockRegistryEntry {
        numeric_id: 147,
        field_name: "GRAY_WOOL",
        registry_id: "minecraft:gray_wool",
    },
    BlockRegistryEntry {
        numeric_id: 148,
        field_name: "LIGHT_GRAY_WOOL",
        registry_id: "minecraft:light_gray_wool",
    },
    BlockRegistryEntry {
        numeric_id: 149,
        field_name: "CYAN_WOOL",
        registry_id: "minecraft:cyan_wool",
    },
    BlockRegistryEntry {
        numeric_id: 150,
        field_name: "PURPLE_WOOL",
        registry_id: "minecraft:purple_wool",
    },
    BlockRegistryEntry {
        numeric_id: 151,
        field_name: "BLUE_WOOL",
        registry_id: "minecraft:blue_wool",
    },
    BlockRegistryEntry {
        numeric_id: 152,
        field_name: "BROWN_WOOL",
        registry_id: "minecraft:brown_wool",
    },
    BlockRegistryEntry {
        numeric_id: 153,
        field_name: "GREEN_WOOL",
        registry_id: "minecraft:green_wool",
    },
    BlockRegistryEntry {
        numeric_id: 154,
        field_name: "RED_WOOL",
        registry_id: "minecraft:red_wool",
    },
    BlockRegistryEntry {
        numeric_id: 155,
        field_name: "BLACK_WOOL",
        registry_id: "minecraft:black_wool",
    },
    BlockRegistryEntry {
        numeric_id: 156,
        field_name: "MOVING_PISTON",
        registry_id: "minecraft:moving_piston",
    },
    BlockRegistryEntry {
        numeric_id: 157,
        field_name: "DANDELION",
        registry_id: "minecraft:dandelion",
    },
    BlockRegistryEntry {
        numeric_id: 158,
        field_name: "GOLDEN_DANDELION",
        registry_id: "minecraft:golden_dandelion",
    },
    BlockRegistryEntry {
        numeric_id: 159,
        field_name: "TORCHFLOWER",
        registry_id: "minecraft:torchflower",
    },
    BlockRegistryEntry {
        numeric_id: 160,
        field_name: "POPPY",
        registry_id: "minecraft:poppy",
    },
    BlockRegistryEntry {
        numeric_id: 161,
        field_name: "BLUE_ORCHID",
        registry_id: "minecraft:blue_orchid",
    },
    BlockRegistryEntry {
        numeric_id: 162,
        field_name: "ALLIUM",
        registry_id: "minecraft:allium",
    },
    BlockRegistryEntry {
        numeric_id: 163,
        field_name: "AZURE_BLUET",
        registry_id: "minecraft:azure_bluet",
    },
    BlockRegistryEntry {
        numeric_id: 164,
        field_name: "RED_TULIP",
        registry_id: "minecraft:red_tulip",
    },
    BlockRegistryEntry {
        numeric_id: 165,
        field_name: "ORANGE_TULIP",
        registry_id: "minecraft:orange_tulip",
    },
    BlockRegistryEntry {
        numeric_id: 166,
        field_name: "WHITE_TULIP",
        registry_id: "minecraft:white_tulip",
    },
    BlockRegistryEntry {
        numeric_id: 167,
        field_name: "PINK_TULIP",
        registry_id: "minecraft:pink_tulip",
    },
    BlockRegistryEntry {
        numeric_id: 168,
        field_name: "OXEYE_DAISY",
        registry_id: "minecraft:oxeye_daisy",
    },
    BlockRegistryEntry {
        numeric_id: 169,
        field_name: "CORNFLOWER",
        registry_id: "minecraft:cornflower",
    },
    BlockRegistryEntry {
        numeric_id: 170,
        field_name: "WITHER_ROSE",
        registry_id: "minecraft:wither_rose",
    },
    BlockRegistryEntry {
        numeric_id: 171,
        field_name: "LILY_OF_THE_VALLEY",
        registry_id: "minecraft:lily_of_the_valley",
    },
    BlockRegistryEntry {
        numeric_id: 172,
        field_name: "BROWN_MUSHROOM",
        registry_id: "minecraft:brown_mushroom",
    },
    BlockRegistryEntry {
        numeric_id: 173,
        field_name: "RED_MUSHROOM",
        registry_id: "minecraft:red_mushroom",
    },
    BlockRegistryEntry {
        numeric_id: 174,
        field_name: "GOLD_BLOCK",
        registry_id: "minecraft:gold_block",
    },
    BlockRegistryEntry {
        numeric_id: 175,
        field_name: "IRON_BLOCK",
        registry_id: "minecraft:iron_block",
    },
    BlockRegistryEntry {
        numeric_id: 176,
        field_name: "BRICKS",
        registry_id: "minecraft:bricks",
    },
    BlockRegistryEntry {
        numeric_id: 177,
        field_name: "TNT",
        registry_id: "minecraft:tnt",
    },
    BlockRegistryEntry {
        numeric_id: 178,
        field_name: "BOOKSHELF",
        registry_id: "minecraft:bookshelf",
    },
    BlockRegistryEntry {
        numeric_id: 179,
        field_name: "CHISELED_BOOKSHELF",
        registry_id: "minecraft:chiseled_bookshelf",
    },
    BlockRegistryEntry {
        numeric_id: 180,
        field_name: "ACACIA_SHELF",
        registry_id: "minecraft:acacia_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 181,
        field_name: "BAMBOO_SHELF",
        registry_id: "minecraft:bamboo_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 182,
        field_name: "BIRCH_SHELF",
        registry_id: "minecraft:birch_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 183,
        field_name: "CHERRY_SHELF",
        registry_id: "minecraft:cherry_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 184,
        field_name: "CRIMSON_SHELF",
        registry_id: "minecraft:crimson_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 185,
        field_name: "DARK_OAK_SHELF",
        registry_id: "minecraft:dark_oak_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 186,
        field_name: "JUNGLE_SHELF",
        registry_id: "minecraft:jungle_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 187,
        field_name: "MANGROVE_SHELF",
        registry_id: "minecraft:mangrove_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 188,
        field_name: "OAK_SHELF",
        registry_id: "minecraft:oak_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 189,
        field_name: "PALE_OAK_SHELF",
        registry_id: "minecraft:pale_oak_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 190,
        field_name: "SPRUCE_SHELF",
        registry_id: "minecraft:spruce_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 191,
        field_name: "WARPED_SHELF",
        registry_id: "minecraft:warped_shelf",
    },
    BlockRegistryEntry {
        numeric_id: 192,
        field_name: "MOSSY_COBBLESTONE",
        registry_id: "minecraft:mossy_cobblestone",
    },
    BlockRegistryEntry {
        numeric_id: 193,
        field_name: "OBSIDIAN",
        registry_id: "minecraft:obsidian",
    },
    BlockRegistryEntry {
        numeric_id: 194,
        field_name: "TORCH",
        registry_id: "minecraft:torch",
    },
    BlockRegistryEntry {
        numeric_id: 195,
        field_name: "WALL_TORCH",
        registry_id: "minecraft:wall_torch",
    },
    BlockRegistryEntry {
        numeric_id: 196,
        field_name: "FIRE",
        registry_id: "minecraft:fire",
    },
    BlockRegistryEntry {
        numeric_id: 197,
        field_name: "SOUL_FIRE",
        registry_id: "minecraft:soul_fire",
    },
    BlockRegistryEntry {
        numeric_id: 198,
        field_name: "SPAWNER",
        registry_id: "minecraft:spawner",
    },
    BlockRegistryEntry {
        numeric_id: 199,
        field_name: "CREAKING_HEART",
        registry_id: "minecraft:creaking_heart",
    },
    BlockRegistryEntry {
        numeric_id: 200,
        field_name: "OAK_STAIRS",
        registry_id: "minecraft:oak_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 201,
        field_name: "CHEST",
        registry_id: "minecraft:chest",
    },
    BlockRegistryEntry {
        numeric_id: 202,
        field_name: "REDSTONE_WIRE",
        registry_id: "minecraft:redstone_wire",
    },
    BlockRegistryEntry {
        numeric_id: 203,
        field_name: "DIAMOND_ORE",
        registry_id: "minecraft:diamond_ore",
    },
    BlockRegistryEntry {
        numeric_id: 204,
        field_name: "DEEPSLATE_DIAMOND_ORE",
        registry_id: "minecraft:deepslate_diamond_ore",
    },
    BlockRegistryEntry {
        numeric_id: 205,
        field_name: "DIAMOND_BLOCK",
        registry_id: "minecraft:diamond_block",
    },
    BlockRegistryEntry {
        numeric_id: 206,
        field_name: "CRAFTING_TABLE",
        registry_id: "minecraft:crafting_table",
    },
    BlockRegistryEntry {
        numeric_id: 207,
        field_name: "WHEAT",
        registry_id: "minecraft:wheat",
    },
    BlockRegistryEntry {
        numeric_id: 208,
        field_name: "FARMLAND",
        registry_id: "minecraft:farmland",
    },
    BlockRegistryEntry {
        numeric_id: 209,
        field_name: "FURNACE",
        registry_id: "minecraft:furnace",
    },
    BlockRegistryEntry {
        numeric_id: 210,
        field_name: "OAK_SIGN",
        registry_id: "minecraft:oak_sign",
    },
    BlockRegistryEntry {
        numeric_id: 211,
        field_name: "SPRUCE_SIGN",
        registry_id: "minecraft:spruce_sign",
    },
    BlockRegistryEntry {
        numeric_id: 212,
        field_name: "BIRCH_SIGN",
        registry_id: "minecraft:birch_sign",
    },
    BlockRegistryEntry {
        numeric_id: 213,
        field_name: "ACACIA_SIGN",
        registry_id: "minecraft:acacia_sign",
    },
    BlockRegistryEntry {
        numeric_id: 214,
        field_name: "CHERRY_SIGN",
        registry_id: "minecraft:cherry_sign",
    },
    BlockRegistryEntry {
        numeric_id: 215,
        field_name: "JUNGLE_SIGN",
        registry_id: "minecraft:jungle_sign",
    },
    BlockRegistryEntry {
        numeric_id: 216,
        field_name: "DARK_OAK_SIGN",
        registry_id: "minecraft:dark_oak_sign",
    },
    BlockRegistryEntry {
        numeric_id: 217,
        field_name: "PALE_OAK_SIGN",
        registry_id: "minecraft:pale_oak_sign",
    },
    BlockRegistryEntry {
        numeric_id: 218,
        field_name: "MANGROVE_SIGN",
        registry_id: "minecraft:mangrove_sign",
    },
    BlockRegistryEntry {
        numeric_id: 219,
        field_name: "BAMBOO_SIGN",
        registry_id: "minecraft:bamboo_sign",
    },
    BlockRegistryEntry {
        numeric_id: 220,
        field_name: "OAK_DOOR",
        registry_id: "minecraft:oak_door",
    },
    BlockRegistryEntry {
        numeric_id: 221,
        field_name: "LADDER",
        registry_id: "minecraft:ladder",
    },
    BlockRegistryEntry {
        numeric_id: 222,
        field_name: "RAIL",
        registry_id: "minecraft:rail",
    },
    BlockRegistryEntry {
        numeric_id: 223,
        field_name: "COBBLESTONE_STAIRS",
        registry_id: "minecraft:cobblestone_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 224,
        field_name: "OAK_WALL_SIGN",
        registry_id: "minecraft:oak_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 225,
        field_name: "SPRUCE_WALL_SIGN",
        registry_id: "minecraft:spruce_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 226,
        field_name: "BIRCH_WALL_SIGN",
        registry_id: "minecraft:birch_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 227,
        field_name: "ACACIA_WALL_SIGN",
        registry_id: "minecraft:acacia_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 228,
        field_name: "CHERRY_WALL_SIGN",
        registry_id: "minecraft:cherry_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 229,
        field_name: "JUNGLE_WALL_SIGN",
        registry_id: "minecraft:jungle_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 230,
        field_name: "DARK_OAK_WALL_SIGN",
        registry_id: "minecraft:dark_oak_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 231,
        field_name: "PALE_OAK_WALL_SIGN",
        registry_id: "minecraft:pale_oak_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 232,
        field_name: "MANGROVE_WALL_SIGN",
        registry_id: "minecraft:mangrove_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 233,
        field_name: "BAMBOO_WALL_SIGN",
        registry_id: "minecraft:bamboo_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 234,
        field_name: "OAK_HANGING_SIGN",
        registry_id: "minecraft:oak_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 235,
        field_name: "SPRUCE_HANGING_SIGN",
        registry_id: "minecraft:spruce_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 236,
        field_name: "BIRCH_HANGING_SIGN",
        registry_id: "minecraft:birch_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 237,
        field_name: "ACACIA_HANGING_SIGN",
        registry_id: "minecraft:acacia_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 238,
        field_name: "CHERRY_HANGING_SIGN",
        registry_id: "minecraft:cherry_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 239,
        field_name: "JUNGLE_HANGING_SIGN",
        registry_id: "minecraft:jungle_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 240,
        field_name: "DARK_OAK_HANGING_SIGN",
        registry_id: "minecraft:dark_oak_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 241,
        field_name: "PALE_OAK_HANGING_SIGN",
        registry_id: "minecraft:pale_oak_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 242,
        field_name: "CRIMSON_HANGING_SIGN",
        registry_id: "minecraft:crimson_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 243,
        field_name: "WARPED_HANGING_SIGN",
        registry_id: "minecraft:warped_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 244,
        field_name: "MANGROVE_HANGING_SIGN",
        registry_id: "minecraft:mangrove_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 245,
        field_name: "BAMBOO_HANGING_SIGN",
        registry_id: "minecraft:bamboo_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 246,
        field_name: "OAK_WALL_HANGING_SIGN",
        registry_id: "minecraft:oak_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 247,
        field_name: "SPRUCE_WALL_HANGING_SIGN",
        registry_id: "minecraft:spruce_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 248,
        field_name: "BIRCH_WALL_HANGING_SIGN",
        registry_id: "minecraft:birch_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 249,
        field_name: "ACACIA_WALL_HANGING_SIGN",
        registry_id: "minecraft:acacia_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 250,
        field_name: "CHERRY_WALL_HANGING_SIGN",
        registry_id: "minecraft:cherry_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 251,
        field_name: "JUNGLE_WALL_HANGING_SIGN",
        registry_id: "minecraft:jungle_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 252,
        field_name: "DARK_OAK_WALL_HANGING_SIGN",
        registry_id: "minecraft:dark_oak_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 253,
        field_name: "PALE_OAK_WALL_HANGING_SIGN",
        registry_id: "minecraft:pale_oak_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 254,
        field_name: "MANGROVE_WALL_HANGING_SIGN",
        registry_id: "minecraft:mangrove_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 255,
        field_name: "CRIMSON_WALL_HANGING_SIGN",
        registry_id: "minecraft:crimson_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 256,
        field_name: "WARPED_WALL_HANGING_SIGN",
        registry_id: "minecraft:warped_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 257,
        field_name: "BAMBOO_WALL_HANGING_SIGN",
        registry_id: "minecraft:bamboo_wall_hanging_sign",
    },
    BlockRegistryEntry {
        numeric_id: 258,
        field_name: "LEVER",
        registry_id: "minecraft:lever",
    },
    BlockRegistryEntry {
        numeric_id: 259,
        field_name: "STONE_PRESSURE_PLATE",
        registry_id: "minecraft:stone_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 260,
        field_name: "IRON_DOOR",
        registry_id: "minecraft:iron_door",
    },
    BlockRegistryEntry {
        numeric_id: 261,
        field_name: "OAK_PRESSURE_PLATE",
        registry_id: "minecraft:oak_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 262,
        field_name: "SPRUCE_PRESSURE_PLATE",
        registry_id: "minecraft:spruce_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 263,
        field_name: "BIRCH_PRESSURE_PLATE",
        registry_id: "minecraft:birch_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 264,
        field_name: "JUNGLE_PRESSURE_PLATE",
        registry_id: "minecraft:jungle_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 265,
        field_name: "ACACIA_PRESSURE_PLATE",
        registry_id: "minecraft:acacia_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 266,
        field_name: "CHERRY_PRESSURE_PLATE",
        registry_id: "minecraft:cherry_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 267,
        field_name: "DARK_OAK_PRESSURE_PLATE",
        registry_id: "minecraft:dark_oak_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 268,
        field_name: "PALE_OAK_PRESSURE_PLATE",
        registry_id: "minecraft:pale_oak_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 269,
        field_name: "MANGROVE_PRESSURE_PLATE",
        registry_id: "minecraft:mangrove_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 270,
        field_name: "BAMBOO_PRESSURE_PLATE",
        registry_id: "minecraft:bamboo_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 271,
        field_name: "REDSTONE_ORE",
        registry_id: "minecraft:redstone_ore",
    },
    BlockRegistryEntry {
        numeric_id: 272,
        field_name: "DEEPSLATE_REDSTONE_ORE",
        registry_id: "minecraft:deepslate_redstone_ore",
    },
    BlockRegistryEntry {
        numeric_id: 273,
        field_name: "REDSTONE_TORCH",
        registry_id: "minecraft:redstone_torch",
    },
    BlockRegistryEntry {
        numeric_id: 274,
        field_name: "REDSTONE_WALL_TORCH",
        registry_id: "minecraft:redstone_wall_torch",
    },
    BlockRegistryEntry {
        numeric_id: 275,
        field_name: "STONE_BUTTON",
        registry_id: "minecraft:stone_button",
    },
    BlockRegistryEntry {
        numeric_id: 276,
        field_name: "SNOW",
        registry_id: "minecraft:snow",
    },
    BlockRegistryEntry {
        numeric_id: 277,
        field_name: "ICE",
        registry_id: "minecraft:ice",
    },
    BlockRegistryEntry {
        numeric_id: 278,
        field_name: "SNOW_BLOCK",
        registry_id: "minecraft:snow_block",
    },
    BlockRegistryEntry {
        numeric_id: 279,
        field_name: "CACTUS",
        registry_id: "minecraft:cactus",
    },
    BlockRegistryEntry {
        numeric_id: 280,
        field_name: "CACTUS_FLOWER",
        registry_id: "minecraft:cactus_flower",
    },
    BlockRegistryEntry {
        numeric_id: 281,
        field_name: "CLAY",
        registry_id: "minecraft:clay",
    },
    BlockRegistryEntry {
        numeric_id: 282,
        field_name: "SUGAR_CANE",
        registry_id: "minecraft:sugar_cane",
    },
    BlockRegistryEntry {
        numeric_id: 283,
        field_name: "JUKEBOX",
        registry_id: "minecraft:jukebox",
    },
    BlockRegistryEntry {
        numeric_id: 284,
        field_name: "OAK_FENCE",
        registry_id: "minecraft:oak_fence",
    },
    BlockRegistryEntry {
        numeric_id: 285,
        field_name: "NETHERRACK",
        registry_id: "minecraft:netherrack",
    },
    BlockRegistryEntry {
        numeric_id: 286,
        field_name: "SOUL_SAND",
        registry_id: "minecraft:soul_sand",
    },
    BlockRegistryEntry {
        numeric_id: 287,
        field_name: "SOUL_SOIL",
        registry_id: "minecraft:soul_soil",
    },
    BlockRegistryEntry {
        numeric_id: 288,
        field_name: "BASALT",
        registry_id: "minecraft:basalt",
    },
    BlockRegistryEntry {
        numeric_id: 289,
        field_name: "POLISHED_BASALT",
        registry_id: "minecraft:polished_basalt",
    },
    BlockRegistryEntry {
        numeric_id: 290,
        field_name: "SOUL_TORCH",
        registry_id: "minecraft:soul_torch",
    },
    BlockRegistryEntry {
        numeric_id: 291,
        field_name: "SOUL_WALL_TORCH",
        registry_id: "minecraft:soul_wall_torch",
    },
    BlockRegistryEntry {
        numeric_id: 292,
        field_name: "COPPER_TORCH",
        registry_id: "minecraft:copper_torch",
    },
    BlockRegistryEntry {
        numeric_id: 293,
        field_name: "COPPER_WALL_TORCH",
        registry_id: "minecraft:copper_wall_torch",
    },
    BlockRegistryEntry {
        numeric_id: 294,
        field_name: "GLOWSTONE",
        registry_id: "minecraft:glowstone",
    },
    BlockRegistryEntry {
        numeric_id: 295,
        field_name: "NETHER_PORTAL",
        registry_id: "minecraft:nether_portal",
    },
    BlockRegistryEntry {
        numeric_id: 296,
        field_name: "CARVED_PUMPKIN",
        registry_id: "minecraft:carved_pumpkin",
    },
    BlockRegistryEntry {
        numeric_id: 297,
        field_name: "JACK_O_LANTERN",
        registry_id: "minecraft:jack_o_lantern",
    },
    BlockRegistryEntry {
        numeric_id: 298,
        field_name: "CAKE",
        registry_id: "minecraft:cake",
    },
    BlockRegistryEntry {
        numeric_id: 299,
        field_name: "REPEATER",
        registry_id: "minecraft:repeater",
    },
    BlockRegistryEntry {
        numeric_id: 300,
        field_name: "WHITE_STAINED_GLASS",
        registry_id: "minecraft:white_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 301,
        field_name: "ORANGE_STAINED_GLASS",
        registry_id: "minecraft:orange_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 302,
        field_name: "MAGENTA_STAINED_GLASS",
        registry_id: "minecraft:magenta_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 303,
        field_name: "LIGHT_BLUE_STAINED_GLASS",
        registry_id: "minecraft:light_blue_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 304,
        field_name: "YELLOW_STAINED_GLASS",
        registry_id: "minecraft:yellow_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 305,
        field_name: "LIME_STAINED_GLASS",
        registry_id: "minecraft:lime_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 306,
        field_name: "PINK_STAINED_GLASS",
        registry_id: "minecraft:pink_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 307,
        field_name: "GRAY_STAINED_GLASS",
        registry_id: "minecraft:gray_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 308,
        field_name: "LIGHT_GRAY_STAINED_GLASS",
        registry_id: "minecraft:light_gray_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 309,
        field_name: "CYAN_STAINED_GLASS",
        registry_id: "minecraft:cyan_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 310,
        field_name: "PURPLE_STAINED_GLASS",
        registry_id: "minecraft:purple_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 311,
        field_name: "BLUE_STAINED_GLASS",
        registry_id: "minecraft:blue_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 312,
        field_name: "BROWN_STAINED_GLASS",
        registry_id: "minecraft:brown_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 313,
        field_name: "GREEN_STAINED_GLASS",
        registry_id: "minecraft:green_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 314,
        field_name: "RED_STAINED_GLASS",
        registry_id: "minecraft:red_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 315,
        field_name: "BLACK_STAINED_GLASS",
        registry_id: "minecraft:black_stained_glass",
    },
    BlockRegistryEntry {
        numeric_id: 316,
        field_name: "OAK_TRAPDOOR",
        registry_id: "minecraft:oak_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 317,
        field_name: "SPRUCE_TRAPDOOR",
        registry_id: "minecraft:spruce_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 318,
        field_name: "BIRCH_TRAPDOOR",
        registry_id: "minecraft:birch_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 319,
        field_name: "JUNGLE_TRAPDOOR",
        registry_id: "minecraft:jungle_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 320,
        field_name: "ACACIA_TRAPDOOR",
        registry_id: "minecraft:acacia_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 321,
        field_name: "CHERRY_TRAPDOOR",
        registry_id: "minecraft:cherry_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 322,
        field_name: "DARK_OAK_TRAPDOOR",
        registry_id: "minecraft:dark_oak_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 323,
        field_name: "PALE_OAK_TRAPDOOR",
        registry_id: "minecraft:pale_oak_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 324,
        field_name: "MANGROVE_TRAPDOOR",
        registry_id: "minecraft:mangrove_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 325,
        field_name: "BAMBOO_TRAPDOOR",
        registry_id: "minecraft:bamboo_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 326,
        field_name: "STONE_BRICKS",
        registry_id: "minecraft:stone_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 327,
        field_name: "MOSSY_STONE_BRICKS",
        registry_id: "minecraft:mossy_stone_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 328,
        field_name: "CRACKED_STONE_BRICKS",
        registry_id: "minecraft:cracked_stone_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 329,
        field_name: "CHISELED_STONE_BRICKS",
        registry_id: "minecraft:chiseled_stone_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 330,
        field_name: "PACKED_MUD",
        registry_id: "minecraft:packed_mud",
    },
    BlockRegistryEntry {
        numeric_id: 331,
        field_name: "MUD_BRICKS",
        registry_id: "minecraft:mud_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 332,
        field_name: "INFESTED_STONE",
        registry_id: "minecraft:infested_stone",
    },
    BlockRegistryEntry {
        numeric_id: 333,
        field_name: "INFESTED_COBBLESTONE",
        registry_id: "minecraft:infested_cobblestone",
    },
    BlockRegistryEntry {
        numeric_id: 334,
        field_name: "INFESTED_STONE_BRICKS",
        registry_id: "minecraft:infested_stone_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 335,
        field_name: "INFESTED_MOSSY_STONE_BRICKS",
        registry_id: "minecraft:infested_mossy_stone_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 336,
        field_name: "INFESTED_CRACKED_STONE_BRICKS",
        registry_id: "minecraft:infested_cracked_stone_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 337,
        field_name: "INFESTED_CHISELED_STONE_BRICKS",
        registry_id: "minecraft:infested_chiseled_stone_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 338,
        field_name: "BROWN_MUSHROOM_BLOCK",
        registry_id: "minecraft:brown_mushroom_block",
    },
    BlockRegistryEntry {
        numeric_id: 339,
        field_name: "RED_MUSHROOM_BLOCK",
        registry_id: "minecraft:red_mushroom_block",
    },
    BlockRegistryEntry {
        numeric_id: 340,
        field_name: "MUSHROOM_STEM",
        registry_id: "minecraft:mushroom_stem",
    },
    BlockRegistryEntry {
        numeric_id: 341,
        field_name: "IRON_BARS",
        registry_id: "minecraft:iron_bars",
    },
    BlockRegistryEntry {
        numeric_id: 342,
        field_name: "IRON_CHAIN",
        registry_id: "minecraft:iron_chain",
    },
    BlockRegistryEntry {
        numeric_id: 343,
        field_name: "GLASS_PANE",
        registry_id: "minecraft:glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 344,
        field_name: "PUMPKIN",
        registry_id: "minecraft:pumpkin",
    },
    BlockRegistryEntry {
        numeric_id: 345,
        field_name: "MELON",
        registry_id: "minecraft:melon",
    },
    BlockRegistryEntry {
        numeric_id: 346,
        field_name: "ATTACHED_PUMPKIN_STEM",
        registry_id: "minecraft:attached_pumpkin_stem",
    },
    BlockRegistryEntry {
        numeric_id: 347,
        field_name: "ATTACHED_MELON_STEM",
        registry_id: "minecraft:attached_melon_stem",
    },
    BlockRegistryEntry {
        numeric_id: 348,
        field_name: "PUMPKIN_STEM",
        registry_id: "minecraft:pumpkin_stem",
    },
    BlockRegistryEntry {
        numeric_id: 349,
        field_name: "MELON_STEM",
        registry_id: "minecraft:melon_stem",
    },
    BlockRegistryEntry {
        numeric_id: 350,
        field_name: "VINE",
        registry_id: "minecraft:vine",
    },
    BlockRegistryEntry {
        numeric_id: 351,
        field_name: "GLOW_LICHEN",
        registry_id: "minecraft:glow_lichen",
    },
    BlockRegistryEntry {
        numeric_id: 352,
        field_name: "RESIN_CLUMP",
        registry_id: "minecraft:resin_clump",
    },
    BlockRegistryEntry {
        numeric_id: 353,
        field_name: "OAK_FENCE_GATE",
        registry_id: "minecraft:oak_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 354,
        field_name: "BRICK_STAIRS",
        registry_id: "minecraft:brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 355,
        field_name: "STONE_BRICK_STAIRS",
        registry_id: "minecraft:stone_brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 356,
        field_name: "MUD_BRICK_STAIRS",
        registry_id: "minecraft:mud_brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 357,
        field_name: "MYCELIUM",
        registry_id: "minecraft:mycelium",
    },
    BlockRegistryEntry {
        numeric_id: 358,
        field_name: "LILY_PAD",
        registry_id: "minecraft:lily_pad",
    },
    BlockRegistryEntry {
        numeric_id: 359,
        field_name: "RESIN_BLOCK",
        registry_id: "minecraft:resin_block",
    },
    BlockRegistryEntry {
        numeric_id: 360,
        field_name: "RESIN_BRICKS",
        registry_id: "minecraft:resin_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 361,
        field_name: "RESIN_BRICK_STAIRS",
        registry_id: "minecraft:resin_brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 362,
        field_name: "RESIN_BRICK_SLAB",
        registry_id: "minecraft:resin_brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 363,
        field_name: "RESIN_BRICK_WALL",
        registry_id: "minecraft:resin_brick_wall",
    },
    BlockRegistryEntry {
        numeric_id: 364,
        field_name: "CHISELED_RESIN_BRICKS",
        registry_id: "minecraft:chiseled_resin_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 365,
        field_name: "NETHER_BRICKS",
        registry_id: "minecraft:nether_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 366,
        field_name: "NETHER_BRICK_FENCE",
        registry_id: "minecraft:nether_brick_fence",
    },
    BlockRegistryEntry {
        numeric_id: 367,
        field_name: "NETHER_BRICK_STAIRS",
        registry_id: "minecraft:nether_brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 368,
        field_name: "NETHER_WART",
        registry_id: "minecraft:nether_wart",
    },
    BlockRegistryEntry {
        numeric_id: 369,
        field_name: "ENCHANTING_TABLE",
        registry_id: "minecraft:enchanting_table",
    },
    BlockRegistryEntry {
        numeric_id: 370,
        field_name: "BREWING_STAND",
        registry_id: "minecraft:brewing_stand",
    },
    BlockRegistryEntry {
        numeric_id: 371,
        field_name: "CAULDRON",
        registry_id: "minecraft:cauldron",
    },
    BlockRegistryEntry {
        numeric_id: 372,
        field_name: "WATER_CAULDRON",
        registry_id: "minecraft:water_cauldron",
    },
    BlockRegistryEntry {
        numeric_id: 373,
        field_name: "LAVA_CAULDRON",
        registry_id: "minecraft:lava_cauldron",
    },
    BlockRegistryEntry {
        numeric_id: 374,
        field_name: "POWDER_SNOW_CAULDRON",
        registry_id: "minecraft:powder_snow_cauldron",
    },
    BlockRegistryEntry {
        numeric_id: 375,
        field_name: "END_PORTAL",
        registry_id: "minecraft:end_portal",
    },
    BlockRegistryEntry {
        numeric_id: 376,
        field_name: "END_PORTAL_FRAME",
        registry_id: "minecraft:end_portal_frame",
    },
    BlockRegistryEntry {
        numeric_id: 377,
        field_name: "END_STONE",
        registry_id: "minecraft:end_stone",
    },
    BlockRegistryEntry {
        numeric_id: 378,
        field_name: "DRAGON_EGG",
        registry_id: "minecraft:dragon_egg",
    },
    BlockRegistryEntry {
        numeric_id: 379,
        field_name: "REDSTONE_LAMP",
        registry_id: "minecraft:redstone_lamp",
    },
    BlockRegistryEntry {
        numeric_id: 380,
        field_name: "COCOA",
        registry_id: "minecraft:cocoa",
    },
    BlockRegistryEntry {
        numeric_id: 381,
        field_name: "SANDSTONE_STAIRS",
        registry_id: "minecraft:sandstone_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 382,
        field_name: "EMERALD_ORE",
        registry_id: "minecraft:emerald_ore",
    },
    BlockRegistryEntry {
        numeric_id: 383,
        field_name: "DEEPSLATE_EMERALD_ORE",
        registry_id: "minecraft:deepslate_emerald_ore",
    },
    BlockRegistryEntry {
        numeric_id: 384,
        field_name: "ENDER_CHEST",
        registry_id: "minecraft:ender_chest",
    },
    BlockRegistryEntry {
        numeric_id: 385,
        field_name: "TRIPWIRE_HOOK",
        registry_id: "minecraft:tripwire_hook",
    },
    BlockRegistryEntry {
        numeric_id: 386,
        field_name: "TRIPWIRE",
        registry_id: "minecraft:tripwire",
    },
    BlockRegistryEntry {
        numeric_id: 387,
        field_name: "EMERALD_BLOCK",
        registry_id: "minecraft:emerald_block",
    },
    BlockRegistryEntry {
        numeric_id: 388,
        field_name: "SPRUCE_STAIRS",
        registry_id: "minecraft:spruce_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 389,
        field_name: "BIRCH_STAIRS",
        registry_id: "minecraft:birch_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 390,
        field_name: "JUNGLE_STAIRS",
        registry_id: "minecraft:jungle_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 391,
        field_name: "COMMAND_BLOCK",
        registry_id: "minecraft:command_block",
    },
    BlockRegistryEntry {
        numeric_id: 392,
        field_name: "BEACON",
        registry_id: "minecraft:beacon",
    },
    BlockRegistryEntry {
        numeric_id: 393,
        field_name: "COBBLESTONE_WALL",
        registry_id: "minecraft:cobblestone_wall",
    },
    BlockRegistryEntry {
        numeric_id: 394,
        field_name: "MOSSY_COBBLESTONE_WALL",
        registry_id: "minecraft:mossy_cobblestone_wall",
    },
    BlockRegistryEntry {
        numeric_id: 395,
        field_name: "FLOWER_POT",
        registry_id: "minecraft:flower_pot",
    },
    BlockRegistryEntry {
        numeric_id: 396,
        field_name: "POTTED_TORCHFLOWER",
        registry_id: "minecraft:potted_torchflower",
    },
    BlockRegistryEntry {
        numeric_id: 397,
        field_name: "POTTED_OAK_SAPLING",
        registry_id: "minecraft:potted_oak_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 398,
        field_name: "POTTED_SPRUCE_SAPLING",
        registry_id: "minecraft:potted_spruce_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 399,
        field_name: "POTTED_BIRCH_SAPLING",
        registry_id: "minecraft:potted_birch_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 400,
        field_name: "POTTED_JUNGLE_SAPLING",
        registry_id: "minecraft:potted_jungle_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 401,
        field_name: "POTTED_ACACIA_SAPLING",
        registry_id: "minecraft:potted_acacia_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 402,
        field_name: "POTTED_CHERRY_SAPLING",
        registry_id: "minecraft:potted_cherry_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 403,
        field_name: "POTTED_DARK_OAK_SAPLING",
        registry_id: "minecraft:potted_dark_oak_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 404,
        field_name: "POTTED_PALE_OAK_SAPLING",
        registry_id: "minecraft:potted_pale_oak_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 405,
        field_name: "POTTED_MANGROVE_PROPAGULE",
        registry_id: "minecraft:potted_mangrove_propagule",
    },
    BlockRegistryEntry {
        numeric_id: 406,
        field_name: "POTTED_FERN",
        registry_id: "minecraft:potted_fern",
    },
    BlockRegistryEntry {
        numeric_id: 407,
        field_name: "POTTED_DANDELION",
        registry_id: "minecraft:potted_dandelion",
    },
    BlockRegistryEntry {
        numeric_id: 408,
        field_name: "POTTED_GOLDEN_DANDELION",
        registry_id: "minecraft:potted_golden_dandelion",
    },
    BlockRegistryEntry {
        numeric_id: 409,
        field_name: "POTTED_POPPY",
        registry_id: "minecraft:potted_poppy",
    },
    BlockRegistryEntry {
        numeric_id: 410,
        field_name: "POTTED_BLUE_ORCHID",
        registry_id: "minecraft:potted_blue_orchid",
    },
    BlockRegistryEntry {
        numeric_id: 411,
        field_name: "POTTED_ALLIUM",
        registry_id: "minecraft:potted_allium",
    },
    BlockRegistryEntry {
        numeric_id: 412,
        field_name: "POTTED_AZURE_BLUET",
        registry_id: "minecraft:potted_azure_bluet",
    },
    BlockRegistryEntry {
        numeric_id: 413,
        field_name: "POTTED_RED_TULIP",
        registry_id: "minecraft:potted_red_tulip",
    },
    BlockRegistryEntry {
        numeric_id: 414,
        field_name: "POTTED_ORANGE_TULIP",
        registry_id: "minecraft:potted_orange_tulip",
    },
    BlockRegistryEntry {
        numeric_id: 415,
        field_name: "POTTED_WHITE_TULIP",
        registry_id: "minecraft:potted_white_tulip",
    },
    BlockRegistryEntry {
        numeric_id: 416,
        field_name: "POTTED_PINK_TULIP",
        registry_id: "minecraft:potted_pink_tulip",
    },
    BlockRegistryEntry {
        numeric_id: 417,
        field_name: "POTTED_OXEYE_DAISY",
        registry_id: "minecraft:potted_oxeye_daisy",
    },
    BlockRegistryEntry {
        numeric_id: 418,
        field_name: "POTTED_CORNFLOWER",
        registry_id: "minecraft:potted_cornflower",
    },
    BlockRegistryEntry {
        numeric_id: 419,
        field_name: "POTTED_LILY_OF_THE_VALLEY",
        registry_id: "minecraft:potted_lily_of_the_valley",
    },
    BlockRegistryEntry {
        numeric_id: 420,
        field_name: "POTTED_WITHER_ROSE",
        registry_id: "minecraft:potted_wither_rose",
    },
    BlockRegistryEntry {
        numeric_id: 421,
        field_name: "POTTED_RED_MUSHROOM",
        registry_id: "minecraft:potted_red_mushroom",
    },
    BlockRegistryEntry {
        numeric_id: 422,
        field_name: "POTTED_BROWN_MUSHROOM",
        registry_id: "minecraft:potted_brown_mushroom",
    },
    BlockRegistryEntry {
        numeric_id: 423,
        field_name: "POTTED_DEAD_BUSH",
        registry_id: "minecraft:potted_dead_bush",
    },
    BlockRegistryEntry {
        numeric_id: 424,
        field_name: "POTTED_CACTUS",
        registry_id: "minecraft:potted_cactus",
    },
    BlockRegistryEntry {
        numeric_id: 425,
        field_name: "CARROTS",
        registry_id: "minecraft:carrots",
    },
    BlockRegistryEntry {
        numeric_id: 426,
        field_name: "POTATOES",
        registry_id: "minecraft:potatoes",
    },
    BlockRegistryEntry {
        numeric_id: 427,
        field_name: "OAK_BUTTON",
        registry_id: "minecraft:oak_button",
    },
    BlockRegistryEntry {
        numeric_id: 428,
        field_name: "SPRUCE_BUTTON",
        registry_id: "minecraft:spruce_button",
    },
    BlockRegistryEntry {
        numeric_id: 429,
        field_name: "BIRCH_BUTTON",
        registry_id: "minecraft:birch_button",
    },
    BlockRegistryEntry {
        numeric_id: 430,
        field_name: "JUNGLE_BUTTON",
        registry_id: "minecraft:jungle_button",
    },
    BlockRegistryEntry {
        numeric_id: 431,
        field_name: "ACACIA_BUTTON",
        registry_id: "minecraft:acacia_button",
    },
    BlockRegistryEntry {
        numeric_id: 432,
        field_name: "CHERRY_BUTTON",
        registry_id: "minecraft:cherry_button",
    },
    BlockRegistryEntry {
        numeric_id: 433,
        field_name: "DARK_OAK_BUTTON",
        registry_id: "minecraft:dark_oak_button",
    },
    BlockRegistryEntry {
        numeric_id: 434,
        field_name: "PALE_OAK_BUTTON",
        registry_id: "minecraft:pale_oak_button",
    },
    BlockRegistryEntry {
        numeric_id: 435,
        field_name: "MANGROVE_BUTTON",
        registry_id: "minecraft:mangrove_button",
    },
    BlockRegistryEntry {
        numeric_id: 436,
        field_name: "BAMBOO_BUTTON",
        registry_id: "minecraft:bamboo_button",
    },
    BlockRegistryEntry {
        numeric_id: 437,
        field_name: "SKELETON_SKULL",
        registry_id: "minecraft:skeleton_skull",
    },
    BlockRegistryEntry {
        numeric_id: 438,
        field_name: "SKELETON_WALL_SKULL",
        registry_id: "minecraft:skeleton_wall_skull",
    },
    BlockRegistryEntry {
        numeric_id: 439,
        field_name: "WITHER_SKELETON_SKULL",
        registry_id: "minecraft:wither_skeleton_skull",
    },
    BlockRegistryEntry {
        numeric_id: 440,
        field_name: "WITHER_SKELETON_WALL_SKULL",
        registry_id: "minecraft:wither_skeleton_wall_skull",
    },
    BlockRegistryEntry {
        numeric_id: 441,
        field_name: "ZOMBIE_HEAD",
        registry_id: "minecraft:zombie_head",
    },
    BlockRegistryEntry {
        numeric_id: 442,
        field_name: "ZOMBIE_WALL_HEAD",
        registry_id: "minecraft:zombie_wall_head",
    },
    BlockRegistryEntry {
        numeric_id: 443,
        field_name: "PLAYER_HEAD",
        registry_id: "minecraft:player_head",
    },
    BlockRegistryEntry {
        numeric_id: 444,
        field_name: "PLAYER_WALL_HEAD",
        registry_id: "minecraft:player_wall_head",
    },
    BlockRegistryEntry {
        numeric_id: 445,
        field_name: "CREEPER_HEAD",
        registry_id: "minecraft:creeper_head",
    },
    BlockRegistryEntry {
        numeric_id: 446,
        field_name: "CREEPER_WALL_HEAD",
        registry_id: "minecraft:creeper_wall_head",
    },
    BlockRegistryEntry {
        numeric_id: 447,
        field_name: "DRAGON_HEAD",
        registry_id: "minecraft:dragon_head",
    },
    BlockRegistryEntry {
        numeric_id: 448,
        field_name: "DRAGON_WALL_HEAD",
        registry_id: "minecraft:dragon_wall_head",
    },
    BlockRegistryEntry {
        numeric_id: 449,
        field_name: "PIGLIN_HEAD",
        registry_id: "minecraft:piglin_head",
    },
    BlockRegistryEntry {
        numeric_id: 450,
        field_name: "PIGLIN_WALL_HEAD",
        registry_id: "minecraft:piglin_wall_head",
    },
    BlockRegistryEntry {
        numeric_id: 451,
        field_name: "ANVIL",
        registry_id: "minecraft:anvil",
    },
    BlockRegistryEntry {
        numeric_id: 452,
        field_name: "CHIPPED_ANVIL",
        registry_id: "minecraft:chipped_anvil",
    },
    BlockRegistryEntry {
        numeric_id: 453,
        field_name: "DAMAGED_ANVIL",
        registry_id: "minecraft:damaged_anvil",
    },
    BlockRegistryEntry {
        numeric_id: 454,
        field_name: "TRAPPED_CHEST",
        registry_id: "minecraft:trapped_chest",
    },
    BlockRegistryEntry {
        numeric_id: 455,
        field_name: "LIGHT_WEIGHTED_PRESSURE_PLATE",
        registry_id: "minecraft:light_weighted_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 456,
        field_name: "HEAVY_WEIGHTED_PRESSURE_PLATE",
        registry_id: "minecraft:heavy_weighted_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 457,
        field_name: "COMPARATOR",
        registry_id: "minecraft:comparator",
    },
    BlockRegistryEntry {
        numeric_id: 458,
        field_name: "DAYLIGHT_DETECTOR",
        registry_id: "minecraft:daylight_detector",
    },
    BlockRegistryEntry {
        numeric_id: 459,
        field_name: "REDSTONE_BLOCK",
        registry_id: "minecraft:redstone_block",
    },
    BlockRegistryEntry {
        numeric_id: 460,
        field_name: "NETHER_QUARTZ_ORE",
        registry_id: "minecraft:nether_quartz_ore",
    },
    BlockRegistryEntry {
        numeric_id: 461,
        field_name: "HOPPER",
        registry_id: "minecraft:hopper",
    },
    BlockRegistryEntry {
        numeric_id: 462,
        field_name: "QUARTZ_BLOCK",
        registry_id: "minecraft:quartz_block",
    },
    BlockRegistryEntry {
        numeric_id: 463,
        field_name: "CHISELED_QUARTZ_BLOCK",
        registry_id: "minecraft:chiseled_quartz_block",
    },
    BlockRegistryEntry {
        numeric_id: 464,
        field_name: "QUARTZ_PILLAR",
        registry_id: "minecraft:quartz_pillar",
    },
    BlockRegistryEntry {
        numeric_id: 465,
        field_name: "QUARTZ_STAIRS",
        registry_id: "minecraft:quartz_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 466,
        field_name: "ACTIVATOR_RAIL",
        registry_id: "minecraft:activator_rail",
    },
    BlockRegistryEntry {
        numeric_id: 467,
        field_name: "DROPPER",
        registry_id: "minecraft:dropper",
    },
    BlockRegistryEntry {
        numeric_id: 468,
        field_name: "WHITE_TERRACOTTA",
        registry_id: "minecraft:white_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 469,
        field_name: "ORANGE_TERRACOTTA",
        registry_id: "minecraft:orange_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 470,
        field_name: "MAGENTA_TERRACOTTA",
        registry_id: "minecraft:magenta_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 471,
        field_name: "LIGHT_BLUE_TERRACOTTA",
        registry_id: "minecraft:light_blue_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 472,
        field_name: "YELLOW_TERRACOTTA",
        registry_id: "minecraft:yellow_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 473,
        field_name: "LIME_TERRACOTTA",
        registry_id: "minecraft:lime_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 474,
        field_name: "PINK_TERRACOTTA",
        registry_id: "minecraft:pink_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 475,
        field_name: "GRAY_TERRACOTTA",
        registry_id: "minecraft:gray_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 476,
        field_name: "LIGHT_GRAY_TERRACOTTA",
        registry_id: "minecraft:light_gray_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 477,
        field_name: "CYAN_TERRACOTTA",
        registry_id: "minecraft:cyan_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 478,
        field_name: "PURPLE_TERRACOTTA",
        registry_id: "minecraft:purple_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 479,
        field_name: "BLUE_TERRACOTTA",
        registry_id: "minecraft:blue_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 480,
        field_name: "BROWN_TERRACOTTA",
        registry_id: "minecraft:brown_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 481,
        field_name: "GREEN_TERRACOTTA",
        registry_id: "minecraft:green_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 482,
        field_name: "RED_TERRACOTTA",
        registry_id: "minecraft:red_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 483,
        field_name: "BLACK_TERRACOTTA",
        registry_id: "minecraft:black_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 484,
        field_name: "WHITE_STAINED_GLASS_PANE",
        registry_id: "minecraft:white_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 485,
        field_name: "ORANGE_STAINED_GLASS_PANE",
        registry_id: "minecraft:orange_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 486,
        field_name: "MAGENTA_STAINED_GLASS_PANE",
        registry_id: "minecraft:magenta_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 487,
        field_name: "LIGHT_BLUE_STAINED_GLASS_PANE",
        registry_id: "minecraft:light_blue_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 488,
        field_name: "YELLOW_STAINED_GLASS_PANE",
        registry_id: "minecraft:yellow_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 489,
        field_name: "LIME_STAINED_GLASS_PANE",
        registry_id: "minecraft:lime_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 490,
        field_name: "PINK_STAINED_GLASS_PANE",
        registry_id: "minecraft:pink_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 491,
        field_name: "GRAY_STAINED_GLASS_PANE",
        registry_id: "minecraft:gray_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 492,
        field_name: "LIGHT_GRAY_STAINED_GLASS_PANE",
        registry_id: "minecraft:light_gray_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 493,
        field_name: "CYAN_STAINED_GLASS_PANE",
        registry_id: "minecraft:cyan_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 494,
        field_name: "PURPLE_STAINED_GLASS_PANE",
        registry_id: "minecraft:purple_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 495,
        field_name: "BLUE_STAINED_GLASS_PANE",
        registry_id: "minecraft:blue_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 496,
        field_name: "BROWN_STAINED_GLASS_PANE",
        registry_id: "minecraft:brown_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 497,
        field_name: "GREEN_STAINED_GLASS_PANE",
        registry_id: "minecraft:green_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 498,
        field_name: "RED_STAINED_GLASS_PANE",
        registry_id: "minecraft:red_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 499,
        field_name: "BLACK_STAINED_GLASS_PANE",
        registry_id: "minecraft:black_stained_glass_pane",
    },
    BlockRegistryEntry {
        numeric_id: 500,
        field_name: "ACACIA_STAIRS",
        registry_id: "minecraft:acacia_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 501,
        field_name: "CHERRY_STAIRS",
        registry_id: "minecraft:cherry_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 502,
        field_name: "DARK_OAK_STAIRS",
        registry_id: "minecraft:dark_oak_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 503,
        field_name: "PALE_OAK_STAIRS",
        registry_id: "minecraft:pale_oak_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 504,
        field_name: "MANGROVE_STAIRS",
        registry_id: "minecraft:mangrove_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 505,
        field_name: "BAMBOO_STAIRS",
        registry_id: "minecraft:bamboo_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 506,
        field_name: "BAMBOO_MOSAIC_STAIRS",
        registry_id: "minecraft:bamboo_mosaic_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 507,
        field_name: "SLIME_BLOCK",
        registry_id: "minecraft:slime_block",
    },
    BlockRegistryEntry {
        numeric_id: 508,
        field_name: "BARRIER",
        registry_id: "minecraft:barrier",
    },
    BlockRegistryEntry {
        numeric_id: 509,
        field_name: "LIGHT",
        registry_id: "minecraft:light",
    },
    BlockRegistryEntry {
        numeric_id: 510,
        field_name: "IRON_TRAPDOOR",
        registry_id: "minecraft:iron_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 511,
        field_name: "PRISMARINE",
        registry_id: "minecraft:prismarine",
    },
    BlockRegistryEntry {
        numeric_id: 512,
        field_name: "PRISMARINE_BRICKS",
        registry_id: "minecraft:prismarine_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 513,
        field_name: "DARK_PRISMARINE",
        registry_id: "minecraft:dark_prismarine",
    },
    BlockRegistryEntry {
        numeric_id: 514,
        field_name: "PRISMARINE_STAIRS",
        registry_id: "minecraft:prismarine_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 515,
        field_name: "PRISMARINE_BRICK_STAIRS",
        registry_id: "minecraft:prismarine_brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 516,
        field_name: "DARK_PRISMARINE_STAIRS",
        registry_id: "minecraft:dark_prismarine_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 517,
        field_name: "PRISMARINE_SLAB",
        registry_id: "minecraft:prismarine_slab",
    },
    BlockRegistryEntry {
        numeric_id: 518,
        field_name: "PRISMARINE_BRICK_SLAB",
        registry_id: "minecraft:prismarine_brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 519,
        field_name: "DARK_PRISMARINE_SLAB",
        registry_id: "minecraft:dark_prismarine_slab",
    },
    BlockRegistryEntry {
        numeric_id: 520,
        field_name: "SEA_LANTERN",
        registry_id: "minecraft:sea_lantern",
    },
    BlockRegistryEntry {
        numeric_id: 521,
        field_name: "HAY_BLOCK",
        registry_id: "minecraft:hay_block",
    },
    BlockRegistryEntry {
        numeric_id: 522,
        field_name: "WHITE_CARPET",
        registry_id: "minecraft:white_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 523,
        field_name: "ORANGE_CARPET",
        registry_id: "minecraft:orange_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 524,
        field_name: "MAGENTA_CARPET",
        registry_id: "minecraft:magenta_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 525,
        field_name: "LIGHT_BLUE_CARPET",
        registry_id: "minecraft:light_blue_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 526,
        field_name: "YELLOW_CARPET",
        registry_id: "minecraft:yellow_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 527,
        field_name: "LIME_CARPET",
        registry_id: "minecraft:lime_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 528,
        field_name: "PINK_CARPET",
        registry_id: "minecraft:pink_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 529,
        field_name: "GRAY_CARPET",
        registry_id: "minecraft:gray_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 530,
        field_name: "LIGHT_GRAY_CARPET",
        registry_id: "minecraft:light_gray_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 531,
        field_name: "CYAN_CARPET",
        registry_id: "minecraft:cyan_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 532,
        field_name: "PURPLE_CARPET",
        registry_id: "minecraft:purple_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 533,
        field_name: "BLUE_CARPET",
        registry_id: "minecraft:blue_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 534,
        field_name: "BROWN_CARPET",
        registry_id: "minecraft:brown_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 535,
        field_name: "GREEN_CARPET",
        registry_id: "minecraft:green_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 536,
        field_name: "RED_CARPET",
        registry_id: "minecraft:red_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 537,
        field_name: "BLACK_CARPET",
        registry_id: "minecraft:black_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 538,
        field_name: "TERRACOTTA",
        registry_id: "minecraft:terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 539,
        field_name: "COAL_BLOCK",
        registry_id: "minecraft:coal_block",
    },
    BlockRegistryEntry {
        numeric_id: 540,
        field_name: "PACKED_ICE",
        registry_id: "minecraft:packed_ice",
    },
    BlockRegistryEntry {
        numeric_id: 541,
        field_name: "SUNFLOWER",
        registry_id: "minecraft:sunflower",
    },
    BlockRegistryEntry {
        numeric_id: 542,
        field_name: "LILAC",
        registry_id: "minecraft:lilac",
    },
    BlockRegistryEntry {
        numeric_id: 543,
        field_name: "ROSE_BUSH",
        registry_id: "minecraft:rose_bush",
    },
    BlockRegistryEntry {
        numeric_id: 544,
        field_name: "PEONY",
        registry_id: "minecraft:peony",
    },
    BlockRegistryEntry {
        numeric_id: 545,
        field_name: "TALL_GRASS",
        registry_id: "minecraft:tall_grass",
    },
    BlockRegistryEntry {
        numeric_id: 546,
        field_name: "LARGE_FERN",
        registry_id: "minecraft:large_fern",
    },
    BlockRegistryEntry {
        numeric_id: 547,
        field_name: "WHITE_BANNER",
        registry_id: "minecraft:white_banner",
    },
    BlockRegistryEntry {
        numeric_id: 548,
        field_name: "ORANGE_BANNER",
        registry_id: "minecraft:orange_banner",
    },
    BlockRegistryEntry {
        numeric_id: 549,
        field_name: "MAGENTA_BANNER",
        registry_id: "minecraft:magenta_banner",
    },
    BlockRegistryEntry {
        numeric_id: 550,
        field_name: "LIGHT_BLUE_BANNER",
        registry_id: "minecraft:light_blue_banner",
    },
    BlockRegistryEntry {
        numeric_id: 551,
        field_name: "YELLOW_BANNER",
        registry_id: "minecraft:yellow_banner",
    },
    BlockRegistryEntry {
        numeric_id: 552,
        field_name: "LIME_BANNER",
        registry_id: "minecraft:lime_banner",
    },
    BlockRegistryEntry {
        numeric_id: 553,
        field_name: "PINK_BANNER",
        registry_id: "minecraft:pink_banner",
    },
    BlockRegistryEntry {
        numeric_id: 554,
        field_name: "GRAY_BANNER",
        registry_id: "minecraft:gray_banner",
    },
    BlockRegistryEntry {
        numeric_id: 555,
        field_name: "LIGHT_GRAY_BANNER",
        registry_id: "minecraft:light_gray_banner",
    },
    BlockRegistryEntry {
        numeric_id: 556,
        field_name: "CYAN_BANNER",
        registry_id: "minecraft:cyan_banner",
    },
    BlockRegistryEntry {
        numeric_id: 557,
        field_name: "PURPLE_BANNER",
        registry_id: "minecraft:purple_banner",
    },
    BlockRegistryEntry {
        numeric_id: 558,
        field_name: "BLUE_BANNER",
        registry_id: "minecraft:blue_banner",
    },
    BlockRegistryEntry {
        numeric_id: 559,
        field_name: "BROWN_BANNER",
        registry_id: "minecraft:brown_banner",
    },
    BlockRegistryEntry {
        numeric_id: 560,
        field_name: "GREEN_BANNER",
        registry_id: "minecraft:green_banner",
    },
    BlockRegistryEntry {
        numeric_id: 561,
        field_name: "RED_BANNER",
        registry_id: "minecraft:red_banner",
    },
    BlockRegistryEntry {
        numeric_id: 562,
        field_name: "BLACK_BANNER",
        registry_id: "minecraft:black_banner",
    },
    BlockRegistryEntry {
        numeric_id: 563,
        field_name: "WHITE_WALL_BANNER",
        registry_id: "minecraft:white_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 564,
        field_name: "ORANGE_WALL_BANNER",
        registry_id: "minecraft:orange_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 565,
        field_name: "MAGENTA_WALL_BANNER",
        registry_id: "minecraft:magenta_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 566,
        field_name: "LIGHT_BLUE_WALL_BANNER",
        registry_id: "minecraft:light_blue_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 567,
        field_name: "YELLOW_WALL_BANNER",
        registry_id: "minecraft:yellow_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 568,
        field_name: "LIME_WALL_BANNER",
        registry_id: "minecraft:lime_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 569,
        field_name: "PINK_WALL_BANNER",
        registry_id: "minecraft:pink_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 570,
        field_name: "GRAY_WALL_BANNER",
        registry_id: "minecraft:gray_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 571,
        field_name: "LIGHT_GRAY_WALL_BANNER",
        registry_id: "minecraft:light_gray_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 572,
        field_name: "CYAN_WALL_BANNER",
        registry_id: "minecraft:cyan_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 573,
        field_name: "PURPLE_WALL_BANNER",
        registry_id: "minecraft:purple_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 574,
        field_name: "BLUE_WALL_BANNER",
        registry_id: "minecraft:blue_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 575,
        field_name: "BROWN_WALL_BANNER",
        registry_id: "minecraft:brown_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 576,
        field_name: "GREEN_WALL_BANNER",
        registry_id: "minecraft:green_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 577,
        field_name: "RED_WALL_BANNER",
        registry_id: "minecraft:red_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 578,
        field_name: "BLACK_WALL_BANNER",
        registry_id: "minecraft:black_wall_banner",
    },
    BlockRegistryEntry {
        numeric_id: 579,
        field_name: "RED_SANDSTONE",
        registry_id: "minecraft:red_sandstone",
    },
    BlockRegistryEntry {
        numeric_id: 580,
        field_name: "CHISELED_RED_SANDSTONE",
        registry_id: "minecraft:chiseled_red_sandstone",
    },
    BlockRegistryEntry {
        numeric_id: 581,
        field_name: "CUT_RED_SANDSTONE",
        registry_id: "minecraft:cut_red_sandstone",
    },
    BlockRegistryEntry {
        numeric_id: 582,
        field_name: "RED_SANDSTONE_STAIRS",
        registry_id: "minecraft:red_sandstone_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 583,
        field_name: "OAK_SLAB",
        registry_id: "minecraft:oak_slab",
    },
    BlockRegistryEntry {
        numeric_id: 584,
        field_name: "SPRUCE_SLAB",
        registry_id: "minecraft:spruce_slab",
    },
    BlockRegistryEntry {
        numeric_id: 585,
        field_name: "BIRCH_SLAB",
        registry_id: "minecraft:birch_slab",
    },
    BlockRegistryEntry {
        numeric_id: 586,
        field_name: "JUNGLE_SLAB",
        registry_id: "minecraft:jungle_slab",
    },
    BlockRegistryEntry {
        numeric_id: 587,
        field_name: "ACACIA_SLAB",
        registry_id: "minecraft:acacia_slab",
    },
    BlockRegistryEntry {
        numeric_id: 588,
        field_name: "CHERRY_SLAB",
        registry_id: "minecraft:cherry_slab",
    },
    BlockRegistryEntry {
        numeric_id: 589,
        field_name: "DARK_OAK_SLAB",
        registry_id: "minecraft:dark_oak_slab",
    },
    BlockRegistryEntry {
        numeric_id: 590,
        field_name: "PALE_OAK_SLAB",
        registry_id: "minecraft:pale_oak_slab",
    },
    BlockRegistryEntry {
        numeric_id: 591,
        field_name: "MANGROVE_SLAB",
        registry_id: "minecraft:mangrove_slab",
    },
    BlockRegistryEntry {
        numeric_id: 592,
        field_name: "BAMBOO_SLAB",
        registry_id: "minecraft:bamboo_slab",
    },
    BlockRegistryEntry {
        numeric_id: 593,
        field_name: "BAMBOO_MOSAIC_SLAB",
        registry_id: "minecraft:bamboo_mosaic_slab",
    },
    BlockRegistryEntry {
        numeric_id: 594,
        field_name: "STONE_SLAB",
        registry_id: "minecraft:stone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 595,
        field_name: "SMOOTH_STONE_SLAB",
        registry_id: "minecraft:smooth_stone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 596,
        field_name: "SANDSTONE_SLAB",
        registry_id: "minecraft:sandstone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 597,
        field_name: "CUT_SANDSTONE_SLAB",
        registry_id: "minecraft:cut_sandstone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 598,
        field_name: "PETRIFIED_OAK_SLAB",
        registry_id: "minecraft:petrified_oak_slab",
    },
    BlockRegistryEntry {
        numeric_id: 599,
        field_name: "COBBLESTONE_SLAB",
        registry_id: "minecraft:cobblestone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 600,
        field_name: "BRICK_SLAB",
        registry_id: "minecraft:brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 601,
        field_name: "STONE_BRICK_SLAB",
        registry_id: "minecraft:stone_brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 602,
        field_name: "MUD_BRICK_SLAB",
        registry_id: "minecraft:mud_brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 603,
        field_name: "NETHER_BRICK_SLAB",
        registry_id: "minecraft:nether_brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 604,
        field_name: "QUARTZ_SLAB",
        registry_id: "minecraft:quartz_slab",
    },
    BlockRegistryEntry {
        numeric_id: 605,
        field_name: "RED_SANDSTONE_SLAB",
        registry_id: "minecraft:red_sandstone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 606,
        field_name: "CUT_RED_SANDSTONE_SLAB",
        registry_id: "minecraft:cut_red_sandstone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 607,
        field_name: "PURPUR_SLAB",
        registry_id: "minecraft:purpur_slab",
    },
    BlockRegistryEntry {
        numeric_id: 608,
        field_name: "SMOOTH_STONE",
        registry_id: "minecraft:smooth_stone",
    },
    BlockRegistryEntry {
        numeric_id: 609,
        field_name: "SMOOTH_SANDSTONE",
        registry_id: "minecraft:smooth_sandstone",
    },
    BlockRegistryEntry {
        numeric_id: 610,
        field_name: "SMOOTH_QUARTZ",
        registry_id: "minecraft:smooth_quartz",
    },
    BlockRegistryEntry {
        numeric_id: 611,
        field_name: "SMOOTH_RED_SANDSTONE",
        registry_id: "minecraft:smooth_red_sandstone",
    },
    BlockRegistryEntry {
        numeric_id: 612,
        field_name: "SPRUCE_FENCE_GATE",
        registry_id: "minecraft:spruce_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 613,
        field_name: "BIRCH_FENCE_GATE",
        registry_id: "minecraft:birch_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 614,
        field_name: "JUNGLE_FENCE_GATE",
        registry_id: "minecraft:jungle_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 615,
        field_name: "ACACIA_FENCE_GATE",
        registry_id: "minecraft:acacia_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 616,
        field_name: "CHERRY_FENCE_GATE",
        registry_id: "minecraft:cherry_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 617,
        field_name: "DARK_OAK_FENCE_GATE",
        registry_id: "minecraft:dark_oak_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 618,
        field_name: "PALE_OAK_FENCE_GATE",
        registry_id: "minecraft:pale_oak_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 619,
        field_name: "MANGROVE_FENCE_GATE",
        registry_id: "minecraft:mangrove_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 620,
        field_name: "BAMBOO_FENCE_GATE",
        registry_id: "minecraft:bamboo_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 621,
        field_name: "SPRUCE_FENCE",
        registry_id: "minecraft:spruce_fence",
    },
    BlockRegistryEntry {
        numeric_id: 622,
        field_name: "BIRCH_FENCE",
        registry_id: "minecraft:birch_fence",
    },
    BlockRegistryEntry {
        numeric_id: 623,
        field_name: "JUNGLE_FENCE",
        registry_id: "minecraft:jungle_fence",
    },
    BlockRegistryEntry {
        numeric_id: 624,
        field_name: "ACACIA_FENCE",
        registry_id: "minecraft:acacia_fence",
    },
    BlockRegistryEntry {
        numeric_id: 625,
        field_name: "CHERRY_FENCE",
        registry_id: "minecraft:cherry_fence",
    },
    BlockRegistryEntry {
        numeric_id: 626,
        field_name: "DARK_OAK_FENCE",
        registry_id: "minecraft:dark_oak_fence",
    },
    BlockRegistryEntry {
        numeric_id: 627,
        field_name: "PALE_OAK_FENCE",
        registry_id: "minecraft:pale_oak_fence",
    },
    BlockRegistryEntry {
        numeric_id: 628,
        field_name: "MANGROVE_FENCE",
        registry_id: "minecraft:mangrove_fence",
    },
    BlockRegistryEntry {
        numeric_id: 629,
        field_name: "BAMBOO_FENCE",
        registry_id: "minecraft:bamboo_fence",
    },
    BlockRegistryEntry {
        numeric_id: 630,
        field_name: "SPRUCE_DOOR",
        registry_id: "minecraft:spruce_door",
    },
    BlockRegistryEntry {
        numeric_id: 631,
        field_name: "BIRCH_DOOR",
        registry_id: "minecraft:birch_door",
    },
    BlockRegistryEntry {
        numeric_id: 632,
        field_name: "JUNGLE_DOOR",
        registry_id: "minecraft:jungle_door",
    },
    BlockRegistryEntry {
        numeric_id: 633,
        field_name: "ACACIA_DOOR",
        registry_id: "minecraft:acacia_door",
    },
    BlockRegistryEntry {
        numeric_id: 634,
        field_name: "CHERRY_DOOR",
        registry_id: "minecraft:cherry_door",
    },
    BlockRegistryEntry {
        numeric_id: 635,
        field_name: "DARK_OAK_DOOR",
        registry_id: "minecraft:dark_oak_door",
    },
    BlockRegistryEntry {
        numeric_id: 636,
        field_name: "PALE_OAK_DOOR",
        registry_id: "minecraft:pale_oak_door",
    },
    BlockRegistryEntry {
        numeric_id: 637,
        field_name: "MANGROVE_DOOR",
        registry_id: "minecraft:mangrove_door",
    },
    BlockRegistryEntry {
        numeric_id: 638,
        field_name: "BAMBOO_DOOR",
        registry_id: "minecraft:bamboo_door",
    },
    BlockRegistryEntry {
        numeric_id: 639,
        field_name: "END_ROD",
        registry_id: "minecraft:end_rod",
    },
    BlockRegistryEntry {
        numeric_id: 640,
        field_name: "CHORUS_PLANT",
        registry_id: "minecraft:chorus_plant",
    },
    BlockRegistryEntry {
        numeric_id: 641,
        field_name: "CHORUS_FLOWER",
        registry_id: "minecraft:chorus_flower",
    },
    BlockRegistryEntry {
        numeric_id: 642,
        field_name: "PURPUR_BLOCK",
        registry_id: "minecraft:purpur_block",
    },
    BlockRegistryEntry {
        numeric_id: 643,
        field_name: "PURPUR_PILLAR",
        registry_id: "minecraft:purpur_pillar",
    },
    BlockRegistryEntry {
        numeric_id: 644,
        field_name: "PURPUR_STAIRS",
        registry_id: "minecraft:purpur_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 645,
        field_name: "END_STONE_BRICKS",
        registry_id: "minecraft:end_stone_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 646,
        field_name: "TORCHFLOWER_CROP",
        registry_id: "minecraft:torchflower_crop",
    },
    BlockRegistryEntry {
        numeric_id: 647,
        field_name: "PITCHER_CROP",
        registry_id: "minecraft:pitcher_crop",
    },
    BlockRegistryEntry {
        numeric_id: 648,
        field_name: "PITCHER_PLANT",
        registry_id: "minecraft:pitcher_plant",
    },
    BlockRegistryEntry {
        numeric_id: 649,
        field_name: "BEETROOTS",
        registry_id: "minecraft:beetroots",
    },
    BlockRegistryEntry {
        numeric_id: 650,
        field_name: "DIRT_PATH",
        registry_id: "minecraft:dirt_path",
    },
    BlockRegistryEntry {
        numeric_id: 651,
        field_name: "END_GATEWAY",
        registry_id: "minecraft:end_gateway",
    },
    BlockRegistryEntry {
        numeric_id: 652,
        field_name: "REPEATING_COMMAND_BLOCK",
        registry_id: "minecraft:repeating_command_block",
    },
    BlockRegistryEntry {
        numeric_id: 653,
        field_name: "CHAIN_COMMAND_BLOCK",
        registry_id: "minecraft:chain_command_block",
    },
    BlockRegistryEntry {
        numeric_id: 654,
        field_name: "FROSTED_ICE",
        registry_id: "minecraft:frosted_ice",
    },
    BlockRegistryEntry {
        numeric_id: 655,
        field_name: "MAGMA_BLOCK",
        registry_id: "minecraft:magma_block",
    },
    BlockRegistryEntry {
        numeric_id: 656,
        field_name: "NETHER_WART_BLOCK",
        registry_id: "minecraft:nether_wart_block",
    },
    BlockRegistryEntry {
        numeric_id: 657,
        field_name: "RED_NETHER_BRICKS",
        registry_id: "minecraft:red_nether_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 658,
        field_name: "BONE_BLOCK",
        registry_id: "minecraft:bone_block",
    },
    BlockRegistryEntry {
        numeric_id: 659,
        field_name: "STRUCTURE_VOID",
        registry_id: "minecraft:structure_void",
    },
    BlockRegistryEntry {
        numeric_id: 660,
        field_name: "OBSERVER",
        registry_id: "minecraft:observer",
    },
    BlockRegistryEntry {
        numeric_id: 661,
        field_name: "SHULKER_BOX",
        registry_id: "minecraft:shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 662,
        field_name: "WHITE_SHULKER_BOX",
        registry_id: "minecraft:white_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 663,
        field_name: "ORANGE_SHULKER_BOX",
        registry_id: "minecraft:orange_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 664,
        field_name: "MAGENTA_SHULKER_BOX",
        registry_id: "minecraft:magenta_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 665,
        field_name: "LIGHT_BLUE_SHULKER_BOX",
        registry_id: "minecraft:light_blue_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 666,
        field_name: "YELLOW_SHULKER_BOX",
        registry_id: "minecraft:yellow_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 667,
        field_name: "LIME_SHULKER_BOX",
        registry_id: "minecraft:lime_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 668,
        field_name: "PINK_SHULKER_BOX",
        registry_id: "minecraft:pink_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 669,
        field_name: "GRAY_SHULKER_BOX",
        registry_id: "minecraft:gray_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 670,
        field_name: "LIGHT_GRAY_SHULKER_BOX",
        registry_id: "minecraft:light_gray_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 671,
        field_name: "CYAN_SHULKER_BOX",
        registry_id: "minecraft:cyan_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 672,
        field_name: "PURPLE_SHULKER_BOX",
        registry_id: "minecraft:purple_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 673,
        field_name: "BLUE_SHULKER_BOX",
        registry_id: "minecraft:blue_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 674,
        field_name: "BROWN_SHULKER_BOX",
        registry_id: "minecraft:brown_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 675,
        field_name: "GREEN_SHULKER_BOX",
        registry_id: "minecraft:green_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 676,
        field_name: "RED_SHULKER_BOX",
        registry_id: "minecraft:red_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 677,
        field_name: "BLACK_SHULKER_BOX",
        registry_id: "minecraft:black_shulker_box",
    },
    BlockRegistryEntry {
        numeric_id: 678,
        field_name: "WHITE_GLAZED_TERRACOTTA",
        registry_id: "minecraft:white_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 679,
        field_name: "ORANGE_GLAZED_TERRACOTTA",
        registry_id: "minecraft:orange_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 680,
        field_name: "MAGENTA_GLAZED_TERRACOTTA",
        registry_id: "minecraft:magenta_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 681,
        field_name: "LIGHT_BLUE_GLAZED_TERRACOTTA",
        registry_id: "minecraft:light_blue_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 682,
        field_name: "YELLOW_GLAZED_TERRACOTTA",
        registry_id: "minecraft:yellow_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 683,
        field_name: "LIME_GLAZED_TERRACOTTA",
        registry_id: "minecraft:lime_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 684,
        field_name: "PINK_GLAZED_TERRACOTTA",
        registry_id: "minecraft:pink_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 685,
        field_name: "GRAY_GLAZED_TERRACOTTA",
        registry_id: "minecraft:gray_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 686,
        field_name: "LIGHT_GRAY_GLAZED_TERRACOTTA",
        registry_id: "minecraft:light_gray_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 687,
        field_name: "CYAN_GLAZED_TERRACOTTA",
        registry_id: "minecraft:cyan_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 688,
        field_name: "PURPLE_GLAZED_TERRACOTTA",
        registry_id: "minecraft:purple_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 689,
        field_name: "BLUE_GLAZED_TERRACOTTA",
        registry_id: "minecraft:blue_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 690,
        field_name: "BROWN_GLAZED_TERRACOTTA",
        registry_id: "minecraft:brown_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 691,
        field_name: "GREEN_GLAZED_TERRACOTTA",
        registry_id: "minecraft:green_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 692,
        field_name: "RED_GLAZED_TERRACOTTA",
        registry_id: "minecraft:red_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 693,
        field_name: "BLACK_GLAZED_TERRACOTTA",
        registry_id: "minecraft:black_glazed_terracotta",
    },
    BlockRegistryEntry {
        numeric_id: 694,
        field_name: "WHITE_CONCRETE",
        registry_id: "minecraft:white_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 695,
        field_name: "ORANGE_CONCRETE",
        registry_id: "minecraft:orange_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 696,
        field_name: "MAGENTA_CONCRETE",
        registry_id: "minecraft:magenta_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 697,
        field_name: "LIGHT_BLUE_CONCRETE",
        registry_id: "minecraft:light_blue_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 698,
        field_name: "YELLOW_CONCRETE",
        registry_id: "minecraft:yellow_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 699,
        field_name: "LIME_CONCRETE",
        registry_id: "minecraft:lime_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 700,
        field_name: "PINK_CONCRETE",
        registry_id: "minecraft:pink_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 701,
        field_name: "GRAY_CONCRETE",
        registry_id: "minecraft:gray_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 702,
        field_name: "LIGHT_GRAY_CONCRETE",
        registry_id: "minecraft:light_gray_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 703,
        field_name: "CYAN_CONCRETE",
        registry_id: "minecraft:cyan_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 704,
        field_name: "PURPLE_CONCRETE",
        registry_id: "minecraft:purple_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 705,
        field_name: "BLUE_CONCRETE",
        registry_id: "minecraft:blue_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 706,
        field_name: "BROWN_CONCRETE",
        registry_id: "minecraft:brown_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 707,
        field_name: "GREEN_CONCRETE",
        registry_id: "minecraft:green_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 708,
        field_name: "RED_CONCRETE",
        registry_id: "minecraft:red_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 709,
        field_name: "BLACK_CONCRETE",
        registry_id: "minecraft:black_concrete",
    },
    BlockRegistryEntry {
        numeric_id: 710,
        field_name: "WHITE_CONCRETE_POWDER",
        registry_id: "minecraft:white_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 711,
        field_name: "ORANGE_CONCRETE_POWDER",
        registry_id: "minecraft:orange_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 712,
        field_name: "MAGENTA_CONCRETE_POWDER",
        registry_id: "minecraft:magenta_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 713,
        field_name: "LIGHT_BLUE_CONCRETE_POWDER",
        registry_id: "minecraft:light_blue_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 714,
        field_name: "YELLOW_CONCRETE_POWDER",
        registry_id: "minecraft:yellow_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 715,
        field_name: "LIME_CONCRETE_POWDER",
        registry_id: "minecraft:lime_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 716,
        field_name: "PINK_CONCRETE_POWDER",
        registry_id: "minecraft:pink_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 717,
        field_name: "GRAY_CONCRETE_POWDER",
        registry_id: "minecraft:gray_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 718,
        field_name: "LIGHT_GRAY_CONCRETE_POWDER",
        registry_id: "minecraft:light_gray_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 719,
        field_name: "CYAN_CONCRETE_POWDER",
        registry_id: "minecraft:cyan_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 720,
        field_name: "PURPLE_CONCRETE_POWDER",
        registry_id: "minecraft:purple_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 721,
        field_name: "BLUE_CONCRETE_POWDER",
        registry_id: "minecraft:blue_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 722,
        field_name: "BROWN_CONCRETE_POWDER",
        registry_id: "minecraft:brown_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 723,
        field_name: "GREEN_CONCRETE_POWDER",
        registry_id: "minecraft:green_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 724,
        field_name: "RED_CONCRETE_POWDER",
        registry_id: "minecraft:red_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 725,
        field_name: "BLACK_CONCRETE_POWDER",
        registry_id: "minecraft:black_concrete_powder",
    },
    BlockRegistryEntry {
        numeric_id: 726,
        field_name: "KELP",
        registry_id: "minecraft:kelp",
    },
    BlockRegistryEntry {
        numeric_id: 727,
        field_name: "KELP_PLANT",
        registry_id: "minecraft:kelp_plant",
    },
    BlockRegistryEntry {
        numeric_id: 728,
        field_name: "DRIED_KELP_BLOCK",
        registry_id: "minecraft:dried_kelp_block",
    },
    BlockRegistryEntry {
        numeric_id: 729,
        field_name: "TURTLE_EGG",
        registry_id: "minecraft:turtle_egg",
    },
    BlockRegistryEntry {
        numeric_id: 730,
        field_name: "SNIFFER_EGG",
        registry_id: "minecraft:sniffer_egg",
    },
    BlockRegistryEntry {
        numeric_id: 731,
        field_name: "DRIED_GHAST",
        registry_id: "minecraft:dried_ghast",
    },
    BlockRegistryEntry {
        numeric_id: 732,
        field_name: "DEAD_TUBE_CORAL_BLOCK",
        registry_id: "minecraft:dead_tube_coral_block",
    },
    BlockRegistryEntry {
        numeric_id: 733,
        field_name: "DEAD_BRAIN_CORAL_BLOCK",
        registry_id: "minecraft:dead_brain_coral_block",
    },
    BlockRegistryEntry {
        numeric_id: 734,
        field_name: "DEAD_BUBBLE_CORAL_BLOCK",
        registry_id: "minecraft:dead_bubble_coral_block",
    },
    BlockRegistryEntry {
        numeric_id: 735,
        field_name: "DEAD_FIRE_CORAL_BLOCK",
        registry_id: "minecraft:dead_fire_coral_block",
    },
    BlockRegistryEntry {
        numeric_id: 736,
        field_name: "DEAD_HORN_CORAL_BLOCK",
        registry_id: "minecraft:dead_horn_coral_block",
    },
    BlockRegistryEntry {
        numeric_id: 737,
        field_name: "TUBE_CORAL_BLOCK",
        registry_id: "minecraft:tube_coral_block",
    },
    BlockRegistryEntry {
        numeric_id: 738,
        field_name: "BRAIN_CORAL_BLOCK",
        registry_id: "minecraft:brain_coral_block",
    },
    BlockRegistryEntry {
        numeric_id: 739,
        field_name: "BUBBLE_CORAL_BLOCK",
        registry_id: "minecraft:bubble_coral_block",
    },
    BlockRegistryEntry {
        numeric_id: 740,
        field_name: "FIRE_CORAL_BLOCK",
        registry_id: "minecraft:fire_coral_block",
    },
    BlockRegistryEntry {
        numeric_id: 741,
        field_name: "HORN_CORAL_BLOCK",
        registry_id: "minecraft:horn_coral_block",
    },
    BlockRegistryEntry {
        numeric_id: 742,
        field_name: "DEAD_TUBE_CORAL",
        registry_id: "minecraft:dead_tube_coral",
    },
    BlockRegistryEntry {
        numeric_id: 743,
        field_name: "DEAD_BRAIN_CORAL",
        registry_id: "minecraft:dead_brain_coral",
    },
    BlockRegistryEntry {
        numeric_id: 744,
        field_name: "DEAD_BUBBLE_CORAL",
        registry_id: "minecraft:dead_bubble_coral",
    },
    BlockRegistryEntry {
        numeric_id: 745,
        field_name: "DEAD_FIRE_CORAL",
        registry_id: "minecraft:dead_fire_coral",
    },
    BlockRegistryEntry {
        numeric_id: 746,
        field_name: "DEAD_HORN_CORAL",
        registry_id: "minecraft:dead_horn_coral",
    },
    BlockRegistryEntry {
        numeric_id: 747,
        field_name: "TUBE_CORAL",
        registry_id: "minecraft:tube_coral",
    },
    BlockRegistryEntry {
        numeric_id: 748,
        field_name: "BRAIN_CORAL",
        registry_id: "minecraft:brain_coral",
    },
    BlockRegistryEntry {
        numeric_id: 749,
        field_name: "BUBBLE_CORAL",
        registry_id: "minecraft:bubble_coral",
    },
    BlockRegistryEntry {
        numeric_id: 750,
        field_name: "FIRE_CORAL",
        registry_id: "minecraft:fire_coral",
    },
    BlockRegistryEntry {
        numeric_id: 751,
        field_name: "HORN_CORAL",
        registry_id: "minecraft:horn_coral",
    },
    BlockRegistryEntry {
        numeric_id: 752,
        field_name: "DEAD_TUBE_CORAL_FAN",
        registry_id: "minecraft:dead_tube_coral_fan",
    },
    BlockRegistryEntry {
        numeric_id: 753,
        field_name: "DEAD_BRAIN_CORAL_FAN",
        registry_id: "minecraft:dead_brain_coral_fan",
    },
    BlockRegistryEntry {
        numeric_id: 754,
        field_name: "DEAD_BUBBLE_CORAL_FAN",
        registry_id: "minecraft:dead_bubble_coral_fan",
    },
    BlockRegistryEntry {
        numeric_id: 755,
        field_name: "DEAD_FIRE_CORAL_FAN",
        registry_id: "minecraft:dead_fire_coral_fan",
    },
    BlockRegistryEntry {
        numeric_id: 756,
        field_name: "DEAD_HORN_CORAL_FAN",
        registry_id: "minecraft:dead_horn_coral_fan",
    },
    BlockRegistryEntry {
        numeric_id: 757,
        field_name: "TUBE_CORAL_FAN",
        registry_id: "minecraft:tube_coral_fan",
    },
    BlockRegistryEntry {
        numeric_id: 758,
        field_name: "BRAIN_CORAL_FAN",
        registry_id: "minecraft:brain_coral_fan",
    },
    BlockRegistryEntry {
        numeric_id: 759,
        field_name: "BUBBLE_CORAL_FAN",
        registry_id: "minecraft:bubble_coral_fan",
    },
    BlockRegistryEntry {
        numeric_id: 760,
        field_name: "FIRE_CORAL_FAN",
        registry_id: "minecraft:fire_coral_fan",
    },
    BlockRegistryEntry {
        numeric_id: 761,
        field_name: "HORN_CORAL_FAN",
        registry_id: "minecraft:horn_coral_fan",
    },
    BlockRegistryEntry {
        numeric_id: 762,
        field_name: "DEAD_TUBE_CORAL_WALL_FAN",
        registry_id: "minecraft:dead_tube_coral_wall_fan",
    },
    BlockRegistryEntry {
        numeric_id: 763,
        field_name: "DEAD_BRAIN_CORAL_WALL_FAN",
        registry_id: "minecraft:dead_brain_coral_wall_fan",
    },
    BlockRegistryEntry {
        numeric_id: 764,
        field_name: "DEAD_BUBBLE_CORAL_WALL_FAN",
        registry_id: "minecraft:dead_bubble_coral_wall_fan",
    },
    BlockRegistryEntry {
        numeric_id: 765,
        field_name: "DEAD_FIRE_CORAL_WALL_FAN",
        registry_id: "minecraft:dead_fire_coral_wall_fan",
    },
    BlockRegistryEntry {
        numeric_id: 766,
        field_name: "DEAD_HORN_CORAL_WALL_FAN",
        registry_id: "minecraft:dead_horn_coral_wall_fan",
    },
    BlockRegistryEntry {
        numeric_id: 767,
        field_name: "TUBE_CORAL_WALL_FAN",
        registry_id: "minecraft:tube_coral_wall_fan",
    },
    BlockRegistryEntry {
        numeric_id: 768,
        field_name: "BRAIN_CORAL_WALL_FAN",
        registry_id: "minecraft:brain_coral_wall_fan",
    },
    BlockRegistryEntry {
        numeric_id: 769,
        field_name: "BUBBLE_CORAL_WALL_FAN",
        registry_id: "minecraft:bubble_coral_wall_fan",
    },
    BlockRegistryEntry {
        numeric_id: 770,
        field_name: "FIRE_CORAL_WALL_FAN",
        registry_id: "minecraft:fire_coral_wall_fan",
    },
    BlockRegistryEntry {
        numeric_id: 771,
        field_name: "HORN_CORAL_WALL_FAN",
        registry_id: "minecraft:horn_coral_wall_fan",
    },
    BlockRegistryEntry {
        numeric_id: 772,
        field_name: "SEA_PICKLE",
        registry_id: "minecraft:sea_pickle",
    },
    BlockRegistryEntry {
        numeric_id: 773,
        field_name: "BLUE_ICE",
        registry_id: "minecraft:blue_ice",
    },
    BlockRegistryEntry {
        numeric_id: 774,
        field_name: "CONDUIT",
        registry_id: "minecraft:conduit",
    },
    BlockRegistryEntry {
        numeric_id: 775,
        field_name: "BAMBOO_SAPLING",
        registry_id: "minecraft:bamboo_sapling",
    },
    BlockRegistryEntry {
        numeric_id: 776,
        field_name: "BAMBOO",
        registry_id: "minecraft:bamboo",
    },
    BlockRegistryEntry {
        numeric_id: 777,
        field_name: "POTTED_BAMBOO",
        registry_id: "minecraft:potted_bamboo",
    },
    BlockRegistryEntry {
        numeric_id: 778,
        field_name: "VOID_AIR",
        registry_id: "minecraft:void_air",
    },
    BlockRegistryEntry {
        numeric_id: 779,
        field_name: "CAVE_AIR",
        registry_id: "minecraft:cave_air",
    },
    BlockRegistryEntry {
        numeric_id: 780,
        field_name: "BUBBLE_COLUMN",
        registry_id: "minecraft:bubble_column",
    },
    BlockRegistryEntry {
        numeric_id: 781,
        field_name: "POLISHED_GRANITE_STAIRS",
        registry_id: "minecraft:polished_granite_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 782,
        field_name: "SMOOTH_RED_SANDSTONE_STAIRS",
        registry_id: "minecraft:smooth_red_sandstone_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 783,
        field_name: "MOSSY_STONE_BRICK_STAIRS",
        registry_id: "minecraft:mossy_stone_brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 784,
        field_name: "POLISHED_DIORITE_STAIRS",
        registry_id: "minecraft:polished_diorite_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 785,
        field_name: "MOSSY_COBBLESTONE_STAIRS",
        registry_id: "minecraft:mossy_cobblestone_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 786,
        field_name: "END_STONE_BRICK_STAIRS",
        registry_id: "minecraft:end_stone_brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 787,
        field_name: "STONE_STAIRS",
        registry_id: "minecraft:stone_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 788,
        field_name: "SMOOTH_SANDSTONE_STAIRS",
        registry_id: "minecraft:smooth_sandstone_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 789,
        field_name: "SMOOTH_QUARTZ_STAIRS",
        registry_id: "minecraft:smooth_quartz_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 790,
        field_name: "GRANITE_STAIRS",
        registry_id: "minecraft:granite_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 791,
        field_name: "ANDESITE_STAIRS",
        registry_id: "minecraft:andesite_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 792,
        field_name: "RED_NETHER_BRICK_STAIRS",
        registry_id: "minecraft:red_nether_brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 793,
        field_name: "POLISHED_ANDESITE_STAIRS",
        registry_id: "minecraft:polished_andesite_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 794,
        field_name: "DIORITE_STAIRS",
        registry_id: "minecraft:diorite_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 795,
        field_name: "POLISHED_GRANITE_SLAB",
        registry_id: "minecraft:polished_granite_slab",
    },
    BlockRegistryEntry {
        numeric_id: 796,
        field_name: "SMOOTH_RED_SANDSTONE_SLAB",
        registry_id: "minecraft:smooth_red_sandstone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 797,
        field_name: "MOSSY_STONE_BRICK_SLAB",
        registry_id: "minecraft:mossy_stone_brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 798,
        field_name: "POLISHED_DIORITE_SLAB",
        registry_id: "minecraft:polished_diorite_slab",
    },
    BlockRegistryEntry {
        numeric_id: 799,
        field_name: "MOSSY_COBBLESTONE_SLAB",
        registry_id: "minecraft:mossy_cobblestone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 800,
        field_name: "END_STONE_BRICK_SLAB",
        registry_id: "minecraft:end_stone_brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 801,
        field_name: "SMOOTH_SANDSTONE_SLAB",
        registry_id: "minecraft:smooth_sandstone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 802,
        field_name: "SMOOTH_QUARTZ_SLAB",
        registry_id: "minecraft:smooth_quartz_slab",
    },
    BlockRegistryEntry {
        numeric_id: 803,
        field_name: "GRANITE_SLAB",
        registry_id: "minecraft:granite_slab",
    },
    BlockRegistryEntry {
        numeric_id: 804,
        field_name: "ANDESITE_SLAB",
        registry_id: "minecraft:andesite_slab",
    },
    BlockRegistryEntry {
        numeric_id: 805,
        field_name: "RED_NETHER_BRICK_SLAB",
        registry_id: "minecraft:red_nether_brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 806,
        field_name: "POLISHED_ANDESITE_SLAB",
        registry_id: "minecraft:polished_andesite_slab",
    },
    BlockRegistryEntry {
        numeric_id: 807,
        field_name: "DIORITE_SLAB",
        registry_id: "minecraft:diorite_slab",
    },
    BlockRegistryEntry {
        numeric_id: 808,
        field_name: "BRICK_WALL",
        registry_id: "minecraft:brick_wall",
    },
    BlockRegistryEntry {
        numeric_id: 809,
        field_name: "PRISMARINE_WALL",
        registry_id: "minecraft:prismarine_wall",
    },
    BlockRegistryEntry {
        numeric_id: 810,
        field_name: "RED_SANDSTONE_WALL",
        registry_id: "minecraft:red_sandstone_wall",
    },
    BlockRegistryEntry {
        numeric_id: 811,
        field_name: "MOSSY_STONE_BRICK_WALL",
        registry_id: "minecraft:mossy_stone_brick_wall",
    },
    BlockRegistryEntry {
        numeric_id: 812,
        field_name: "GRANITE_WALL",
        registry_id: "minecraft:granite_wall",
    },
    BlockRegistryEntry {
        numeric_id: 813,
        field_name: "STONE_BRICK_WALL",
        registry_id: "minecraft:stone_brick_wall",
    },
    BlockRegistryEntry {
        numeric_id: 814,
        field_name: "MUD_BRICK_WALL",
        registry_id: "minecraft:mud_brick_wall",
    },
    BlockRegistryEntry {
        numeric_id: 815,
        field_name: "NETHER_BRICK_WALL",
        registry_id: "minecraft:nether_brick_wall",
    },
    BlockRegistryEntry {
        numeric_id: 816,
        field_name: "ANDESITE_WALL",
        registry_id: "minecraft:andesite_wall",
    },
    BlockRegistryEntry {
        numeric_id: 817,
        field_name: "RED_NETHER_BRICK_WALL",
        registry_id: "minecraft:red_nether_brick_wall",
    },
    BlockRegistryEntry {
        numeric_id: 818,
        field_name: "SANDSTONE_WALL",
        registry_id: "minecraft:sandstone_wall",
    },
    BlockRegistryEntry {
        numeric_id: 819,
        field_name: "END_STONE_BRICK_WALL",
        registry_id: "minecraft:end_stone_brick_wall",
    },
    BlockRegistryEntry {
        numeric_id: 820,
        field_name: "DIORITE_WALL",
        registry_id: "minecraft:diorite_wall",
    },
    BlockRegistryEntry {
        numeric_id: 821,
        field_name: "SCAFFOLDING",
        registry_id: "minecraft:scaffolding",
    },
    BlockRegistryEntry {
        numeric_id: 822,
        field_name: "LOOM",
        registry_id: "minecraft:loom",
    },
    BlockRegistryEntry {
        numeric_id: 823,
        field_name: "BARREL",
        registry_id: "minecraft:barrel",
    },
    BlockRegistryEntry {
        numeric_id: 824,
        field_name: "SMOKER",
        registry_id: "minecraft:smoker",
    },
    BlockRegistryEntry {
        numeric_id: 825,
        field_name: "BLAST_FURNACE",
        registry_id: "minecraft:blast_furnace",
    },
    BlockRegistryEntry {
        numeric_id: 826,
        field_name: "CARTOGRAPHY_TABLE",
        registry_id: "minecraft:cartography_table",
    },
    BlockRegistryEntry {
        numeric_id: 827,
        field_name: "FLETCHING_TABLE",
        registry_id: "minecraft:fletching_table",
    },
    BlockRegistryEntry {
        numeric_id: 828,
        field_name: "GRINDSTONE",
        registry_id: "minecraft:grindstone",
    },
    BlockRegistryEntry {
        numeric_id: 829,
        field_name: "LECTERN",
        registry_id: "minecraft:lectern",
    },
    BlockRegistryEntry {
        numeric_id: 830,
        field_name: "SMITHING_TABLE",
        registry_id: "minecraft:smithing_table",
    },
    BlockRegistryEntry {
        numeric_id: 831,
        field_name: "STONECUTTER",
        registry_id: "minecraft:stonecutter",
    },
    BlockRegistryEntry {
        numeric_id: 832,
        field_name: "BELL",
        registry_id: "minecraft:bell",
    },
    BlockRegistryEntry {
        numeric_id: 833,
        field_name: "LANTERN",
        registry_id: "minecraft:lantern",
    },
    BlockRegistryEntry {
        numeric_id: 834,
        field_name: "SOUL_LANTERN",
        registry_id: "minecraft:soul_lantern",
    },
    BlockRegistryEntry {
        numeric_id: 835,
        field_name: "CAMPFIRE",
        registry_id: "minecraft:campfire",
    },
    BlockRegistryEntry {
        numeric_id: 836,
        field_name: "SOUL_CAMPFIRE",
        registry_id: "minecraft:soul_campfire",
    },
    BlockRegistryEntry {
        numeric_id: 837,
        field_name: "SWEET_BERRY_BUSH",
        registry_id: "minecraft:sweet_berry_bush",
    },
    BlockRegistryEntry {
        numeric_id: 838,
        field_name: "WARPED_STEM",
        registry_id: "minecraft:warped_stem",
    },
    BlockRegistryEntry {
        numeric_id: 839,
        field_name: "STRIPPED_WARPED_STEM",
        registry_id: "minecraft:stripped_warped_stem",
    },
    BlockRegistryEntry {
        numeric_id: 840,
        field_name: "WARPED_HYPHAE",
        registry_id: "minecraft:warped_hyphae",
    },
    BlockRegistryEntry {
        numeric_id: 841,
        field_name: "STRIPPED_WARPED_HYPHAE",
        registry_id: "minecraft:stripped_warped_hyphae",
    },
    BlockRegistryEntry {
        numeric_id: 842,
        field_name: "WARPED_NYLIUM",
        registry_id: "minecraft:warped_nylium",
    },
    BlockRegistryEntry {
        numeric_id: 843,
        field_name: "WARPED_FUNGUS",
        registry_id: "minecraft:warped_fungus",
    },
    BlockRegistryEntry {
        numeric_id: 844,
        field_name: "WARPED_WART_BLOCK",
        registry_id: "minecraft:warped_wart_block",
    },
    BlockRegistryEntry {
        numeric_id: 845,
        field_name: "WARPED_ROOTS",
        registry_id: "minecraft:warped_roots",
    },
    BlockRegistryEntry {
        numeric_id: 846,
        field_name: "NETHER_SPROUTS",
        registry_id: "minecraft:nether_sprouts",
    },
    BlockRegistryEntry {
        numeric_id: 847,
        field_name: "CRIMSON_STEM",
        registry_id: "minecraft:crimson_stem",
    },
    BlockRegistryEntry {
        numeric_id: 848,
        field_name: "STRIPPED_CRIMSON_STEM",
        registry_id: "minecraft:stripped_crimson_stem",
    },
    BlockRegistryEntry {
        numeric_id: 849,
        field_name: "CRIMSON_HYPHAE",
        registry_id: "minecraft:crimson_hyphae",
    },
    BlockRegistryEntry {
        numeric_id: 850,
        field_name: "STRIPPED_CRIMSON_HYPHAE",
        registry_id: "minecraft:stripped_crimson_hyphae",
    },
    BlockRegistryEntry {
        numeric_id: 851,
        field_name: "CRIMSON_NYLIUM",
        registry_id: "minecraft:crimson_nylium",
    },
    BlockRegistryEntry {
        numeric_id: 852,
        field_name: "CRIMSON_FUNGUS",
        registry_id: "minecraft:crimson_fungus",
    },
    BlockRegistryEntry {
        numeric_id: 853,
        field_name: "SHROOMLIGHT",
        registry_id: "minecraft:shroomlight",
    },
    BlockRegistryEntry {
        numeric_id: 854,
        field_name: "WEEPING_VINES",
        registry_id: "minecraft:weeping_vines",
    },
    BlockRegistryEntry {
        numeric_id: 855,
        field_name: "WEEPING_VINES_PLANT",
        registry_id: "minecraft:weeping_vines_plant",
    },
    BlockRegistryEntry {
        numeric_id: 856,
        field_name: "TWISTING_VINES",
        registry_id: "minecraft:twisting_vines",
    },
    BlockRegistryEntry {
        numeric_id: 857,
        field_name: "TWISTING_VINES_PLANT",
        registry_id: "minecraft:twisting_vines_plant",
    },
    BlockRegistryEntry {
        numeric_id: 858,
        field_name: "CRIMSON_ROOTS",
        registry_id: "minecraft:crimson_roots",
    },
    BlockRegistryEntry {
        numeric_id: 859,
        field_name: "CRIMSON_PLANKS",
        registry_id: "minecraft:crimson_planks",
    },
    BlockRegistryEntry {
        numeric_id: 860,
        field_name: "WARPED_PLANKS",
        registry_id: "minecraft:warped_planks",
    },
    BlockRegistryEntry {
        numeric_id: 861,
        field_name: "CRIMSON_SLAB",
        registry_id: "minecraft:crimson_slab",
    },
    BlockRegistryEntry {
        numeric_id: 862,
        field_name: "WARPED_SLAB",
        registry_id: "minecraft:warped_slab",
    },
    BlockRegistryEntry {
        numeric_id: 863,
        field_name: "CRIMSON_PRESSURE_PLATE",
        registry_id: "minecraft:crimson_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 864,
        field_name: "WARPED_PRESSURE_PLATE",
        registry_id: "minecraft:warped_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 865,
        field_name: "CRIMSON_FENCE",
        registry_id: "minecraft:crimson_fence",
    },
    BlockRegistryEntry {
        numeric_id: 866,
        field_name: "WARPED_FENCE",
        registry_id: "minecraft:warped_fence",
    },
    BlockRegistryEntry {
        numeric_id: 867,
        field_name: "CRIMSON_TRAPDOOR",
        registry_id: "minecraft:crimson_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 868,
        field_name: "WARPED_TRAPDOOR",
        registry_id: "minecraft:warped_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 869,
        field_name: "CRIMSON_FENCE_GATE",
        registry_id: "minecraft:crimson_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 870,
        field_name: "WARPED_FENCE_GATE",
        registry_id: "minecraft:warped_fence_gate",
    },
    BlockRegistryEntry {
        numeric_id: 871,
        field_name: "CRIMSON_STAIRS",
        registry_id: "minecraft:crimson_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 872,
        field_name: "WARPED_STAIRS",
        registry_id: "minecraft:warped_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 873,
        field_name: "CRIMSON_BUTTON",
        registry_id: "minecraft:crimson_button",
    },
    BlockRegistryEntry {
        numeric_id: 874,
        field_name: "WARPED_BUTTON",
        registry_id: "minecraft:warped_button",
    },
    BlockRegistryEntry {
        numeric_id: 875,
        field_name: "CRIMSON_DOOR",
        registry_id: "minecraft:crimson_door",
    },
    BlockRegistryEntry {
        numeric_id: 876,
        field_name: "WARPED_DOOR",
        registry_id: "minecraft:warped_door",
    },
    BlockRegistryEntry {
        numeric_id: 877,
        field_name: "CRIMSON_SIGN",
        registry_id: "minecraft:crimson_sign",
    },
    BlockRegistryEntry {
        numeric_id: 878,
        field_name: "WARPED_SIGN",
        registry_id: "minecraft:warped_sign",
    },
    BlockRegistryEntry {
        numeric_id: 879,
        field_name: "CRIMSON_WALL_SIGN",
        registry_id: "minecraft:crimson_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 880,
        field_name: "WARPED_WALL_SIGN",
        registry_id: "minecraft:warped_wall_sign",
    },
    BlockRegistryEntry {
        numeric_id: 881,
        field_name: "STRUCTURE_BLOCK",
        registry_id: "minecraft:structure_block",
    },
    BlockRegistryEntry {
        numeric_id: 882,
        field_name: "JIGSAW",
        registry_id: "minecraft:jigsaw",
    },
    BlockRegistryEntry {
        numeric_id: 883,
        field_name: "TEST_BLOCK",
        registry_id: "minecraft:test_block",
    },
    BlockRegistryEntry {
        numeric_id: 884,
        field_name: "TEST_INSTANCE_BLOCK",
        registry_id: "minecraft:test_instance_block",
    },
    BlockRegistryEntry {
        numeric_id: 885,
        field_name: "COMPOSTER",
        registry_id: "minecraft:composter",
    },
    BlockRegistryEntry {
        numeric_id: 886,
        field_name: "TARGET",
        registry_id: "minecraft:target",
    },
    BlockRegistryEntry {
        numeric_id: 887,
        field_name: "BEE_NEST",
        registry_id: "minecraft:bee_nest",
    },
    BlockRegistryEntry {
        numeric_id: 888,
        field_name: "BEEHIVE",
        registry_id: "minecraft:beehive",
    },
    BlockRegistryEntry {
        numeric_id: 889,
        field_name: "HONEY_BLOCK",
        registry_id: "minecraft:honey_block",
    },
    BlockRegistryEntry {
        numeric_id: 890,
        field_name: "HONEYCOMB_BLOCK",
        registry_id: "minecraft:honeycomb_block",
    },
    BlockRegistryEntry {
        numeric_id: 891,
        field_name: "NETHERITE_BLOCK",
        registry_id: "minecraft:netherite_block",
    },
    BlockRegistryEntry {
        numeric_id: 892,
        field_name: "ANCIENT_DEBRIS",
        registry_id: "minecraft:ancient_debris",
    },
    BlockRegistryEntry {
        numeric_id: 893,
        field_name: "CRYING_OBSIDIAN",
        registry_id: "minecraft:crying_obsidian",
    },
    BlockRegistryEntry {
        numeric_id: 894,
        field_name: "RESPAWN_ANCHOR",
        registry_id: "minecraft:respawn_anchor",
    },
    BlockRegistryEntry {
        numeric_id: 895,
        field_name: "POTTED_CRIMSON_FUNGUS",
        registry_id: "minecraft:potted_crimson_fungus",
    },
    BlockRegistryEntry {
        numeric_id: 896,
        field_name: "POTTED_WARPED_FUNGUS",
        registry_id: "minecraft:potted_warped_fungus",
    },
    BlockRegistryEntry {
        numeric_id: 897,
        field_name: "POTTED_CRIMSON_ROOTS",
        registry_id: "minecraft:potted_crimson_roots",
    },
    BlockRegistryEntry {
        numeric_id: 898,
        field_name: "POTTED_WARPED_ROOTS",
        registry_id: "minecraft:potted_warped_roots",
    },
    BlockRegistryEntry {
        numeric_id: 899,
        field_name: "LODESTONE",
        registry_id: "minecraft:lodestone",
    },
    BlockRegistryEntry {
        numeric_id: 900,
        field_name: "BLACKSTONE",
        registry_id: "minecraft:blackstone",
    },
    BlockRegistryEntry {
        numeric_id: 901,
        field_name: "BLACKSTONE_STAIRS",
        registry_id: "minecraft:blackstone_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 902,
        field_name: "BLACKSTONE_WALL",
        registry_id: "minecraft:blackstone_wall",
    },
    BlockRegistryEntry {
        numeric_id: 903,
        field_name: "BLACKSTONE_SLAB",
        registry_id: "minecraft:blackstone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 904,
        field_name: "POLISHED_BLACKSTONE",
        registry_id: "minecraft:polished_blackstone",
    },
    BlockRegistryEntry {
        numeric_id: 905,
        field_name: "POLISHED_BLACKSTONE_BRICKS",
        registry_id: "minecraft:polished_blackstone_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 906,
        field_name: "CRACKED_POLISHED_BLACKSTONE_BRICKS",
        registry_id: "minecraft:cracked_polished_blackstone_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 907,
        field_name: "CHISELED_POLISHED_BLACKSTONE",
        registry_id: "minecraft:chiseled_polished_blackstone",
    },
    BlockRegistryEntry {
        numeric_id: 908,
        field_name: "POLISHED_BLACKSTONE_BRICK_SLAB",
        registry_id: "minecraft:polished_blackstone_brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 909,
        field_name: "POLISHED_BLACKSTONE_BRICK_STAIRS",
        registry_id: "minecraft:polished_blackstone_brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 910,
        field_name: "POLISHED_BLACKSTONE_BRICK_WALL",
        registry_id: "minecraft:polished_blackstone_brick_wall",
    },
    BlockRegistryEntry {
        numeric_id: 911,
        field_name: "GILDED_BLACKSTONE",
        registry_id: "minecraft:gilded_blackstone",
    },
    BlockRegistryEntry {
        numeric_id: 912,
        field_name: "POLISHED_BLACKSTONE_STAIRS",
        registry_id: "minecraft:polished_blackstone_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 913,
        field_name: "POLISHED_BLACKSTONE_SLAB",
        registry_id: "minecraft:polished_blackstone_slab",
    },
    BlockRegistryEntry {
        numeric_id: 914,
        field_name: "POLISHED_BLACKSTONE_PRESSURE_PLATE",
        registry_id: "minecraft:polished_blackstone_pressure_plate",
    },
    BlockRegistryEntry {
        numeric_id: 915,
        field_name: "POLISHED_BLACKSTONE_BUTTON",
        registry_id: "minecraft:polished_blackstone_button",
    },
    BlockRegistryEntry {
        numeric_id: 916,
        field_name: "POLISHED_BLACKSTONE_WALL",
        registry_id: "minecraft:polished_blackstone_wall",
    },
    BlockRegistryEntry {
        numeric_id: 917,
        field_name: "CHISELED_NETHER_BRICKS",
        registry_id: "minecraft:chiseled_nether_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 918,
        field_name: "CRACKED_NETHER_BRICKS",
        registry_id: "minecraft:cracked_nether_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 919,
        field_name: "QUARTZ_BRICKS",
        registry_id: "minecraft:quartz_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 920,
        field_name: "CANDLE",
        registry_id: "minecraft:candle",
    },
    BlockRegistryEntry {
        numeric_id: 921,
        field_name: "WHITE_CANDLE",
        registry_id: "minecraft:white_candle",
    },
    BlockRegistryEntry {
        numeric_id: 922,
        field_name: "ORANGE_CANDLE",
        registry_id: "minecraft:orange_candle",
    },
    BlockRegistryEntry {
        numeric_id: 923,
        field_name: "MAGENTA_CANDLE",
        registry_id: "minecraft:magenta_candle",
    },
    BlockRegistryEntry {
        numeric_id: 924,
        field_name: "LIGHT_BLUE_CANDLE",
        registry_id: "minecraft:light_blue_candle",
    },
    BlockRegistryEntry {
        numeric_id: 925,
        field_name: "YELLOW_CANDLE",
        registry_id: "minecraft:yellow_candle",
    },
    BlockRegistryEntry {
        numeric_id: 926,
        field_name: "LIME_CANDLE",
        registry_id: "minecraft:lime_candle",
    },
    BlockRegistryEntry {
        numeric_id: 927,
        field_name: "PINK_CANDLE",
        registry_id: "minecraft:pink_candle",
    },
    BlockRegistryEntry {
        numeric_id: 928,
        field_name: "GRAY_CANDLE",
        registry_id: "minecraft:gray_candle",
    },
    BlockRegistryEntry {
        numeric_id: 929,
        field_name: "LIGHT_GRAY_CANDLE",
        registry_id: "minecraft:light_gray_candle",
    },
    BlockRegistryEntry {
        numeric_id: 930,
        field_name: "CYAN_CANDLE",
        registry_id: "minecraft:cyan_candle",
    },
    BlockRegistryEntry {
        numeric_id: 931,
        field_name: "PURPLE_CANDLE",
        registry_id: "minecraft:purple_candle",
    },
    BlockRegistryEntry {
        numeric_id: 932,
        field_name: "BLUE_CANDLE",
        registry_id: "minecraft:blue_candle",
    },
    BlockRegistryEntry {
        numeric_id: 933,
        field_name: "BROWN_CANDLE",
        registry_id: "minecraft:brown_candle",
    },
    BlockRegistryEntry {
        numeric_id: 934,
        field_name: "GREEN_CANDLE",
        registry_id: "minecraft:green_candle",
    },
    BlockRegistryEntry {
        numeric_id: 935,
        field_name: "RED_CANDLE",
        registry_id: "minecraft:red_candle",
    },
    BlockRegistryEntry {
        numeric_id: 936,
        field_name: "BLACK_CANDLE",
        registry_id: "minecraft:black_candle",
    },
    BlockRegistryEntry {
        numeric_id: 937,
        field_name: "CANDLE_CAKE",
        registry_id: "minecraft:candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 938,
        field_name: "WHITE_CANDLE_CAKE",
        registry_id: "minecraft:white_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 939,
        field_name: "ORANGE_CANDLE_CAKE",
        registry_id: "minecraft:orange_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 940,
        field_name: "MAGENTA_CANDLE_CAKE",
        registry_id: "minecraft:magenta_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 941,
        field_name: "LIGHT_BLUE_CANDLE_CAKE",
        registry_id: "minecraft:light_blue_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 942,
        field_name: "YELLOW_CANDLE_CAKE",
        registry_id: "minecraft:yellow_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 943,
        field_name: "LIME_CANDLE_CAKE",
        registry_id: "minecraft:lime_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 944,
        field_name: "PINK_CANDLE_CAKE",
        registry_id: "minecraft:pink_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 945,
        field_name: "GRAY_CANDLE_CAKE",
        registry_id: "minecraft:gray_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 946,
        field_name: "LIGHT_GRAY_CANDLE_CAKE",
        registry_id: "minecraft:light_gray_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 947,
        field_name: "CYAN_CANDLE_CAKE",
        registry_id: "minecraft:cyan_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 948,
        field_name: "PURPLE_CANDLE_CAKE",
        registry_id: "minecraft:purple_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 949,
        field_name: "BLUE_CANDLE_CAKE",
        registry_id: "minecraft:blue_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 950,
        field_name: "BROWN_CANDLE_CAKE",
        registry_id: "minecraft:brown_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 951,
        field_name: "GREEN_CANDLE_CAKE",
        registry_id: "minecraft:green_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 952,
        field_name: "RED_CANDLE_CAKE",
        registry_id: "minecraft:red_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 953,
        field_name: "BLACK_CANDLE_CAKE",
        registry_id: "minecraft:black_candle_cake",
    },
    BlockRegistryEntry {
        numeric_id: 954,
        field_name: "AMETHYST_BLOCK",
        registry_id: "minecraft:amethyst_block",
    },
    BlockRegistryEntry {
        numeric_id: 955,
        field_name: "BUDDING_AMETHYST",
        registry_id: "minecraft:budding_amethyst",
    },
    BlockRegistryEntry {
        numeric_id: 956,
        field_name: "AMETHYST_CLUSTER",
        registry_id: "minecraft:amethyst_cluster",
    },
    BlockRegistryEntry {
        numeric_id: 957,
        field_name: "LARGE_AMETHYST_BUD",
        registry_id: "minecraft:large_amethyst_bud",
    },
    BlockRegistryEntry {
        numeric_id: 958,
        field_name: "MEDIUM_AMETHYST_BUD",
        registry_id: "minecraft:medium_amethyst_bud",
    },
    BlockRegistryEntry {
        numeric_id: 959,
        field_name: "SMALL_AMETHYST_BUD",
        registry_id: "minecraft:small_amethyst_bud",
    },
    BlockRegistryEntry {
        numeric_id: 960,
        field_name: "TUFF",
        registry_id: "minecraft:tuff",
    },
    BlockRegistryEntry {
        numeric_id: 961,
        field_name: "TUFF_SLAB",
        registry_id: "minecraft:tuff_slab",
    },
    BlockRegistryEntry {
        numeric_id: 962,
        field_name: "TUFF_STAIRS",
        registry_id: "minecraft:tuff_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 963,
        field_name: "TUFF_WALL",
        registry_id: "minecraft:tuff_wall",
    },
    BlockRegistryEntry {
        numeric_id: 964,
        field_name: "POLISHED_TUFF",
        registry_id: "minecraft:polished_tuff",
    },
    BlockRegistryEntry {
        numeric_id: 965,
        field_name: "POLISHED_TUFF_SLAB",
        registry_id: "minecraft:polished_tuff_slab",
    },
    BlockRegistryEntry {
        numeric_id: 966,
        field_name: "POLISHED_TUFF_STAIRS",
        registry_id: "minecraft:polished_tuff_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 967,
        field_name: "POLISHED_TUFF_WALL",
        registry_id: "minecraft:polished_tuff_wall",
    },
    BlockRegistryEntry {
        numeric_id: 968,
        field_name: "CHISELED_TUFF",
        registry_id: "minecraft:chiseled_tuff",
    },
    BlockRegistryEntry {
        numeric_id: 969,
        field_name: "TUFF_BRICKS",
        registry_id: "minecraft:tuff_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 970,
        field_name: "TUFF_BRICK_SLAB",
        registry_id: "minecraft:tuff_brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 971,
        field_name: "TUFF_BRICK_STAIRS",
        registry_id: "minecraft:tuff_brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 972,
        field_name: "TUFF_BRICK_WALL",
        registry_id: "minecraft:tuff_brick_wall",
    },
    BlockRegistryEntry {
        numeric_id: 973,
        field_name: "CHISELED_TUFF_BRICKS",
        registry_id: "minecraft:chiseled_tuff_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 974,
        field_name: "CALCITE",
        registry_id: "minecraft:calcite",
    },
    BlockRegistryEntry {
        numeric_id: 975,
        field_name: "TINTED_GLASS",
        registry_id: "minecraft:tinted_glass",
    },
    BlockRegistryEntry {
        numeric_id: 976,
        field_name: "POWDER_SNOW",
        registry_id: "minecraft:powder_snow",
    },
    BlockRegistryEntry {
        numeric_id: 977,
        field_name: "SCULK_SENSOR",
        registry_id: "minecraft:sculk_sensor",
    },
    BlockRegistryEntry {
        numeric_id: 978,
        field_name: "CALIBRATED_SCULK_SENSOR",
        registry_id: "minecraft:calibrated_sculk_sensor",
    },
    BlockRegistryEntry {
        numeric_id: 979,
        field_name: "SCULK",
        registry_id: "minecraft:sculk",
    },
    BlockRegistryEntry {
        numeric_id: 980,
        field_name: "SCULK_VEIN",
        registry_id: "minecraft:sculk_vein",
    },
    BlockRegistryEntry {
        numeric_id: 981,
        field_name: "SCULK_CATALYST",
        registry_id: "minecraft:sculk_catalyst",
    },
    BlockRegistryEntry {
        numeric_id: 982,
        field_name: "SCULK_SHRIEKER",
        registry_id: "minecraft:sculk_shrieker",
    },
    BlockRegistryEntry {
        numeric_id: 983,
        field_name: "COPPER_BLOCK",
        registry_id: "minecraft:copper_block",
    },
    BlockRegistryEntry {
        numeric_id: 984,
        field_name: "EXPOSED_COPPER",
        registry_id: "minecraft:exposed_copper",
    },
    BlockRegistryEntry {
        numeric_id: 985,
        field_name: "WEATHERED_COPPER",
        registry_id: "minecraft:weathered_copper",
    },
    BlockRegistryEntry {
        numeric_id: 986,
        field_name: "OXIDIZED_COPPER",
        registry_id: "minecraft:oxidized_copper",
    },
    BlockRegistryEntry {
        numeric_id: 987,
        field_name: "COPPER_ORE",
        registry_id: "minecraft:copper_ore",
    },
    BlockRegistryEntry {
        numeric_id: 988,
        field_name: "DEEPSLATE_COPPER_ORE",
        registry_id: "minecraft:deepslate_copper_ore",
    },
    BlockRegistryEntry {
        numeric_id: 989,
        field_name: "OXIDIZED_CUT_COPPER",
        registry_id: "minecraft:oxidized_cut_copper",
    },
    BlockRegistryEntry {
        numeric_id: 990,
        field_name: "WEATHERED_CUT_COPPER",
        registry_id: "minecraft:weathered_cut_copper",
    },
    BlockRegistryEntry {
        numeric_id: 991,
        field_name: "EXPOSED_CUT_COPPER",
        registry_id: "minecraft:exposed_cut_copper",
    },
    BlockRegistryEntry {
        numeric_id: 992,
        field_name: "CUT_COPPER",
        registry_id: "minecraft:cut_copper",
    },
    BlockRegistryEntry {
        numeric_id: 993,
        field_name: "OXIDIZED_CHISELED_COPPER",
        registry_id: "minecraft:oxidized_chiseled_copper",
    },
    BlockRegistryEntry {
        numeric_id: 994,
        field_name: "WEATHERED_CHISELED_COPPER",
        registry_id: "minecraft:weathered_chiseled_copper",
    },
    BlockRegistryEntry {
        numeric_id: 995,
        field_name: "EXPOSED_CHISELED_COPPER",
        registry_id: "minecraft:exposed_chiseled_copper",
    },
    BlockRegistryEntry {
        numeric_id: 996,
        field_name: "CHISELED_COPPER",
        registry_id: "minecraft:chiseled_copper",
    },
    BlockRegistryEntry {
        numeric_id: 997,
        field_name: "WAXED_OXIDIZED_CHISELED_COPPER",
        registry_id: "minecraft:waxed_oxidized_chiseled_copper",
    },
    BlockRegistryEntry {
        numeric_id: 998,
        field_name: "WAXED_WEATHERED_CHISELED_COPPER",
        registry_id: "minecraft:waxed_weathered_chiseled_copper",
    },
    BlockRegistryEntry {
        numeric_id: 999,
        field_name: "WAXED_EXPOSED_CHISELED_COPPER",
        registry_id: "minecraft:waxed_exposed_chiseled_copper",
    },
    BlockRegistryEntry {
        numeric_id: 1000,
        field_name: "WAXED_CHISELED_COPPER",
        registry_id: "minecraft:waxed_chiseled_copper",
    },
    BlockRegistryEntry {
        numeric_id: 1001,
        field_name: "OXIDIZED_CUT_COPPER_STAIRS",
        registry_id: "minecraft:oxidized_cut_copper_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1002,
        field_name: "WEATHERED_CUT_COPPER_STAIRS",
        registry_id: "minecraft:weathered_cut_copper_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1003,
        field_name: "EXPOSED_CUT_COPPER_STAIRS",
        registry_id: "minecraft:exposed_cut_copper_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1004,
        field_name: "CUT_COPPER_STAIRS",
        registry_id: "minecraft:cut_copper_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1005,
        field_name: "OXIDIZED_CUT_COPPER_SLAB",
        registry_id: "minecraft:oxidized_cut_copper_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1006,
        field_name: "WEATHERED_CUT_COPPER_SLAB",
        registry_id: "minecraft:weathered_cut_copper_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1007,
        field_name: "EXPOSED_CUT_COPPER_SLAB",
        registry_id: "minecraft:exposed_cut_copper_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1008,
        field_name: "CUT_COPPER_SLAB",
        registry_id: "minecraft:cut_copper_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1009,
        field_name: "WAXED_COPPER_BLOCK",
        registry_id: "minecraft:waxed_copper_block",
    },
    BlockRegistryEntry {
        numeric_id: 1010,
        field_name: "WAXED_WEATHERED_COPPER",
        registry_id: "minecraft:waxed_weathered_copper",
    },
    BlockRegistryEntry {
        numeric_id: 1011,
        field_name: "WAXED_EXPOSED_COPPER",
        registry_id: "minecraft:waxed_exposed_copper",
    },
    BlockRegistryEntry {
        numeric_id: 1012,
        field_name: "WAXED_OXIDIZED_COPPER",
        registry_id: "minecraft:waxed_oxidized_copper",
    },
    BlockRegistryEntry {
        numeric_id: 1013,
        field_name: "WAXED_OXIDIZED_CUT_COPPER",
        registry_id: "minecraft:waxed_oxidized_cut_copper",
    },
    BlockRegistryEntry {
        numeric_id: 1014,
        field_name: "WAXED_WEATHERED_CUT_COPPER",
        registry_id: "minecraft:waxed_weathered_cut_copper",
    },
    BlockRegistryEntry {
        numeric_id: 1015,
        field_name: "WAXED_EXPOSED_CUT_COPPER",
        registry_id: "minecraft:waxed_exposed_cut_copper",
    },
    BlockRegistryEntry {
        numeric_id: 1016,
        field_name: "WAXED_CUT_COPPER",
        registry_id: "minecraft:waxed_cut_copper",
    },
    BlockRegistryEntry {
        numeric_id: 1017,
        field_name: "WAXED_OXIDIZED_CUT_COPPER_STAIRS",
        registry_id: "minecraft:waxed_oxidized_cut_copper_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1018,
        field_name: "WAXED_WEATHERED_CUT_COPPER_STAIRS",
        registry_id: "minecraft:waxed_weathered_cut_copper_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1019,
        field_name: "WAXED_EXPOSED_CUT_COPPER_STAIRS",
        registry_id: "minecraft:waxed_exposed_cut_copper_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1020,
        field_name: "WAXED_CUT_COPPER_STAIRS",
        registry_id: "minecraft:waxed_cut_copper_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1021,
        field_name: "WAXED_OXIDIZED_CUT_COPPER_SLAB",
        registry_id: "minecraft:waxed_oxidized_cut_copper_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1022,
        field_name: "WAXED_WEATHERED_CUT_COPPER_SLAB",
        registry_id: "minecraft:waxed_weathered_cut_copper_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1023,
        field_name: "WAXED_EXPOSED_CUT_COPPER_SLAB",
        registry_id: "minecraft:waxed_exposed_cut_copper_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1024,
        field_name: "WAXED_CUT_COPPER_SLAB",
        registry_id: "minecraft:waxed_cut_copper_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1025,
        field_name: "COPPER_DOOR",
        registry_id: "minecraft:copper_door",
    },
    BlockRegistryEntry {
        numeric_id: 1026,
        field_name: "EXPOSED_COPPER_DOOR",
        registry_id: "minecraft:exposed_copper_door",
    },
    BlockRegistryEntry {
        numeric_id: 1027,
        field_name: "OXIDIZED_COPPER_DOOR",
        registry_id: "minecraft:oxidized_copper_door",
    },
    BlockRegistryEntry {
        numeric_id: 1028,
        field_name: "WEATHERED_COPPER_DOOR",
        registry_id: "minecraft:weathered_copper_door",
    },
    BlockRegistryEntry {
        numeric_id: 1029,
        field_name: "WAXED_COPPER_DOOR",
        registry_id: "minecraft:waxed_copper_door",
    },
    BlockRegistryEntry {
        numeric_id: 1030,
        field_name: "WAXED_EXPOSED_COPPER_DOOR",
        registry_id: "minecraft:waxed_exposed_copper_door",
    },
    BlockRegistryEntry {
        numeric_id: 1031,
        field_name: "WAXED_OXIDIZED_COPPER_DOOR",
        registry_id: "minecraft:waxed_oxidized_copper_door",
    },
    BlockRegistryEntry {
        numeric_id: 1032,
        field_name: "WAXED_WEATHERED_COPPER_DOOR",
        registry_id: "minecraft:waxed_weathered_copper_door",
    },
    BlockRegistryEntry {
        numeric_id: 1033,
        field_name: "COPPER_TRAPDOOR",
        registry_id: "minecraft:copper_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 1034,
        field_name: "EXPOSED_COPPER_TRAPDOOR",
        registry_id: "minecraft:exposed_copper_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 1035,
        field_name: "OXIDIZED_COPPER_TRAPDOOR",
        registry_id: "minecraft:oxidized_copper_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 1036,
        field_name: "WEATHERED_COPPER_TRAPDOOR",
        registry_id: "minecraft:weathered_copper_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 1037,
        field_name: "WAXED_COPPER_TRAPDOOR",
        registry_id: "minecraft:waxed_copper_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 1038,
        field_name: "WAXED_EXPOSED_COPPER_TRAPDOOR",
        registry_id: "minecraft:waxed_exposed_copper_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 1039,
        field_name: "WAXED_OXIDIZED_COPPER_TRAPDOOR",
        registry_id: "minecraft:waxed_oxidized_copper_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 1040,
        field_name: "WAXED_WEATHERED_COPPER_TRAPDOOR",
        registry_id: "minecraft:waxed_weathered_copper_trapdoor",
    },
    BlockRegistryEntry {
        numeric_id: 1041,
        field_name: "COPPER_GRATE",
        registry_id: "minecraft:copper_grate",
    },
    BlockRegistryEntry {
        numeric_id: 1042,
        field_name: "EXPOSED_COPPER_GRATE",
        registry_id: "minecraft:exposed_copper_grate",
    },
    BlockRegistryEntry {
        numeric_id: 1043,
        field_name: "WEATHERED_COPPER_GRATE",
        registry_id: "minecraft:weathered_copper_grate",
    },
    BlockRegistryEntry {
        numeric_id: 1044,
        field_name: "OXIDIZED_COPPER_GRATE",
        registry_id: "minecraft:oxidized_copper_grate",
    },
    BlockRegistryEntry {
        numeric_id: 1045,
        field_name: "WAXED_COPPER_GRATE",
        registry_id: "minecraft:waxed_copper_grate",
    },
    BlockRegistryEntry {
        numeric_id: 1046,
        field_name: "WAXED_EXPOSED_COPPER_GRATE",
        registry_id: "minecraft:waxed_exposed_copper_grate",
    },
    BlockRegistryEntry {
        numeric_id: 1047,
        field_name: "WAXED_WEATHERED_COPPER_GRATE",
        registry_id: "minecraft:waxed_weathered_copper_grate",
    },
    BlockRegistryEntry {
        numeric_id: 1048,
        field_name: "WAXED_OXIDIZED_COPPER_GRATE",
        registry_id: "minecraft:waxed_oxidized_copper_grate",
    },
    BlockRegistryEntry {
        numeric_id: 1049,
        field_name: "COPPER_BULB",
        registry_id: "minecraft:copper_bulb",
    },
    BlockRegistryEntry {
        numeric_id: 1050,
        field_name: "EXPOSED_COPPER_BULB",
        registry_id: "minecraft:exposed_copper_bulb",
    },
    BlockRegistryEntry {
        numeric_id: 1051,
        field_name: "WEATHERED_COPPER_BULB",
        registry_id: "minecraft:weathered_copper_bulb",
    },
    BlockRegistryEntry {
        numeric_id: 1052,
        field_name: "OXIDIZED_COPPER_BULB",
        registry_id: "minecraft:oxidized_copper_bulb",
    },
    BlockRegistryEntry {
        numeric_id: 1053,
        field_name: "WAXED_COPPER_BULB",
        registry_id: "minecraft:waxed_copper_bulb",
    },
    BlockRegistryEntry {
        numeric_id: 1054,
        field_name: "WAXED_EXPOSED_COPPER_BULB",
        registry_id: "minecraft:waxed_exposed_copper_bulb",
    },
    BlockRegistryEntry {
        numeric_id: 1055,
        field_name: "WAXED_WEATHERED_COPPER_BULB",
        registry_id: "minecraft:waxed_weathered_copper_bulb",
    },
    BlockRegistryEntry {
        numeric_id: 1056,
        field_name: "WAXED_OXIDIZED_COPPER_BULB",
        registry_id: "minecraft:waxed_oxidized_copper_bulb",
    },
    BlockRegistryEntry {
        numeric_id: 1057,
        field_name: "COPPER_CHEST",
        registry_id: "minecraft:copper_chest",
    },
    BlockRegistryEntry {
        numeric_id: 1058,
        field_name: "EXPOSED_COPPER_CHEST",
        registry_id: "minecraft:exposed_copper_chest",
    },
    BlockRegistryEntry {
        numeric_id: 1059,
        field_name: "WEATHERED_COPPER_CHEST",
        registry_id: "minecraft:weathered_copper_chest",
    },
    BlockRegistryEntry {
        numeric_id: 1060,
        field_name: "OXIDIZED_COPPER_CHEST",
        registry_id: "minecraft:oxidized_copper_chest",
    },
    BlockRegistryEntry {
        numeric_id: 1061,
        field_name: "WAXED_COPPER_CHEST",
        registry_id: "minecraft:waxed_copper_chest",
    },
    BlockRegistryEntry {
        numeric_id: 1062,
        field_name: "WAXED_EXPOSED_COPPER_CHEST",
        registry_id: "minecraft:waxed_exposed_copper_chest",
    },
    BlockRegistryEntry {
        numeric_id: 1063,
        field_name: "WAXED_WEATHERED_COPPER_CHEST",
        registry_id: "minecraft:waxed_weathered_copper_chest",
    },
    BlockRegistryEntry {
        numeric_id: 1064,
        field_name: "WAXED_OXIDIZED_COPPER_CHEST",
        registry_id: "minecraft:waxed_oxidized_copper_chest",
    },
    BlockRegistryEntry {
        numeric_id: 1065,
        field_name: "COPPER_GOLEM_STATUE",
        registry_id: "minecraft:copper_golem_statue",
    },
    BlockRegistryEntry {
        numeric_id: 1066,
        field_name: "EXPOSED_COPPER_GOLEM_STATUE",
        registry_id: "minecraft:exposed_copper_golem_statue",
    },
    BlockRegistryEntry {
        numeric_id: 1067,
        field_name: "WEATHERED_COPPER_GOLEM_STATUE",
        registry_id: "minecraft:weathered_copper_golem_statue",
    },
    BlockRegistryEntry {
        numeric_id: 1068,
        field_name: "OXIDIZED_COPPER_GOLEM_STATUE",
        registry_id: "minecraft:oxidized_copper_golem_statue",
    },
    BlockRegistryEntry {
        numeric_id: 1069,
        field_name: "WAXED_COPPER_GOLEM_STATUE",
        registry_id: "minecraft:waxed_copper_golem_statue",
    },
    BlockRegistryEntry {
        numeric_id: 1070,
        field_name: "WAXED_EXPOSED_COPPER_GOLEM_STATUE",
        registry_id: "minecraft:waxed_exposed_copper_golem_statue",
    },
    BlockRegistryEntry {
        numeric_id: 1071,
        field_name: "WAXED_WEATHERED_COPPER_GOLEM_STATUE",
        registry_id: "minecraft:waxed_weathered_copper_golem_statue",
    },
    BlockRegistryEntry {
        numeric_id: 1072,
        field_name: "WAXED_OXIDIZED_COPPER_GOLEM_STATUE",
        registry_id: "minecraft:waxed_oxidized_copper_golem_statue",
    },
    BlockRegistryEntry {
        numeric_id: 1073,
        field_name: "LIGHTNING_ROD",
        registry_id: "minecraft:lightning_rod",
    },
    BlockRegistryEntry {
        numeric_id: 1074,
        field_name: "EXPOSED_LIGHTNING_ROD",
        registry_id: "minecraft:exposed_lightning_rod",
    },
    BlockRegistryEntry {
        numeric_id: 1075,
        field_name: "WEATHERED_LIGHTNING_ROD",
        registry_id: "minecraft:weathered_lightning_rod",
    },
    BlockRegistryEntry {
        numeric_id: 1076,
        field_name: "OXIDIZED_LIGHTNING_ROD",
        registry_id: "minecraft:oxidized_lightning_rod",
    },
    BlockRegistryEntry {
        numeric_id: 1077,
        field_name: "WAXED_LIGHTNING_ROD",
        registry_id: "minecraft:waxed_lightning_rod",
    },
    BlockRegistryEntry {
        numeric_id: 1078,
        field_name: "WAXED_EXPOSED_LIGHTNING_ROD",
        registry_id: "minecraft:waxed_exposed_lightning_rod",
    },
    BlockRegistryEntry {
        numeric_id: 1079,
        field_name: "WAXED_WEATHERED_LIGHTNING_ROD",
        registry_id: "minecraft:waxed_weathered_lightning_rod",
    },
    BlockRegistryEntry {
        numeric_id: 1080,
        field_name: "WAXED_OXIDIZED_LIGHTNING_ROD",
        registry_id: "minecraft:waxed_oxidized_lightning_rod",
    },
    BlockRegistryEntry {
        numeric_id: 1081,
        field_name: "POINTED_DRIPSTONE",
        registry_id: "minecraft:pointed_dripstone",
    },
    BlockRegistryEntry {
        numeric_id: 1082,
        field_name: "DRIPSTONE_BLOCK",
        registry_id: "minecraft:dripstone_block",
    },
    BlockRegistryEntry {
        numeric_id: 1083,
        field_name: "CAVE_VINES",
        registry_id: "minecraft:cave_vines",
    },
    BlockRegistryEntry {
        numeric_id: 1084,
        field_name: "CAVE_VINES_PLANT",
        registry_id: "minecraft:cave_vines_plant",
    },
    BlockRegistryEntry {
        numeric_id: 1085,
        field_name: "SPORE_BLOSSOM",
        registry_id: "minecraft:spore_blossom",
    },
    BlockRegistryEntry {
        numeric_id: 1086,
        field_name: "AZALEA",
        registry_id: "minecraft:azalea",
    },
    BlockRegistryEntry {
        numeric_id: 1087,
        field_name: "FLOWERING_AZALEA",
        registry_id: "minecraft:flowering_azalea",
    },
    BlockRegistryEntry {
        numeric_id: 1088,
        field_name: "MOSS_CARPET",
        registry_id: "minecraft:moss_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 1089,
        field_name: "PINK_PETALS",
        registry_id: "minecraft:pink_petals",
    },
    BlockRegistryEntry {
        numeric_id: 1090,
        field_name: "WILDFLOWERS",
        registry_id: "minecraft:wildflowers",
    },
    BlockRegistryEntry {
        numeric_id: 1091,
        field_name: "LEAF_LITTER",
        registry_id: "minecraft:leaf_litter",
    },
    BlockRegistryEntry {
        numeric_id: 1092,
        field_name: "MOSS_BLOCK",
        registry_id: "minecraft:moss_block",
    },
    BlockRegistryEntry {
        numeric_id: 1093,
        field_name: "BIG_DRIPLEAF",
        registry_id: "minecraft:big_dripleaf",
    },
    BlockRegistryEntry {
        numeric_id: 1094,
        field_name: "BIG_DRIPLEAF_STEM",
        registry_id: "minecraft:big_dripleaf_stem",
    },
    BlockRegistryEntry {
        numeric_id: 1095,
        field_name: "SMALL_DRIPLEAF",
        registry_id: "minecraft:small_dripleaf",
    },
    BlockRegistryEntry {
        numeric_id: 1096,
        field_name: "HANGING_ROOTS",
        registry_id: "minecraft:hanging_roots",
    },
    BlockRegistryEntry {
        numeric_id: 1097,
        field_name: "ROOTED_DIRT",
        registry_id: "minecraft:rooted_dirt",
    },
    BlockRegistryEntry {
        numeric_id: 1098,
        field_name: "MUD",
        registry_id: "minecraft:mud",
    },
    BlockRegistryEntry {
        numeric_id: 1099,
        field_name: "DEEPSLATE",
        registry_id: "minecraft:deepslate",
    },
    BlockRegistryEntry {
        numeric_id: 1100,
        field_name: "COBBLED_DEEPSLATE",
        registry_id: "minecraft:cobbled_deepslate",
    },
    BlockRegistryEntry {
        numeric_id: 1101,
        field_name: "COBBLED_DEEPSLATE_STAIRS",
        registry_id: "minecraft:cobbled_deepslate_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1102,
        field_name: "COBBLED_DEEPSLATE_SLAB",
        registry_id: "minecraft:cobbled_deepslate_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1103,
        field_name: "COBBLED_DEEPSLATE_WALL",
        registry_id: "minecraft:cobbled_deepslate_wall",
    },
    BlockRegistryEntry {
        numeric_id: 1104,
        field_name: "POLISHED_DEEPSLATE",
        registry_id: "minecraft:polished_deepslate",
    },
    BlockRegistryEntry {
        numeric_id: 1105,
        field_name: "POLISHED_DEEPSLATE_STAIRS",
        registry_id: "minecraft:polished_deepslate_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1106,
        field_name: "POLISHED_DEEPSLATE_SLAB",
        registry_id: "minecraft:polished_deepslate_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1107,
        field_name: "POLISHED_DEEPSLATE_WALL",
        registry_id: "minecraft:polished_deepslate_wall",
    },
    BlockRegistryEntry {
        numeric_id: 1108,
        field_name: "DEEPSLATE_TILES",
        registry_id: "minecraft:deepslate_tiles",
    },
    BlockRegistryEntry {
        numeric_id: 1109,
        field_name: "DEEPSLATE_TILE_STAIRS",
        registry_id: "minecraft:deepslate_tile_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1110,
        field_name: "DEEPSLATE_TILE_SLAB",
        registry_id: "minecraft:deepslate_tile_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1111,
        field_name: "DEEPSLATE_TILE_WALL",
        registry_id: "minecraft:deepslate_tile_wall",
    },
    BlockRegistryEntry {
        numeric_id: 1112,
        field_name: "DEEPSLATE_BRICKS",
        registry_id: "minecraft:deepslate_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 1113,
        field_name: "DEEPSLATE_BRICK_STAIRS",
        registry_id: "minecraft:deepslate_brick_stairs",
    },
    BlockRegistryEntry {
        numeric_id: 1114,
        field_name: "DEEPSLATE_BRICK_SLAB",
        registry_id: "minecraft:deepslate_brick_slab",
    },
    BlockRegistryEntry {
        numeric_id: 1115,
        field_name: "DEEPSLATE_BRICK_WALL",
        registry_id: "minecraft:deepslate_brick_wall",
    },
    BlockRegistryEntry {
        numeric_id: 1116,
        field_name: "CHISELED_DEEPSLATE",
        registry_id: "minecraft:chiseled_deepslate",
    },
    BlockRegistryEntry {
        numeric_id: 1117,
        field_name: "CRACKED_DEEPSLATE_BRICKS",
        registry_id: "minecraft:cracked_deepslate_bricks",
    },
    BlockRegistryEntry {
        numeric_id: 1118,
        field_name: "CRACKED_DEEPSLATE_TILES",
        registry_id: "minecraft:cracked_deepslate_tiles",
    },
    BlockRegistryEntry {
        numeric_id: 1119,
        field_name: "INFESTED_DEEPSLATE",
        registry_id: "minecraft:infested_deepslate",
    },
    BlockRegistryEntry {
        numeric_id: 1120,
        field_name: "SMOOTH_BASALT",
        registry_id: "minecraft:smooth_basalt",
    },
    BlockRegistryEntry {
        numeric_id: 1121,
        field_name: "RAW_IRON_BLOCK",
        registry_id: "minecraft:raw_iron_block",
    },
    BlockRegistryEntry {
        numeric_id: 1122,
        field_name: "RAW_COPPER_BLOCK",
        registry_id: "minecraft:raw_copper_block",
    },
    BlockRegistryEntry {
        numeric_id: 1123,
        field_name: "RAW_GOLD_BLOCK",
        registry_id: "minecraft:raw_gold_block",
    },
    BlockRegistryEntry {
        numeric_id: 1124,
        field_name: "POTTED_AZALEA",
        registry_id: "minecraft:potted_azalea_bush",
    },
    BlockRegistryEntry {
        numeric_id: 1125,
        field_name: "POTTED_FLOWERING_AZALEA",
        registry_id: "minecraft:potted_flowering_azalea_bush",
    },
    BlockRegistryEntry {
        numeric_id: 1126,
        field_name: "OCHRE_FROGLIGHT",
        registry_id: "minecraft:ochre_froglight",
    },
    BlockRegistryEntry {
        numeric_id: 1127,
        field_name: "VERDANT_FROGLIGHT",
        registry_id: "minecraft:verdant_froglight",
    },
    BlockRegistryEntry {
        numeric_id: 1128,
        field_name: "PEARLESCENT_FROGLIGHT",
        registry_id: "minecraft:pearlescent_froglight",
    },
    BlockRegistryEntry {
        numeric_id: 1129,
        field_name: "FROGSPAWN",
        registry_id: "minecraft:frogspawn",
    },
    BlockRegistryEntry {
        numeric_id: 1130,
        field_name: "REINFORCED_DEEPSLATE",
        registry_id: "minecraft:reinforced_deepslate",
    },
    BlockRegistryEntry {
        numeric_id: 1131,
        field_name: "DECORATED_POT",
        registry_id: "minecraft:decorated_pot",
    },
    BlockRegistryEntry {
        numeric_id: 1132,
        field_name: "CRAFTER",
        registry_id: "minecraft:crafter",
    },
    BlockRegistryEntry {
        numeric_id: 1133,
        field_name: "TRIAL_SPAWNER",
        registry_id: "minecraft:trial_spawner",
    },
    BlockRegistryEntry {
        numeric_id: 1134,
        field_name: "VAULT",
        registry_id: "minecraft:vault",
    },
    BlockRegistryEntry {
        numeric_id: 1135,
        field_name: "HEAVY_CORE",
        registry_id: "minecraft:heavy_core",
    },
    BlockRegistryEntry {
        numeric_id: 1136,
        field_name: "PALE_MOSS_BLOCK",
        registry_id: "minecraft:pale_moss_block",
    },
    BlockRegistryEntry {
        numeric_id: 1137,
        field_name: "PALE_MOSS_CARPET",
        registry_id: "minecraft:pale_moss_carpet",
    },
    BlockRegistryEntry {
        numeric_id: 1138,
        field_name: "PALE_HANGING_MOSS",
        registry_id: "minecraft:pale_hanging_moss",
    },
    BlockRegistryEntry {
        numeric_id: 1139,
        field_name: "OPEN_EYEBLOSSOM",
        registry_id: "minecraft:open_eyeblossom",
    },
    BlockRegistryEntry {
        numeric_id: 1140,
        field_name: "CLOSED_EYEBLOSSOM",
        registry_id: "minecraft:closed_eyeblossom",
    },
    BlockRegistryEntry {
        numeric_id: 1141,
        field_name: "POTTED_OPEN_EYEBLOSSOM",
        registry_id: "minecraft:potted_open_eyeblossom",
    },
    BlockRegistryEntry {
        numeric_id: 1142,
        field_name: "POTTED_CLOSED_EYEBLOSSOM",
        registry_id: "minecraft:potted_closed_eyeblossom",
    },
    BlockRegistryEntry {
        numeric_id: 1143,
        field_name: "FIREFLY_BUSH",
        registry_id: "minecraft:firefly_bush",
    },
];

const AXIS_VALUES: &[&str] = &["x", "y", "z"];
const BOOLEAN_VALUES: &[&str] = &["false", "true"];
const FACING_HORIZONTAL_VALUES: &[&str] = &["north", "south", "west", "east"];
const HALF_VALUES: &[&str] = &["top", "bottom"];
const STAIR_SHAPE_VALUES: &[&str] = &[
    "straight",
    "inner_left",
    "inner_right",
    "outer_left",
    "outer_right",
];
const CHEST_TYPE_VALUES: &[&str] = &["single", "left", "right"];
const AGE_0_7_VALUES: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7"];
const EMPTY_PROPERTIES: &[BlockPropertyDefinition] = &[];
const GRASS_BLOCK_PROPERTIES: &[BlockPropertyDefinition] = &[BlockPropertyDefinition {
    name: "snowy",
    values: BOOLEAN_VALUES,
    default_value: "false",
}];
const OAK_LOG_PROPERTIES: &[BlockPropertyDefinition] = &[BlockPropertyDefinition {
    name: "axis",
    values: AXIS_VALUES,
    default_value: "y",
}];
const OAK_STAIRS_PROPERTIES: &[BlockPropertyDefinition] = &[
    BlockPropertyDefinition {
        name: "facing",
        values: FACING_HORIZONTAL_VALUES,
        default_value: "north",
    },
    BlockPropertyDefinition {
        name: "half",
        values: HALF_VALUES,
        default_value: "bottom",
    },
    BlockPropertyDefinition {
        name: "shape",
        values: STAIR_SHAPE_VALUES,
        default_value: "straight",
    },
    BlockPropertyDefinition {
        name: "waterlogged",
        values: BOOLEAN_VALUES,
        default_value: "false",
    },
];
const CHEST_PROPERTIES: &[BlockPropertyDefinition] = &[
    BlockPropertyDefinition {
        name: "facing",
        values: FACING_HORIZONTAL_VALUES,
        default_value: "north",
    },
    BlockPropertyDefinition {
        name: "type",
        values: CHEST_TYPE_VALUES,
        default_value: "single",
    },
    BlockPropertyDefinition {
        name: "waterlogged",
        values: BOOLEAN_VALUES,
        default_value: "false",
    },
];
const WHEAT_PROPERTIES: &[BlockPropertyDefinition] = &[BlockPropertyDefinition {
    name: "age",
    values: AGE_0_7_VALUES,
    default_value: "0",
}];

const AIR_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "none",
    sound_type: "empty",
    destroy_time: 0.0,
    explosion_resistance: 0.0,
    has_collision: false,
    occludes: false,
    light_emission: 0,
    pathfind_land: true,
    pathfind_air: true,
    can_survive_without_support: true,
};
const STONE_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "stone",
    sound_type: "stone",
    destroy_time: 1.5,
    explosion_resistance: 6.0,
    has_collision: true,
    occludes: true,
    light_emission: 0,
    pathfind_land: false,
    pathfind_air: false,
    can_survive_without_support: true,
};
const GRASS_BLOCK_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "grass",
    sound_type: "grass",
    destroy_time: 0.6,
    explosion_resistance: 0.6,
    has_collision: true,
    occludes: true,
    light_emission: 0,
    pathfind_land: false,
    pathfind_air: false,
    can_survive_without_support: true,
};
const OAK_LOG_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "wood",
    sound_type: "wood",
    destroy_time: 2.0,
    explosion_resistance: 2.0,
    has_collision: true,
    occludes: true,
    light_emission: 0,
    pathfind_land: false,
    pathfind_air: false,
    can_survive_without_support: true,
};
const OAK_STAIRS_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "wood",
    sound_type: "wood",
    destroy_time: 2.0,
    explosion_resistance: 3.0,
    has_collision: true,
    occludes: false,
    light_emission: 0,
    pathfind_land: false,
    pathfind_air: false,
    can_survive_without_support: true,
};
const CHEST_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "wood",
    sound_type: "wood",
    destroy_time: 2.5,
    explosion_resistance: 2.5,
    has_collision: true,
    occludes: false,
    light_emission: 0,
    pathfind_land: false,
    pathfind_air: false,
    can_survive_without_support: true,
};
const WHEAT_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "plant",
    sound_type: "crop",
    destroy_time: 0.0,
    explosion_resistance: 0.0,
    has_collision: false,
    occludes: false,
    light_emission: 0,
    pathfind_land: true,
    pathfind_air: true,
    can_survive_without_support: false,
};
const TORCH_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "none",
    sound_type: "wood",
    destroy_time: 0.0,
    explosion_resistance: 0.0,
    has_collision: false,
    occludes: false,
    light_emission: 14,
    pathfind_land: true,
    pathfind_air: true,
    can_survive_without_support: false,
};
// All simple plants/flowers use .instabreak() in Java → destroyTime=0.0, explosionResistance=0.0,
// noCollision(), sound(SoundType.GRASS).
const INSTABREAK_PLANT_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "plant",
    sound_type: "grass",
    destroy_time: 0.0,
    explosion_resistance: 0.0,
    has_collision: false,
    occludes: false,
    light_emission: 0,
    pathfind_land: true,
    pathfind_air: true,
    can_survive_without_support: false,
};

pub fn registry_entry_by_id(registry_id: &str) -> Option<&'static BlockRegistryEntry> {
    BLOCK_REGISTRY
        .iter()
        .find(|entry| entry.registry_id == registry_id)
}

pub fn representative_state_definition(registry_id: &str) -> Option<BlockStateDefinition> {
    Some(match registry_id {
        "minecraft:air" => BlockStateDefinition {
            registry_id: "minecraft:air",
            properties: EMPTY_PROPERTIES,
            physical: AIR_PHYSICAL,
            collision_shape: ShapeKind::Empty,
            occlusion_shape: ShapeKind::Empty,
        },
        "minecraft:stone" => BlockStateDefinition {
            registry_id: "minecraft:stone",
            properties: EMPTY_PROPERTIES,
            physical: STONE_PHYSICAL,
            collision_shape: ShapeKind::FullCube,
            occlusion_shape: ShapeKind::FullCube,
        },
        "minecraft:grass_block" => BlockStateDefinition {
            registry_id: "minecraft:grass_block",
            properties: GRASS_BLOCK_PROPERTIES,
            physical: GRASS_BLOCK_PHYSICAL,
            collision_shape: ShapeKind::FullCube,
            occlusion_shape: ShapeKind::FullCube,
        },
        "minecraft:oak_log" => BlockStateDefinition {
            registry_id: "minecraft:oak_log",
            properties: OAK_LOG_PROPERTIES,
            physical: OAK_LOG_PHYSICAL,
            collision_shape: ShapeKind::FullCube,
            occlusion_shape: ShapeKind::FullCube,
        },
        "minecraft:oak_stairs" => BlockStateDefinition {
            registry_id: "minecraft:oak_stairs",
            properties: OAK_STAIRS_PROPERTIES,
            physical: OAK_STAIRS_PHYSICAL,
            collision_shape: ShapeKind::Custom,
            occlusion_shape: ShapeKind::Custom,
        },
        "minecraft:chest" => BlockStateDefinition {
            registry_id: "minecraft:chest",
            properties: CHEST_PROPERTIES,
            physical: CHEST_PHYSICAL,
            collision_shape: ShapeKind::Custom,
            occlusion_shape: ShapeKind::Custom,
        },
        "minecraft:wheat" => BlockStateDefinition {
            registry_id: "minecraft:wheat",
            properties: WHEAT_PROPERTIES,
            physical: WHEAT_PHYSICAL,
            collision_shape: ShapeKind::Empty,
            occlusion_shape: ShapeKind::Empty,
        },
        "minecraft:torch" => BlockStateDefinition {
            registry_id: "minecraft:torch",
            properties: EMPTY_PROPERTIES,
            physical: TORCH_PHYSICAL,
            collision_shape: ShapeKind::Empty,
            occlusion_shape: ShapeKind::Empty,
        },
        // Instabreak plants — all use .instabreak() in Java (destroyTime=0.0, noCollision)
        "minecraft:short_grass"
        | "minecraft:fern"
        | "minecraft:dead_bush"
        | "minecraft:bush"
        | "minecraft:short_dry_grass"
        | "minecraft:dandelion"
        | "minecraft:golden_dandelion"
        | "minecraft:torchflower"
        | "minecraft:poppy"
        | "minecraft:blue_orchid"
        | "minecraft:allium"
        | "minecraft:azure_bluet"
        | "minecraft:red_tulip"
        | "minecraft:orange_tulip"
        | "minecraft:white_tulip"
        | "minecraft:pink_tulip"
        | "minecraft:oxeye_daisy"
        | "minecraft:cornflower"
        | "minecraft:wither_rose"
        | "minecraft:lily_of_the_valley"
        | "minecraft:brown_mushroom"
        | "minecraft:red_mushroom"
        | "minecraft:wildflowers"
        | "minecraft:firefly_bush"
        | "minecraft:tall_grass"
        | "minecraft:large_fern"
        | "minecraft:rose_bush"
        | "minecraft:peony"
        | "minecraft:lilac"
        | "minecraft:sunflower" => {
            // Use the static registry_id string so the lifetime requirement is satisfied
            let static_id = registry_entry_by_id(registry_id)?.registry_id;
            return Some(BlockStateDefinition {
                registry_id: static_id,
                properties: EMPTY_PROPERTIES,
                physical: INSTABREAK_PLANT_PHYSICAL,
                collision_shape: ShapeKind::Empty,
                occlusion_shape: ShapeKind::Empty,
            });
        }
        _ => return None,
    })
}

pub fn possible_state_count(definition: &BlockStateDefinition) -> usize {
    definition
        .properties
        .iter()
        .map(|property| property.values.len())
        .product::<usize>()
        .max(1)
}

pub fn default_state(definition: &BlockStateDefinition) -> BTreeMap<&'static str, &'static str> {
    definition
        .properties
        .iter()
        .map(|property| (property.name, property.default_value))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        default_state, possible_state_count, registry_entry_by_id, representative_state_definition,
        BLOCK_REGISTRY, VANILLA_BLOCK_REGISTRY_COUNT,
    };
    use crate::block_metadata::ShapeKind;
    use std::collections::BTreeSet;

    #[test]
    fn block_registry_order_matches_blocks_static_registration_surface() {
        assert_eq!(VANILLA_BLOCK_REGISTRY_COUNT, 1144);
        assert_eq!(BLOCK_REGISTRY.len(), VANILLA_BLOCK_REGISTRY_COUNT);
        assert_eq!(BLOCK_REGISTRY[0].registry_id, "minecraft:air");
        assert_eq!(BLOCK_REGISTRY[1].registry_id, "minecraft:stone");
        assert_eq!(BLOCK_REGISTRY[9].registry_id, "minecraft:dirt");
        assert_eq!(
            BLOCK_REGISTRY.last().unwrap().registry_id,
            "minecraft:firefly_bush"
        );
        assert!(BLOCK_REGISTRY
            .iter()
            .enumerate()
            .all(|(idx, entry)| entry.numeric_id == idx));
        let ids = BLOCK_REGISTRY
            .iter()
            .map(|entry| entry.registry_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), BLOCK_REGISTRY.len());
    }

    #[test]
    fn representative_state_counts_match_vanilla_property_products() {
        for (id, count) in [
            ("minecraft:stone", 1),
            ("minecraft:grass_block", 2),
            ("minecraft:oak_log", 3),
            ("minecraft:oak_stairs", 80),
            ("minecraft:chest", 24),
            ("minecraft:wheat", 8),
        ] {
            let definition =
                representative_state_definition(id).expect("known representative block");
            assert_eq!(possible_state_count(&definition), count, "{id}");
        }
    }

    #[test]
    fn default_states_expose_property_defaults() {
        let stairs = representative_state_definition("minecraft:oak_stairs").unwrap();
        let defaults = default_state(&stairs);
        assert_eq!(defaults["facing"], "north");
        assert_eq!(defaults["half"], "bottom");
        assert_eq!(defaults["shape"], "straight");
        assert_eq!(defaults["waterlogged"], "false");
    }

    #[test]
    fn representative_properties_cover_physics_and_pathfinding_flags() {
        let air = representative_state_definition("minecraft:air").unwrap();
        assert!(!air.physical.has_collision);
        assert!(air.physical.pathfind_land);
        assert_eq!(air.collision_shape, ShapeKind::Empty);

        let stone = representative_state_definition("minecraft:stone").unwrap();
        assert_eq!(stone.physical.map_color, "stone");
        assert_eq!(stone.physical.destroy_time, 1.5);
        assert_eq!(stone.physical.explosion_resistance, 6.0);
        assert!(stone.physical.occludes);
        assert_eq!(stone.occlusion_shape, ShapeKind::FullCube);
        assert!(!stone.physical.pathfind_land);

        let torch = representative_state_definition("minecraft:torch").unwrap();
        assert_eq!(torch.physical.light_emission, 14);
        assert!(!torch.physical.can_survive_without_support);
    }

    #[test]
    fn registry_lookup_resolves_representative_ids() {
        assert_eq!(
            registry_entry_by_id("minecraft:oak_stairs")
                .unwrap()
                .field_name,
            "OAK_STAIRS"
        );
        assert_eq!(
            registry_entry_by_id("minecraft:chest").unwrap().field_name,
            "CHEST"
        );
        assert!(registry_entry_by_id("minecraft:not_a_block").is_none());
    }
}
