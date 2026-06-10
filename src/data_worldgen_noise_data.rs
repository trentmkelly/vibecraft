#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::worldgen::{
    builtin_normal_noise_parameters, NormalNoiseParameters, NORMAL_NOISE_PARAMETERS,
};

const NOISE_DATA_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/worldgen/NoiseData.java");

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "NoiseData.java is missing sentinel: {sentinel}"
        );
    }
}

fn parse_noise_json(id: &'static str) -> NormalNoiseParameters {
    let path = format!(
        "../decompiled-server-26.1.2/data/minecraft/worldgen/noise/{}.json",
        id.strip_prefix("minecraft:").unwrap()
    );
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read noise JSON {path}: {err}"));
    let value: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|err| panic!("invalid noise JSON {path}: {err}"));
    let first_octave = value
        .get("firstOctave")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_else(|| panic!("noise JSON {path} missing firstOctave"));
    let first_octave = i32::try_from(first_octave)
        .unwrap_or_else(|_| panic!("noise JSON {path} firstOctave overflows i32"));
    let amplitudes = value
        .get("amplitudes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("noise JSON {path} missing amplitudes"))
        .iter()
        .map(|value| {
            value
                .as_f64()
                .unwrap_or_else(|| panic!("noise JSON {path} has non-number amplitude"))
        })
        .collect::<Vec<_>>();
    let leaked_amplitudes: &'static [f64] = Box::leak(amplitudes.into_boxed_slice());

    NormalNoiseParameters {
        id,
        first_octave,
        amplitudes: leaked_amplitudes,
    }
}

fn vanilla_noise_json_ids() -> Vec<String> {
    fn visit(root: &std::path::Path, dir: &std::path::Path, ids: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir)
            .unwrap_or_else(|err| panic!("failed to read noise directory {dir:?}: {err}"))
        {
            let entry =
                entry.unwrap_or_else(|err| panic!("failed to read noise directory entry: {err}"));
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, ids);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .unwrap_or_else(|err| panic!("noise path {path:?} is outside root {root:?}: {err}"))
                .with_extension("");
            ids.push(format!(
                "minecraft:{}",
                relative.to_string_lossy().replace('\\', "/")
            ));
        }
    }

    let root = std::path::Path::new("../decompiled-server-26.1.2/data/minecraft/worldgen/noise");
    let mut ids = Vec::new();
    visit(root, root, &mut ids);
    ids.sort();
    ids
}

#[test]
fn noise_data_java_bootstrap_shape_matches_decompilation() {
    assert_eq!(NOISE_DATA_JAVA.lines().count(), 93);
    assert_source_contains_all(
        NOISE_DATA_JAVA,
        &[
            "public static final NormalNoise.NoiseParameters DEFAULT_SHIFT = new NormalNoise.NoiseParameters(-3, 1.0, 1.0, 1.0, 0.0);",
            "registerBiomeNoises(context, 0, Noises.TEMPERATURE, Noises.VEGETATION, Noises.CONTINENTALNESS, Noises.EROSION);",
            "registerBiomeNoises(context, -2, Noises.TEMPERATURE_LARGE, Noises.VEGETATION_LARGE, Noises.CONTINENTALNESS_LARGE, Noises.EROSION_LARGE);",
            "register(context, Noises.RIDGE, -7, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0);",
            "context.register(Noises.SHIFT, DEFAULT_SHIFT);",
            "register(context, Noises.CAVE_CHEESE, -8, 0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0);",
            "register(context, Noises.JAGGED, -16, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);",
            "register(context, Noises.SOUL_SAND_LAYER, -8, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334);",
            "register(context, Noises.NETHER_STATE_SELECTOR, -4, 1.0);",
            "private static void registerBiomeNoises(",
            "private static void register(",
        ],
    );

    assert_eq!(count_occurrences(NOISE_DATA_JAVA, "Noises."), 62);
    assert_eq!(count_occurrences(NOISE_DATA_JAVA, "context.register("), 2);
    assert_eq!(count_occurrences(NOISE_DATA_JAVA, "register(context,"), 57);
    assert_eq!(
        count_occurrences(NOISE_DATA_JAVA, "registerBiomeNoises("),
        3
    );
}

#[test]
fn rust_normal_noise_table_matches_java_registration_order_and_values() {
    assert_eq!(NORMAL_NOISE_PARAMETERS.len(), 62);
    assert_eq!(
        NORMAL_NOISE_PARAMETERS
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:temperature",
            "minecraft:vegetation",
            "minecraft:continentalness",
            "minecraft:erosion",
            "minecraft:temperature_large",
            "minecraft:vegetation_large",
            "minecraft:continentalness_large",
            "minecraft:erosion_large",
            "minecraft:nether/temperature",
            "minecraft:nether/vegetation",
            "minecraft:ridge",
            "minecraft:offset",
            "minecraft:aquifer_barrier",
            "minecraft:aquifer_fluid_level_floodedness",
            "minecraft:aquifer_lava",
            "minecraft:aquifer_fluid_level_spread",
            "minecraft:pillar",
            "minecraft:pillar_rareness",
            "minecraft:pillar_thickness",
            "minecraft:spaghetti_2d",
            "minecraft:spaghetti_2d_elevation",
            "minecraft:spaghetti_2d_modulator",
            "minecraft:spaghetti_2d_thickness",
            "minecraft:spaghetti_3d_1",
            "minecraft:spaghetti_3d_2",
            "minecraft:spaghetti_3d_rarity",
            "minecraft:spaghetti_3d_thickness",
            "minecraft:spaghetti_roughness",
            "minecraft:spaghetti_roughness_modulator",
            "minecraft:cave_entrance",
            "minecraft:cave_layer",
            "minecraft:cave_cheese",
            "minecraft:ore_veininess",
            "minecraft:ore_vein_a",
            "minecraft:ore_vein_b",
            "minecraft:ore_gap",
            "minecraft:noodle",
            "minecraft:noodle_thickness",
            "minecraft:noodle_ridge_a",
            "minecraft:noodle_ridge_b",
            "minecraft:jagged",
            "minecraft:surface",
            "minecraft:surface_secondary",
            "minecraft:clay_bands_offset",
            "minecraft:badlands_pillar",
            "minecraft:badlands_pillar_roof",
            "minecraft:badlands_surface",
            "minecraft:iceberg_pillar",
            "minecraft:iceberg_pillar_roof",
            "minecraft:iceberg_surface",
            "minecraft:surface_swamp",
            "minecraft:calcite",
            "minecraft:gravel",
            "minecraft:powder_snow",
            "minecraft:packed_ice",
            "minecraft:ice",
            "minecraft:soul_sand_layer",
            "minecraft:gravel_layer",
            "minecraft:patch",
            "minecraft:netherrack",
            "minecraft:nether_wart",
            "minecraft:nether_state_selector",
        ]
    );

    assert_eq!(
        *builtin_normal_noise_parameters("offset").unwrap(),
        NormalNoiseParameters {
            id: "minecraft:offset",
            first_octave: -3,
            amplitudes: &[1.0, 1.0, 1.0, 0.0],
        }
    );
    assert_eq!(
        *builtin_normal_noise_parameters("minecraft:cave_cheese").unwrap(),
        NormalNoiseParameters {
            id: "minecraft:cave_cheese",
            first_octave: -8,
            amplitudes: &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0],
        }
    );
    assert_eq!(
        *builtin_normal_noise_parameters("minecraft:soul_sand_layer").unwrap(),
        NormalNoiseParameters {
            id: "minecraft:soul_sand_layer",
            first_octave: -8,
            amplitudes: &[1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334],
        }
    );
}

#[test]
fn vanilla_noise_json_files_match_rust_normal_noise_parameters() {
    let rust_ids = NORMAL_NOISE_PARAMETERS
        .iter()
        .map(|entry| entry.id.to_string())
        .collect::<Vec<_>>();
    let mut sorted_rust_ids = rust_ids.clone();
    sorted_rust_ids.sort();
    assert_eq!(vanilla_noise_json_ids(), sorted_rust_ids);

    for parameters in NORMAL_NOISE_PARAMETERS {
        assert_eq!(parse_noise_json(parameters.id), *parameters);
    }
}
