//! Behavioural tests for the full `/execute` surface (`ExecuteCommand.java`).

use super::*;
use crate::command::{
    CommandLootPredicate, CustomBossBar, EntityRelation, EntityRelationKind, EntityRotation,
    LoadedChunk,
};

const GM: LevelBasedPermissionSet = LevelBasedPermissionSet::GAMEMASTER;

fn run(state: &mut ServerCommandState, input: &str) -> Result<CommandResult, CommandError> {
    execute_builtin_command(state, GM, input)
}

fn steve_state() -> ServerCommandState {
    ServerCommandState {
        online_players: vec![
            NameAndId::create_offline("Steve"),
            NameAndId::create_offline("Alex"),
        ],
        command_source_player: Some(NameAndId::create_offline("Steve")),
        command_source_entity: Some(entity_ref("Steve")),
        command_source_position: Vec3 {
            x: 1.0,
            y: 64.0,
            z: 1.0,
        },
        entity_states: vec![
            EntityState {
                entity: entity_ref("Steve"),
                kind: EntityKind::Player,
                dimension: "minecraft:overworld".to_string(),
            },
            EntityState {
                entity: entity_ref("Alex"),
                kind: EntityKind::Player,
                dimension: "minecraft:overworld".to_string(),
            },
        ],
        entity_positions: vec![EntityPosition {
            entity: entity_ref("Alex"),
            dimension: "minecraft:overworld".to_string(),
            position: Vec3 {
                x: 10.0,
                y: 70.0,
                z: -4.0,
            },
        }],
        ..ServerCommandState::default()
    }
}

fn add_objective(state: &mut ServerCommandState, name: &str) {
    state.scoreboard_objectives.push(ScoreboardObjective {
        name: name.to_string(),
        criteria: "dummy".to_string(),
        display_name: name.to_string(),
        render_type: "integer".to_string(),
        display_auto_update: false,
        number_format: None,
    });
}

fn set_score(state: &mut ServerCommandState, owner: &str, objective: &str, value: i32) {
    state.scoreboard_scores.retain(|s| !(s.owner == owner && s.objective == objective));
    state.scoreboard_scores.push(ScoreboardScore {
        owner: owner.to_string(),
        objective: objective.to_string(),
        value,
        locked: false,
        display_name: None,
        number_format: None,
    });
}

fn score(state: &ServerCommandState, owner: &str, objective: &str) -> Option<i32> {
    state
        .scoreboard_scores
        .iter()
        .find(|s| s.owner == owner && s.objective == objective)
        .map(|s| s.value)
}

fn last_source(state: &ServerCommandState) -> ExecuteSourceSnapshot {
    state.execute_events.last().unwrap().sources[0].clone()
}

#[test]
fn positioned_center_corrects_absolute_x_and_z_only() {
    let mut state = steve_state();
    run(&mut state, "execute positioned 4 65 9 run say hi").unwrap();
    let source = last_source(&state);
    assert_eq!((source.position.x, source.position.y, source.position.z), (4.5, 65.0, 9.5));

    run(&mut state, "execute positioned 4.0 65 9.25 run say hi").unwrap();
    let source = last_source(&state);
    assert_eq!((source.position.x, source.position.y, source.position.z), (4.0, 65.0, 9.25));

    run(&mut state, "execute positioned ~1 ~ ~-1 run say hi").unwrap();
    let source = last_source(&state);
    assert_eq!((source.position.x, source.position.y, source.position.z), (2.0, 64.0, 0.0));
}

#[test]
fn positioned_local_coordinates_use_source_rotation_and_anchor() {
    let mut state = steve_state();
    state.command_source_position = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    // yaw 0 faces +z, so ^ ^ ^3 is three blocks ahead of the feet anchor.
    run(&mut state, "execute positioned ^ ^ ^3 run say hi").unwrap();
    let source = last_source(&state);
    assert!(source.position.x.abs() < 1e-6);
    assert!(source.position.y.abs() < 1e-6);
    assert!((source.position.z - 3.0).abs() < 1e-6);

    // Anchored eyes lifts the local origin by the entity eye height.
    run(&mut state, "execute anchored eyes positioned ^ ^ ^ run say hi").unwrap();
    assert!((last_source(&state).position.y - 1.62).abs() < 1e-9);
}

#[test]
fn mixed_local_and_world_coordinates_are_a_syntax_error() {
    let mut state = steve_state();
    assert_eq!(
        run(&mut state, "execute positioned ^ 1 2 run say hi"),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        run(&mut state, "execute positioned 1 ^ 2 run say hi"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn as_and_at_fork_over_selector_results_and_copy_entity_state() {
    let mut state = steve_state();
    state.entity_rotations.push(EntityRotation {
        entity: entity_ref("Alex"),
        x_rot: 15.0,
        y_rot: 90.0,
    });
    run(&mut state, "execute as @a at @s run say hi").unwrap();
    let event = state.execute_events.last().unwrap();
    assert_eq!(event.sources.len(), 2);
    let alex = event
        .sources
        .iter()
        .find(|s| s.entity.as_ref().unwrap().id == "Alex")
        .unwrap();
    assert_eq!((alex.position.x, alex.position.y, alex.position.z), (10.0, 70.0, -4.0));
    assert_eq!((alex.pitch, alex.yaw), (15.0, 90.0));

    // `positioned as` copies only the position, `rotated as` only the rotation.
    run(&mut state, "execute positioned as Alex run say hi").unwrap();
    let source = last_source(&state);
    assert_eq!(source.position.x, 10.0);
    assert_eq!(source.yaw, 0.0);
    run(&mut state, "execute rotated as Alex run say hi").unwrap();
    let source = last_source(&state);
    assert_eq!((source.pitch, source.yaw), (15.0, 90.0));
    assert_eq!(source.position.x, 1.0);
}

#[test]
fn rotated_absolute_and_relative_take_yaw_then_pitch() {
    let mut state = steve_state();
    state.command_source_yaw = 10.0;
    state.command_source_pitch = 5.0;
    run(&mut state, "execute rotated 90 45 run say hi").unwrap();
    let source = last_source(&state);
    assert_eq!((source.yaw, source.pitch), (90.0, 45.0));
    run(&mut state, "execute rotated ~5 ~-5 run say hi").unwrap();
    let source = last_source(&state);
    assert_eq!((source.yaw, source.pitch), (15.0, 0.0));
}

#[test]
fn align_floors_only_the_selected_axes() {
    let mut state = steve_state();
    state.command_source_position = Vec3 { x: 1.7, y: 64.9, z: -2.2 };
    run(&mut state, "execute align xz run say hi").unwrap();
    let source = last_source(&state);
    assert_eq!((source.position.x, source.position.y, source.position.z), (1.0, 64.9, -3.0));
    run(&mut state, "execute align y run say hi").unwrap();
    assert_eq!(last_source(&state).position.y, 64.0);
    assert_eq!(
        run(&mut state, "execute align xx run say hi"),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        run(&mut state, "execute align w run say hi"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn in_rescales_coordinates_between_overworld_and_nether() {
    let mut state = steve_state();
    state.command_source_position = Vec3 { x: 80.0, y: 64.0, z: -16.0 };
    run(&mut state, "execute in minecraft:the_nether run say hi").unwrap();
    let source = last_source(&state);
    assert_eq!(source.dimension, "minecraft:the_nether");
    assert_eq!((source.position.x, source.position.z), (10.0, -2.0));
    run(&mut state, "execute in minecraft:the_nether in minecraft:overworld run say hi").unwrap();
    let source = last_source(&state);
    assert_eq!((source.position.x, source.position.z), (80.0, -16.0));
}

#[test]
fn in_unknown_dimension_reports_argument_dimension_invalid() {
    let mut state = steve_state();
    assert_eq!(
        run(&mut state, "execute in foo:bar run say hi"),
        Err(CommandError::Translatable {
            key: "argument.dimension.invalid",
            args: vec!["foo:bar".to_string()],
        })
    );
}

#[test]
fn anchored_and_facing_use_java_rotation_math() {
    let mut state = steve_state();
    state.command_source_position = Vec3 { x: 0.0, y: 64.0, z: 0.0 };
    run(&mut state, "execute facing 0.0 64.0 10.0 run say hi").unwrap();
    let source = last_source(&state);
    assert_eq!(source.yaw, 0.0);
    assert!(source.pitch.abs() < 1e-3);
    run(&mut state, "execute facing entity Alex feet run say hi").unwrap();
    let source = last_source(&state);
    // Alex is at +x, -z of the source: yaw = atan2(dz, dx) - 90.
    assert!((source.yaw - (-4.0_f64.atan2(10.0).to_degrees() as f32 - 90.0)).abs() < 0.01);
}

#[test]
fn on_relations_follow_mounts_and_explicit_links() {
    let mut state = steve_state();
    state.entity_mounts.push(EntityMount {
        target: entity_ref("Steve"),
        vehicle: entity_ref("Alex"),
    });
    state.entity_relations.push(EntityRelation {
        entity: entity_ref("Steve"),
        kind: EntityRelationKind::Target,
        target: entity_ref("Alex"),
    });
    run(&mut state, "execute on vehicle run say hi").unwrap();
    assert_eq!(last_source(&state).entity, Some(entity_ref("Alex")));
    run(&mut state, "execute on target run say hi").unwrap();
    assert_eq!(last_source(&state).entity, Some(entity_ref("Alex")));
    run(&mut state, "execute as Alex on passengers run say hi").unwrap();
    assert_eq!(last_source(&state).entity, Some(entity_ref("Steve")));

    // No relation of that kind: the fork is empty and nothing runs, silently.
    let events = state.execute_events.len();
    let result = run(&mut state, "execute on owner run say hi").unwrap();
    assert_eq!(result.success_count, 0);
    assert_eq!(result.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(state.execute_events.len(), events);

    // Removed entities are filtered out.
    state.killed_entities.push(entity_ref("Alex"));
    run(&mut state, "execute on vehicle run say hi").unwrap();
    assert_eq!(state.execute_events.len(), events);
}

#[test]
fn summon_creates_the_entity_and_redirects_the_source_to_it() {
    let mut state = steve_state();
    run(&mut state, "execute summon minecraft:pig run say hi").unwrap();
    let summoned = state.summoned_entities.last().unwrap().clone();
    assert_eq!(summoned.entity_type, "minecraft:pig");
    assert_eq!(last_source(&state).entity, Some(summoned.entity));
}

#[test]
fn store_result_and_success_write_scores() {
    let mut state = steve_state();
    add_objective(&mut state, "obj");
    let result = run(&mut state, "execute store result score Steve obj run give Steve minecraft:stone 5");
    assert!(result.is_ok());
    assert_eq!(score(&state, "Steve", "obj"), Some(1));

    run(&mut state, "execute store success score Steve obj run give Steve minecraft:stone 5").unwrap();
    assert_eq!(score(&state, "Steve", "obj"), Some(1));

    // A failing command stores 0 for both variants and is not forked, so it reports the error.
    set_score(&mut state, "Steve", "obj", 7);
    assert!(run(&mut state, "execute store success score Steve obj run gamemode bogus Steve").is_err());
    assert_eq!(score(&state, "Steve", "obj"), Some(0));
    set_score(&mut state, "Steve", "obj", 7);
    assert!(run(&mut state, "execute store result score Steve obj run gamemode bogus Steve").is_err());
    assert_eq!(score(&state, "Steve", "obj"), Some(0));
}

#[test]
fn store_chains_callbacks_in_order_and_uses_wildcard_holders() {
    let mut state = steve_state();
    add_objective(&mut state, "a");
    add_objective(&mut state, "b");
    set_score(&mut state, "Alex", "a", 9);
    run(
        &mut state,
        "execute store success score Steve a store result score * b run give Steve minecraft:stone 3",
    )
    .unwrap();
    assert_eq!(score(&state, "Steve", "a"), Some(1));
    // `*` expands when the store stage runs, before the first callback created Steve's score,
    // so only Alex (already tracked) receives the second store.
    assert_eq!(score(&state, "Alex", "b"), Some(1));
    assert_eq!(score(&state, "Steve", "b"), None);
}

#[test]
fn store_into_unknown_objective_or_bossbar_is_an_error() {
    let mut state = steve_state();
    assert_eq!(
        run(&mut state, "execute store result score Steve nope run say hi"),
        Err(CommandError::ScoreboardObjectiveNotFound)
    );
    assert_eq!(
        run(&mut state, "execute store result bossbar minecraft:nope value run say hi"),
        Err(CommandError::BossBarUnknown)
    );
}

#[test]
fn store_into_bossbar_value_and_max() {
    let mut state = steve_state();
    state.bossbars.push(CustomBossBar {
        id: "minecraft:bar".to_string(),
        name: "Bar".to_string(),
        color: BossBarCommandColor::White,
        overlay: BossBarCommandOverlay::Progress,
        value: 0,
        max: 100,
        visible: true,
        players: Vec::new(),
    });
    run(&mut state, "execute store result bossbar minecraft:bar value run give Steve minecraft:stone 42").unwrap();
    assert_eq!(state.bossbars[0].value, 1);
    run(&mut state, "execute store success bossbar minecraft:bar max run give Steve minecraft:stone 42").unwrap();
    assert_eq!(state.bossbars[0].max, 1);
}

#[test]
fn store_into_storage_nbt_scales_and_narrows_like_java_casts() {
    let mut state = steve_state();
    add_objective(&mut state, "obj");
    set_score(&mut state, "Steve", "obj", 300);
    // The stored value is the numeric result of the command: `scoreboard players get`.
    run(&mut state, "execute store result storage minecraft:test n int 2.5 run scoreboard players get Steve obj").unwrap();
    run(&mut state, "execute store result storage minecraft:test b byte 1 run scoreboard players get Steve obj").unwrap();
    run(&mut state, "execute store result storage minecraft:test s short 1000 run scoreboard players get Steve obj").unwrap();
    run(&mut state, "execute store result storage minecraft:test d double 0.5 run scoreboard players get Steve obj").unwrap();
    let nbt = &state
        .macro_storage_nbt_sources
        .iter()
        .find(|s| s.id == "minecraft:test")
        .unwrap()
        .nbt;
    let Tag::Compound(fields) = nbt else { panic!("compound") };
    let get = |name: &str| fields.iter().find(|(k, _)| k == name).unwrap().1.clone();
    assert_eq!(get("n"), Tag::Int(750));
    assert_eq!(get("b"), Tag::Byte(300_i32 as i8));
    assert_eq!(get("s"), Tag::Short(300_000_i32 as i16));
    assert_eq!(get("d"), Tag::Double(150.0));
}

#[test]
fn store_into_player_entity_nbt_is_silently_ignored() {
    let mut state = steve_state();
    add_objective(&mut state, "obj");
    set_score(&mut state, "Steve", "obj", 5);
    run(&mut state, "execute store result entity Steve foo int 1 run scoreboard players get Steve obj").unwrap();
    assert!(state.macro_entity_nbt_sources.is_empty());
}

#[test]
fn store_into_block_nbt_requires_a_block_entity() {
    let mut state = steve_state();
    add_objective(&mut state, "obj");
    set_score(&mut state, "Steve", "obj", 5);
    assert_eq!(
        run(&mut state, "execute store result block 0 64 0 foo int 1 run scoreboard players get Steve obj"),
        Err(CommandError::Translatable {
            key: "commands.data.block.invalid",
            args: Vec::new(),
        })
    );
    state.macro_block_nbt_sources.push(CommandBlockNbtSource {
        pos: BlockPos { x: 0, y: 64, z: 0 },
        nbt: Tag::Compound(Vec::new()),
    });
    run(&mut state, "execute store result block 0 64 0 foo int 1 run scoreboard players get Steve obj").unwrap();
    assert_eq!(
        state.macro_block_nbt_sources[0].nbt,
        Tag::Compound(vec![("foo".to_string(), Tag::Int(5))])
    );
}

#[test]
fn conditions_filter_forks_silently() {
    let mut state = steve_state();
    let result = run(&mut state, "execute if entity @e[type=minecraft:zombie] run say hi").unwrap();
    assert_eq!(result.success_count, 0);
    assert_eq!(result.feedback_key, NO_COMMAND_FEEDBACK);
    assert!(state.chat_events.is_empty());
    let result = run(&mut state, "execute unless entity @e[type=minecraft:zombie] run say hi").unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(state.chat_events.len(), 1);
}

#[test]
fn terminal_conditions_report_pass_and_fail_messages() {
    let mut state = steve_state();
    let pass = run(&mut state, "execute if entity @a").unwrap();
    assert_eq!(pass.success_count, 2);
    assert_eq!(pass.feedback_key, "commands.execute.conditional.pass_count");
    assert_eq!(state.feedback_args, vec!["2".to_string()]);
    assert!(!pass.broadcast_to_admins);

    assert_eq!(
        run(&mut state, "execute if entity @e[type=minecraft:zombie]"),
        Err(CommandError::ExecuteConditionFailed)
    );

    state.feedback_args.clear();
    let unless = run(&mut state, "execute unless entity @e[type=minecraft:zombie]").unwrap();
    assert_eq!(unless.success_count, 1);
    assert_eq!(unless.feedback_key, "commands.execute.conditional.pass");
    assert!(state.feedback_args.is_empty());

    assert_eq!(
        run(&mut state, "execute unless entity @a"),
        Err(CommandError::Translatable {
            key: "commands.execute.conditional.fail_count",
            args: vec!["2".to_string()],
        })
    );
}

#[test]
fn if_score_compares_and_matches() {
    let mut state = steve_state();
    add_objective(&mut state, "a");
    add_objective(&mut state, "b");
    set_score(&mut state, "Steve", "a", 5);
    set_score(&mut state, "Alex", "b", 8);
    for (op, expected) in [("=", false), ("<", true), ("<=", true), (">", false), (">=", false)] {
        let result = run(&mut state, &format!("execute if score Steve a {op} Alex b"));
        assert_eq!(result.is_ok(), expected, "operator {op}");
    }
    assert!(run(&mut state, "execute if score Steve a matches 3..6").is_ok());
    assert!(run(&mut state, "execute if score Steve a matches ..4").is_err());
    assert!(run(&mut state, "execute if score Steve a matches 5").is_ok());
    // A missing score never matches, in either polarity of the comparison.
    assert!(run(&mut state, "execute if score Nobody a matches ..100").is_err());
    assert!(run(&mut state, "execute unless score Nobody a matches ..100").is_ok());
    assert_eq!(
        run(&mut state, "execute if score Steve missing matches 1"),
        Err(CommandError::ScoreboardObjectiveNotFound)
    );
}

#[test]
fn if_block_matches_id_tag_properties_and_nbt() {
    let mut state = steve_state();
    state.blocks.push(BlockStateEntry {
        dimension: "minecraft:overworld".to_string(),
        position: BlockPos { x: 1, y: 2, z: 3 },
        block: "minecraft:oak_stairs[facing=north]".to_string(),
    });
    assert!(run(&mut state, "execute if block 1 2 3 minecraft:oak_stairs").is_ok());
    assert!(run(&mut state, "execute if block 1 2 3 oak_stairs[facing=north]").is_ok());
    assert!(run(&mut state, "execute if block 1 2 3 oak_stairs[facing=south]").is_err());
    assert!(run(&mut state, "execute if block 1 2 3 #minecraft:stairs").is_ok());
    assert!(run(&mut state, "execute if block 1 2 3 #minecraft:logs").is_err());
    assert!(run(&mut state, "execute if block 1 2 3 stone").is_err());
    assert!(run(&mut state, "execute unless block 4 4 4 stone").is_ok());
    // Air is the default block; unknown ids are a syntax error before anything runs.
    assert!(run(&mut state, "execute if block 9 9 9 air").is_ok());
    assert_eq!(
        run(&mut state, "execute if block 1 2 3 not_a_block"),
        Err(CommandError::InvalidSyntax)
    );
    state.macro_block_nbt_sources.push(CommandBlockNbtSource {
        pos: BlockPos { x: 1, y: 2, z: 3 },
        nbt: Tag::Compound(vec![("x".to_string(), Tag::Int(4))]),
    });
    assert!(run(&mut state, "execute if block 1 2 3 oak_stairs{x:4}").is_ok());
    assert!(run(&mut state, "execute if block 1 2 3 oak_stairs{x:5}").is_err());
}

#[test]
fn if_block_reports_unloaded_chunks_as_a_forked_silent_failure() {
    let mut state = steve_state();
    state.loaded_chunks = Some(vec![LoadedChunk {
        dimension: "minecraft:overworld".to_string(),
        chunk_x: 0,
        chunk_z: 0,
        entity_ticking: true,
    }]);
    assert!(run(&mut state, "execute if block 1 2 3 air").is_ok());
    // The terminal (non-forked) form surfaces the error message.
    assert_eq!(
        run(&mut state, "execute if block 100 2 3 air"),
        Err(CommandError::Translatable {
            key: "argument.pos.unloaded",
            args: Vec::new(),
        })
    );
    assert!(run(&mut state, "execute if loaded 1 2 3").is_ok());
    assert!(run(&mut state, "execute if loaded 100 2 3").is_err());
    state.loaded_chunks.as_mut().unwrap()[0].entity_ticking = false;
    assert!(run(&mut state, "execute if loaded 1 2 3").is_err());
    // As a modifier the error is swallowed by the fork and simply filters the source.
    let result = run(&mut state, "execute if block 100 2 3 air run say hi").unwrap();
    assert_eq!(result.success_count, 0);
}

#[test]
fn if_blocks_compares_regions_with_all_and_masked() {
    let mut state = steve_state();
    let put = |state: &mut ServerCommandState, x: i32, block: &str| {
        state.blocks.push(BlockStateEntry {
            dimension: "minecraft:overworld".to_string(),
            position: BlockPos { x, y: 0, z: 0 },
            block: block.to_string(),
        });
    };
    put(&mut state, 0, "minecraft:stone");
    put(&mut state, 1, "minecraft:dirt");
    put(&mut state, 10, "minecraft:stone");
    put(&mut state, 11, "minecraft:dirt");
    let pass = run(&mut state, "execute if blocks 0 0 0 1 0 0 10 0 0 all").unwrap();
    assert_eq!(pass.success_count, 2);
    assert_eq!(pass.feedback_key, "commands.execute.conditional.pass_count");
    assert!(run(&mut state, "execute if blocks 0 0 0 1 0 0 20 0 0 all").is_err());
    // Masked skips air in the source region.
    // `masked` skips the air blocks of the source region, so only the two solid blocks count.
    assert_eq!(
        run(&mut state, "execute if blocks 0 0 0 3 0 0 10 0 0 masked").unwrap().success_count,
        2
    );
    assert_eq!(
        run(&mut state, "execute if blocks 0 0 0 3 0 0 11 0 0 all").unwrap_err(),
        CommandError::ExecuteConditionFailed
    );
    assert_eq!(
        run(&mut state, "execute unless blocks 0 0 0 1 0 0 10 0 0 all").unwrap_err(),
        CommandError::Translatable {
            key: "commands.execute.conditional.fail_count",
            args: vec!["2".to_string()],
        }
    );
    assert_eq!(
        run(&mut state, "execute if blocks 0 0 0 100 100 100 0 0 0 all").unwrap_err(),
        CommandError::Translatable {
            key: "commands.execute.blocks.toobig",
            args: vec!["32768".to_string(), "1030301".to_string()],
        }
    );
}

#[test]
fn if_dimension_biome_and_stopwatch() {
    let mut state = steve_state();
    assert!(run(&mut state, "execute if dimension minecraft:overworld").is_ok());
    assert!(run(&mut state, "execute if dimension minecraft:the_nether").is_err());
    assert!(run(&mut state, "execute in minecraft:the_nether if dimension minecraft:the_nether").is_ok());
    state.biomes.push(BiomeEntry {
        dimension: "minecraft:overworld".to_string(),
        position: BlockPos { x: 0, y: 0, z: 0 },
        biome: "minecraft:desert".to_string(),
    });
    assert!(run(&mut state, "execute if biome 1 1 1 minecraft:desert").is_ok());
    assert!(run(&mut state, "execute if biome 1 1 1 minecraft:plains").is_err());
    assert!(run(&mut state, "execute if biome 1 1 1 #minecraft:is_overworld").is_ok());
    state.stopwatches.push(StopwatchState {
        id: "minecraft:sw".to_string(),
        creation_time_millis: 0,
        accumulated_elapsed_millis: 2500,
    });
    assert!(run(&mut state, "execute if stopwatch minecraft:sw 2..3").is_ok());
    assert!(run(&mut state, "execute if stopwatch minecraft:sw 3..").is_err());
    assert_eq!(
        run(&mut state, "execute if stopwatch minecraft:none 0.."),
        Err(CommandError::StopwatchDoesNotExist)
    );
}

#[test]
fn if_data_counts_matching_nodes() {
    let mut state = steve_state();
    state.macro_storage_nbt_sources.push(CommandStorageNbtSource {
        id: "minecraft:s".to_string(),
        nbt: Tag::Compound(vec![(
            "list".to_string(),
            Tag::List(vec![Tag::Int(1), Tag::Int(2), Tag::Int(3)]),
        )]),
    });
    let pass = run(&mut state, "execute if data storage minecraft:s list[]").unwrap();
    assert_eq!(pass.success_count, 3);
    assert!(run(&mut state, "execute if data storage minecraft:s nope").is_err());
    assert!(run(&mut state, "execute unless data storage minecraft:s nope").is_ok());
    // Entity data needs exactly one entity.
    assert_eq!(
        run(&mut state, "execute if data entity @a foo"),
        Err(CommandError::Translatable {
            key: "argument.entity.toomany",
            args: Vec::new(),
        })
    );
}

#[test]
fn if_items_counts_matching_slot_contents() {
    let mut state = steve_state();
    state.entity_item_slots.push(CommandEntityItemSlot {
        entity: entity_ref("Steve"),
        slot: "container.0".to_string(),
        item: Some(CommandItemStack {
            item: "minecraft:stone".to_string(),
            count: 12,
        }),
    });
    state.entity_item_slots.push(CommandEntityItemSlot {
        entity: entity_ref("Steve"),
        slot: "container.1".to_string(),
        item: Some(CommandItemStack {
            item: "minecraft:dirt".to_string(),
            count: 3,
        }),
    });
    let stone = run(&mut state, "execute if items entity Steve container.* minecraft:stone").unwrap();
    assert_eq!(stone.success_count, 12);
    let any = run(&mut state, "execute if items entity Steve container.* *").unwrap();
    assert_eq!(any.success_count, 15);
    assert!(run(&mut state, "execute if items entity Steve container.1 minecraft:stone").is_err());
    state.block_item_slots.push(CommandBlockItemSlot {
        pos: BlockPos { x: 5, y: 5, z: 5 },
        slot: "container.0".to_string(),
        item: Some(CommandItemStack {
            item: "minecraft:stone".to_string(),
            count: 4,
        }),
    });
    let block = run(&mut state, "execute if items block 5 5 5 container.0 minecraft:stone").unwrap();
    assert_eq!(block.success_count, 4);
    assert!(run(&mut state, "execute if items block 6 5 5 container.0 *").is_err());
    // A plain condition needs entities: none found is `argument.entity.notfound.entity`.
    assert_eq!(
        run(&mut state, "execute if items entity @e[type=minecraft:zombie] container.0 *"),
        Err(CommandError::Translatable {
            key: "argument.entity.notfound.entity",
            args: Vec::new(),
        })
    );
}

#[test]
fn if_predicate_evaluates_structural_loot_conditions() {
    let mut state = steve_state();
    state.loot_predicates.push(CommandLootPredicate {
        id: "minecraft:not_raining".to_string(),
        definition: crate::storage::nbt::parse_snbt(
            "{condition:\"minecraft:inverted\",term:{condition:\"minecraft:weather_check\",raining:1b}}",
        )
        .unwrap(),
    });
    assert!(run(&mut state, "execute if predicate minecraft:not_raining").is_ok());
    state.weather.mode = WeatherMode::Rain;
    assert!(run(&mut state, "execute if predicate minecraft:not_raining").is_err());
    assert_eq!(
        run(&mut state, "execute if predicate minecraft:missing"),
        Err(CommandError::Translatable {
            key: "argument.predicate.unknown",
            args: vec!["minecraft:missing".to_string()],
        })
    );
}

fn function(id: &str, commands: &[&str]) -> CommandFunctionDefinition {
    CommandFunctionDefinition {
        id: id.to_string(),
        commands: commands.iter().map(|c| (*c).to_string()).collect(),
        macro_parameters: Vec::new(),
    }
}

#[test]
fn if_function_uses_the_functions_return_value() {
    let mut state = steve_state();
    state.available_functions = vec![
        function("minecraft:ret5", &["say inside", "return 5", "say unreachable"]),
        function("minecraft:ret0", &["return 0"]),
        function("minecraft:fail", &["return fail"]),
        function("minecraft:fall", &["say only"]),
    ];
    assert!(run(&mut state, "execute if function minecraft:ret5 run say ok").is_ok());
    assert_eq!(state.chat_events.len(), 2, "the function ran, then the continuation");
    assert!(state.chat_events.iter().all(|e| !e.message.contains("unreachable")));

    for (name, if_passes) in [
        ("minecraft:ret5", true),
        ("minecraft:ret0", false),
        ("minecraft:fail", false),
        ("minecraft:fall", false),
    ] {
        state.chat_events.clear();
        run(&mut state, &format!("execute if function {name} run say yes")).unwrap();
        let ran_if = state.chat_events.iter().any(|e| e.message == "yes");
        assert_eq!(ran_if, if_passes, "if function {name}");
        state.chat_events.clear();
        run(&mut state, &format!("execute unless function {name} run say yes")).unwrap();
        let ran_unless = state.chat_events.iter().any(|e| e.message == "yes");
        assert_eq!(ran_unless, !if_passes, "unless function {name}");
    }
}

#[test]
fn if_function_errors_and_empty_tags() {
    let mut state = steve_state();
    // The condition forks, so the unknown-function error is swallowed (forked chains never
    // print errors) and nothing runs.
    let unknown = run(&mut state, "execute if function minecraft:unknown run say hi").unwrap();
    assert_eq!(unknown.success_count, 0);
    assert!(state.chat_events.is_empty());
    // `if function` cannot end a command: it has no executes handler.
    assert_eq!(
        run(&mut state, "execute if function minecraft:unknown"),
        Err(CommandError::InvalidSyntax)
    );
    // An empty tag queues nothing, so even `unless` runs nothing.
    state.function_tags.push(CommandFunctionTag {
        id: "minecraft:empty".to_string(),
        functions: Vec::new(),
    });
    let result = run(&mut state, "execute unless function #minecraft:empty run say hi").unwrap();
    assert_eq!(result.success_count, 0);
    assert!(state.chat_events.is_empty());
}

#[test]
fn return_inside_execute_records_events_and_completes_store_callbacks() {
    let mut state = steve_state();
    add_objective(&mut state, "obj");
    run(&mut state, "execute store result score Steve obj run return 7").unwrap();
    assert_eq!(score(&state, "Steve", "obj"), Some(7));
    assert!(matches!(
        state.return_events.last(),
        Some(ReturnCommandEvent::Success { value: 7, .. })
    ));
    run(&mut state, "execute store success score Steve obj run return fail").unwrap();
    assert_eq!(score(&state, "Steve", "obj"), Some(0));
    assert!(matches!(
        state.return_events.last(),
        Some(ReturnCommandEvent::Failure { .. })
    ));
}

#[test]
fn return_run_uses_only_the_first_source_result() {
    let mut state = steve_state();
    run(&mut state, "execute as @a run return run give @s minecraft:stone 1").unwrap();
    assert!(matches!(
        state.return_events.last(),
        Some(ReturnCommandEvent::Success { .. })
    ));
    let returns = state.return_events.len();
    // With no sources the returning frame fails (`FallthroughTask`).
    run(&mut state, "execute as @e[type=minecraft:zombie] run return run say hi").unwrap();
    assert_eq!(state.return_events.len(), returns);
}

#[test]
fn fork_limit_stops_the_chain_with_command_fork_limit() {
    let mut state = steve_state();
    state.game_rules.iter_mut().find(|r| r.name == "max_command_forks").unwrap().value =
        GameRuleValue::Int(2);
    // Two sources fit (`size + new >= limit` fails at 2), so `as @a` itself trips the limit.
    let result = run(&mut state, "execute as @a run say hi").unwrap();
    // Forked mode is silent: the error is not reported and nothing ran.
    assert_eq!(result.success_count, 0);
    assert!(state.chat_events.is_empty());
    // A non-forked stage reports `command.forkLimit`.
    state.game_rules.iter_mut().find(|r| r.name == "max_command_forks").unwrap().value =
        GameRuleValue::Int(1);
    assert_eq!(
        run(&mut state, "execute positioned 1 1 1 run say hi"),
        Err(CommandError::Translatable {
            key: "command.forkLimit",
            args: vec!["1".to_string()],
        })
    );
}

#[test]
fn sequence_length_rule_bounds_the_number_of_executions() {
    let mut state = steve_state();
    state.game_rules.iter_mut().find(|r| r.name == "max_command_sequence_length").unwrap().value =
        GameRuleValue::Int(2);
    // Cost: one for the `as` modifier stage plus one per executed command: the second
    // command hits the empty quota.
    run(&mut state, "execute as @a run say hi").unwrap();
    assert_eq!(state.chat_events.len(), 1);
}

#[test]
fn execute_restores_the_original_command_source() {
    let mut state = steve_state();
    run(&mut state, "execute as Alex at Alex positioned 5 5 5 in minecraft:the_end run say hi").unwrap();
    assert_eq!(state.command_source_entity, Some(entity_ref("Steve")));
    assert_eq!(state.command_source_position.x, 1.0);
    assert_eq!(state.command_source_dimension, "minecraft:overworld");
    assert!(!state.execution.active);
}

#[test]
fn syntax_errors_are_reported_before_anything_runs() {
    let mut state = steve_state();
    for bad in [
        "execute",
        "execute run",
        "execute as",
        "execute as @a",
        "execute if",
        "execute if score Steve",
        "execute store result score Steve",
        "execute bogus run say hi",
        "execute positioned 1 2",
        "execute anchored nose run say hi",
        "execute run execute",
    ] {
        assert_eq!(run(&mut state, bad), Err(CommandError::InvalidSyntax), "{bad}");
    }
    assert!(state.chat_events.is_empty());
}

#[test]
fn positioned_over_sets_y_to_the_heightmap() {
    let mut state = steve_state();
    for y in 0..5 {
        state.blocks.push(BlockStateEntry {
            dimension: "minecraft:overworld".to_string(),
            position: BlockPos { x: 1, y, z: 1 },
            block: "minecraft:stone".to_string(),
        });
    }
    run(&mut state, "execute positioned over world_surface run say hi").unwrap();
    assert_eq!(last_source(&state).position.y, 5.0);
    run(&mut state, "execute positioned over motion_blocking run say hi").unwrap();
    assert_eq!(last_source(&state).position.y, 5.0);
    // An empty column reports the dimension's minimum build height.
    state.command_source_position = Vec3 { x: 50.0, y: 0.0, z: 50.0 };
    run(&mut state, "execute positioned over ocean_floor run say hi").unwrap();
    assert_eq!(last_source(&state).position.y, -64.0);
    assert_eq!(
        run(&mut state, "execute positioned over bogus run say hi"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn nested_run_execute_flattens_into_one_chain() {
    let mut state = steve_state();
    run(&mut state, "execute as @a run execute at @s run say hi").unwrap();
    assert_eq!(state.chat_events.len(), 2);
    assert_eq!(state.execute_events.len(), 1);
}

#[test]
fn selectors_filter_by_tag_score_and_type() {
    let mut state = steve_state();
    add_objective(&mut state, "obj");
    set_score(&mut state, "Alex", "obj", 3);
    state.entity_tags.push(EntityTags {
        entity: entity_ref("Steve"),
        tags: vec!["red".to_string()],
    });
    run(&mut state, "execute as @a[tag=red] run say hi").unwrap();
    assert_eq!(state.chat_events.len(), 1);
    state.chat_events.clear();
    run(&mut state, "execute as @a[scores={obj=1..}] run say hi").unwrap();
    assert_eq!(state.chat_events.len(), 1);
    assert_eq!(
        state.chat_events[0].sender.as_ref().unwrap().name,
        "Alex"
    );
}

mod math {
    use crate::command::execute_math::*;

    #[test]
    fn mth_atan2_tracks_libm_in_every_quadrant() {
        for (y, x) in [
            (0.0, 1.0),
            (1.0, 1.0),
            (1.0, 0.0),
            (1.0, -1.0),
            (0.0, -1.0),
            (-1.0, -1.0),
            (-1.0, 0.0),
            (-1.0, 1.0),
            (3.0, 4.0),
            (-12.5, 0.25),
        ] {
            let expected = f64::atan2(y, x);
            assert!(
                (mth_atan2(y, x) - expected).abs() < 1e-4, // Mth.atan2 is a table + fastInvSqrt approximation
                "atan2({y}, {x}): {} vs {expected}",
                mth_atan2(y, x)
            );
        }
        assert!(mth_atan2(f64::NAN, 1.0).is_nan());
    }

    #[test]
    fn mth_sin_and_cos_use_the_65536_entry_table() {
        assert_eq!(mth_sin(0.0), 0.0);
        assert_eq!(mth_cos(0.0), 1.0);
        assert!((mth_sin(std::f64::consts::FRAC_PI_2) - 1.0).abs() < 1e-6);
        assert!((mth_cos(std::f64::consts::PI) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn wrap_degrees_matches_mth_bounds() {
        assert_eq!(mth_wrap_degrees(180.0), -180.0);
        assert_eq!(mth_wrap_degrees(-180.0), -180.0);
        assert_eq!(mth_wrap_degrees(190.0), -170.0);
        assert_eq!(mth_wrap_degrees(-190.0), 170.0);
        assert_eq!(mth_wrap_degrees(720.0), 0.0);
    }

    #[test]
    fn local_coordinates_follow_yaw_and_pitch() {
        // Yaw 90 faces -x: one block forwards is (-1, 0, 0).
        let ahead = apply_local_coordinates_to_rotation(0.0, 90.0, 0.0, 0.0, 1.0);
        assert!((ahead.x + 1.0).abs() < 1e-5 && ahead.y.abs() < 1e-5 && ahead.z.abs() < 1e-5);
        // Pitch -90 looks straight up.
        let up = apply_local_coordinates_to_rotation(-90.0, 0.0, 0.0, 0.0, 1.0);
        assert!((up.y - 1.0).abs() < 1e-5);
        // Left of a south-facing (yaw 0) source is +x.
        let left = apply_local_coordinates_to_rotation(0.0, 0.0, 1.0, 0.0, 0.0);
        assert!((left.x - 1.0).abs() < 1e-5);
    }
}
