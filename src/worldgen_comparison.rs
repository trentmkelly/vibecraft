#![allow(dead_code)]

use crate::seed_validation::{build_seed_parity_sample, ChunkCoord, SeedParitySample};
use crate::storage::chunk::{LevelChunk, ChunkSection};
use crate::storage::nbt::Tag;
use crate::worldgen::{
    blending_output_for_old_height, block_predicate_test, carver_is_start_chunk, configured_carver,
    density_function_type, height_provider_sample_with_rolls, normal_noise_value_factor,
    placement_modifier_positions, surface_condition_test, surface_rule_apply, BlockPos,
    BlockPredicate, BlockPredicateContext, CaveSurface, HeightProvider, PlacementModifier,
    SurfaceConditionSource, SurfaceMaterialContext, SurfaceRuleSource, VerticalAnchor,
    WorldGenerationHeightContext, BLOCK_PREDICATE_TYPES, BUILTIN_NOISE_GENERATOR_SETTINGS,
    BUILTIN_NOISE_ROUTERS, CONFIGURED_CARVERS, CONFIGURED_FEATURES, DENSITY_FUNCTION_TYPES,
    FLAT_GENERATOR_PRESETS, HEIGHT_PROVIDER_TYPES, NORMAL_NOISE_PARAMETERS,
    PLACED_FEATURE_BOOTSTRAP_SOURCES, STRUCTURE_FAMILIES, STRUCTURE_PIECE_TYPES,
    SURFACE_CONDITION_TYPES, SURFACE_RULE_TYPES, SYNTH_NOISE_SOURCES, WORLD_PRESETS,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenChunkComparison {
    pub seed: i64,
    pub chunk: ChunkCoord,
    pub fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenComparisonDiff {
    pub seed: i64,
    pub chunk: ChunkCoord,
    pub left_fingerprint: u64,
    pub right_fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenSourceFamilyGolden {
    pub family: &'static str,
    pub seed: i64,
    pub chunk: ChunkCoord,
    pub registry_items: usize,
    pub codec_items: usize,
    pub fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenChunkSignature {
    pub chunk: ChunkCoord,
    pub status: String,
    pub section_count: usize,
    pub non_empty_section_count: usize,
    pub heightmaps: Vec<WorldgenNamedArraySignature>,
    pub block_palette: Vec<String>,
    pub biome_palette: Vec<String>,
    pub sections: Vec<WorldgenSectionSignature>,
    pub structures: WorldgenStructureSignature,
    pub payload_fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenSectionSignature {
    pub y: i8,
    pub block_palette: Vec<String>,
    pub block_data_entries: usize,
    pub block_data_fingerprint: u64,
    pub biome_palette: Vec<String>,
    pub biome_data_entries: usize,
    pub biome_data_fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenNamedArraySignature {
    pub name: String,
    pub entries: usize,
    pub fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenStructureSignature {
    pub start_keys: Vec<String>,
    pub reference_keys: Vec<String>,
}

pub fn build_worldgen_chunk_comparisons(
    seeds: &[i64],
    chunks: &[ChunkCoord],
) -> Vec<WorldgenChunkComparison> {
    seeds
        .iter()
        .flat_map(|seed| {
            chunks.iter().map(move |chunk| {
                let sample = build_seed_parity_sample(*seed, chunk.x, chunk.z);
                WorldgenChunkComparison {
                    seed: *seed,
                    chunk: *chunk,
                    fingerprint: fingerprint_sample(&sample),
                }
            })
        })
        .collect()
}

pub fn build_worldgen_source_family_goldens(
    seed: i64,
    chunk: ChunkCoord,
) -> Vec<WorldgenSourceFamilyGolden> {
    let sample = build_seed_parity_sample(seed, chunk.x, chunk.z);
    let context = WorldGenerationHeightContext {
        min_y: -64,
        height: 384,
    };
    let block_context = BlockPredicateContext {
        min_y: -64,
        height: 384,
        block: "minecraft:grass_block",
        fluid: "minecraft:empty",
        solid: true,
        replaceable: false,
        unobstructed: true,
    };
    let placement_origin = BlockPos {
        x: chunk.x * 16,
        y: 72,
        z: chunk.z * 16,
    };
    let material_context = SurfaceMaterialContext {
        seed,
        random_algorithm: crate::worldgen::RandomAlgorithm::Xoroshiro,
        x: chunk.x * 16,
        y: 64,
        z: chunk.z * 16,
        biome: "minecraft:plains",
        stone_depth_above: 0,
        stone_depth_below: 4,
        surface_depth: 3,
        preliminary_surface_y: 68,
        water_height: 63,
        temperature: 0.8,
        noise: 0.25,
        steep: false,
        hole: false,
    };

    vec![
        source_family_golden(
            "blending",
            seed,
            chunk,
            1,
            1,
            format!(
                "{:?}",
                blending_output_for_old_height(Some(72.0), Some(2.0))
            ),
        ),
        source_family_golden(
            "block_predicate",
            seed,
            chunk,
            BLOCK_PREDICATE_TYPES.len(),
            BLOCK_PREDICATE_TYPES.len(),
            format!(
                "{:?}:{:?}",
                BLOCK_PREDICATE_TYPES,
                block_predicate_test(BlockPredicate::Solid, block_context, 64)
            ),
        ),
        source_family_golden(
            "carver",
            seed,
            chunk,
            CONFIGURED_CARVERS.len(),
            3,
            format!(
                "{:?}:{:?}",
                CONFIGURED_CARVERS,
                carver_is_start_chunk(configured_carver("cave").unwrap(), 0.15)
            ),
        ),
        source_family_golden(
            "feature",
            seed,
            chunk,
            CONFIGURED_FEATURES.len(),
            PLACED_FEATURE_BOOTSTRAP_SOURCES.len(),
            format!("{:?}", CONFIGURED_FEATURES),
        ),
        source_family_golden(
            "flat_generator",
            seed,
            chunk,
            FLAT_GENERATOR_PRESETS.len(),
            FLAT_GENERATOR_PRESETS.len(),
            format!("{:?}", FLAT_GENERATOR_PRESETS),
        ),
        source_family_golden(
            "height_provider",
            seed,
            chunk,
            HEIGHT_PROVIDER_TYPES.len(),
            HEIGHT_PROVIDER_TYPES.len(),
            format!(
                "{:?}:{:?}",
                HEIGHT_PROVIDER_TYPES,
                height_provider_sample_with_rolls(
                    HeightProvider::Trapezoid {
                        min_inclusive: VerticalAnchor::Absolute(40),
                        max_inclusive: VerticalAnchor::Absolute(80),
                        plateau: 8,
                    },
                    context,
                    4,
                    9,
                    0,
                )
            ),
        ),
        source_family_golden(
            "material_rule",
            seed,
            chunk,
            SURFACE_RULE_TYPES.len(),
            SURFACE_CONDITION_TYPES.len(),
            format!(
                "{:?}:{:?}",
                surface_condition_test(
                    &SurfaceConditionSource::StoneDepth {
                        offset: 1,
                        add_surface_depth: true,
                        secondary_depth_range: 0,
                        surface: CaveSurface::Floor,
                    },
                    &material_context,
                    &context,
                ),
                surface_rule_apply(
                    &SurfaceRuleSource::Block("minecraft:grass_block"),
                    &material_context,
                    &context,
                )
            ),
        ),
        source_family_golden(
            "placement_modifier",
            seed,
            chunk,
            PLACED_FEATURE_BOOTSTRAP_SOURCES.len(),
            9,
            format!(
                "{:?}",
                (
                    placement_modifier_positions(
                        PlacementModifier::Count { count: 2 },
                        placement_origin,
                        2,
                        0,
                        0,
                    ),
                    placement_modifier_positions(
                        PlacementModifier::InSquare,
                        placement_origin,
                        3,
                        5,
                        0,
                    ),
                    placement_modifier_positions(
                        PlacementModifier::RandomOffset {
                            xz_spread: 2,
                            y_spread: 1,
                        },
                        placement_origin,
                        3,
                        5,
                        7,
                    ),
                )
            ),
        ),
        source_family_golden(
            "preset",
            seed,
            chunk,
            WORLD_PRESETS.len()
                + BUILTIN_NOISE_GENERATOR_SETTINGS.len()
                + BUILTIN_NOISE_ROUTERS.len(),
            3,
            format!(
                "{:?}:{:?}:{:?}",
                WORLD_PRESETS, BUILTIN_NOISE_GENERATOR_SETTINGS, BUILTIN_NOISE_ROUTERS
            ),
        ),
        source_family_golden(
            "structure",
            seed,
            chunk,
            STRUCTURE_FAMILIES.len() + STRUCTURE_PIECE_TYPES.len(),
            STRUCTURE_PIECE_TYPES.len(),
            format!("{:?}:{:?}", STRUCTURE_FAMILIES, sample.structure_chunks),
        ),
        source_family_golden(
            "synth_noise",
            seed,
            chunk,
            NORMAL_NOISE_PARAMETERS.len(),
            SYNTH_NOISE_SOURCES.len() + DENSITY_FUNCTION_TYPES.len(),
            format!(
                "{:?}:{:?}:{:?}",
                SYNTH_NOISE_SOURCES,
                normal_noise_value_factor(NORMAL_NOISE_PARAMETERS[11]),
                density_function_type("old_blended_noise")
            ),
        ),
    ]
}

fn source_family_golden(
    family: &'static str,
    seed: i64,
    chunk: ChunkCoord,
    registry_items: usize,
    codec_items: usize,
    payload: String,
) -> WorldgenSourceFamilyGolden {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    mix_bytes(&mut hash, family.as_bytes());
    mix_i64(&mut hash, seed);
    mix_i32(&mut hash, chunk.x);
    mix_i32(&mut hash, chunk.z);
    mix_bytes(&mut hash, payload.as_bytes());
    WorldgenSourceFamilyGolden {
        family,
        seed,
        chunk,
        registry_items,
        codec_items,
        fingerprint: hash,
    }
}

pub fn diff_worldgen_comparisons(
    left: &[WorldgenChunkComparison],
    right: &[WorldgenChunkComparison],
) -> Vec<WorldgenComparisonDiff> {
    left.iter()
        .zip(right)
        .filter_map(|(left, right)| {
            if left.seed == right.seed
                && left.chunk == right.chunk
                && left.fingerprint != right.fingerprint
            {
                Some(WorldgenComparisonDiff {
                    seed: left.seed,
                    chunk: left.chunk,
                    left_fingerprint: left.fingerprint,
                    right_fingerprint: right.fingerprint,
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn build_chunk_signature(chunk: &LevelChunk) -> WorldgenChunkSignature {
    let sections = chunk
        .sections
        .iter()
        .map(section_signature)
        .collect::<Vec<_>>();
    let mut block_palette = sections
        .iter()
        .flat_map(|section| section.block_palette.clone())
        .collect::<Vec<_>>();
    block_palette.sort();
    block_palette.dedup();
    let mut biome_palette = sections
        .iter()
        .flat_map(|section| section.biome_palette.clone())
        .collect::<Vec<_>>();
    biome_palette.sort();
    biome_palette.dedup();
    let mut heightmaps = chunk
        .heightmaps
        .iter()
        .map(|(name, tag)| named_array_signature(name, tag))
        .collect::<Vec<_>>();
    heightmaps.sort_by(|left, right| left.name.cmp(&right.name));

    WorldgenChunkSignature {
        chunk: ChunkCoord {
            x: chunk.pos.x,
            z: chunk.pos.z,
        },
        status: chunk.status.clone(),
        section_count: chunk.sections.len(),
        non_empty_section_count: sections
            .iter()
            .filter(|section| !section.block_palette.is_empty())
            .count(),
        heightmaps,
        block_palette,
        biome_palette,
        structures: structure_signature(&chunk.structures),
        payload_fingerprint: tag_fingerprint(
            &chunk.to_nbt(crate::storage::datafix::TARGET_DATA_VERSION),
        ),
        sections,
    }
}

fn section_signature(section: &ChunkSection) -> WorldgenSectionSignature {
    let block_palette = palette_names(&section.block_states, true);
    let biome_palette = palette_names(&section.biomes, false);
    let block_data = container_data(&section.block_states);
    let biome_data = container_data(&section.biomes);
    WorldgenSectionSignature {
        y: section.y,
        block_palette,
        block_data_entries: block_data.len(),
        block_data_fingerprint: fingerprint_i64s(block_data),
        biome_palette,
        biome_data_entries: biome_data.len(),
        biome_data_fingerprint: fingerprint_i64s(biome_data),
    }
}

fn named_array_signature(name: &str, tag: &Tag) -> WorldgenNamedArraySignature {
    let values = match tag {
        Tag::LongArray(values) => values.as_slice(),
        _ => &[],
    };
    WorldgenNamedArraySignature {
        name: name.to_string(),
        entries: values.len(),
        fingerprint: fingerprint_i64s(values),
    }
}

fn structure_signature(tag: &Tag) -> WorldgenStructureSignature {
    let starts = compound_field(tag, "starts")
        .map(compound_keys)
        .unwrap_or_default();
    let references = compound_field(tag, "References")
        .or_else(|| compound_field(tag, "references"))
        .map(compound_keys)
        .unwrap_or_default();
    WorldgenStructureSignature {
        start_keys: starts,
        reference_keys: references,
    }
}

fn palette_names(container: &Tag, block_states: bool) -> Vec<String> {
    let mut names = compound_field(container, "palette")
        .and_then(|tag| match tag {
            Tag::List(values) => Some(values.as_slice()),
            _ => None,
        })
        .unwrap_or(&[])
        .iter()
        .filter_map(|tag| {
            if block_states {
                string_field(tag, "Name")
            } else if let Tag::String(value) = tag {
                Some(value.as_str())
            } else {
                None
            }
        })
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn container_data(container: &Tag) -> &[i64] {
    match compound_field(container, "data") {
        Some(Tag::LongArray(values)) => values.as_slice(),
        _ => &[],
    }
}

fn compound_field<'a>(tag: &'a Tag, name: &str) -> Option<&'a Tag> {
    match tag {
        Tag::Compound(fields) => fields
            .iter()
            .find(|(field_name, _)| field_name == name)
            .map(|(_, value)| value),
        _ => None,
    }
}

fn string_field<'a>(tag: &'a Tag, name: &str) -> Option<&'a str> {
    match compound_field(tag, name) {
        Some(Tag::String(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn compound_keys(tag: &Tag) -> Vec<String> {
    let mut keys = match tag {
        Tag::Compound(fields) => fields.iter().map(|(name, _)| name.clone()).collect(),
        _ => Vec::new(),
    };
    keys.sort();
    keys
}

fn tag_fingerprint(tag: &Tag) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    mix_tag(&mut hash, tag);
    hash
}

fn fingerprint_i64s(values: &[i64]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for value in values {
        mix_i64(&mut hash, *value);
    }
    hash
}

fn mix_tag(hash: &mut u64, tag: &Tag) {
    match tag {
        Tag::End => mix_bytes(hash, &[0]),
        Tag::Byte(value) => mix_bytes(hash, &[*value as u8]),
        Tag::Short(value) => mix_bytes(hash, &value.to_le_bytes()),
        Tag::Int(value) => mix_bytes(hash, &value.to_le_bytes()),
        Tag::Long(value) => mix_i64(hash, *value),
        Tag::Float(value) => mix_bytes(hash, &value.to_le_bytes()),
        Tag::Double(value) => mix_bytes(hash, &value.to_le_bytes()),
        Tag::ByteArray(values) => {
            for value in values {
                mix_bytes(hash, &[*value as u8]);
            }
        }
        Tag::String(value) => mix_bytes(hash, value.as_bytes()),
        Tag::List(values) => {
            for value in values {
                mix_tag(hash, value);
            }
        }
        Tag::Compound(fields) => {
            let sorted = fields
                .iter()
                .map(|(name, tag)| (name.as_str(), tag))
                .collect::<BTreeMap<_, _>>();
            for (name, tag) in sorted {
                mix_bytes(hash, name.as_bytes());
                mix_tag(hash, tag);
            }
        }
        Tag::IntArray(values) => {
            for value in values {
                mix_i32(hash, *value);
            }
        }
        Tag::LongArray(values) => {
            for value in values {
                mix_i64(hash, *value);
            }
        }
    }
}

fn fingerprint_sample(sample: &SeedParitySample) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    mix_i64(&mut hash, sample.seed);
    mix_i32(&mut hash, sample.chunk.x);
    mix_i32(&mut hash, sample.chunk.z);
    mix_i32(&mut hash, sample.biome_sample_block.0);
    mix_i32(&mut hash, sample.biome_sample_block.1);
    mix_i32(&mut hash, sample.biome_sample_block.2);
    mix_i32(&mut hash, sample.spawn_search_candidate.0);
    mix_i32(&mut hash, sample.spawn_search_candidate.1);
    mix_i64(&mut hash, sample.decoration_seed);
    mix_i64(&mut hash, sample.feature_seed);
    mix_i64(&mut hash, sample.slime_chunk_seed);
    mix_i64(&mut hash, sample.loot_sequence_seed.lo);
    mix_i64(&mut hash, sample.loot_sequence_seed.hi);
    for (id, chunk) in &sample.structure_chunks {
        mix_bytes(&mut hash, id.as_bytes());
        mix_i32(&mut hash, chunk.x);
        mix_i32(&mut hash, chunk.z);
    }
    hash
}

fn mix_i32(hash: &mut u64, value: i32) {
    mix_bytes(hash, &value.to_le_bytes());
}

fn mix_i64(hash: &mut u64, value: i64) {
    mix_bytes(hash, &value.to_le_bytes());
}

fn mix_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEEDS: &[i64] = &[0, 1, -1, 12_345, 8_675_309, i64::MIN + 1, i64::MAX];
    const CHUNKS: &[ChunkCoord] = &[
        ChunkCoord { x: 0, z: 0 },
        ChunkCoord { x: 1, z: -1 },
        ChunkCoord { x: -12, z: 34 },
        ChunkCoord { x: 1875, z: -1875 },
        ChunkCoord {
            x: i32::MIN / 1024,
            z: i32::MAX / 1024,
        },
    ];

    #[test]
    fn worldgen_chunk_comparison_matrix_is_deterministic_across_many_seeds_and_coordinates() {
        let first = build_worldgen_chunk_comparisons(SEEDS, CHUNKS);
        let second = build_worldgen_chunk_comparisons(SEEDS, CHUNKS);

        assert_eq!(first, second);
        assert_eq!(first.len(), SEEDS.len() * CHUNKS.len());
        assert!(first.iter().all(|entry| entry.fingerprint != 0));
    }

    #[test]
    fn worldgen_source_family_golden_matrix_is_deterministic() {
        let chunk = ChunkCoord { x: -12, z: 34 };
        let first = build_worldgen_source_family_goldens(8_675_309, chunk);
        let second = build_worldgen_source_family_goldens(8_675_309, chunk);

        assert_eq!(first, second);
        assert_eq!(
            first
                .iter()
                .map(|entry| (
                    entry.family,
                    entry.registry_items,
                    entry.codec_items,
                    entry.fingerprint
                ))
                .collect::<Vec<_>>(),
            vec![
                ("blending", 1, 1, 0x73ac_67d6_03ec_92f7),
                ("block_predicate", 13, 13, 0xc3ee_462f_0289_7bb6),
                ("carver", 4, 3, 0x4250_358c_87d5_b357),
                ("feature", 221, 9, 0x4406_a49b_7ed4_41f7),
                ("flat_generator", 9, 9, 0xd90a_d773_babb_cb17),
                ("height_provider", 6, 6, 0x7703_bfeb_40d0_6186),
                ("material_rule", 4, 11, 0x7094_bb91_9777_7c3a),
                ("placement_modifier", 9, 9, 0xb7be_3703_8b30_0f4f),
                ("preset", 21, 3, 0x950d_dea0_8cd8_0621),
                ("structure", 77, 56, 0x4ce0_499c_e60f_f4bb),
                ("synth_noise", 63, 40, 0xcb94_13d2_e01a_94cc),
            ]
        );
    }

    #[test]
    fn worldgen_chunk_comparison_fingerprints_change_with_seed_or_coordinate() {
        let baseline =
            build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 0, z: 0 }])[0].clone();
        let different_seed =
            build_worldgen_chunk_comparisons(&[54_321], &[ChunkCoord { x: 0, z: 0 }])[0].clone();
        let different_chunk =
            build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 2, z: 3 }])[0].clone();

        assert_ne!(baseline.fingerprint, different_seed.fingerprint);
        assert_ne!(baseline.fingerprint, different_chunk.fingerprint);
    }

    #[test]
    fn worldgen_comparison_diff_reports_matching_seed_chunk_fingerprint_changes() {
        let left = build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 0, z: 0 }]);
        let mut right = left.clone();
        right[0].fingerprint ^= 0xfeed;

        assert_eq!(
            diff_worldgen_comparisons(&left, &right),
            vec![WorldgenComparisonDiff {
                seed: 12_345,
                chunk: ChunkCoord { x: 0, z: 0 },
                left_fingerprint: left[0].fingerprint,
                right_fingerprint: right[0].fingerprint,
            }]
        );
    }

    #[test]
    fn worldgen_comparison_diff_ignores_mismatched_rows_so_matrices_fail_closed_elsewhere() {
        let left = build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 0, z: 0 }]);
        let right = build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 1, z: 0 }]);

        assert!(diff_worldgen_comparisons(&left, &right).is_empty());
    }

    #[test]
    fn chunk_signature_normalizes_generated_chunk_shape_for_vanilla_fixture_diffs() {
        let chunk = crate::worldgen::generate_overworld_chunk_for_preset(
            crate::storage::region::ChunkPos { x: 0, z: 0 },
            "flat",
        )
        .expect("flat preset should generate a concrete chunk");
        let signature = build_chunk_signature(&chunk);

        assert_eq!(signature.chunk, ChunkCoord { x: 0, z: 0 });
        assert_eq!(signature.status, "minecraft:full");
        assert_eq!(signature.section_count, 1);
        assert_eq!(signature.non_empty_section_count, 1);
        assert!(
            signature.heightmaps.iter().any(|heightmap| {
                (heightmap.name == "WORLD_SURFACE_WG" || heightmap.name == "WORLD_SURFACE")
                    && heightmap.entries > 0
            })
        );
        assert!(
            signature.heightmaps.iter().any(|heightmap| {
                (heightmap.name == "OCEAN_FLOOR_WG" || heightmap.name == "OCEAN_FLOOR")
                    && heightmap.entries > 0
            })
        );
        assert!(signature
            .block_palette
            .contains(&"minecraft:grass_block".to_string()));
        assert_eq!(signature.biome_palette, vec!["minecraft:plains".to_string()]);
        assert_ne!(signature.payload_fingerprint, 0);
    }

    #[test]
    fn chunk_signature_captures_structure_keys_and_section_data_fingerprints() {
        let mut chunk = crate::worldgen::generate_overworld_chunk_for_preset(
            crate::storage::region::ChunkPos { x: 2, z: -3 },
            "flat",
        )
        .expect("flat preset should generate a concrete chunk");
        chunk.structures = Tag::Compound(vec![
            (
                "starts".to_string(),
                Tag::Compound(vec![("minecraft:village".to_string(), Tag::Compound(vec![]))]),
            ),
            (
                "References".to_string(),
                Tag::Compound(vec![(
                    "minecraft:mineshaft".to_string(),
                    Tag::LongArray(vec![1, 2, 3]),
                )]),
            ),
        ]);
        let first = build_chunk_signature(&chunk);
        let second = build_chunk_signature(&chunk);

        assert_eq!(first, second);
        assert_eq!(
            first.structures.start_keys,
            vec!["minecraft:village".to_string()]
        );
        assert_eq!(
            first.structures.reference_keys,
            vec!["minecraft:mineshaft".to_string()]
        );
        assert!(first
            .sections
            .iter()
            .all(|section| section.block_data_fingerprint != 0));
    }
}
