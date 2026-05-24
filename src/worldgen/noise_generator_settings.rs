use super::*;

pub const NETHER_LEVEL_STEM: LevelStemPreset = LevelStemPreset {
    dimension: "minecraft:the_nether",
    generator: "minecraft:noise",
    biome_source: "minecraft:multi_noise/nether",
    noise_settings: Some("minecraft:nether"),
};

pub const END_LEVEL_STEM: LevelStemPreset = LevelStemPreset {
    dimension: "minecraft:the_end",
    generator: "minecraft:noise",
    biome_source: "minecraft:the_end",
    noise_settings: Some("minecraft:end"),
};

pub const WORLD_PRESETS: &[WorldPresetEntry] = &[
    WorldPresetEntry {
        id: "minecraft:normal",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:multi_noise/overworld",
            noise_settings: Some("minecraft:overworld"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:flat",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:flat",
            biome_source: "minecraft:plains",
            noise_settings: None,
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:large_biomes",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:multi_noise/overworld",
            noise_settings: Some("minecraft:large_biomes"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:amplified",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:multi_noise/overworld",
            noise_settings: Some("minecraft:amplified"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:single_biome_surface",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:fixed/plains",
            noise_settings: Some("minecraft:overworld"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:debug_all_block_states",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:debug",
            biome_source: "minecraft:plains",
            noise_settings: None,
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
];

pub const OVERWORLD_SPAWN_TARGET: &[ClimateParameterPoint] = &[
    ClimateParameterPoint {
        temperature: span(-1.0, 1.0),
        humidity: span(-1.0, 1.0),
        continentalness: span(-0.11, 1.0),
        erosion: span(-1.0, 1.0),
        depth: span(0.0, 0.0),
        weirdness: span(-1.0, -0.16),
        offset: 0,
    },
    ClimateParameterPoint {
        temperature: span(-1.0, 1.0),
        humidity: span(-1.0, 1.0),
        continentalness: span(-0.11, 1.0),
        erosion: span(-1.0, 1.0),
        depth: span(0.0, 0.0),
        weirdness: span(0.16, 1.0),
        offset: 0,
    },
];

pub const BUILTIN_NOISE_GENERATOR_SETTINGS: &[NoiseGeneratorSettings] = &[
    NoiseGeneratorSettings {
        id: "minecraft:overworld",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: false,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:large_biomes",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: true,
            amplified: false,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:amplified",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: true,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:nether",
        noise: NETHER_NOISE_SETTINGS,
        default_block: "minecraft:netherrack",
        default_fluid: "minecraft:lava",
        noise_router: NoiseRouterPreset::Nether,
        surface_rule: SurfaceRulePreset::Nether,
        spawn_target: &[],
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:end",
        noise: END_NOISE_SETTINGS,
        default_block: "minecraft:end_stone",
        default_fluid: "minecraft:air",
        noise_router: NoiseRouterPreset::End,
        surface_rule: SurfaceRulePreset::End,
        spawn_target: &[],
        sea_level: 0,
        disable_mob_generation: true,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:caves",
        noise: CAVES_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Caves,
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: true,
            surface: true,
        },
        spawn_target: &[],
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:floating_islands",
        noise: FLOATING_ISLANDS_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::FloatingIslands,
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: false,
            surface: false,
        },
        spawn_target: &[],
        sea_level: -64,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
];

pub const EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS: &[NoiseSettingsRegistryExpectation] = &[
    NoiseSettingsRegistryExpectation {
        id: "minecraft:amplified",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        router_id: "minecraft:amplified",
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target_len: 2,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:caves",
        noise: CAVES_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        router_id: "minecraft:caves",
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: true,
            surface: true,
        },
        spawn_target_len: 0,
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:end",
        noise: END_NOISE_SETTINGS,
        default_block: "minecraft:end_stone",
        default_fluid: "minecraft:air",
        router_id: "minecraft:end",
        surface_rule: SurfaceRulePreset::End,
        spawn_target_len: 0,
        sea_level: 0,
        disable_mob_generation: true,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:floating_islands",
        noise: FLOATING_ISLANDS_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        router_id: "minecraft:floating_islands",
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: false,
            surface: false,
        },
        spawn_target_len: 0,
        sea_level: -64,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:large_biomes",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        router_id: "minecraft:large_biomes",
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target_len: 2,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:nether",
        noise: NETHER_NOISE_SETTINGS,
        default_block: "minecraft:netherrack",
        default_fluid: "minecraft:lava",
        router_id: "minecraft:nether",
        surface_rule: SurfaceRulePreset::Nether,
        spawn_target_len: 0,
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:overworld",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        router_id: "minecraft:overworld",
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target_len: 2,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
];

pub const END_ISLANDS_DENSITY: DensityFunction = DensityFunction::EndIslands { seed: 0 };
pub const TEST_NEGATIVE_DENSITY: DensityFunction = DensityFunction::Constant(-2.0);
pub const TEST_POSITIVE_DENSITY: DensityFunction = DensityFunction::Constant(3.0);
pub const TEST_CACHE_ALL_IN_CELL_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::CacheAllInCell,
    input: &TEST_POSITIVE_DENSITY,
};
pub const TEST_RANGE_CHOICE_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &Y_DENSITY,
    min_inclusive: -1.0,
    max_exclusive: 1.0,
    when_in_range: &TEST_POSITIVE_DENSITY,
    when_out_of_range: &TEST_NEGATIVE_DENSITY,
};

impl NoiseSettings {
    pub const fn new(min_y: i32, height: i32, size_horizontal: i32, size_vertical: i32) -> Self {
        Self {
            min_y,
            height,
            size_horizontal,
            size_vertical,
        }
    }

    pub fn validate(self) -> Result<(), String> {
        if self.min_y + self.height > 2032 {
            return Err("min_y + height cannot be higher than: 2032".to_string());
        }
        if self.height % 16 != 0 {
            return Err("height has to be a multiple of 16".to_string());
        }
        if self.min_y % 16 != 0 {
            return Err("min_y has to be a multiple of 16".to_string());
        }
        if !(1..=4).contains(&self.size_horizontal) {
            return Err("size_horizontal must be in 1..=4".to_string());
        }
        if !(1..=4).contains(&self.size_vertical) {
            return Err("size_vertical must be in 1..=4".to_string());
        }
        Ok(())
    }

    pub fn cell_height(self) -> i32 {
        self.size_vertical * 4
    }

    pub fn cell_width(self) -> i32 {
        self.size_horizontal * 4
    }

    pub fn clamp_to_height(self, min_y: i32, max_y: i32) -> Self {
        let new_min_y = self.min_y.max(min_y);
        let new_height = (self.min_y + self.height).min(max_y + 1) - new_min_y;
        Self::new(
            new_min_y,
            new_height,
            self.size_horizontal,
            self.size_vertical,
        )
    }
}

pub fn builtin_density_function(id: &str) -> Option<&'static DensityFunctionEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_DENSITY_FUNCTIONS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn density_function_type(id: &str) -> Option<&'static DensityFunctionType> {
    DENSITY_FUNCTION_TYPES.iter().find(|kind| kind.id == id)
}

pub fn builtin_noise_router(id: &str) -> Option<&'static NoiseRouterEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_NOISE_ROUTERS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn noise_router_id_for_settings(settings: NoiseGeneratorSettings) -> &'static str {
    match settings.noise_router {
        NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: false,
        } => "minecraft:overworld",
        NoiseRouterPreset::Overworld {
            large_biomes: true,
            amplified: false,
        } => "minecraft:large_biomes",
        NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: true,
        } => "minecraft:amplified",
        NoiseRouterPreset::Overworld { .. } => "minecraft:overworld",
        NoiseRouterPreset::Nether => "minecraft:nether",
        NoiseRouterPreset::End => "minecraft:end",
        NoiseRouterPreset::Caves => "minecraft:caves",
        NoiseRouterPreset::FloatingIslands => "minecraft:floating_islands",
    }
}

/// Returns the Y value of the highest non-air block + 1 (or 0 if not present).

pub fn cave_generation_family(id: &str) -> Option<&'static CaveGenerationFamily> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    CAVE_GENERATION_FAMILIES.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}
