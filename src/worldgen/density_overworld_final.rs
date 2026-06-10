use super::*;

// === Overworld final_density computation ===
// Mirrors Java NoiseRouterData.overworld() (non-amplified, non-large-biomes) inline build.
// Java source: NoiseRouterData.java, methods overworld(), underground(), postProcess(),
// slideOverworld(), and DensityFunctions.lerp(factor, first, second).

// Shared reference to the registered sloped_cheese function.
pub const OVERWORLD_SLOPED_CHEESE_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/sloped_cheese");

// Shared references to registered cave density functions.
pub const OVERWORLD_CAVES_ENTRANCES_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/caves/entrances");
pub const OVERWORLD_CAVES_SPAGHETTI_2D_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/caves/spaghetti_2d");
pub const OVERWORLD_CAVES_PILLARS_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/caves/pillars");
pub const OVERWORLD_CAVES_NOODLE_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/caves/noodle");

// --- underground() ---
// layerNoiseSource = noise(cave_layer, xzScale=1.0, yScale=8.0)
pub const OVERWORLD_UNDERGROUND_CAVE_LAYER_NOISE_DENSITY: DensityFunction =
    DensityFunction::Noise {
        noise: "minecraft:cave_layer",
        xz_scale: 1.0,
        y_scale: 8.0,
    };
// layerNoiseSource.square()
pub const OVERWORLD_UNDERGROUND_CAVE_LAYER_SQUARED_DENSITY: DensityFunction =
    DensityFunction::Mapped {
        kind: MappedDensityFunction::Square,
        input: &OVERWORLD_UNDERGROUND_CAVE_LAYER_NOISE_DENSITY,
    };
// 4.0 * layerNoiseSource.square()  → layerizedCavernsFunction
pub const OVERWORLD_UNDERGROUND_LAYER_CAVERNS_SCALE_DENSITY: DensityFunction =
    DensityFunction::Constant(4.0);
pub const OVERWORLD_UNDERGROUND_LAYERIZED_CAVERNS_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &OVERWORLD_UNDERGROUND_LAYER_CAVERNS_SCALE_DENSITY,
        argument2: &OVERWORLD_UNDERGROUND_CAVE_LAYER_SQUARED_DENSITY,
    };
// cheese = noise(cave_cheese, xzScale=1.0, yScale=2.0/3.0)
pub const OVERWORLD_UNDERGROUND_CAVE_CHEESE_NOISE_DENSITY: DensityFunction =
    DensityFunction::Noise {
        noise: "minecraft:cave_cheese",
        xz_scale: 1.0,
        y_scale: 0.6666666666666666,
    };
// clamp(0.27 + cheese, -1, 1)
pub const OVERWORLD_UNDERGROUND_CHEESE_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Constant(0.27);
pub const OVERWORLD_UNDERGROUND_CHEESE_WITH_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &OVERWORLD_UNDERGROUND_CHEESE_OFFSET_DENSITY,
        argument2: &OVERWORLD_UNDERGROUND_CAVE_CHEESE_NOISE_DENSITY,
    };
pub const OVERWORLD_UNDERGROUND_CHEESE_CLAMPED_DENSITY: DensityFunction = DensityFunction::Clamp {
    input: &OVERWORLD_UNDERGROUND_CHEESE_WITH_OFFSET_DENSITY,
    min: -1.0,
    max: 1.0,
};
// clamp(1.5 + (-0.64 * slopedCheese), 0, 0.5)
pub const OVERWORLD_UNDERGROUND_SLOPED_CHEESE_NEG_SCALE_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.64);
pub const OVERWORLD_UNDERGROUND_NEG_SLOPED_CHEESE_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &OVERWORLD_UNDERGROUND_SLOPED_CHEESE_NEG_SCALE_DENSITY,
        argument2: &OVERWORLD_SLOPED_CHEESE_REFERENCE_DENSITY,
    };
pub const OVERWORLD_UNDERGROUND_SLOPED_CHEESE_SHIFT_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Constant(1.5);
pub const OVERWORLD_UNDERGROUND_SLOPED_CHEESE_SHIFTED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &OVERWORLD_UNDERGROUND_SLOPED_CHEESE_SHIFT_OFFSET_DENSITY,
        argument2: &OVERWORLD_UNDERGROUND_NEG_SLOPED_CHEESE_DENSITY,
    };
pub const OVERWORLD_UNDERGROUND_SLOPED_CHEESE_CLAMPED_DENSITY: DensityFunction =
    DensityFunction::Clamp {
        input: &OVERWORLD_UNDERGROUND_SLOPED_CHEESE_SHIFTED_DENSITY,
        min: 0.0,
        max: 0.5,
    };
// solidifiedCheeseWithTopSlide = cheeseClamped + slopedCheeseClamped
pub const OVERWORLD_UNDERGROUND_SOLID_CHEESE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_UNDERGROUND_CHEESE_CLAMPED_DENSITY,
    argument2: &OVERWORLD_UNDERGROUND_SLOPED_CHEESE_CLAMPED_DENSITY,
};
// baseCaveDensity = layerizedCaverns + solidifiedCheese
pub const OVERWORLD_UNDERGROUND_BASE_CAVE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_UNDERGROUND_LAYERIZED_CAVERNS_DENSITY,
    argument2: &OVERWORLD_UNDERGROUND_SOLID_CHEESE_DENSITY,
};
// undergroundSubtractions = min(min(baseCave, entrances), spaghetti2D + spaghettiRoughness)
pub const OVERWORLD_UNDERGROUND_INNER_MIN_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Min,
    argument1: &OVERWORLD_UNDERGROUND_BASE_CAVE_DENSITY,
    argument2: &OVERWORLD_CAVES_ENTRANCES_REFERENCE_DENSITY,
};
pub const OVERWORLD_UNDERGROUND_SPAGHETTI_SUM_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_CAVES_SPAGHETTI_2D_REFERENCE_DENSITY,
    argument2: &ENTRANCES_ROUGHNESS_REFERENCE_DENSITY,
};
pub const OVERWORLD_UNDERGROUND_SUBTRACTIONS_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Min,
    argument1: &OVERWORLD_UNDERGROUND_INNER_MIN_DENSITY,
    argument2: &OVERWORLD_UNDERGROUND_SPAGHETTI_SUM_DENSITY,
};
// pillars = rangeChoice(pillarsRef, -1e6, 0.03, constant(-1e6), pillarsRef)
pub const OVERWORLD_UNDERGROUND_PILLARS_FILL_DENSITY: DensityFunction =
    DensityFunction::Constant(-1_000_000.0);
pub const OVERWORLD_UNDERGROUND_PILLARS_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &OVERWORLD_CAVES_PILLARS_REFERENCE_DENSITY,
    min_inclusive: -1_000_000.0,
    max_exclusive: 0.03,
    when_in_range: &OVERWORLD_UNDERGROUND_PILLARS_FILL_DENSITY,
    when_out_of_range: &OVERWORLD_CAVES_PILLARS_REFERENCE_DENSITY,
};
// underground = max(undergroundSubtractions, pillars)
pub const OVERWORLD_UNDERGROUND_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Max,
    argument1: &OVERWORLD_UNDERGROUND_SUBTRACTIONS_DENSITY,
    argument2: &OVERWORLD_UNDERGROUND_PILLARS_DENSITY,
};

// --- caves = rangeChoice(slopedCheese, -1e6, 1.5625, surfaceWithEntrances, underground) ---
// surfaceWithEntrances = min(slopedCheese, 5.0 * entrances)
pub const OVERWORLD_CAVES_SURFACE_ENTRANCES_SCALE_DENSITY: DensityFunction =
    DensityFunction::Constant(5.0);
pub const OVERWORLD_CAVES_SURFACE_ENTRANCES_SCALED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &OVERWORLD_CAVES_SURFACE_ENTRANCES_SCALE_DENSITY,
        argument2: &OVERWORLD_CAVES_ENTRANCES_REFERENCE_DENSITY,
    };
pub const OVERWORLD_SURFACE_WITH_ENTRANCES_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Min,
    argument1: &OVERWORLD_SLOPED_CHEESE_REFERENCE_DENSITY,
    argument2: &OVERWORLD_CAVES_SURFACE_ENTRANCES_SCALED_DENSITY,
};
pub const OVERWORLD_CAVES_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &OVERWORLD_SLOPED_CHEESE_REFERENCE_DENSITY,
    min_inclusive: -1_000_000.0,
    max_exclusive: 1.5625,
    when_in_range: &OVERWORLD_SURFACE_WITH_ENTRANCES_DENSITY,
    when_out_of_range: &OVERWORLD_UNDERGROUND_DENSITY,
};

// --- slideOverworld(amplified=false, caves) ---
// slide(caves, minY=-64, height=384, topStartY=80, topEndY=64, topTarget=-0.078125,
//       bottomStartY=0, bottomEndY=24, bottomTarget=0.1171875)
// topFactor    = yClampedGradient(minY+height-topStartY, minY+height-topEndY, 1.0, 0.0)
//              = yClampedGradient(240, 256, 1.0, 0.0)
// bottomFactor = yClampedGradient(minY+bottomStartY, minY+bottomEndY, 0.0, 1.0)
//              = yClampedGradient(-64, -40, 0.0, 1.0)
// lerp(factor, first, second) = add(mul(factor, add(second, constant(-first))), constant(first))
pub const OVERWORLD_SLIDE_TOP_GRADIENT_DENSITY: DensityFunction =
    DensityFunction::YClampedGradient {
        from_y: 240,
        to_y: 256,
        from_value: 1.0,
        to_value: 0.0,
    };
// lerp(topFactor, -0.078125, caves) = mul(topFactor, caves+0.078125) + (-0.078125)
pub const OVERWORLD_SLIDE_TOP_OFFSET_ADD_DENSITY: DensityFunction =
    DensityFunction::Constant(0.078125);
pub const OVERWORLD_SLIDE_TOP_CAVES_ADJUSTED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_CAVES_DENSITY,
    argument2: &OVERWORLD_SLIDE_TOP_OFFSET_ADD_DENSITY,
};
pub const OVERWORLD_SLIDE_TOP_MUL_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_SLIDE_TOP_GRADIENT_DENSITY,
    argument2: &OVERWORLD_SLIDE_TOP_CAVES_ADJUSTED_DENSITY,
};
pub const OVERWORLD_SLIDE_TOP_NEG_TARGET_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.078125);
pub const OVERWORLD_TOP_SLIDE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_SLIDE_TOP_MUL_DENSITY,
    argument2: &OVERWORLD_SLIDE_TOP_NEG_TARGET_DENSITY,
};
pub const OVERWORLD_SLIDE_BOTTOM_GRADIENT_DENSITY: DensityFunction =
    DensityFunction::YClampedGradient {
        from_y: -64,
        to_y: -40,
        from_value: 0.0,
        to_value: 1.0,
    };
// lerp(bottomFactor, 0.1171875, topSlide) = mul(bottomFactor, topSlide-0.1171875) + 0.1171875
pub const OVERWORLD_SLIDE_BOTTOM_NEG_TARGET_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.1171875);
pub const OVERWORLD_SLIDE_BOTTOM_TOP_ADJUSTED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_TOP_SLIDE_DENSITY,
    argument2: &OVERWORLD_SLIDE_BOTTOM_NEG_TARGET_DENSITY,
};
pub const OVERWORLD_SLIDE_BOTTOM_MUL_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_SLIDE_BOTTOM_GRADIENT_DENSITY,
    argument2: &OVERWORLD_SLIDE_BOTTOM_TOP_ADJUSTED_DENSITY,
};
pub const OVERWORLD_SLIDE_BOTTOM_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Constant(0.1171875);
pub const OVERWORLD_SLIDE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_SLIDE_BOTTOM_MUL_DENSITY,
    argument2: &OVERWORLD_SLIDE_BOTTOM_OFFSET_DENSITY,
};

// --- postProcess(slide) = Squeeze(Mul(Interpolated(BlendDensity(slide)), 0.64)) ---
pub const OVERWORLD_FINAL_BLEND_DENSITY: DensityFunction = DensityFunction::BlendDensity {
    input: &OVERWORLD_SLIDE_DENSITY,
};
pub const OVERWORLD_FINAL_INTERPOLATED_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Interpolated,
    input: &OVERWORLD_FINAL_BLEND_DENSITY,
};
pub const OVERWORLD_FINAL_MUL_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(0.64);
pub const OVERWORLD_FINAL_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_FINAL_INTERPOLATED_DENSITY,
    argument2: &OVERWORLD_FINAL_MUL_SCALE_DENSITY,
};
pub const OVERWORLD_FINAL_POST_PROCESS_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Squeeze,
    input: &OVERWORLD_FINAL_SCALED_DENSITY,
};

// fullNoise = min(postProcess(slideOverworld(caves)), noodle)
pub const OVERWORLD_FINAL_DENSITY_CONST: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Min,
    argument1: &OVERWORLD_FINAL_POST_PROCESS_DENSITY,
    argument2: &OVERWORLD_CAVES_NOODLE_REFERENCE_DENSITY,
};
