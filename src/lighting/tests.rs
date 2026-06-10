//! Lighting engine test suite (F1–F19 in
//! `RustCraft/CHECKLIST_LIGHTING.md`). Each test names the checklist item it
//! satisfies and exercises one well-defined invariant; the engines are
//! covered end-to-end through the public `LevelLightEngine` API and the
//! `compute_chunk_lighting` chunk-pipeline entry point.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::lighting::block_light_properties::light_properties_for;
use crate::lighting::chunk_sky_light_sources::ChunkSkyLightSources;
use crate::lighting::data_layer::{DataLayer, MAX_LIGHT_LEVEL, SECTION_VOLUME};
use crate::lighting::direction::Direction;
use crate::lighting::level_height::LevelHeightAccessor;
use crate::lighting::level_light_engine::LevelLightEngine;
use crate::lighting::light_chunk::{
    BlockLightSourceConsumer, LightBlockProperties, LightChunkGetter,
};
use crate::lighting::light_engine::has_different_light_properties;
use crate::lighting::light_layer::LightLayer;
use crate::lighting::positions::{
    block_pos_as_long, block_pos_x, block_pos_y, block_pos_z, block_to_section,
    section_pos_as_long, section_pos_x, section_pos_y, section_pos_z,
};
use crate::lighting::queue_entry::{
    decrease_all_directions, decrease_skip_one_direction, get_from_level,
    increase_light_from_emission, increase_only_one_direction, increase_skip_one_direction,
    increase_sky_source_in_directions, is_from_empty_shape, is_increase_from_emission,
    should_propagate_in_direction,
};

// ---- F1. DataLayer nibble parity ----
#[test]
fn f1_data_layer_nibble_layout_matches_java() {
    let mut layer = DataLayer::empty();
    layer.set(0, 0, 0, 3);
    layer.set(1, 0, 0, 12);
    layer.set(0, 1, 0, 5);
    layer.set(15, 15, 15, 14);

    let bytes = layer.clone().into_bytes();
    assert_eq!(bytes.len(), 2048);
    // (y=0,z=0): byte 0, low nibble = x=0, high nibble = x=1 -> 0x3 | (0xC << 4) = 0xC3
    assert_eq!(bytes[0], 0xC3);
    // (y=1,z=0,x=0) -> index 256, byte 128, low nibble = 5
    assert_eq!(bytes[128] & 0x0F, 5);
    // (y=15,z=15,x=15) -> top half of last byte
    assert_eq!(bytes[2047] >> 4, 14);

    // is_definitely_homogenous flips false after a write.
    assert!(!layer.is_definitely_homogenous());

    // Fresh default-15 layer round-trips to 0xFF nibbles without writes.
    let homogenous = DataLayer::with_default(15);
    assert!(homogenous.is_definitely_filled_with(15));
    let bytes = homogenous.into_bytes();
    assert!(bytes.iter().all(|b| *b == 0xFF));
}

#[test]
fn f1_data_layer_lazy_default_is_empty() {
    let layer = DataLayer::empty();
    assert!(layer.is_empty());
    assert!(layer.is_definitely_homogenous());
    assert_eq!(layer.get(5, 6, 7), 0);
}

// ---- F2. SectionPos packing ----
#[test]
fn f2_section_pos_round_trips_through_java_layout() {
    for (x, y, z) in [
        (0, 0, 0),
        (1, 2, 3),
        (-1, -2, -3),
        (1_000_000, -64, -1_000_000),
        (-2_000_000, 319, 2_000_000),
    ] {
        let packed = section_pos_as_long(x, y, z);
        assert_eq!(section_pos_x(packed), x, "x parity for ({x},{y},{z})");
        assert_eq!(section_pos_y(packed), y, "y parity for ({x},{y},{z})");
        assert_eq!(section_pos_z(packed), z, "z parity for ({x},{y},{z})");
    }

    // block_to_section uses arithmetic shift.
    let node = block_pos_as_long(33, -65, -16);
    let section = block_to_section(node);
    assert_eq!(section_pos_x(section), 2);
    assert_eq!(section_pos_y(section), -5);
    assert_eq!(section_pos_z(section), -1);
}

// ---- F3. QueueEntry bitpacking ----
#[test]
fn f3_queue_entry_round_trips_for_every_constructor() {
    let entry = increase_light_from_emission(14, true);
    assert_eq!(get_from_level(entry), 14);
    assert!(is_from_empty_shape(entry));
    assert!(is_increase_from_emission(entry));
    // All six directions propagate.
    for d in [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ] {
        assert!(should_propagate_in_direction(entry, d));
    }

    let entry = decrease_all_directions(12);
    assert_eq!(get_from_level(entry), 12);
    assert!(!is_increase_from_emission(entry));

    let entry = decrease_skip_one_direction(7, Direction::Up);
    assert_eq!(get_from_level(entry), 7);
    assert!(!should_propagate_in_direction(entry, Direction::Up));
    assert!(should_propagate_in_direction(entry, Direction::Down));

    let entry = increase_skip_one_direction(5, false, Direction::North);
    assert_eq!(get_from_level(entry), 5);
    assert!(!should_propagate_in_direction(entry, Direction::North));

    let entry = increase_only_one_direction(3, false, Direction::East);
    assert_eq!(get_from_level(entry), 3);
    assert!(should_propagate_in_direction(entry, Direction::East));
    for d in [
        Direction::Down,
        Direction::Up,
        Direction::West,
        Direction::South,
        Direction::North,
    ] {
        assert!(!should_propagate_in_direction(entry, d));
    }

    let entry = increase_sky_source_in_directions(true, false, true, false, true);
    assert_eq!(get_from_level(entry), 15);
    assert!(should_propagate_in_direction(entry, Direction::Down));
    assert!(!should_propagate_in_direction(entry, Direction::North));
    assert!(should_propagate_in_direction(entry, Direction::South));
    assert!(!should_propagate_in_direction(entry, Direction::West));
    assert!(should_propagate_in_direction(entry, Direction::East));
}

// ---- F4. Section-type lifecycle ----
#[test]
fn f4_section_state_transitions_when_enabling_disabling() {
    use crate::lighting::storage::{LayerLightSectionStorage, SectionType};
    let mut storage = LayerLightSectionStorage::new_block();
    let section_node = section_pos_as_long(0, 0, 0);
    assert_eq!(
        storage.get_debug_section_type(section_node),
        SectionType::Empty
    );

    storage.update_section_status(section_node, false);
    assert_eq!(
        storage.get_debug_section_type(section_node),
        SectionType::LightAndData
    );

    storage.update_section_status(section_node, true);
    // section is empty but the neighbour count is non-zero (its own 3x3x3
    // includes itself), so its state should fall back to LIGHT_ONLY only if
    // a neighbour is still non-empty. With no neighbours, state returns to
    // EMPTY.
    assert_eq!(
        storage.get_debug_section_type(section_node),
        SectionType::Empty
    );

    // `setLightEnabled(zero, true)` adds the column to the source set.
    let zero = section_pos_as_long(0, 0, 0) & !((1i64 << 20) - 1);
    storage.set_light_enabled(zero, true);
    assert!(storage.light_on_in_column(zero));
}

// ---- F12 (companion to F4). runLightUpdates accounting ----
#[test]
fn f12_run_light_updates_drains_queues_and_counts_nodes() {
    let chunks = single_chunk_with(|chunk| {
        // Place a torch at (0, 1, 0) inside chunk (0, 0).
        chunk.insert(
            (0, 1, 0),
            LightBlockProperties {
                opacity: 0,
                emission: 14,
                uses_shape_for_light_occlusion: false,
                occlusion_shape_occludes_full_face: false,
            },
        );
    });
    let getter = SimpleChunkGetter::new(level_height_overworld_test(), chunks);
    let mut engine = LevelLightEngine::new(level_height_overworld_test(), true, true);
    engine.update_section_status(0, 0, 0, false);
    engine.set_light_enabled(&getter, 0, 0, true);
    engine.propagate_light_sources(&getter, 0, 0);
    let count = engine.run_light_updates(&getter);
    assert!(
        count > 0,
        "engine should have drained at least one queued entry"
    );
    assert!(!engine.has_light_work());
    assert!(engine.block_engine.as_ref().unwrap().last_run_node_count() > 0);
}

// ---- F5. Block-light single-section propagation ----
#[test]
fn f5_block_light_propagates_from_emission_with_min_opacity() {
    let chunks = single_chunk_with(|chunk| {
        // Torch at (8, 1, 8) in chunk (0, 0).
        chunk.insert(
            (8, 1, 8),
            LightBlockProperties {
                opacity: 0,
                emission: 14,
                uses_shape_for_light_occlusion: false,
                occlusion_shape_occludes_full_face: false,
            },
        );
    });
    let getter = SimpleChunkGetter::new(level_height_overworld_test(), chunks);
    let mut engine = LevelLightEngine::new(level_height_overworld_test(), true, false);
    for sy in level_height_overworld_test().min_section_y()
        ..=level_height_overworld_test().max_section_y()
    {
        engine.update_section_status(0, sy, 0, false);
    }
    engine.set_light_enabled(&getter, 0, 0, true);
    engine.propagate_light_sources(&getter, 0, 0);
    engine.run_light_updates(&getter);

    // At source: level 14 (block doesn't dampen itself in our model — Java
    // sets level to `emission` directly via `setStoredLevel` during the
    // increaseFromEmission step).
    assert_eq!(get_block_light_at(&engine, 8, 1, 8), 14);
    assert_eq!(get_block_light_at(&engine, 9, 1, 8), 13);
    assert_eq!(get_block_light_at(&engine, 10, 1, 8), 12);
    // 14 blocks away the level reaches zero.
    assert_eq!(get_block_light_at(&engine, 8 + 14, 1, 8), 0);
}

// ---- F6. Block-light across section boundary ----
#[test]
fn f6_block_light_propagates_across_section_boundary() {
    let chunks = single_chunk_with(|chunk| {
        // Torch at (0, 15, 0) — top voxel of section y=0. Light should bleed
        // into y=16 (section y=1) at level 13.
        chunk.insert(
            (0, 15, 0),
            LightBlockProperties {
                opacity: 0,
                emission: 14,
                uses_shape_for_light_occlusion: false,
                occlusion_shape_occludes_full_face: false,
            },
        );
    });
    let getter = SimpleChunkGetter::new(level_height_overworld_test(), chunks);
    let mut engine = LevelLightEngine::new(level_height_overworld_test(), true, false);
    for sy in level_height_overworld_test().min_section_y()
        ..=level_height_overworld_test().max_section_y()
    {
        engine.update_section_status(0, sy, 0, false);
    }
    engine.set_light_enabled(&getter, 0, 0, true);
    engine.propagate_light_sources(&getter, 0, 0);
    engine.run_light_updates(&getter);

    assert_eq!(get_block_light_at(&engine, 0, 15, 0), 14);
    // y=16 lives in section y=1. The boundary crossing costs 1 opacity.
    assert_eq!(get_block_light_at(&engine, 0, 16, 0), 13);
}

// ---- F7. Block-light across chunk boundary ----
#[test]
fn f7_block_light_propagates_across_chunk_boundary() {
    let mut chunks: HashMap<(i32, i32), TestChunk> = HashMap::new();
    let mut left = TestChunk::default();
    // Torch at world (15, 4, 8) -> chunk (0, 0) local (15, 4, 8).
    left.insert(
        (15, 4, 8),
        LightBlockProperties {
            opacity: 0,
            emission: 14,
            uses_shape_for_light_occlusion: false,
            occlusion_shape_occludes_full_face: false,
        },
    );
    chunks.insert((0, 0), left);
    chunks.insert((1, 0), TestChunk::default());
    let getter = SimpleChunkGetter::new(level_height_overworld_test(), chunks);
    let mut engine = LevelLightEngine::new(level_height_overworld_test(), true, false);
    for cx in 0..=1 {
        for sy in level_height_overworld_test().min_section_y()
            ..=level_height_overworld_test().max_section_y()
        {
            engine.update_section_status(cx, sy, 0, false);
        }
        engine.set_light_enabled(&getter, cx, 0, true);
        engine.propagate_light_sources(&getter, cx, 0);
    }
    engine.run_light_updates(&getter);

    assert_eq!(get_block_light_at(&engine, 15, 4, 8), 14);
    // world (16, 4, 8) is in chunk (1, 0) local (0, 4, 8). After crossing
    // the chunk boundary the level is 13.
    assert_eq!(get_block_light_at(&engine, 16, 4, 8), 13);
}

// ---- F8. Sky-light open column ----
#[test]
fn f8_sky_light_open_column_is_fully_15() {
    let chunks = single_chunk_with(|_| {});
    let getter = SimpleChunkGetter::new(level_height_overworld_test(), chunks);
    let mut engine = LevelLightEngine::new(level_height_overworld_test(), false, true);
    for sy in level_height_overworld_test().min_section_y()
        ..=level_height_overworld_test().max_section_y()
    {
        engine.update_section_status(0, sy, 0, false);
    }
    engine.set_light_enabled(&getter, 0, 0, true);
    engine.propagate_light_sources(&getter, 0, 0);
    engine.run_light_updates(&getter);

    // Every voxel in the column should report sky-light 15 because no
    // occluder exists.
    for y in [
        level_height_overworld_test().min_y(),
        0,
        64,
        level_height_overworld_test().max_y(),
    ] {
        let sky = get_sky_light_at(&engine, 4, y, 4);
        assert_eq!(sky, 15, "sky light at y={y} should be 15, got {sky}");
    }
}

// ---- F9. Sky-light blocked column ----
#[test]
fn f9_sky_light_blocked_column_decays_below_stone() {
    let blocker_y = 70;
    let chunks = single_chunk_with(|chunk| {
        chunk.insert(
            (4, blocker_y, 4),
            LightBlockProperties {
                opacity: 15,
                emission: 0,
                uses_shape_for_light_occlusion: false,
                occlusion_shape_occludes_full_face: true,
            },
        );
    });
    let getter = SimpleChunkGetter::new(level_height_overworld_test(), chunks);
    let mut engine = LevelLightEngine::new(level_height_overworld_test(), false, true);
    for sy in level_height_overworld_test().min_section_y()
        ..=level_height_overworld_test().max_section_y()
    {
        engine.update_section_status(0, sy, 0, false);
    }
    engine.set_light_enabled(&getter, 0, 0, true);
    engine.propagate_light_sources(&getter, 0, 0);
    engine.run_light_updates(&getter);

    assert_eq!(get_sky_light_at(&engine, 4, blocker_y + 1, 4), 15);
    assert_eq!(get_sky_light_at(&engine, 4, blocker_y, 4), 0);
    // The open column at (5, _, 4) gets sky-light 15 all the way down. The
    // shadowed column at (4, _, 4) recovers via cardinal bleed from its
    // open neighbours: directly under the blocker (y=blocker_y-1) the
    // four-neighbour minimum bleed is 14.
    assert_eq!(get_sky_light_at(&engine, 5, blocker_y, 4), 15);
    assert_eq!(get_sky_light_at(&engine, 4, blocker_y - 1, 4), 14);
}

// ---- F10. Sky-light cardinal bleed under overhang ----
#[test]
fn f10_sky_light_cardinal_bleed_under_overhang() {
    let chunks = single_chunk_with(|chunk| {
        // Build a flat ceiling at y=80 across the 4x4 region with corners at
        // (4,4)..(7,7). Outside that region the column is open, so light
        // pours in around the edges.
        for x in 4..=7 {
            for z in 4..=7 {
                chunk.insert(
                    (x, 80, z),
                    LightBlockProperties {
                        opacity: 15,
                        emission: 0,
                        uses_shape_for_light_occlusion: false,
                        occlusion_shape_occludes_full_face: true,
                    },
                );
            }
        }
    });
    let getter = SimpleChunkGetter::new(level_height_overworld_test(), chunks);
    let mut engine = LevelLightEngine::new(level_height_overworld_test(), false, true);
    for sy in level_height_overworld_test().min_section_y()
        ..=level_height_overworld_test().max_section_y()
    {
        engine.update_section_status(0, sy, 0, false);
    }
    engine.set_light_enabled(&getter, 0, 0, true);
    engine.propagate_light_sources(&getter, 0, 0);
    engine.run_light_updates(&getter);

    // Just below the ceiling, the centre (5, 79, 5) is two blocks away from
    // any open column on either side. Each step in costs 1 opacity, so the
    // edge column gets 14 and the centre gets 13.
    assert_eq!(get_sky_light_at(&engine, 4, 79, 5), 14);
    // Centre under the overhang: 2 steps from the open edge -> 13.
    assert_eq!(get_sky_light_at(&engine, 5, 79, 5), 13);
}

// ---- Diagnostic: modern decoration blocks must report opacity 0 ----
#[test]
fn diagnostic_modern_decoration_blocks_are_transparent() {
    use crate::lighting::block_light_properties::light_properties_for;
    // Java vanilla `BlockBehaviour.getLightDampening` returns 0 for any
    // block whose state propagates sky-light down — which is the default
    // for every `noCollision()` decoration block. The previous lookup
    // fallback treated everything not in the representative-state table as
    // fully opaque (opacity 15) and that's exactly what made forest grass
    // render as dark patches under leaf litter, pink petals, moss carpet,
    // and friends.
    for name in [
        "minecraft:leaf_litter",
        "minecraft:moss_carpet",
        "minecraft:pink_petals",
        "minecraft:wildflowers",
        "minecraft:firefly_bush",
        "minecraft:bush",
        "minecraft:oak_sapling",
        "minecraft:cherry_sapling",
        "minecraft:vine",
        "minecraft:spore_blossom",
        "minecraft:big_dripleaf",
        "minecraft:small_dripleaf",
        "minecraft:hanging_moss",
        "minecraft:crimson_roots",
        "minecraft:warped_roots",
        "minecraft:bamboo",
    ] {
        let props = light_properties_for(name);
        assert_eq!(props.opacity, 0, "{name} should be opacity 0");
        assert!(
            !props.occlusion_shape_occludes_full_face,
            "{name} should not seal its face below"
        );
    }
}

// ---- Diagnostic: cardinal bleed from a vertical shaft into a horizontal cavern ----
#[test]
fn diagnostic_sky_light_bleeds_horizontally_from_shaft_into_cavern() {
    // Build a stone ceiling at y=80 across the full chunk, with a single
    // 1x1 vertical shaft at (x=8, z=8) that lets sky reach down to y=64,
    // where the cavern opens up horizontally (a flat air corridor at y=64,
    // capped by stone at y=63 below and stone at y=65 above except the
    // shaft column). Sky-light has to travel down the shaft, then bleed
    // sideways at y=64 through the cavern, dropping by 1 per block.
    let chunks = single_chunk_with(|chunk| {
        let stone = LightBlockProperties {
            opacity: 15,
            emission: 0,
            uses_shape_for_light_occlusion: false,
            occlusion_shape_occludes_full_face: true,
        };
        for x in 0..16_i32 {
            for z in 0..16_i32 {
                if x == 8 && z == 8 {
                    continue;
                }
                chunk.insert((x, 80, z), stone);
                // Cap the cavern's ceiling everywhere except the shaft.
                chunk.insert((x, 65, z), stone);
            }
        }
        // Floor below the cavern.
        for x in 0..16_i32 {
            for z in 0..16_i32 {
                chunk.insert((x, 63, z), stone);
            }
        }
    });
    let getter = SimpleChunkGetter::new(level_height_overworld_test(), chunks);
    let mut engine = LevelLightEngine::new(level_height_overworld_test(), false, true);
    for sy in level_height_overworld_test().min_section_y()
        ..=level_height_overworld_test().max_section_y()
    {
        engine.update_section_status(0, sy, 0, false);
    }
    engine.set_light_enabled(&getter, 0, 0, true);
    engine.propagate_light_sources(&getter, 0, 0);
    engine.run_light_updates(&getter);

    // Inside the shaft column the cells are sources (no occluder above them
    // in their own column).
    assert_eq!(get_sky_light_at(&engine, 8, 79, 8), 15);
    assert_eq!(get_sky_light_at(&engine, 8, 64, 8), 15);
    // One block sideways at the cavern level the bleed has cost 1.
    assert_eq!(get_sky_light_at(&engine, 9, 64, 8), 14);
    assert_eq!(get_sky_light_at(&engine, 10, 64, 8), 13);
    // Deep into the cavern the gradient continues until it bottoms out at 0.
    assert_eq!(get_sky_light_at(&engine, 14, 64, 8), 9);
    assert_eq!(get_sky_light_at(&engine, 15, 64, 8), 8);
}

// ---- F11. Sky-light occlusion-shape fallback ----
#[test]
fn f11_sky_light_partial_shape_fallback_documented() {
    // RustCraft does not yet wire voxel shapes through `LightBlockProperties`;
    // F11 verifies the documented fallback behaviour: a block with
    // `uses_shape_for_light_occlusion = true` but a non-sealed face acts as
    // a transparent block to the lighting engine.
    let chunks = single_chunk_with(|chunk| {
        // Half-occluding "slab" stand-in: dampens by 0 (so light passes),
        // claims to use shape, but its full-face flag is false.
        chunk.insert(
            (3, 50, 3),
            LightBlockProperties {
                opacity: 0,
                emission: 0,
                uses_shape_for_light_occlusion: true,
                occlusion_shape_occludes_full_face: false,
            },
        );
    });
    let getter = SimpleChunkGetter::new(level_height_overworld_test(), chunks);
    let mut engine = LevelLightEngine::new(level_height_overworld_test(), false, true);
    for sy in level_height_overworld_test().min_section_y()
        ..=level_height_overworld_test().max_section_y()
    {
        engine.update_section_status(0, sy, 0, false);
    }
    engine.set_light_enabled(&getter, 0, 0, true);
    engine.propagate_light_sources(&getter, 0, 0);
    engine.run_light_updates(&getter);

    // With shape declared but face open, sky-light still passes (0 opacity).
    assert_eq!(get_sky_light_at(&engine, 3, 49, 3), 15);
}

// ---- F13. hasDifferentLightProperties ----
#[test]
fn f13_has_different_light_properties_for_common_pairs() {
    let air = LightBlockProperties::AIR;
    let stone = light_properties_for("minecraft:stone");
    let torch = light_properties_for("minecraft:torch");
    let glass = light_properties_for("minecraft:glass");
    assert!(has_different_light_properties(air, stone));
    assert!(has_different_light_properties(air, torch));
    assert!(has_different_light_properties(stone, torch));
    assert!(!has_different_light_properties(stone, stone));
    assert!(!has_different_light_properties(air, glass));
    // glass has opacity 0, same as air, no emission, same shape flags.
}

// ---- F14. Block-state opacity sourcing ----
#[test]
fn f14_opacity_values_match_vanilla_block_metadata() {
    assert_eq!(light_properties_for("minecraft:air").opacity, 0);
    assert_eq!(light_properties_for("minecraft:stone").opacity, 15);
    assert_eq!(light_properties_for("minecraft:water").opacity, 1);
    assert_eq!(light_properties_for("minecraft:ice").opacity, 2);
    assert_eq!(light_properties_for("minecraft:oak_leaves").opacity, 1);
    assert_eq!(light_properties_for("minecraft:cobweb").opacity, 1);
    assert_eq!(light_properties_for("minecraft:glass").opacity, 0);
    assert_eq!(light_properties_for("minecraft:lava").opacity, 0);
}

// ---- F15. Emission sourcing ----
#[test]
fn f15_emission_values_match_vanilla_block_metadata() {
    assert_eq!(light_properties_for("minecraft:torch").emission, 14);
    assert_eq!(light_properties_for("minecraft:soul_torch").emission, 10);
    assert_eq!(light_properties_for("minecraft:redstone_torch").emission, 7);
    assert_eq!(light_properties_for("minecraft:glowstone").emission, 15);
    assert_eq!(light_properties_for("minecraft:sea_lantern").emission, 15);
    assert_eq!(
        light_properties_for("minecraft:jack_o_lantern").emission,
        15
    );
    assert_eq!(light_properties_for("minecraft:lava").emission, 15);
    assert_eq!(light_properties_for("minecraft:magma_block").emission, 3);
}

// ---- F16/F17/F18/F19. End-to-end via compute_chunk_lighting ----
#[test]
fn f16_generated_chunk_produces_dark_underground_and_lit_surface() {
    use crate::lighting::compute_chunk_lighting;
    use crate::storage::chunk::LevelChunk;
    use crate::storage::region::ChunkPos;

    let mut chunk = build_test_chunk_with_floor();
    let level_height = LevelHeightAccessor::new(-64, 384);
    let _ = compute_chunk_lighting(&mut chunk, level_height, true);

    // Surface block at y=63 is the topmost stone; y=64 is air with full sky.
    assert_eq!(sky_light_in_chunk(&chunk, 4, 64, 4), 15);
    // Deep underground (y=10) is fully dark for sky light.
    assert_eq!(sky_light_in_chunk(&chunk, 4, 10, 4), 0);
    assert!(chunk.light_correct, "chunk should be marked light_correct");
    let _ = chunk.pos == (ChunkPos { x: 0, z: 0 });
    let _ = LevelChunk::empty(ChunkPos { x: 0, z: 0 }).pos;
}

#[test]
fn f17_generated_chunk_glowstone_lights_block_layer() {
    use crate::lighting::compute_chunk_lighting;

    let mut chunk = build_test_chunk_with_floor();
    // Place glowstone at world (8, 64, 8) (chunk-local same).
    chunk.set_block_state(8, 64, 8, "minecraft:glowstone");
    let level_height = LevelHeightAccessor::new(-64, 384);
    let _ = compute_chunk_lighting(&mut chunk, level_height, true);

    assert!(block_light_in_chunk(&chunk, 8, 64, 8) >= 14);
}

#[test]
fn f18_no_fullbright_leak_under_overhang() {
    use crate::lighting::compute_chunk_lighting;

    let mut chunk = build_test_chunk_with_floor();
    // Stamp a stone ceiling at y=100 across the entire chunk.
    for x in 0..16 {
        for z in 0..16 {
            chunk.set_block_state(x, 100, z, "minecraft:stone");
        }
    }
    let level_height = LevelHeightAccessor::new(-64, 384);
    let _ = compute_chunk_lighting(&mut chunk, level_height, true);

    // Under the ceiling, far from the edges (centre of chunk), sky-light
    // must be 0.
    assert_eq!(sky_light_in_chunk(&chunk, 8, 80, 8), 0);
}

#[test]
fn f19_compute_chunk_lighting_sets_light_correct() {
    use crate::lighting::compute_chunk_lighting;

    let mut chunk = build_test_chunk_with_floor();
    assert!(!chunk.light_correct);
    let level_height = LevelHeightAccessor::new(-64, 384);
    let _ = compute_chunk_lighting(&mut chunk, level_height, true);
    assert!(chunk.light_correct);
}

// ---- Helper: minimal in-memory chunk source for tests ----

#[derive(Default, Clone)]
struct TestChunk {
    blocks: HashMap<(i32, i32, i32), LightBlockProperties>,
}

impl TestChunk {
    fn insert(&mut self, pos: (i32, i32, i32), props: LightBlockProperties) {
        self.blocks.insert(pos, props);
    }
}

struct SimpleChunkGetter {
    level_height: LevelHeightAccessor,
    chunks: HashMap<(i32, i32), TestChunk>,
    sky_cache: RefCell<HashMap<(i32, i32), ChunkSkyLightSources>>,
}

impl SimpleChunkGetter {
    fn new(level_height: LevelHeightAccessor, chunks: HashMap<(i32, i32), TestChunk>) -> Self {
        Self {
            level_height,
            chunks,
            sky_cache: RefCell::new(HashMap::new()),
        }
    }
}

impl LightChunkGetter for SimpleChunkGetter {
    fn light_properties_at(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
    ) -> LightBlockProperties {
        let chunk_x = world_x.div_euclid(16);
        let chunk_z = world_z.div_euclid(16);
        let Some(chunk) = self.chunks.get(&(chunk_x, chunk_z)) else {
            return LightBlockProperties::BEDROCK_FALLBACK;
        };
        chunk
            .blocks
            .get(&(world_x, world_y, world_z))
            .copied()
            .unwrap_or(LightBlockProperties::AIR)
    }

    fn find_block_light_sources(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        consumer: &mut dyn BlockLightSourceConsumer,
    ) {
        let Some(chunk) = self.chunks.get(&(chunk_x, chunk_z)) else {
            return;
        };
        for ((x, y, z), props) in &chunk.blocks {
            if props.emission > 0 {
                consumer.accept(*x, *y, *z, props.emission);
            }
        }
    }

    fn sky_light_sources(&self, chunk_x: i32, chunk_z: i32) -> Option<ChunkSkyLightSources> {
        if !self.chunks.contains_key(&(chunk_x, chunk_z)) {
            return None;
        }
        if let Some(cached) = self.sky_cache.borrow().get(&(chunk_x, chunk_z)) {
            return Some(cached.clone());
        }
        let chunk_min_x = chunk_x * 16;
        let chunk_min_z = chunk_z * 16;
        let top = self.level_height.max_y();
        let bottom = self.level_height.min_y();
        let mut sources = ChunkSkyLightSources::new(self.level_height);
        sources.fill_from_columns(chunk_min_x, chunk_min_z, top, bottom, |x, y, z| {
            self.light_properties_at(x, y, z)
        });
        self.sky_cache
            .borrow_mut()
            .insert((chunk_x, chunk_z), sources.clone());
        Some(sources)
    }

    fn level(&self) -> LevelHeightAccessor {
        self.level_height
    }
}

fn level_height_overworld_test() -> LevelHeightAccessor {
    LevelHeightAccessor::new(-64, 384)
}

fn single_chunk_with<F>(setup: F) -> HashMap<(i32, i32), TestChunk>
where
    F: FnOnce(&mut TestChunk),
{
    let mut chunk = TestChunk::default();
    setup(&mut chunk);
    let mut map = HashMap::new();
    map.insert((0, 0), chunk);
    map
}

fn get_block_light_at(engine: &LevelLightEngine, x: i32, y: i32, z: i32) -> i32 {
    engine
        .block_engine
        .as_ref()
        .map(|e| e.get_light_value(block_pos_as_long(x, y, z)))
        .unwrap_or(0)
}

fn get_sky_light_at(engine: &LevelLightEngine, x: i32, y: i32, z: i32) -> i32 {
    engine
        .sky_engine
        .as_ref()
        .map(|e| e.get_light_value(block_pos_as_long(x, y, z)))
        .unwrap_or(0)
}

fn build_test_chunk_with_floor() -> crate::storage::chunk::LevelChunk {
    use crate::storage::chunk::LevelChunk;
    use crate::storage::region::ChunkPos;
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.min_section_y = -4;
    // Initialize 24 sections of empty air with no light data so the engine
    // computes from scratch.
    let mut sections: Vec<crate::storage::chunk::ChunkSection> = Vec::new();
    for section_y in -4..20 {
        sections.push(crate::storage::chunk::ChunkSection {
            y: section_y as i8,
            block_states: crate::storage::chunk::PalettedContainer::single(
                crate::storage::nbt::Tag::Compound(vec![(
                    "Name".to_string(),
                    crate::storage::nbt::Tag::String("minecraft:air".to_string()),
                )]),
                SECTION_VOLUME,
            )
            .to_nbt(),
            biomes: crate::storage::chunk::PalettedContainer::single(
                crate::storage::nbt::Tag::String("minecraft:plains".to_string()),
                crate::storage::chunk::BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: None,
        });
    }
    chunk.sections = sections;
    // Fill stone from y=-64 up to y=63, leaving y=64+ as air -> sky-surface
    // sits at y=64.
    for y in -64..=63 {
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block_state(x, y, z, "minecraft:stone");
            }
        }
    }
    chunk.light_correct = false;
    chunk
}

fn sky_light_in_chunk(chunk: &crate::storage::chunk::LevelChunk, x: i32, y: i32, z: i32) -> i32 {
    light_array_get(chunk, LightLayer::Sky, x, y, z)
}

fn block_light_in_chunk(chunk: &crate::storage::chunk::LevelChunk, x: i32, y: i32, z: i32) -> i32 {
    light_array_get(chunk, LightLayer::Block, x, y, z)
}

fn light_array_get(
    chunk: &crate::storage::chunk::LevelChunk,
    layer: LightLayer,
    x: i32,
    y: i32,
    z: i32,
) -> i32 {
    let section_y = y.div_euclid(16) as i8;
    let local_x = x.rem_euclid(16) as usize;
    let local_y = y.rem_euclid(16) as usize;
    let local_z = z.rem_euclid(16) as usize;
    let bytes = chunk
        .sections
        .iter()
        .find(|s| s.y == section_y)
        .and_then(|s| match layer {
            LightLayer::Block => s.block_light.as_ref(),
            LightLayer::Sky => s.sky_light.as_ref(),
        });
    if let Some(bytes) = bytes {
        let index = (local_y << 8) | (local_z << 4) | local_x;
        let byte = bytes[index >> 1] as u8;
        let nibble = index & 1;
        return ((byte >> (4 * nibble)) & 0x0F) as i32;
    }
    // No stored layer for this section. Java's `SkyLightSectionStorage`
    // walks up the column until it finds a layer; if every section above is
    // also empty, the answer is 15 for sky-light. For block-light the
    // default is 0.
    if matches!(layer, LightLayer::Block) {
        return 0;
    }
    for upper_section_y in (section_y + 1)..=20 {
        let bytes = chunk
            .sections
            .iter()
            .find(|s| s.y == upper_section_y)
            .and_then(|s| s.sky_light.as_ref());
        let Some(bytes) = bytes else { continue };
        let index = (local_y << 8) | (local_z << 4) | local_x;
        let byte = bytes[index >> 1] as u8;
        let nibble = index & 1;
        return ((byte >> (4 * nibble)) & 0x0F) as i32;
    }
    // Above every stored sky-light section -> default 15.
    15
}
