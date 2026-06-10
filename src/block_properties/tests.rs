use super::*;

fn physics(name: &str) -> &'static StatePhysics {
    state_physics_by_name(name).unwrap_or_else(|| panic!("missing {name}"))
}

#[test]
fn tables_cover_every_block_and_state() {
    assert_eq!(TABLES.blocks.len(), block_state_entries().len());
    assert_eq!(TABLES.states.len(), VANILLA_BLOCK_STATE_COUNT_26_1_2);
    assert_eq!(TABLES.shapes.len(), 942);
    assert!(TABLES.sound_types.len() >= 123);

    for state in &TABLES.states {
        for index in [
            state.shape,
            state.collision_shape,
            state.occlusion_shape,
            state.interaction_shape,
            state.support_shape,
        ] {
            assert!((index as usize) < TABLES.shapes.len());
        }
        assert!((state.sound_type as usize) < TABLES.sound_types.len());
        assert!(map_color_name(state.map_color).is_some());
    }
}

#[test]
fn block_level_constants_match_java_blocks_registrations() {
    // Java: Blocks.STONE .strength(1.5F, 6.0F); explosion resistance is block-level.
    let stone = block_physics("minecraft:stone").expect("stone");
    assert_eq!(stone.explosion_resistance, 6.0);
    assert_eq!(stone.friction, 0.6); // BlockBehaviour default friction
    assert_eq!(stone.speed_factor, 1.0);
    assert_eq!(stone.jump_factor, 1.0);

    // Java: .friction(0.8F) on slime, .friction(0.98F) on ice/packed/blue ice.
    assert_eq!(block_physics("minecraft:slime_block").expect("slime").friction, 0.8);
    assert_eq!(block_physics("minecraft:ice").expect("ice").friction, 0.98);
    assert_eq!(block_physics("minecraft:blue_ice").expect("blue ice").friction, 0.989);

    // Java: .speedFactor(0.4F) soul sand, .jumpFactor(0.5F) honey block.
    assert_eq!(block_physics("minecraft:soul_sand").expect("soul sand").speed_factor, 0.4);
    assert_eq!(block_physics("minecraft:honey_block").expect("honey").jump_factor, 0.5);

    // Java: obsidian .strength(50.0F, 1200.0F), bedrock resistance 3600000.
    assert_eq!(
        block_physics("minecraft:obsidian").expect("obsidian").explosion_resistance,
        1200.0
    );
    assert_eq!(
        block_physics("minecraft:bedrock").expect("bedrock").explosion_resistance,
        3_600_000.0
    );
}

#[test]
fn state_physics_match_java_block_state_base_accessors() {
    let stone = physics("minecraft:stone");
    assert_eq!(stone.destroy_speed, 1.5);
    assert!(stone.requires_correct_tool_for_drops);
    assert!(stone.blocks_motion && stone.can_occlude && stone.is_solid_render);
    assert_eq!(map_color_name(stone.map_color), Some("STONE"));
    assert_eq!(state_sound_type(stone).name, "STONE");
    assert!(!stone.pathfind_land && !stone.pathfind_air && !stone.pathfind_water);

    let bedrock = physics("minecraft:bedrock");
    assert_eq!(bedrock.destroy_speed, -1.0); // unbreakable

    let torch = physics("minecraft:torch");
    assert_eq!(torch.light_emission, 14);
    assert!(!torch.blocks_motion);
    assert!(torch.pathfind_land);

    let air = physics("minecraft:air");
    assert!(air.is_air && !air.blocks_motion && air.replaceable);
    assert_eq!(shape(air.shape), &[] as &[ShapeBox]);
    assert_eq!(shape(air.collision_shape), &[] as &[ShapeBox]);

    let water = physics("minecraft:water");
    assert!(water.liquid && water.replaceable);
}

#[test]
fn fluid_states_and_light_match_java() {
    assert_eq!(
        physics("minecraft:water").fluid,
        StateFluid::Water { amount: 8, source: true }
    );
    assert_eq!(
        physics("minecraft:water[level=2]").fluid,
        StateFluid::Water { amount: 6, source: false }
    );

    // Waterlogged states carry a water source fluid.
    assert_eq!(
        physics("minecraft:oak_stairs[waterlogged=true]").fluid,
        StateFluid::Water { amount: 8, source: true }
    );

    // Light dampening: solid block = 15, ice = 1 (translucent), air = 0.
    assert_eq!(physics("minecraft:stone").light_dampening, 15);
    assert_eq!(physics("minecraft:ice").light_dampening, 1);
    assert_eq!(physics("minecraft:air").light_dampening, 0);

    // Lit-state-dependent light emission on redstone torches.
    assert_eq!(physics("minecraft:redstone_torch[lit=true]").light_emission, 7);
    assert_eq!(physics("minecraft:redstone_torch[lit=false]").light_emission, 0);
}

#[test]
fn push_reactions_and_block_flags_match_java() {
    // Push reactions. Note: obsidian is NORMAL here — Java blocks pushing it via a
    // hardcoded check in PistonBaseBlock.isPushable (line 241), not the property.
    assert_eq!(physics("minecraft:obsidian").push_reaction, PushReaction::Normal);
    assert_eq!(physics("minecraft:anvil").push_reaction, PushReaction::Block);
    assert_eq!(physics("minecraft:piston").push_reaction, PushReaction::Block);
    assert_eq!(physics("minecraft:stone").push_reaction, PushReaction::Normal);
    assert_eq!(physics("minecraft:torch").push_reaction, PushReaction::Destroy);

    // Block-entity holders and signal sources.
    assert!(physics("minecraft:chest").has_block_entity);
    assert!(physics("minecraft:lever[powered=true]").is_signal_source);
    assert!(physics("minecraft:grass_block").is_randomly_ticking);
    assert!(!physics("minecraft:dirt").is_randomly_ticking);
}

#[test]
fn voxel_shapes_match_java_block_shape_definitions() {
    // Full cube.
    let stone = physics("minecraft:stone");
    assert_eq!(shape(stone.shape), &[[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]);
    assert_eq!(shape(stone.collision_shape), &[[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]);

    // Java TorchBlock AABB: box(6, 0, 6, 10, 10, 10) / 16.
    let torch = physics("minecraft:torch");
    assert_eq!(shape(torch.shape), &[[0.375, 0.0, 0.375, 0.625, 0.625, 0.625]]);
    assert_eq!(shape(torch.collision_shape), &[] as &[ShapeBox]);

    // Java SlabBlock bottom AABB: box(0, 0, 0, 16, 8, 16) / 16.
    let slab = physics("minecraft:oak_slab[type=bottom]");
    assert_eq!(shape(slab.shape), &[[0.0, 0.0, 0.0, 1.0, 0.5, 1.0]]);

    // Java FenceBlock collision is 1.5 blocks tall.
    let fence = physics("minecraft:oak_fence");
    assert!(shape(fence.collision_shape)
        .iter()
        .all(|aabb| (aabb[4] - 1.5).abs() < 1e-9));

    // Stairs decompose into multiple boxes.
    let stairs = physics("minecraft:oak_stairs");
    assert!(shape(stairs.shape).len() > 1);
}

#[test]
fn sound_type_table_pins_java_sound_events() {
    let stone = state_sound_type(physics("minecraft:stone"));
    assert_eq!(stone.volume, 1.0);
    assert_eq!(stone.pitch, 1.0);
    assert_eq!(stone.break_sound, "minecraft:block.stone.break");
    assert_eq!(stone.step_sound, "minecraft:block.stone.step");
    assert_eq!(stone.place_sound, "minecraft:block.stone.place");
    assert_eq!(stone.hit_sound, "minecraft:block.stone.hit");
    assert_eq!(stone.fall_sound, "minecraft:block.stone.fall");

    let grass = state_sound_type(physics("minecraft:grass_block"));
    assert_eq!(grass.name, "GRASS");
    assert_eq!(grass.break_sound, "minecraft:block.grass.break");
}

#[test]
fn full_report_round_trips_against_vendored_json() {
    // Decode the vendored gz independently and verify every state record matches
    // what the parsed tables expose, in both directions.
    use std::io::Read;
    let raw: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vanilla-data/reports/block_properties_26_1_2.json.gz"
    ));
    let mut decoded = String::new();
    flate2::read::GzDecoder::new(raw)
        .read_to_string(&mut decoded)
        .expect("gzip");
    let root: serde_json::Value = serde_json::from_str(&decoded).expect("json");

    let mut checked = 0usize;
    for (name, block) in root["blocks"].as_object().expect("blocks") {
        let block_physics = block_physics(name).unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(
            block_physics.explosion_resistance,
            block["explosion_resistance"].as_f64().expect("resistance") as f32,
            "{name}"
        );
        for state in block["states"].as_array().expect("states") {
            let id = state["id"].as_i64().expect("id") as i32;
            let parsed = state_physics(id).unwrap_or_else(|| panic!("missing state {id}"));
            assert_eq!(
                parsed.destroy_speed,
                state["destroy_speed"].as_f64().expect("speed") as f32,
                "{name} {id}"
            );
            assert_eq!(
                parsed.light_emission as u64,
                state["light_emission"].as_u64().expect("light"),
                "{name} {id}"
            );
            assert_eq!(
                parsed.blocks_motion,
                state["blocks_motion"].as_bool().expect("motion"),
                "{name} {id}"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, VANILLA_BLOCK_STATE_COUNT_26_1_2);
}
