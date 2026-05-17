#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiomeKey {
    pub id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateParameter {
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateTarget {
    pub temperature: i64,
    pub humidity: i64,
    pub continentalness: i64,
    pub erosion: i64,
    pub depth: i64,
    pub weirdness: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateParameterPoint {
    pub temperature: ClimateParameter,
    pub humidity: ClimateParameter,
    pub continentalness: ClimateParameter,
    pub erosion: ClimateParameter,
    pub depth: ClimateParameter,
    pub weirdness: ClimateParameter,
    pub offset: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateBiomeEntry {
    pub parameters: ClimateParameterPoint,
    pub biome: &'static str,
}

pub const QUANTIZATION_FACTOR: f32 = 10_000.0;

pub const BUILTIN_BIOMES: &[BiomeKey] = &[
    BiomeKey {
        id: "minecraft:the_void",
    },
    BiomeKey {
        id: "minecraft:plains",
    },
    BiomeKey {
        id: "minecraft:sunflower_plains",
    },
    BiomeKey {
        id: "minecraft:snowy_plains",
    },
    BiomeKey {
        id: "minecraft:ice_spikes",
    },
    BiomeKey {
        id: "minecraft:desert",
    },
    BiomeKey {
        id: "minecraft:swamp",
    },
    BiomeKey {
        id: "minecraft:mangrove_swamp",
    },
    BiomeKey {
        id: "minecraft:forest",
    },
    BiomeKey {
        id: "minecraft:flower_forest",
    },
    BiomeKey {
        id: "minecraft:birch_forest",
    },
    BiomeKey {
        id: "minecraft:dark_forest",
    },
    BiomeKey {
        id: "minecraft:pale_garden",
    },
    BiomeKey {
        id: "minecraft:old_growth_birch_forest",
    },
    BiomeKey {
        id: "minecraft:old_growth_pine_taiga",
    },
    BiomeKey {
        id: "minecraft:old_growth_spruce_taiga",
    },
    BiomeKey {
        id: "minecraft:taiga",
    },
    BiomeKey {
        id: "minecraft:snowy_taiga",
    },
    BiomeKey {
        id: "minecraft:savanna",
    },
    BiomeKey {
        id: "minecraft:savanna_plateau",
    },
    BiomeKey {
        id: "minecraft:windswept_hills",
    },
    BiomeKey {
        id: "minecraft:windswept_gravelly_hills",
    },
    BiomeKey {
        id: "minecraft:windswept_forest",
    },
    BiomeKey {
        id: "minecraft:windswept_savanna",
    },
    BiomeKey {
        id: "minecraft:jungle",
    },
    BiomeKey {
        id: "minecraft:sparse_jungle",
    },
    BiomeKey {
        id: "minecraft:bamboo_jungle",
    },
    BiomeKey {
        id: "minecraft:badlands",
    },
    BiomeKey {
        id: "minecraft:eroded_badlands",
    },
    BiomeKey {
        id: "minecraft:wooded_badlands",
    },
    BiomeKey {
        id: "minecraft:meadow",
    },
    BiomeKey {
        id: "minecraft:cherry_grove",
    },
    BiomeKey {
        id: "minecraft:grove",
    },
    BiomeKey {
        id: "minecraft:snowy_slopes",
    },
    BiomeKey {
        id: "minecraft:frozen_peaks",
    },
    BiomeKey {
        id: "minecraft:jagged_peaks",
    },
    BiomeKey {
        id: "minecraft:stony_peaks",
    },
    BiomeKey {
        id: "minecraft:river",
    },
    BiomeKey {
        id: "minecraft:frozen_river",
    },
    BiomeKey {
        id: "minecraft:beach",
    },
    BiomeKey {
        id: "minecraft:snowy_beach",
    },
    BiomeKey {
        id: "minecraft:stony_shore",
    },
    BiomeKey {
        id: "minecraft:warm_ocean",
    },
    BiomeKey {
        id: "minecraft:lukewarm_ocean",
    },
    BiomeKey {
        id: "minecraft:deep_lukewarm_ocean",
    },
    BiomeKey {
        id: "minecraft:ocean",
    },
    BiomeKey {
        id: "minecraft:deep_ocean",
    },
    BiomeKey {
        id: "minecraft:cold_ocean",
    },
    BiomeKey {
        id: "minecraft:deep_cold_ocean",
    },
    BiomeKey {
        id: "minecraft:frozen_ocean",
    },
    BiomeKey {
        id: "minecraft:deep_frozen_ocean",
    },
    BiomeKey {
        id: "minecraft:mushroom_fields",
    },
    BiomeKey {
        id: "minecraft:dripstone_caves",
    },
    BiomeKey {
        id: "minecraft:lush_caves",
    },
    BiomeKey {
        id: "minecraft:deep_dark",
    },
    BiomeKey {
        id: "minecraft:nether_wastes",
    },
    BiomeKey {
        id: "minecraft:warped_forest",
    },
    BiomeKey {
        id: "minecraft:crimson_forest",
    },
    BiomeKey {
        id: "minecraft:soul_sand_valley",
    },
    BiomeKey {
        id: "minecraft:basalt_deltas",
    },
    BiomeKey {
        id: "minecraft:the_end",
    },
    BiomeKey {
        id: "minecraft:end_highlands",
    },
    BiomeKey {
        id: "minecraft:end_midlands",
    },
    BiomeKey {
        id: "minecraft:small_end_islands",
    },
    BiomeKey {
        id: "minecraft:end_barrens",
    },
];

pub const NETHER_BIOME_PARAMETERS: &[ClimateBiomeEntry] = &[
    ClimateBiomeEntry {
        parameters: climate_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        biome: "minecraft:nether_wastes",
    },
    ClimateBiomeEntry {
        parameters: climate_point(0.0, -0.5, 0.0, 0.0, 0.0, 0.0, 0.0),
        biome: "minecraft:soul_sand_valley",
    },
    ClimateBiomeEntry {
        parameters: climate_point(0.4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        biome: "minecraft:crimson_forest",
    },
    ClimateBiomeEntry {
        parameters: climate_point(0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.375),
        biome: "minecraft:warped_forest",
    },
    ClimateBiomeEntry {
        parameters: climate_point(-0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.175),
        biome: "minecraft:basalt_deltas",
    },
];

pub const fn quantize_coord(coord: f32) -> i64 {
    (coord * QUANTIZATION_FACTOR) as i64
}

pub fn unquantize_coord(coord: i64) -> f32 {
    coord as f32 / QUANTIZATION_FACTOR
}

pub const fn climate_target(
    temperature: f32,
    humidity: f32,
    continentalness: f32,
    erosion: f32,
    depth: f32,
    weirdness: f32,
) -> ClimateTarget {
    ClimateTarget {
        temperature: quantize_coord(temperature),
        humidity: quantize_coord(humidity),
        continentalness: quantize_coord(continentalness),
        erosion: quantize_coord(erosion),
        depth: quantize_coord(depth),
        weirdness: quantize_coord(weirdness),
    }
}

pub const fn point(value: f32) -> ClimateParameter {
    span(value, value)
}

pub const fn span(min: f32, max: f32) -> ClimateParameter {
    ClimateParameter {
        min: quantize_coord(min),
        max: quantize_coord(max),
    }
}

pub const fn climate_point(
    temperature: f32,
    humidity: f32,
    continentalness: f32,
    erosion: f32,
    depth: f32,
    weirdness: f32,
    offset: f32,
) -> ClimateParameterPoint {
    ClimateParameterPoint {
        temperature: point(temperature),
        humidity: point(humidity),
        continentalness: point(continentalness),
        erosion: point(erosion),
        depth: point(depth),
        weirdness: point(weirdness),
        offset: quantize_coord(offset),
    }
}

impl ClimateParameter {
    pub fn distance(self, target: i64) -> i64 {
        let above = target - self.max;
        let below = self.min - target;
        if above > 0 {
            above
        } else {
            below.max(0)
        }
    }
}

impl ClimateParameterPoint {
    pub fn fitness(self, target: ClimateTarget) -> i64 {
        square(self.temperature.distance(target.temperature))
            + square(self.humidity.distance(target.humidity))
            + square(self.continentalness.distance(target.continentalness))
            + square(self.erosion.distance(target.erosion))
            + square(self.depth.distance(target.depth))
            + square(self.weirdness.distance(target.weirdness))
            + square(self.offset)
    }
}

pub fn select_climate_biome(
    entries: &[ClimateBiomeEntry],
    target: ClimateTarget,
) -> Option<&'static str> {
    entries
        .iter()
        .min_by_key(|entry| entry.parameters.fitness(target))
        .map(|entry| entry.biome)
}

pub fn builtin_biome(id: &str) -> Option<&'static BiomeKey> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_BIOMES.iter().find(|biome| {
        biome
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|biome_name| biome_name == name)
    })
}

const fn square(value: i64) -> i64 {
    value * value
}

#[cfg(test)]
mod tests {
    use super::{
        builtin_biome, climate_point, climate_target, quantize_coord, select_climate_biome,
        unquantize_coord, ClimateBiomeEntry, BUILTIN_BIOMES, NETHER_BIOME_PARAMETERS,
    };

    #[test]
    fn builtin_biome_registry_keys_match_26_1_2_biomes_order() {
        assert_eq!(BUILTIN_BIOMES.len(), 65);
        assert_eq!(BUILTIN_BIOMES.first().unwrap().id, "minecraft:the_void");
        assert_eq!(BUILTIN_BIOMES[1].id, "minecraft:plains");
        assert_eq!(BUILTIN_BIOMES[12].id, "minecraft:pale_garden");
        assert_eq!(BUILTIN_BIOMES[55].id, "minecraft:nether_wastes");
        assert_eq!(BUILTIN_BIOMES.last().unwrap().id, "minecraft:end_barrens");
        assert_eq!(builtin_biome("plains").unwrap().id, "minecraft:plains");
        assert!(builtin_biome("missing").is_none());
    }

    #[test]
    fn climate_quantization_and_parameter_distance_match_vanilla_rules() {
        assert_eq!(quantize_coord(0.375), 3750);
        assert_eq!(quantize_coord(-0.5), -5000);
        assert_eq!(unquantize_coord(3750), 0.375);

        let point = climate_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.175);
        assert_eq!(point.offset, 1750);
        assert_eq!(point.temperature.distance(quantize_coord(0.4)), 4000);
        assert_eq!(point.temperature.distance(quantize_coord(0.0)), 0);
    }

    #[test]
    fn nether_multi_noise_parameters_select_nearest_biome() {
        assert_eq!(NETHER_BIOME_PARAMETERS.len(), 5);
        assert_eq!(
            select_climate_biome(
                NETHER_BIOME_PARAMETERS,
                climate_target(0.4, 0.0, 0.0, 0.0, 0.0, 0.0)
            ),
            Some("minecraft:crimson_forest")
        );
        assert_eq!(
            select_climate_biome(
                NETHER_BIOME_PARAMETERS,
                climate_target(0.0, 0.5, 0.0, 0.0, 0.0, 0.0)
            ),
            Some("minecraft:warped_forest")
        );
        assert_eq!(
            select_climate_biome(
                NETHER_BIOME_PARAMETERS,
                climate_target(-0.5, 0.0, 0.0, 0.0, 0.0, 0.0)
            ),
            Some("minecraft:basalt_deltas")
        );
    }

    #[test]
    fn climate_selection_uses_first_entry_when_fitness_ties() {
        let entries = [
            ClimateBiomeEntry {
                parameters: climate_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                biome: "minecraft:plains",
            },
            ClimateBiomeEntry {
                parameters: climate_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                biome: "minecraft:forest",
            },
        ];
        assert_eq!(
            select_climate_biome(&entries, climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)),
            Some("minecraft:plains")
        );
    }
}
