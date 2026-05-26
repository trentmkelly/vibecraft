use super::*;

#[test]
fn real_surface_generation_mode_depends_on_world_seed() {
    let chunk_seed_0 = super::super::generate_overworld_chunk_for_preset_with_mode(
        ChunkPos { x: 0, z: 0 },
        "normal",
        super::super::LiveChunkGenerationMode::RealSurface,
        0,
    )
    .expect("real-surface mode should generate seed 0 terrain");
    let chunk_seed_1 = super::super::generate_overworld_chunk_for_preset_with_mode(
        ChunkPos { x: 0, z: 0 },
        "normal",
        super::super::LiveChunkGenerationMode::RealSurface,
        1,
    )
    .expect("real-surface mode should generate seed 1 terrain");

    assert_ne!(
        chunk_seed_0.heightmaps.get("WORLD_SURFACE_WG"),
        chunk_seed_1.heightmaps.get("WORLD_SURFACE_WG"),
        "real-surface generation must not ignore the world seed"
    );
}

#[test]
fn generator_method_facade_exposes_vanilla_status_task_names() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    let pos = ChunkPos { x: 1, z: -1 };

    assert_eq!(
        super::super::generator_create_structures_for_stem(pos, &normal.overworld)
            .unwrap()
            .status,
        "minecraft:structure_starts"
    );
    assert_eq!(
        super::super::generator_create_references_for_stem(pos, &normal.overworld)
            .unwrap()
            .status,
        "minecraft:structure_references"
    );
    assert_eq!(
        super::super::generator_create_biomes_for_stem(pos, &normal.overworld)
            .unwrap()
            .status,
        "minecraft:biomes"
    );
    assert_eq!(
        super::super::generator_fill_from_noise_for_stem(pos, &normal.overworld)
            .unwrap()
            .status,
        "minecraft:noise"
    );
    assert_eq!(
        super::super::generator_build_surface_for_stem(pos, &normal.overworld)
            .unwrap()
            .status,
        "minecraft:surface"
    );
    assert_eq!(
        super::super::generator_apply_carvers_for_stem(pos, &normal.overworld)
            .unwrap()
            .status,
        "minecraft:carvers"
    );
    assert_eq!(
        super::super::generator_apply_biome_decoration_for_stem(pos, &normal.overworld)
            .unwrap()
            .status,
        "minecraft:features"
    );
    assert_eq!(
        super::super::generator_spawn_original_mobs_for_stem(pos, &normal.overworld)
            .unwrap()
            .status,
        "minecraft:spawn"
    );

    let noise_chunk =
        super::super::generator_fill_from_noise_for_stem(pos, &normal.overworld).unwrap();
    assert_eq!(noise_chunk.sections.len(), 24);
    assert!(noise_chunk.heightmaps.contains_key("WORLD_SURFACE_WG"));
}

#[test]
fn generator_apply_carvers_mutates_surface_chunk_blocks() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    let pos = ChunkPos { x: 1, z: -1 };
    let surface = super::super::generator_build_surface_for_stem(pos, &normal.overworld)
        .expect("surface chunk must generate before carvers");
    let carvers = super::super::generator_apply_carvers_for_stem(pos, &normal.overworld)
        .expect("carver status must execute");

    assert_eq!(carvers.status, "minecraft:carvers");
    assert!(
        carvers
            .carving_mask
            .as_ref()
            .is_some_and(|mask| !mask.is_empty()),
        "carver execution should persist a carving mask on the generated chunk"
    );
    assert_ne!(
        surface.heightmaps.get("WORLD_SURFACE"),
        carvers.heightmaps.get("WORLD_SURFACE"),
        "carver execution should recompute client heightmaps after mutating blocks"
    );

    let settings = super::super::builtin_noise_generator_settings("overworld").unwrap();
    let mut changed_blocks = 0;
    'scan: for y in settings.noise.min_y..settings.noise.min_y + settings.noise.height {
        for z in pos.z * 16..pos.z * 16 + 16 {
            for x in pos.x * 16..pos.x * 16 + 16 {
                if surface.get_block_state(x, y, z) != carvers.get_block_state(x, y, z) {
                    changed_blocks += 1;
                    if changed_blocks >= 16 {
                        break 'scan;
                    }
                }
            }
        }
    }
    assert!(
        changed_blocks >= 16,
        "configured carvers should replace terrain blocks with cave air or lava"
    );
}

#[test]
fn generator_apply_carvers_selects_carvers_from_resolved_biome_source() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    let pos = ChunkPos { x: 0, z: 0 };

    let super::super::ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = &normal.overworld.generator
    else {
        panic!("normal overworld should use a noise generator");
    };
    assert_eq!(
        super::super::carvers_for_biome_source_and_noise_settings(
            biome_source_model,
            pos,
            noise_settings,
            0,
        ),
        super::super::OVERWORLD_COMMON_CARVERS
    );

    let super::super::ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = &normal.nether.generator
    else {
        panic!("normal nether should use a noise generator");
    };
    assert_eq!(
        super::super::carvers_for_biome_source_and_noise_settings(
            biome_source_model,
            pos,
            noise_settings,
            0,
        ),
        super::super::NETHER_COMMON_CARVERS
    );

    let super::super::ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = &normal.end.generator
    else {
        panic!("normal end should use a noise generator");
    };
    assert_eq!(
        super::super::carvers_for_biome_source_and_noise_settings(
            biome_source_model,
            pos,
            noise_settings,
            0,
        ),
        &[] as &[&'static str]
    );
}

#[test]
fn cave_tunnel_carver_mutates_chunk_blocks_and_mask() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut chunk = LevelChunk::empty(pos);
    let height_context = WorldGenerationHeightContext {
        min_y: -64,
        height: 384,
    };
    chunk.min_section_y = 3;
    chunk.sections = (3..=5)
        .map(|section_y| ChunkSection {
            y: section_y,
            block_states: PalettedContainer::single(
                Tag::Compound(vec![(
                    "Name".to_string(),
                    Tag::String("minecraft:stone".to_string()),
                )]),
                SECTION_VOLUME,
            )
            .to_nbt(),
            biomes: PalettedContainer::single(
                Tag::String("minecraft:plains".to_string()),
                BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: Some(crate::lighting::data_layer::fullbright_sky_layer_bytes()),
        })
        .collect();

    let cave = super::super::configured_carver("cave").unwrap();
    let mut mask = Vec::new();
    let settings = super::super::builtin_noise_generator_settings("overworld").unwrap();
    let noise_router = super::super::OVERWORLD_NOISE_ROUTER;
    let noise_chunk = super::super::NoiseChunk::new(0, 0, *settings, 0, noise_router);
    let carved = super::super::carve_cave_tunnel_into_chunk(super::super::CaveTunnelChunkInput {
        chunk: &mut chunk,
        height_context,
        carver: cave,
        chunk_min_x: 0,
        chunk_min_z: 0,
        tunnel_seed: 12_345,
        x: 8.0,
        y: 64.0,
        z: 8.0,
        horizontal_radius_multiplier: 1.0,
        vertical_radius_multiplier: 1.0,
        thickness: 2.5,
        horizontal_rotation: 0.0,
        vertical_rotation: 0.0,
        start_step: 0,
        distance: 24,
        y_scale: 1.0,
        floor_level: -0.7,
        mask: &mut mask,
        depth: 0,
        settings,
        noise_chunk: &noise_chunk,
        aquifer: None,
    });

    assert!(carved > 0, "cave tunnel walking should carve stone blocks");
    assert!(
        !mask.is_empty(),
        "cave tunnel carving should record mask bits"
    );
    assert!(
        (48..=80).any(|y| (0..16).any(|z| (0..16)
            .any(|x| chunk.get_block_state(x, y, z).as_deref() == Some("minecraft:cave_air")))),
        "cave tunnel carving should replace at least one local stone block with cave air"
    );
}

#[test]
fn canyon_tunnel_carver_mutates_chunk_blocks_and_mask() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut chunk = LevelChunk::empty(pos);
    let height_context = WorldGenerationHeightContext {
        min_y: -64,
        height: 384,
    };
    chunk.min_section_y = 2;
    chunk.sections = (2..=5)
        .map(|section_y| ChunkSection {
            y: section_y,
            block_states: PalettedContainer::single(
                Tag::Compound(vec![(
                    "Name".to_string(),
                    Tag::String("minecraft:stone".to_string()),
                )]),
                SECTION_VOLUME,
            )
            .to_nbt(),
            biomes: PalettedContainer::single(
                Tag::String("minecraft:plains".to_string()),
                BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: Some(crate::lighting::data_layer::fullbright_sky_layer_bytes()),
        })
        .collect();

    let canyon = super::super::configured_carver("canyon").unwrap();
    let CarverShape::Canyon { shape, .. } = canyon.shape else {
        panic!("canyon configured carver must use canyon shape");
    };
    let mut mask = Vec::new();
    let settings = super::super::builtin_noise_generator_settings("overworld").unwrap();
    let noise_router = super::super::OVERWORLD_NOISE_ROUTER;
    let noise_chunk = super::super::NoiseChunk::new(0, 0, *settings, 0, noise_router);
    let carved =
        super::super::carve_canyon_tunnel_into_chunk(super::super::CanyonTunnelChunkInput {
            chunk: &mut chunk,
            height_context,
            carver: canyon,
            chunk_min_x: 0,
            chunk_min_z: 0,
            tunnel_seed: 98_765,
            x: 8.0,
            y: 56.0,
            z: 8.0,
            thickness: 4.0,
            horizontal_rotation: 0.0,
            vertical_rotation: 0.0,
            distance: 32,
            y_scale: 3.0,
            shape: &shape,
            mask: &mut mask,
            settings,
            noise_chunk: &noise_chunk,
            aquifer: None,
        });

    assert!(
        carved > 0,
        "canyon tunnel walking should carve stone blocks"
    );
    assert!(
        !mask.is_empty(),
        "canyon tunnel carving should record mask bits"
    );
    assert!(
        (32..=80).any(|y| (0..16).any(|z| (0..16)
            .any(|x| chunk.get_block_state(x, y, z).as_deref() == Some("minecraft:cave_air")))),
        "canyon tunnel carving should replace at least one local stone block with cave air"
    );
}

#[test]
fn carving_mask_indices_pack_to_persisted_bitset_words() {
    let packed = super::super::pack_carving_mask_indices(&[0, 1, 63, 64, 130]);
    assert_eq!(packed.len(), 3);
    assert_eq!(packed[0] as u64, 0x8000_0000_0000_0003);
    assert_eq!(packed[1] as u64, 0x0000_0000_0000_0001);
    assert_eq!(packed[2] as u64, 0x0000_0000_0000_0004);
}

#[test]
fn overworld_chunk_at_origin_has_correct_section_count_and_heightmaps() {
    // Mirrors Java's ChunkStatus parity expectation: overworld (0,0) produced by the
    // noise generator must have 24 sections (minY=-64, height=384, 384/16=24) and
    // the WORLD_SURFACE_WG and OCEAN_FLOOR_WG heightmaps required for worldgen.
    // Source: decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Heightmap.java
    let preset = super::super::resolve_world_preset("normal").unwrap();
    let chunk = super::super::generate_chunk_for_stem(ChunkPos { x: 0, z: 0 }, &preset.overworld)
        .expect("overworld chunk generation must not fail at (0,0)");

    let settings = super::super::builtin_noise_generator_settings("overworld").unwrap();
    let expected_sections = (settings.noise.height / 16) as usize;
    assert_eq!(
        chunk.sections.len(),
        expected_sections,
        "expected {expected_sections} sections for height={}",
        settings.noise.height
    );
    assert!(
        chunk.heightmaps.contains_key("WORLD_SURFACE_WG"),
        "WORLD_SURFACE_WG heightmap must be present"
    );
    assert!(
        chunk.heightmaps.contains_key("OCEAN_FLOOR_WG"),
        "OCEAN_FLOOR_WG heightmap must be present"
    );
}

#[test]
fn lightweight_tree_context_heights_match_noise_chunk_heightmaps() {
    let settings = super::super::builtin_noise_generator_settings("overworld").unwrap();
    let router =
        super::super::builtin_noise_router(super::super::noise_router_id_for_settings(*settings))
            .unwrap()
            .router;
    for (seed, pos) in [
        (0, ChunkPos { x: 1, z: -1 }),
        (0, ChunkPos { x: -1, z: -1 }),
        (1, ChunkPos { x: 2, z: 0 }),
        (42, ChunkPos { x: -2, z: 3 }),
    ] {
        let chunk = super::super::fill_from_noise_chunk(pos, settings, seed, router);
        let full_heights = super::super::tree_decoration_terrain_heights_from_wg(&chunk, settings);
        let lightweight_heights =
            super::super::noise_tree_context_heights(pos, settings, seed, router);

        assert_eq!(
                lightweight_heights.ocean_floor, full_heights.ocean_floor,
                "lightweight ocean floor heightmap should match full noise chunk for seed={seed} pos=({}, {})",
                pos.x, pos.z
            );
        assert_eq!(
                lightweight_heights.world_surface, full_heights.world_surface,
                "lightweight world surface heightmap should match full noise chunk for seed={seed} pos=({}, {})",
                pos.x, pos.z
            );
    }
}

#[test]
fn noise_preview_trees_follow_biome_generation_settings() {
    let settings = super::super::builtin_noise_generator_settings("overworld").unwrap();
    let terrain_heights = [settings.sea_level + 8; 16 * 16];
    let plains = super::super::noise_preview_tree_blocks(
        ChunkPos { x: 0, z: 0 },
        0,
        settings,
        "minecraft:plains",
        &terrain_heights,
    );
    let forest = super::super::noise_preview_tree_blocks(
        ChunkPos { x: 0, z: 0 },
        0,
        settings,
        "minecraft:forest",
        &terrain_heights,
    );
    let unknown = super::super::noise_preview_tree_blocks(
        ChunkPos { x: 0, z: 0 },
        0,
        settings,
        "minecraft:badlands",
        &terrain_heights,
    );

    assert!(forest.len() > plains.len());
    assert!(forest
        .iter()
        .any(|block| block.state == "minecraft:birch_log"));
    let cherry = super::super::noise_preview_tree_blocks(
        ChunkPos { x: 0, z: 0 },
        0,
        settings,
        "minecraft:cherry_grove",
        &terrain_heights,
    );
    let swamp = super::super::noise_preview_tree_blocks(
        ChunkPos { x: 0, z: 0 },
        0,
        settings,
        "minecraft:swamp",
        &terrain_heights,
    );
    let mangrove = super::super::noise_preview_tree_blocks(
        ChunkPos { x: 0, z: 0 },
        0,
        settings,
        "minecraft:mangrove_swamp",
        &terrain_heights,
    );
    let dark_forest = super::super::noise_preview_tree_blocks(
        ChunkPos { x: 0, z: 0 },
        0,
        settings,
        "minecraft:dark_forest",
        &terrain_heights,
    );
    assert!(cherry
        .iter()
        .any(|block| block.state == "minecraft:cherry_log"));
    assert!(swamp.iter().any(|block| block.state == "minecraft:oak_log"));
    assert!(mangrove
        .iter()
        .any(|block| block.state == "minecraft:mangrove_log"));
    assert!(dark_forest
        .iter()
        .any(|block| block.state == "minecraft:dark_oak_log"));
    assert!(unknown.is_empty());
}

#[test]
fn forest_preview_chunk_uses_forest_decoration_palette() {
    let settings = super::super::builtin_noise_generator_settings("overworld").unwrap();
    let chunk = super::super::materialize_noise_preview_chunk(
        ChunkPos { x: 0, z: 0 },
        &BiomeSourceModel::Fixed {
            biome: "minecraft:forest",
        },
        settings,
    );

    assert!(chunk.sections.iter().any(|section| {
        let Tag::Compound(block_states) = &section.block_states else {
            return false;
        };
        let Some((_, Tag::List(palette))) = block_states.iter().find(|(name, _)| name == "palette")
        else {
            return false;
        };
        palette.contains(&super::super::block_state_tag("minecraft:birch_log"))
            || palette.contains(&super::super::block_state_tag("minecraft:birch_leaves"))
    }));
}

#[test]
fn preview_tree_overlay_updates_world_surface_wg_heightmap() {
    let settings = super::super::builtin_noise_generator_settings("overworld").unwrap();
    let mut terrain_heights = [0; 16 * 16];
    for z in 0..16 {
        for x in 0..16 {
            terrain_heights[z * 16 + x] =
                super::super::noise_preview_terrain_height(x as i32, z as i32, settings).clamp(
                    settings.noise.min_y + 1,
                    settings.noise.min_y + settings.noise.height,
                );
        }
    }
    let overlay = super::super::noise_preview_tree_blocks(
        ChunkPos { x: 0, z: 0 },
        0,
        settings,
        "minecraft:forest",
        &terrain_heights,
    );
    let tallest = overlay
        .iter()
        .max_by_key(|block| block.pos.y)
        .expect("forest preview should emit visible tree blocks");
    let column = tallest.pos.z as usize * 16 + tallest.pos.x as usize;

    let chunk = super::super::materialize_noise_preview_chunk(
        ChunkPos { x: 0, z: 0 },
        &BiomeSourceModel::Fixed {
            biome: "minecraft:forest",
        },
        settings,
    );
    let Tag::LongArray(world_surface_wg) = chunk.heightmaps.get("WORLD_SURFACE_WG").unwrap() else {
        panic!("WORLD_SURFACE_WG should be stored as a long array");
    };

    assert_eq!(
        unpack_heightmap_column(world_surface_wg, column),
        tallest.pos.y + 1
    );
    assert!(
        tallest.pos.y + 1 > terrain_heights[column].max(settings.sea_level + 1),
        "tree overlay should raise the world surface above terrain"
    );
}

#[test]
fn noise_preview_ground_cover_follows_biome_features() {
    let settings = super::super::builtin_noise_generator_settings("overworld").unwrap();
    let terrain_heights = [settings.sea_level + 8; 16 * 16];
    let plains = super::super::noise_preview_ground_cover_blocks(
        ChunkPos { x: 0, z: 0 },
        settings,
        "minecraft:plains",
        &terrain_heights,
    );
    let sunflower = super::super::noise_preview_ground_cover_blocks(
        ChunkPos { x: 0, z: 0 },
        settings,
        "minecraft:sunflower_plains",
        &terrain_heights,
    );
    let unknown = super::super::noise_preview_ground_cover_blocks(
        ChunkPos { x: 0, z: 0 },
        settings,
        "minecraft:badlands",
        &terrain_heights,
    );

    assert!(plains
        .iter()
        .any(|block| matches!(block.state, "minecraft:short_grass" | "minecraft:dandelion")));
    assert!(sunflower
        .iter()
        .any(|block| block.state == "minecraft:sunflower"));
    assert!(sunflower.len() >= plains.len());
    assert!(unknown.is_empty());
}

#[test]
fn resolved_generators_answer_base_height_and_column_queries() {
    let flat = super::super::resolve_world_preset("flat").unwrap();
    assert_eq!(
        super::super::generator_base_height_for_stem(
            0,
            0,
            HeightmapKind::WorldSurfaceWg,
            &flat.overworld
        )
        .unwrap(),
        4
    );
    let flat_column = super::super::generator_base_column_for_stem(0, 0, &flat.overworld).unwrap();
    assert_eq!(flat_column.min_y, super::super::FLAT_GENERATOR_MIN_Y);
    assert_eq!(flat_column.states[0], "minecraft:bedrock");
    assert_eq!(flat_column.states[3], "minecraft:grass_block");

    let normal = super::super::resolve_world_preset("normal").unwrap();
    let world_surface = super::super::generator_base_height_for_stem(
        96,
        -48,
        HeightmapKind::WorldSurfaceWg,
        &normal.overworld,
    )
    .unwrap();
    let ocean_floor = super::super::generator_base_height_for_stem(
        96,
        -48,
        HeightmapKind::OceanFloorWg,
        &normal.overworld,
    )
    .unwrap();
    assert!(world_surface >= ocean_floor);

    let column = super::super::generator_base_column_for_stem(96, -48, &normal.overworld).unwrap();
    assert_eq!(column.min_y, super::super::OVERWORLD_NOISE_SETTINGS.min_y);
    assert_eq!(
        column.states.len(),
        super::super::OVERWORLD_NOISE_SETTINGS.height as usize
    );
    assert_eq!(column.states[0], "minecraft:bedrock");
    assert!(column.states.contains(&"minecraft:stone"));
    assert!(column.states.contains(&"minecraft:air"));
}

#[test]
fn unresolved_debug_generation_fails_closed() {
    assert_eq!(
        super::super::generate_overworld_chunk_for_preset(
            ChunkPos { x: 0, z: 0 },
            "debug_all_block_states"
        )
        .unwrap_err(),
        "Debug chunk generation for minecraft:overworld is not implemented".to_string()
    );
}
