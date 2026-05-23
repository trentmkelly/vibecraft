use super::*;

pub const OVERWORLD_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    barrier: DensityFunction::Noise {
        noise: "minecraft:aquifer_barrier",
        xz_scale: 1.0,
        y_scale: 0.5,
    },
    fluid_level_floodedness: DensityFunction::Noise {
        noise: "minecraft:aquifer_fluid_level_floodedness",
        xz_scale: 1.0,
        y_scale: 0.67,
    },
    fluid_level_spread: DensityFunction::Noise {
        noise: "minecraft:aquifer_fluid_level_spread",
        xz_scale: 1.0,
        y_scale: 0.7142857142857143,
    },
    lava: DensityFunction::Noise {
        noise: "minecraft:aquifer_lava",
        xz_scale: 1.0,
        y_scale: 1.0,
    },
    temperature: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:temperature",
    },
    vegetation: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:vegetation",
    },
    continents: DensityFunction::Reference("minecraft:overworld/continents"),
    erosion: DensityFunction::Reference("minecraft:overworld/erosion"),
    depth: DensityFunction::Reference("minecraft:overworld/depth"),
    ridges: DensityFunction::Reference("minecraft:overworld/ridges"),
    preliminary_surface_level: OVERWORLD_PRELIMINARY_SURFACE_LEVEL_DENSITY,
    final_density: DensityFunction::Reference("minecraft:overworld/final_density"),
    vein_toggle: OVERWORLD_VEIN_TOGGLE_DENSITY,
    vein_ridged: OVERWORLD_VEIN_RIDGED_DENSITY,
    vein_gap: DensityFunction::Noise {
        noise: "minecraft:ore_gap",
        xz_scale: 1.0,
        y_scale: 1.0,
    },
};

pub const LARGE_BIOMES_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    temperature: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:temperature_large",
    },
    vegetation: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:vegetation_large",
    },
    continents: DensityFunction::Reference("minecraft:overworld_large_biomes/continents"),
    erosion: DensityFunction::Reference("minecraft:overworld_large_biomes/erosion"),
    depth: DensityFunction::Reference("minecraft:overworld_large_biomes/depth"),
    preliminary_surface_level: OVERWORLD_LARGE_BIOMES_PRELIMINARY_SURFACE_LEVEL_DENSITY,
    final_density: DensityFunction::Reference("minecraft:overworld_large_biomes/final_density"),
    ..OVERWORLD_NOISE_ROUTER
};

pub const AMPLIFIED_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    depth: DensityFunction::Reference("minecraft:overworld_amplified/depth"),
    preliminary_surface_level: OVERWORLD_AMPLIFIED_PRELIMINARY_SURFACE_LEVEL_DENSITY,
    final_density: DensityFunction::Reference("minecraft:overworld_amplified/final_density"),
    ..OVERWORLD_NOISE_ROUTER
};

pub const NETHER_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    temperature: DensityFunction::ShiftedNoise {
        shift_x: &ZERO_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &ZERO_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:nether/temperature",
    },
    vegetation: DensityFunction::ShiftedNoise {
        shift_x: &ZERO_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &ZERO_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:nether/vegetation",
    },
    final_density: DensityFunction::Reference("minecraft:nether/final_density"),
    ..NoiseRouter::simple(ZERO_DENSITY)
};

pub const CAVES_NOISE_ROUTER: NoiseRouter =
    NoiseRouter::simple(DensityFunction::Reference("minecraft:caves/final_density"));
pub const FLOATING_ISLANDS_NOISE_ROUTER: NoiseRouter = NoiseRouter::simple(
    DensityFunction::Reference("minecraft:floating_islands/final_density"),
);
pub const END_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    erosion: DensityFunction::Marker {
        kind: DensityMarker::Cache2D,
        input: &END_ISLANDS_DENSITY,
    },
    final_density: DensityFunction::Reference("minecraft:end/final_density"),
    ..NoiseRouter::simple(ZERO_DENSITY)
};
pub const NONE_NOISE_ROUTER: NoiseRouter = NoiseRouter::simple(ZERO_DENSITY);

pub const BUILTIN_NOISE_ROUTERS: &[NoiseRouterEntry] = &[
    NoiseRouterEntry {
        id: "minecraft:overworld",
        router: OVERWORLD_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:large_biomes",
        router: LARGE_BIOMES_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:amplified",
        router: AMPLIFIED_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:nether",
        router: NETHER_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:end",
        router: END_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:caves",
        router: CAVES_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:floating_islands",
        router: FLOATING_ISLANDS_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:none",
        router: NONE_NOISE_ROUTER,
    },
];

