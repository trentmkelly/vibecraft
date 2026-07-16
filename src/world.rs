#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DimensionType {
    pub id: &'static str,
    pub has_fixed_time: bool,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub has_ender_dragon_fight: bool,
    pub coordinate_scale: f64,
    pub min_y: i32,
    pub height: i32,
    pub logical_height: i32,
    pub infiniburn: &'static str,
    pub ambient_light: f32,
    pub monster_spawn_light_level: MonsterSpawnLightLevel,
    pub monster_spawn_block_light_limit: u8,
    pub skybox: Skybox,
    pub cardinal_light: CardinalLight,
    pub bed_rule: BedRule,
    pub respawn_anchor_works: bool,
    pub can_start_raid: bool,
    pub default_clock: Option<&'static str>,
    pub timelines: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterSpawnLightLevel {
    Constant(u8),
    Uniform {
        min_inclusive: u8,
        max_inclusive: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skybox {
    None,
    Overworld,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardinalLight {
    Default,
    Nether,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BedRule {
    pub can_set_spawn: BedSpawnRule,
    pub can_sleep: SleepRule,
    pub explodes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BedSpawnRule {
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepRule {
    WhenDark,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelStem {
    pub id: &'static str,
    pub dimension_type: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldPreset {
    pub id: &'static str,
    pub dimensions: &'static [PresetDimension],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresetDimension {
    pub level_stem: &'static str,
    pub dimension_type: &'static str,
    pub generator: ChunkGeneratorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkGeneratorKind {
    Noise {
        biome_source: BiomeSourceKind,
        noise_settings: &'static str,
    },
    Flat {
        settings: FlatGeneratorSettings,
    },
    Debug {
        biome: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiomeSourceKind {
    MultiNoisePreset(&'static str),
    Fixed(&'static str),
    TheEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlatGeneratorSettings {
    pub biome: &'static str,
    pub layers: &'static [FlatLayer],
    pub lakes: bool,
    pub features: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlatLayer {
    pub block: &'static str,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldOptions {
    pub seed: i64,
    pub generate_structures: bool,
    pub generate_bonus_chest: bool,
    pub demo: bool,
    pub legacy_custom_options: Option<String>,
}

pub const OVERWORLD_MIN_Y: i32 = -64;
pub const OVERWORLD_LEVEL_HEIGHT: i32 = 384;
pub const OVERWORLD_GENERATION_HEIGHT: i32 = 384;
pub const OVERWORLD_LOGICAL_HEIGHT: i32 = 384;
pub const OVERWORLD_CLOUD_HEIGHT: f32 = 192.33;

pub const NETHER_MIN_Y: i32 = 0;
pub const NETHER_LEVEL_HEIGHT: i32 = 256;
pub const NETHER_GENERATION_HEIGHT: i32 = 128;
pub const NETHER_LOGICAL_HEIGHT: i32 = 128;

pub const END_MIN_Y: i32 = 0;
pub const END_LEVEL_HEIGHT: i32 = 256;
pub const END_GENERATION_HEIGHT: i32 = 128;
pub const END_LOGICAL_HEIGHT: i32 = 256;
pub const END_ISLAND_BASE_Y: i32 = 63;

pub const OVERWORLD: DimensionType = DimensionType {
    id: "minecraft:overworld",
    has_fixed_time: false,
    has_skylight: true,
    has_ceiling: false,
    has_ender_dragon_fight: false,
    coordinate_scale: 1.0,
    min_y: OVERWORLD_MIN_Y,
    height: OVERWORLD_LEVEL_HEIGHT,
    logical_height: OVERWORLD_LOGICAL_HEIGHT,
    infiniburn: "#minecraft:infiniburn_overworld",
    ambient_light: 0.0,
    monster_spawn_light_level: MonsterSpawnLightLevel::Uniform {
        min_inclusive: 0,
        max_inclusive: 7,
    },
    monster_spawn_block_light_limit: 0,
    skybox: Skybox::Overworld,
    cardinal_light: CardinalLight::Default,
    bed_rule: BedRule {
        can_set_spawn: BedSpawnRule::Always,
        can_sleep: SleepRule::WhenDark,
        explodes: false,
    },
    respawn_anchor_works: false,
    can_start_raid: true,
    default_clock: Some("minecraft:overworld"),
    timelines: "#minecraft:in_overworld",
};

pub const OVERWORLD_CAVES: DimensionType = DimensionType {
    has_ceiling: true,
    ..OVERWORLD
};

pub const NETHER: DimensionType = DimensionType {
    id: "minecraft:the_nether",
    has_fixed_time: true,
    has_skylight: false,
    has_ceiling: true,
    has_ender_dragon_fight: false,
    coordinate_scale: 8.0,
    min_y: NETHER_MIN_Y,
    height: NETHER_LEVEL_HEIGHT,
    logical_height: NETHER_LOGICAL_HEIGHT,
    infiniburn: "#minecraft:infiniburn_nether",
    ambient_light: 0.1,
    monster_spawn_light_level: MonsterSpawnLightLevel::Constant(7),
    monster_spawn_block_light_limit: 15,
    skybox: Skybox::None,
    cardinal_light: CardinalLight::Nether,
    bed_rule: BedRule {
        can_set_spawn: BedSpawnRule::Never,
        can_sleep: SleepRule::Never,
        explodes: true,
    },
    respawn_anchor_works: true,
    can_start_raid: false,
    default_clock: None,
    timelines: "#minecraft:in_nether",
};

pub const END: DimensionType = DimensionType {
    id: "minecraft:the_end",
    has_fixed_time: true,
    has_skylight: true,
    has_ceiling: false,
    has_ender_dragon_fight: true,
    coordinate_scale: 1.0,
    min_y: END_MIN_Y,
    height: END_LEVEL_HEIGHT,
    logical_height: END_LOGICAL_HEIGHT,
    infiniburn: "#minecraft:infiniburn_end",
    ambient_light: 0.25,
    monster_spawn_light_level: MonsterSpawnLightLevel::Constant(15),
    monster_spawn_block_light_limit: 0,
    skybox: Skybox::End,
    cardinal_light: CardinalLight::Default,
    bed_rule: BedRule {
        can_set_spawn: BedSpawnRule::Never,
        can_sleep: SleepRule::Never,
        explodes: true,
    },
    respawn_anchor_works: false,
    can_start_raid: true,
    default_clock: Some("minecraft:the_end"),
    timelines: "#minecraft:in_end",
};

pub const LEVEL_STEMS: &[LevelStem] = &[
    LevelStem {
        id: "minecraft:overworld",
        dimension_type: "minecraft:overworld",
    },
    LevelStem {
        id: "minecraft:the_nether",
        dimension_type: "minecraft:the_nether",
    },
    LevelStem {
        id: "minecraft:the_end",
        dimension_type: "minecraft:the_end",
    },
];

pub const FLAT_OVERWORLD_LAYERS: &[FlatLayer] = &[
    FlatLayer {
        block: "minecraft:bedrock",
        height: 1,
    },
    FlatLayer {
        block: "minecraft:dirt",
        height: 2,
    },
    FlatLayer {
        block: "minecraft:grass_block",
        height: 1,
    },
];

pub const DEFAULT_FLAT_GENERATOR_SETTINGS: FlatGeneratorSettings = FlatGeneratorSettings {
    biome: "minecraft:plains",
    layers: FLAT_OVERWORLD_LAYERS,
    lakes: false,
    features: false,
};

pub const NORMAL_OVERWORLD: PresetDimension = PresetDimension {
    level_stem: "minecraft:overworld",
    dimension_type: "minecraft:overworld",
    generator: ChunkGeneratorKind::Noise {
        biome_source: BiomeSourceKind::MultiNoisePreset("minecraft:overworld"),
        noise_settings: "minecraft:overworld",
    },
};

pub const LARGE_BIOMES_OVERWORLD: PresetDimension = PresetDimension {
    generator: ChunkGeneratorKind::Noise {
        biome_source: BiomeSourceKind::MultiNoisePreset("minecraft:overworld"),
        noise_settings: "minecraft:large_biomes",
    },
    ..NORMAL_OVERWORLD
};

pub const AMPLIFIED_OVERWORLD: PresetDimension = PresetDimension {
    generator: ChunkGeneratorKind::Noise {
        biome_source: BiomeSourceKind::MultiNoisePreset("minecraft:overworld"),
        noise_settings: "minecraft:amplified",
    },
    ..NORMAL_OVERWORLD
};

pub const SINGLE_BIOME_SURFACE_OVERWORLD: PresetDimension = PresetDimension {
    generator: ChunkGeneratorKind::Noise {
        biome_source: BiomeSourceKind::Fixed("minecraft:plains"),
        noise_settings: "minecraft:overworld",
    },
    ..NORMAL_OVERWORLD
};

pub const FLAT_OVERWORLD: PresetDimension = PresetDimension {
    generator: ChunkGeneratorKind::Flat {
        settings: DEFAULT_FLAT_GENERATOR_SETTINGS,
    },
    ..NORMAL_OVERWORLD
};

pub const DEBUG_OVERWORLD: PresetDimension = PresetDimension {
    generator: ChunkGeneratorKind::Debug {
        biome: "minecraft:plains",
    },
    ..NORMAL_OVERWORLD
};

pub const NETHER_PRESET_DIMENSION: PresetDimension = PresetDimension {
    level_stem: "minecraft:the_nether",
    dimension_type: "minecraft:the_nether",
    generator: ChunkGeneratorKind::Noise {
        biome_source: BiomeSourceKind::MultiNoisePreset("minecraft:nether"),
        noise_settings: "minecraft:nether",
    },
};

pub const END_PRESET_DIMENSION: PresetDimension = PresetDimension {
    level_stem: "minecraft:the_end",
    dimension_type: "minecraft:the_end",
    generator: ChunkGeneratorKind::Noise {
        biome_source: BiomeSourceKind::TheEnd,
        noise_settings: "minecraft:end",
    },
};

pub const NORMAL_WORLD_DIMENSIONS: &[PresetDimension] = &[
    NORMAL_OVERWORLD,
    NETHER_PRESET_DIMENSION,
    END_PRESET_DIMENSION,
];
pub const LARGE_BIOMES_WORLD_DIMENSIONS: &[PresetDimension] = &[
    LARGE_BIOMES_OVERWORLD,
    NETHER_PRESET_DIMENSION,
    END_PRESET_DIMENSION,
];
pub const AMPLIFIED_WORLD_DIMENSIONS: &[PresetDimension] = &[
    AMPLIFIED_OVERWORLD,
    NETHER_PRESET_DIMENSION,
    END_PRESET_DIMENSION,
];
pub const SINGLE_BIOME_SURFACE_WORLD_DIMENSIONS: &[PresetDimension] = &[
    SINGLE_BIOME_SURFACE_OVERWORLD,
    NETHER_PRESET_DIMENSION,
    END_PRESET_DIMENSION,
];
pub const FLAT_WORLD_DIMENSIONS: &[PresetDimension] = &[
    FLAT_OVERWORLD,
    NETHER_PRESET_DIMENSION,
    END_PRESET_DIMENSION,
];
pub const DEBUG_WORLD_DIMENSIONS: &[PresetDimension] = &[
    DEBUG_OVERWORLD,
    NETHER_PRESET_DIMENSION,
    END_PRESET_DIMENSION,
];

pub const BUILTIN_WORLD_PRESETS: &[WorldPreset] = &[
    WorldPreset {
        id: "minecraft:normal",
        dimensions: NORMAL_WORLD_DIMENSIONS,
    },
    WorldPreset {
        id: "minecraft:flat",
        dimensions: FLAT_WORLD_DIMENSIONS,
    },
    WorldPreset {
        id: "minecraft:large_biomes",
        dimensions: LARGE_BIOMES_WORLD_DIMENSIONS,
    },
    WorldPreset {
        id: "minecraft:amplified",
        dimensions: AMPLIFIED_WORLD_DIMENSIONS,
    },
    WorldPreset {
        id: "minecraft:single_biome_surface",
        dimensions: SINGLE_BIOME_SURFACE_WORLD_DIMENSIONS,
    },
    WorldPreset {
        id: "minecraft:debug_all_block_states",
        dimensions: DEBUG_WORLD_DIMENSIONS,
    },
];

impl WorldOptions {
    pub const DEMO_SEED: i64 = -343_522_682;

    pub fn new(seed: i64, generate_structures: bool, generate_bonus_chest: bool) -> Self {
        Self {
            seed,
            generate_structures,
            generate_bonus_chest,
            demo: false,
            legacy_custom_options: None,
        }
    }

    pub fn demo_options() -> Self {
        Self {
            seed: Self::DEMO_SEED,
            generate_structures: true,
            generate_bonus_chest: true,
            demo: true,
            legacy_custom_options: None,
        }
    }

    /// Java `WorldOptions.defaultWithRandomSeed`: normal generation with a
    /// random seed and no bonus chest.
    pub fn default_with_random_seed() -> Self {
        Self::new(random_seed(), true, false)
    }

    /// Java `WorldOptions.testWorldWithRandomSeed`: random seed with
    /// structures and bonus chests disabled for deterministic test setup.
    pub fn test_world_with_random_seed() -> Self {
        Self::new(random_seed(), false, false)
    }

    pub fn from_server_inputs(
        level_seed: &str,
        generate_structures: bool,
        bonus_chest: bool,
        demo: bool,
    ) -> Self {
        if demo {
            return Self::demo_options();
        }
        Self::new(
            parse_seed(level_seed).unwrap_or_else(random_seed),
            generate_structures,
            bonus_chest,
        )
    }

    pub fn with_bonus_chest(&self, generate_bonus_chest: bool) -> Self {
        Self {
            generate_bonus_chest,
            ..self.clone()
        }
    }

    pub fn with_structures(&self, generate_structures: bool) -> Self {
        Self {
            generate_structures,
            ..self.clone()
        }
    }

    pub fn with_seed(&self, seed: Option<i64>) -> Self {
        Self {
            seed: seed.unwrap_or_else(random_seed),
            ..self.clone()
        }
    }

    pub fn is_old_customized_world(&self) -> bool {
        self.legacy_custom_options.is_some()
    }
}

pub fn parse_seed(seed: &str) -> Option<i64> {
    let seed = seed.trim();
    if seed.is_empty() {
        None
    } else {
        seed.parse::<i64>()
            .ok()
            .or_else(|| Some(java_string_hash(seed)))
    }
}

pub fn random_seed() -> i64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_else(|err| err.duration().as_nanos());
    let mixed = nanos ^ nanos.rotate_left(29) ^ u128::from(std::process::id());
    mixed as i64
}

fn java_string_hash(value: &str) -> i64 {
    value.encode_utf16().fold(0i32, |hash, code_unit| {
        hash.wrapping_mul(31).wrapping_add(i32::from(code_unit))
    }) as i64
}

impl DimensionType {
    pub fn validate(self) -> Result<(), String> {
        if self.height < 16 {
            return Err("height has to be at least 16".to_string());
        }
        if self.logical_height > self.height {
            return Err("logical_height cannot be higher than height".to_string());
        }
        if self.height % 16 != 0 {
            return Err("height has to be multiple of 16".to_string());
        }
        if self.min_y % 16 != 0 {
            return Err("min_y has to be a multiple of 16".to_string());
        }
        Ok(())
    }

    pub fn teleportation_scale_to(self, other: Self) -> f64 {
        self.coordinate_scale / other.coordinate_scale
    }

    pub fn storage_folder(self, base: &Path) -> PathBuf {
        let id = self.id.strip_prefix("minecraft:").unwrap_or(self.id);
        if id == "overworld" {
            base.to_path_buf()
        } else {
            base.join("dimensions").join("minecraft").join(id)
        }
    }
}

pub fn builtin_dimension_type(id: &str) -> Option<DimensionType> {
    match id {
        "minecraft:overworld" | "overworld" => Some(OVERWORLD),
        "minecraft:overworld_caves" | "overworld_caves" => Some(OVERWORLD_CAVES),
        "minecraft:the_nether" | "the_nether" => Some(NETHER),
        "minecraft:the_end" | "the_end" => Some(END),
        _ => None,
    }
}

pub fn builtin_world_preset(id: &str) -> Option<&'static WorldPreset> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_WORLD_PRESETS.iter().find(|preset| {
        preset
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|preset_name| preset_name == name)
    })
}

impl WorldPreset {
    pub fn overworld(self) -> Option<PresetDimension> {
        self.dimensions
            .iter()
            .copied()
            .find(|dimension| dimension.level_stem == "minecraft:overworld")
    }

    pub fn validate(self) -> Result<(), String> {
        if self.overworld().is_none() {
            return Err("Missing overworld dimension".to_string());
        }
        for (idx, dimension) in self.dimensions.iter().enumerate() {
            if dimension.level_stem == "minecraft:overworld" && idx != 0 {
                return Err(
                    "overworld dimension must be first in vanilla world dimension order"
                        .to_string(),
                );
            }
            if builtin_dimension_type(dimension.dimension_type).is_none() {
                return Err(format!(
                    "unknown dimension type {}",
                    dimension.dimension_type
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        builtin_dimension_type, builtin_world_preset, BedSpawnRule, BiomeSourceKind, CardinalLight,
        ChunkGeneratorKind, MonsterSpawnLightLevel, Skybox, SleepRule, WorldOptions,
        BUILTIN_WORLD_PRESETS, DEFAULT_FLAT_GENERATOR_SETTINGS, END, END_GENERATION_HEIGHT,
        END_ISLAND_BASE_Y, LEVEL_STEMS, NETHER, NETHER_GENERATION_HEIGHT, OVERWORLD,
        OVERWORLD_CAVES, OVERWORLD_CLOUD_HEIGHT, OVERWORLD_GENERATION_HEIGHT,
    };
    use std::path::Path;

    #[test]
    fn builtin_level_stems_match_vanilla_ids() {
        assert_eq!(LEVEL_STEMS[0].id, "minecraft:overworld");
        assert_eq!(LEVEL_STEMS[1].id, "minecraft:the_nether");
        assert_eq!(LEVEL_STEMS[2].id, "minecraft:the_end");
        assert_eq!(
            LEVEL_STEMS
                .iter()
                .map(|stem| stem.dimension_type)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld",
                "minecraft:the_nether",
                "minecraft:the_end"
            ]
        );
    }

    #[test]
    fn builtin_world_presets_match_26_1_2_bootstrap_keys_and_order() {
        assert_eq!(
            BUILTIN_WORLD_PRESETS
                .iter()
                .map(|preset| preset.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:normal",
                "minecraft:flat",
                "minecraft:large_biomes",
                "minecraft:amplified",
                "minecraft:single_biome_surface",
                "minecraft:debug_all_block_states",
            ]
        );

        for preset in BUILTIN_WORLD_PRESETS {
            assert!(preset.validate().is_ok());
            assert_eq!(
                preset
                    .dimensions
                    .iter()
                    .map(|dimension| dimension.level_stem)
                    .collect::<Vec<_>>(),
                vec![
                    "minecraft:overworld",
                    "minecraft:the_nether",
                    "minecraft:the_end"
                ]
            );
        }
    }

    #[test]
    fn world_preset_generators_match_worldpresets_bootstrap() {
        let normal = builtin_world_preset("normal").unwrap();
        assert_eq!(
            normal.overworld().unwrap().generator,
            ChunkGeneratorKind::Noise {
                biome_source: BiomeSourceKind::MultiNoisePreset("minecraft:overworld"),
                noise_settings: "minecraft:overworld",
            }
        );

        let large_biomes = builtin_world_preset("minecraft:large_biomes").unwrap();
        assert_eq!(
            large_biomes.overworld().unwrap().generator,
            ChunkGeneratorKind::Noise {
                biome_source: BiomeSourceKind::MultiNoisePreset("minecraft:overworld"),
                noise_settings: "minecraft:large_biomes",
            }
        );

        let amplified = builtin_world_preset("amplified").unwrap();
        assert_eq!(
            amplified.overworld().unwrap().generator,
            ChunkGeneratorKind::Noise {
                biome_source: BiomeSourceKind::MultiNoisePreset("minecraft:overworld"),
                noise_settings: "minecraft:amplified",
            }
        );

        let single_biome = builtin_world_preset("single_biome_surface").unwrap();
        assert_eq!(
            single_biome.overworld().unwrap().generator,
            ChunkGeneratorKind::Noise {
                biome_source: BiomeSourceKind::Fixed("minecraft:plains"),
                noise_settings: "minecraft:overworld",
            }
        );

        let flat = builtin_world_preset("flat").unwrap();
        assert_eq!(
            flat.overworld().unwrap().generator,
            ChunkGeneratorKind::Flat {
                settings: DEFAULT_FLAT_GENERATOR_SETTINGS,
            }
        );
        assert_eq!(
            DEFAULT_FLAT_GENERATOR_SETTINGS
                .layers
                .iter()
                .map(|layer| (layer.block, layer.height))
                .collect::<Vec<_>>(),
            vec![
                ("minecraft:bedrock", 1),
                ("minecraft:dirt", 2),
                ("minecraft:grass_block", 1),
            ]
        );

        let debug = builtin_world_preset("debug_all_block_states").unwrap();
        assert_eq!(
            debug.overworld().unwrap().generator,
            ChunkGeneratorKind::Debug {
                biome: "minecraft:plains",
            }
        );
    }

    #[test]
    fn overworld_dimension_type_matches_26_1_2_data() {
        let Some(overworld) = builtin_dimension_type("overworld") else {
            panic!("overworld dimension type must be registered");
        };
        assert!(overworld.validate().is_ok());
        assert!(overworld.has_skylight);
        assert!(!overworld.has_ceiling);
        assert_eq!(overworld.min_y, -64);
        assert_eq!(overworld.height, 384);
        assert_eq!(overworld.logical_height, 384);
        assert_eq!(OVERWORLD_GENERATION_HEIGHT, 384);
        assert_eq!(OVERWORLD_CLOUD_HEIGHT, 192.33);
        assert_eq!(
            overworld.monster_spawn_light_level,
            MonsterSpawnLightLevel::Uniform {
                min_inclusive: 0,
                max_inclusive: 7
            }
        );
        assert_eq!(overworld.bed_rule.can_set_spawn, BedSpawnRule::Always);
        assert_eq!(overworld.bed_rule.can_sleep, SleepRule::WhenDark);
        assert!(!overworld.respawn_anchor_works);
        assert!(overworld.can_start_raid);
    }

    #[test]
    fn nether_dimension_type_matches_26_1_2_data() {
        let Some(nether) = builtin_dimension_type("the_nether") else {
            panic!("nether dimension type must be registered");
        };
        assert!(nether.validate().is_ok());
        assert!(nether.has_fixed_time);
        assert!(!nether.has_skylight);
        assert!(nether.has_ceiling);
        assert_eq!(nether.coordinate_scale, 8.0);
        assert_eq!(nether.logical_height, 128);
        assert_eq!(NETHER_GENERATION_HEIGHT, 128);
        assert_eq!(
            nether.monster_spawn_light_level,
            MonsterSpawnLightLevel::Constant(7)
        );
        assert_eq!(nether.monster_spawn_block_light_limit, 15);
        assert_eq!(nether.skybox, Skybox::None);
        assert_eq!(nether.cardinal_light, CardinalLight::Nether);
        assert!(nether.bed_rule.explodes);
        assert!(nether.respawn_anchor_works);
        assert!(!nether.can_start_raid);
    }

    #[test]
    fn end_dimension_type_matches_26_1_2_data() {
        let Some(end) = builtin_dimension_type("the_end") else {
            panic!("end dimension type must be registered");
        };
        assert!(end.validate().is_ok());
        assert!(end.has_fixed_time);
        assert!(end.has_skylight);
        assert!(end.has_ender_dragon_fight);
        assert_eq!(end.logical_height, 256);
        assert_eq!(END_GENERATION_HEIGHT, 128);
        assert_eq!(END_ISLAND_BASE_Y, 63);
        assert_eq!(
            end.monster_spawn_light_level,
            MonsterSpawnLightLevel::Constant(15)
        );
        assert_eq!(end.skybox, Skybox::End);
        assert!(end.bed_rule.explodes);
        assert!(!end.respawn_anchor_works);
    }

    #[test]
    fn dimension_storage_paths_and_teleport_scale_match_vanilla_rules() {
        let base = Path::new("world");
        assert_eq!(OVERWORLD.storage_folder(base), Path::new("world"));
        assert_eq!(
            NETHER.storage_folder(base),
            Path::new("world/dimensions/minecraft/the_nether")
        );
        assert_eq!(OVERWORLD.teleportation_scale_to(NETHER), 0.125);
        assert_eq!(NETHER.teleportation_scale_to(OVERWORLD), 8.0);
    }

    #[test]
    fn builtin_dimension_lookup_includes_overworld_caves() {
        assert_eq!(builtin_dimension_type("overworld"), Some(OVERWORLD));
        assert_eq!(builtin_dimension_type("minecraft:the_end"), Some(END));
        assert_eq!(
            builtin_dimension_type("overworld_caves"),
            Some(OVERWORLD_CAVES)
        );
        let Some(overworld_caves) = builtin_dimension_type("overworld_caves") else {
            panic!("overworld_caves dimension type must be registered");
        };
        assert!(overworld_caves.has_ceiling);
        assert_eq!(overworld_caves.min_y, OVERWORLD.min_y);
    }

    #[test]
    fn world_options_parse_seed_like_vanilla() {
        assert_eq!(super::parse_seed(""), None);
        assert_eq!(super::parse_seed(" 123 "), Some(123));
        assert_eq!(super::parse_seed("-5"), Some(-5));
        assert_eq!(
            super::parse_seed("North Carolina"),
            Some(WorldOptions::DEMO_SEED)
        );
    }

    #[test]
    fn world_options_follow_dedicated_server_and_demo_rules() {
        let normal = WorldOptions::from_server_inputs("8675309", false, true, false);
        assert_eq!(normal.seed, 8_675_309);
        assert!(!normal.generate_structures);
        assert!(normal.generate_bonus_chest);
        assert!(!normal.demo);

        let generated = WorldOptions::from_server_inputs("", true, false, false);
        assert!(generated.generate_structures);
        assert!(!generated.generate_bonus_chest);
        assert!(!generated.demo);

        let demo = WorldOptions::from_server_inputs("ignored", false, false, true);
        assert_eq!(demo, WorldOptions::demo_options());
        assert_eq!(demo.seed, WorldOptions::DEMO_SEED);
        assert!(demo.generate_structures);
        assert!(demo.generate_bonus_chest);
        assert!(demo.demo);
    }

    #[test]
    fn world_options_random_seed_factories_match_java_defaults() {
        let normal = WorldOptions::default_with_random_seed();
        assert!(normal.generate_structures);
        assert!(!normal.generate_bonus_chest);
        assert!(!normal.demo);

        let test = WorldOptions::test_world_with_random_seed();
        assert!(!test.generate_structures);
        assert!(!test.generate_bonus_chest);
        assert!(!test.demo);
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn world_options_source_matches_java_26_1_2_contract() {
        const JAVA_SOURCE: &str =
            vibecraft_java_source!("/net/minecraft/world/level/levelgen/WorldOptions.java");
        for fragment in [
            "public static WorldOptions defaultWithRandomSeed()",
            "public static WorldOptions testWorldWithRandomSeed()",
            "public boolean isOldCustomizedWorld()",
            "public WorldOptions withBonusChest(final boolean generateBonusChest)",
            "public WorldOptions withStructures(final boolean generateStructures)",
            "public WorldOptions withSeed(final OptionalLong seed)",
            "public static OptionalLong parseSeed(String seedString)",
            "return OptionalLong.of(seedString.hashCode());",
            "public static long randomSeed()",
        ] {
            assert!(JAVA_SOURCE.contains(fragment), "missing Java source fragment: {fragment}");
        }
    }
}
