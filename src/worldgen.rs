#![allow(dead_code)]

use crate::biome::{span, ClimateParameterPoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoiseSettings {
    pub min_y: i32,
    pub height: i32,
    pub size_horizontal: i32,
    pub size_vertical: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoiseGeneratorSettings {
    pub id: &'static str,
    pub noise: NoiseSettings,
    pub default_block: &'static str,
    pub default_fluid: &'static str,
    pub noise_router: NoiseRouterPreset,
    pub surface_rule: SurfaceRulePreset,
    pub spawn_target: &'static [ClimateParameterPoint],
    pub sea_level: i32,
    pub disable_mob_generation: bool,
    pub aquifers_enabled: bool,
    pub ore_veins_enabled: bool,
    pub legacy_random_source: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseRouterPreset {
    Overworld { large_biomes: bool, amplified: bool },
    Nether,
    End,
    Caves,
    FloatingIslands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceRulePreset {
    Overworld,
    Nether,
    End,
    OverworldLike {
        bedrock_roof: bool,
        bedrock_floor: bool,
        surface: bool,
    },
}

pub const OVERWORLD_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(-64, 384, 1, 2);
pub const NETHER_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 128, 1, 2);
pub const END_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 128, 2, 1);
pub const CAVES_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(-64, 192, 1, 2);
pub const FLOATING_ISLANDS_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 256, 2, 1);

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

pub fn builtin_noise_generator_settings(id: &str) -> Option<&'static NoiseGeneratorSettings> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_NOISE_GENERATOR_SETTINGS.iter().find(|settings| {
        settings
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|settings_name| settings_name == name)
    })
}

#[cfg(test)]
mod tests {
    use super::{
        builtin_noise_generator_settings, NoiseRouterPreset, NoiseSettings, SurfaceRulePreset,
        BUILTIN_NOISE_GENERATOR_SETTINGS, CAVES_NOISE_SETTINGS, END_NOISE_SETTINGS,
        FLOATING_ISLANDS_NOISE_SETTINGS, NETHER_NOISE_SETTINGS, OVERWORLD_NOISE_SETTINGS,
        OVERWORLD_SPAWN_TARGET,
    };
    use crate::biome::quantize_coord;

    #[test]
    fn noise_settings_presets_match_26_1_2_constants() {
        assert_eq!(OVERWORLD_NOISE_SETTINGS, NoiseSettings::new(-64, 384, 1, 2));
        assert_eq!(NETHER_NOISE_SETTINGS, NoiseSettings::new(0, 128, 1, 2));
        assert_eq!(END_NOISE_SETTINGS, NoiseSettings::new(0, 128, 2, 1));
        assert_eq!(CAVES_NOISE_SETTINGS, NoiseSettings::new(-64, 192, 1, 2));
        assert_eq!(
            FLOATING_ISLANDS_NOISE_SETTINGS,
            NoiseSettings::new(0, 256, 2, 1)
        );
        assert_eq!(OVERWORLD_NOISE_SETTINGS.cell_width(), 4);
        assert_eq!(OVERWORLD_NOISE_SETTINGS.cell_height(), 8);
    }

    #[test]
    fn noise_settings_validation_and_clamp_follow_vanilla_rules() {
        assert!(NoiseSettings::new(-64, 384, 1, 2).validate().is_ok());
        assert_eq!(
            NoiseSettings::new(-63, 384, 1, 2).validate().unwrap_err(),
            "min_y has to be a multiple of 16"
        );
        assert_eq!(
            NoiseSettings::new(-64, 383, 1, 2).validate().unwrap_err(),
            "height has to be a multiple of 16"
        );
        assert_eq!(
            NoiseSettings::new(0, 2033, 1, 2).validate().unwrap_err(),
            "min_y + height cannot be higher than: 2032"
        );
        assert_eq!(
            NoiseSettings::new(-64, 384, 1, 2).clamp_to_height(0, 255),
            NoiseSettings::new(0, 256, 1, 2)
        );
    }

    #[test]
    fn noise_generator_settings_bootstrap_matches_vanilla_order_and_flags() {
        assert_eq!(
            BUILTIN_NOISE_GENERATOR_SETTINGS
                .iter()
                .map(|settings| settings.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld",
                "minecraft:large_biomes",
                "minecraft:amplified",
                "minecraft:nether",
                "minecraft:end",
                "minecraft:caves",
                "minecraft:floating_islands",
            ]
        );

        let overworld = builtin_noise_generator_settings("overworld").unwrap();
        assert_eq!(overworld.default_block, "minecraft:stone");
        assert_eq!(overworld.default_fluid, "minecraft:water");
        assert_eq!(
            overworld.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: false,
                amplified: false
            }
        );
        assert_eq!(overworld.surface_rule, SurfaceRulePreset::Overworld);
        assert_eq!(overworld.sea_level, 63);
        assert!(overworld.aquifers_enabled);
        assert!(overworld.ore_veins_enabled);
        assert!(!overworld.legacy_random_source);

        let large = builtin_noise_generator_settings("large_biomes").unwrap();
        assert_eq!(
            large.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: true,
                amplified: false
            }
        );
        let amplified = builtin_noise_generator_settings("amplified").unwrap();
        assert_eq!(
            amplified.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: false,
                amplified: true
            }
        );

        let nether = builtin_noise_generator_settings("nether").unwrap();
        assert_eq!(nether.default_block, "minecraft:netherrack");
        assert_eq!(nether.default_fluid, "minecraft:lava");
        assert_eq!(nether.sea_level, 32);
        assert!(nether.legacy_random_source);

        let end = builtin_noise_generator_settings("end").unwrap();
        assert_eq!(end.default_block, "minecraft:end_stone");
        assert_eq!(end.default_fluid, "minecraft:air");
        assert!(end.disable_mob_generation);
        assert_eq!(end.sea_level, 0);
    }

    #[test]
    fn overworld_spawn_target_matches_overworld_biome_builder() {
        assert_eq!(OVERWORLD_SPAWN_TARGET.len(), 2);
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].continentalness.min,
            quantize_coord(-0.11)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].continentalness.max,
            quantize_coord(1.0)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].weirdness.min,
            quantize_coord(-1.0)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].weirdness.max,
            quantize_coord(-0.16)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[1].weirdness.min,
            quantize_coord(0.16)
        );
        assert_eq!(OVERWORLD_SPAWN_TARGET[1].weirdness.max, quantize_coord(1.0));
    }
}
