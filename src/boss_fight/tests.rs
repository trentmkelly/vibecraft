use super::*;

#[test]
fn dragon_phase_ids_parts_and_attributes_match_decompiled_boss_sources() {
    assert_eq!(DRAGON_PHASES.len(), 11);
    assert_eq!(dragon_phase_by_id(0).name, "HoldingPattern");
    assert_eq!(dragon_phase_by_id(5).name, "SittingFlaming");
    assert!(dragon_phase_by_id(6).sitting);
    assert_eq!(dragon_phase_by_id(999).name, "HoldingPattern");

    assert_eq!(DRAGON_PARTS.len(), 8);
    assert_eq!(
        DRAGON_PARTS[0],
        DragonPartDef {
            name: "head",
            width: 1.0,
            height: 1.0
        }
    );
    assert_eq!(
        DRAGON_PARTS[2],
        DragonPartDef {
            name: "body",
            width: 5.0,
            height: 3.0
        }
    );
    assert_eq!(
        DRAGON_PARTS[6],
        DragonPartDef {
            name: "wing",
            width: 4.0,
            height: 2.0
        }
    );
    assert_eq!(ENDER_DRAGON_MAX_HEALTH, 200.0);
    assert_eq!(ENDER_DRAGON_CAMERA_DISTANCE, 16.0);
    assert_eq!(ENDER_DRAGON_SITTING_ALLOWED_DAMAGE_PERCENTAGE, 0.25);
}

#[test]
fn dragon_fight_tick_plan_tracks_players_tickets_scans_and_bossbar() {
    assert_eq!(DRAGON_FIGHT_PLAYER_SCAN_INTERVAL, 20);
    assert_eq!(DRAGON_FIGHT_CRYSTAL_SCAN_INTERVAL, 100);
    assert_eq!(DRAGON_FIGHT_MAX_TICKS_BEFORE_RESPAWN, 1_200);
    assert_eq!(DRAGON_FIGHT_ARENA_TICKET_LEVEL, 9);
    assert_eq!(DRAGON_GATEWAY_COUNT, 20);
    assert_eq!(DRAGON_GATEWAY_DISTANCE, 96);
    assert_eq!(DRAGON_SPAWN_Y, 128);

    let empty = dragon_fight_tick_plan(DragonFightState {
        ticks_since_last_player_scan: 19,
        ticks_since_dragon_seen: 1_199,
        ticks_since_crystals_scanned: 99,
        dragon_killed: false,
        players_tracking_fight: 0,
        arena_loaded: true,
        needs_state_scanning: true,
        respawn_stage_active: true,
    });
    assert!(empty.scan_players);
    assert!(empty.remove_arena_ticket);
    assert!(!empty.add_arena_ticket);
    assert!(!empty.find_or_create_dragon);

    let active = dragon_fight_tick_plan(DragonFightState {
        players_tracking_fight: 1,
        ..DragonFightState {
            ticks_since_last_player_scan: 19,
            ticks_since_dragon_seen: 1_199,
            ticks_since_crystals_scanned: 99,
            dragon_killed: false,
            players_tracking_fight: 0,
            arena_loaded: true,
            needs_state_scanning: true,
            respawn_stage_active: true,
        }
    });
    assert!(active.add_arena_ticket);
    assert!(active.scan_legacy_state);
    assert!(active.tick_respawn_stage);
    assert!(active.find_or_create_dragon);
    assert!(active.scan_crystals);
}

#[test]
fn dragon_bossbar_death_respawn_and_crystal_rules_are_visible_to_clients() {
    assert_eq!(
        BossBarState::ender_dragon(false, 100.0),
        BossBarState {
            visible: true,
            progress: 0.5,
            color: BossBarColor::Pink,
            overlay: BossBarOverlay::Progress,
            play_music: true,
            darken_screen: false,
            create_world_fog: true,
        }
    );
    assert!(!BossBarState::ender_dragon(true, 0.0).visible);

    assert_eq!(DRAGON_RESPAWN_STAGES.len(), 5);
    assert_eq!(dragon_death_plan(179), vec![]);
    assert_eq!(dragon_death_plan(180), vec![DragonDeathPlan::Explode]);
    assert_eq!(
        dragon_death_plan(200),
        vec![
            DragonDeathPlan::Explode,
            DragonDeathPlan::DropExperience,
            DragonDeathPlan::SpawnExitPortalAndGateway,
            DragonDeathPlan::Remove,
        ]
    );

    assert_eq!(
        end_crystal_damage_result(true, false, false),
        EndCrystalDamageResult::IgnoredDragon
    );
    assert_eq!(
        end_crystal_damage_result(false, false, true),
        EndCrystalDamageResult::RemovedOnly
    );
    assert_eq!(
        end_crystal_damage_result(false, false, false),
        EndCrystalDamageResult::ExplodeAndNotifyFight
    );
}

#[test]
fn wither_attributes_invulnerability_bossbar_and_healing_match_vanilla() {
    assert_eq!(WITHER_MAX_HEALTH, 300.0);
    assert_eq!(WITHER_MOVEMENT_SPEED, 0.6);
    assert_eq!(WITHER_FLYING_SPEED, 0.6);
    assert_eq!(WITHER_FOLLOW_RANGE, 40.0);
    assert_eq!(WITHER_ARMOR, 4.0);
    assert_eq!(WITHER_XP_REWARD, 50);
    assert_eq!(make_wither_invulnerable_health(WITHER_MAX_HEALTH), 100.0);
    assert!(wither_is_powered(150.0, 300.0));
    assert!(!wither_is_powered(151.0, 300.0));

    let spawning = wither_ai_tick(10, 220, 0, true, 100.0);
    assert_eq!(spawning.invulnerable_ticks, 219);
    assert_eq!(spawning.heal_amount, 10.0);
    assert!(!spawning.explode);
    assert_eq!(
        spawning.bossbar_progress,
        BossBarState::wither(100.0, 219).progress
    );

    let explode = wither_ai_tick(20, 1, 0, true, 100.0);
    assert!(explode.explode);
    assert_eq!(explode.invulnerable_ticks, 0);

    let normal = wither_ai_tick(20, 0, 1, true, 150.0);
    assert_eq!(normal.heal_amount, 1.0);
    assert!(normal.destroy_blocks_now);
    assert_eq!(normal.bossbar_progress, 0.5);

    let bossbar = BossBarState::wither(150.0, 0);
    assert!(bossbar.visible);
    assert_eq!(bossbar.color, BossBarColor::Purple);
    assert_eq!(bossbar.overlay, BossBarOverlay::Progress);
    assert_eq!(bossbar.progress, 0.5);
}

#[test]
fn wither_damage_immunities_powered_projectile_gate_and_block_destroy_timer_match_vanilla() {
    assert_eq!(
        wither_damage_result(10, false, 0, WitherDamageSourceKind::Generic),
        WitherDamageResult::Ignored
    );
    assert_eq!(
        wither_damage_result(
            10,
            false,
            0,
            WitherDamageSourceKind::BypassesInvulnerability
        ),
        WitherDamageResult::Accepted {
            schedule_block_destroy: true
        }
    );
    assert_eq!(
        wither_damage_result(0, true, 0, WitherDamageSourceKind::Arrow),
        WitherDamageResult::Ignored
    );
    assert_eq!(
        wither_damage_result(0, false, 20, WitherDamageSourceKind::Generic),
        WitherDamageResult::Accepted {
            schedule_block_destroy: false
        }
    );
    assert_eq!(
        wither_damage_result(0, false, 0, WitherDamageSourceKind::WitherFriend),
        WitherDamageResult::Ignored
    );
}

#[test]
fn wither_block_destroy_delay_scan_volume_and_filters_match_java() {
    assert!(!wither_can_destroy_block(true, false));
    assert!(!wither_can_destroy_block(false, true));
    assert!(wither_can_destroy_block(false, false));
    assert_eq!(wither_block_destroy_scan_volume(0.9, 3.5), (1, 3));

    assert_eq!(
        wither_block_destroy_plan(20, true, false),
        WitherBlockDestroyPlan {
            next_destroy_blocks_tick: 19,
            should_scan_blocks: false,
            emit_level_event_1022: false,
        }
    );
    assert_eq!(
        wither_block_destroy_plan(1, false, true),
        WitherBlockDestroyPlan {
            next_destroy_blocks_tick: 0,
            should_scan_blocks: false,
            emit_level_event_1022: false,
        }
    );
    assert_eq!(
        wither_block_destroy_plan(1, true, false),
        WitherBlockDestroyPlan {
            next_destroy_blocks_tick: 0,
            should_scan_blocks: true,
            emit_level_event_1022: false,
        }
    );
    assert_eq!(
        wither_block_destroy_plan(1, true, true),
        WitherBlockDestroyPlan {
            next_destroy_blocks_tick: 0,
            should_scan_blocks: true,
            emit_level_event_1022: true,
        }
    );
}

#[test]
fn wither_summoning_precondition_matches_skull_block_pattern() {
    let x_axis = |x, y, z| match (x, y, z) {
        (-1, 2, 0) | (0, 2, 0) | (1, 2, 0) => "minecraft:wither_skeleton_skull",
        (-1, 1, 0) | (0, 1, 0) | (1, 1, 0) => "minecraft:soul_sand",
        (0, 0, 0) => "minecraft:soul_soil",
        _ => "minecraft:air",
    };
    assert_eq!(
        wither_summon_precondition_allows_item(
            WitherSummonPrecondition {
                item: "minecraft:wither_skeleton_skull",
                placed_y: 64,
                min_y: -64,
                peaceful: false,
                client_side: false,
            },
            x_axis,
        ),
        Some(WitherSummonPlan {
            axis: WitherSummonAxis::X,
            spawn_offset: (0, 1, 0),
            yaw_degrees: 0,
            invulnerable_ticks: WITHER_INVULNERABLE_TICKS,
        })
    );

    let z_axis = |x, y, z| match (x, y, z) {
        (0, 2, -1) | (0, 2, 0) | (0, 2, 1) => "minecraft:wither_skeleton_wall_skull",
        (0, 1, -1) | (0, 1, 0) | (0, 1, 1) => "minecraft:soul_soil",
        (0, 0, 0) => "minecraft:soul_sand",
        _ => "minecraft:air",
    };
    assert_eq!(wither_summon_full_pattern(z_axis).unwrap().yaw_degrees, 90);

    for blocked in [
        WitherSummonPrecondition {
            item: "minecraft:skeleton_skull",
            placed_y: 64,
            min_y: -64,
            peaceful: false,
            client_side: false,
        },
        WitherSummonPrecondition {
            item: "minecraft:wither_skeleton_skull",
            placed_y: -63,
            min_y: -64,
            peaceful: false,
            client_side: false,
        },
        WitherSummonPrecondition {
            item: "minecraft:wither_skeleton_skull",
            placed_y: 64,
            min_y: -64,
            peaceful: true,
            client_side: false,
        },
        WitherSummonPrecondition {
            item: "minecraft:wither_skeleton_skull",
            placed_y: 64,
            min_y: -64,
            peaceful: false,
            client_side: true,
        },
    ] {
        assert_eq!(
            wither_summon_precondition_allows_item(blocked, x_axis),
            None
        );
    }

    let obstructed_air_slot = |x, y, z| match (x, y, z) {
        (-1, 0, 0) => "minecraft:stone",
        (-1, 2, 0) | (0, 2, 0) | (1, 2, 0) => "minecraft:wither_skeleton_skull",
        (-1, 1, 0) | (0, 1, 0) | (1, 1, 0) => "minecraft:soul_sand",
        (0, 0, 0) => "minecraft:soul_soil",
        _ => "minecraft:air",
    };
    assert_eq!(wither_summon_full_pattern(obstructed_air_slot), None);
}

#[test]
fn wither_death_drop_matches_java_nether_star_behavior() {
    assert_eq!(
        wither_death_loot_plan(),
        WitherDeathLootPlan {
            drop_item: "minecraft:nether_star",
            extended_lifetime: true,
            death_explosion: false,
            bossbar_visible_after_removal: false,
        }
    );
    assert_eq!(WITHER_SPAWN_EXPLOSION_POWER, 7.0);
}

#[test]
fn wither_powered_target_movement_matches_java_ai_step_gate() {
    let unpowered_above_target = wither_target_movement_step(WitherTargetMovementInput {
        powered: false,
        has_main_target: true,
        self_y: 68.0,
        target_y: 64.0,
        target_dx: 4.0,
        target_dz: 0.0,
        delta: Vec3Plan {
            x: 0.0,
            y: -0.2,
            z: 0.0,
        },
    });
    assert!((unpowered_above_target.y - 0.3).abs() < 0.0001);
    assert!((unpowered_above_target.x - 0.3).abs() < 0.0001);
    assert_eq!(wither_yaw_from_delta(unpowered_above_target), Some(-90.0));

    let powered_above_target = wither_target_movement_step(WitherTargetMovementInput {
        powered: true,
        has_main_target: true,
        self_y: 68.0,
        target_y: 64.0,
        target_dx: 4.0,
        target_dz: 0.0,
        delta: Vec3Plan {
            x: 0.0,
            y: -0.2,
            z: 0.0,
        },
    });
    assert!((powered_above_target.y - -0.12).abs() < 0.0001);
    assert!((powered_above_target.x - 0.3).abs() < 0.0001);

    let powered_below_target = wither_target_movement_step(WitherTargetMovementInput {
        powered: true,
        has_main_target: true,
        self_y: 63.0,
        target_y: 64.0,
        target_dx: 0.0,
        target_dz: 2.0,
        delta: Vec3Plan {
            x: 0.0,
            y: -0.2,
            z: 0.0,
        },
    });
    assert!((powered_below_target.y - 0.3).abs() < 0.0001);
    assert_eq!(powered_below_target.z, 0.0);
    assert_eq!(wither_yaw_from_delta(powered_below_target), None);

    let no_target = wither_target_movement_step(WitherTargetMovementInput {
        powered: false,
        has_main_target: false,
        self_y: 60.0,
        target_y: 80.0,
        target_dx: 20.0,
        target_dz: 20.0,
        delta: Vec3Plan {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    });
    assert_eq!(
        no_target,
        Vec3Plan {
            x: 1.0,
            y: 0.6,
            z: 1.0
        }
    );
}

#[test]
fn wither_skull_launch_head_offsets_direction_and_dangerous_roll_match_java() {
    let main = wither_skull_launch_plan(
        0,
        Vec3Plan {
            x: 10.0,
            y: 64.0,
            z: -5.0,
        },
        90.0,
        1.0,
        Vec3Plan {
            x: 10.0,
            y: 67.0,
            z: -1.0,
        },
        0.0005,
        false,
    );
    assert_eq!(
        main.origin,
        Vec3Plan {
            x: 10.0,
            y: 67.0,
            z: -5.0
        }
    );
    assert_eq!(
        main.direction,
        Vec3Plan {
            x: 0.0,
            y: 0.0,
            z: 1.0
        }
    );
    assert!(main.dangerous);
    assert_eq!(main.level_event, Some(1024));

    let side = wither_skull_launch_plan(
        2,
        Vec3Plan {
            x: 10.0,
            y: 64.0,
            z: -5.0,
        },
        90.0,
        1.0,
        Vec3Plan {
            x: 10.0,
            y: 66.2,
            z: -3.7,
        },
        0.0,
        true,
    );
    assert!((side.origin.x - 10.0).abs() < 0.0001);
    assert!((side.origin.y - 66.2).abs() < 0.0001);
    assert!((side.origin.z - -6.3).abs() < 0.0001);
    assert!(!side.dangerous);
    assert_eq!(side.level_event, None);

    let side_head_one = wither_head_position(
        1,
        Vec3Plan {
            x: 10.0,
            y: 64.0,
            z: -5.0,
        },
        90.0,
        1.0,
    );
    assert!((side_head_one.z - -3.7).abs() < 0.0001);
}

#[test]
fn wither_shield_threshold_invulnerability_and_self_heal_rate_match_java() {
    assert!(!wither_is_powered(151.0, WITHER_MAX_HEALTH));
    assert!(wither_is_powered(150.0, WITHER_MAX_HEALTH));

    assert_eq!(
        wither_damage_result(0, true, 0, WitherDamageSourceKind::Arrow),
        WitherDamageResult::Ignored
    );
    assert_eq!(
        wither_damage_result(0, true, 0, WitherDamageSourceKind::WindCharge),
        WitherDamageResult::Ignored
    );
    assert_eq!(
        wither_damage_result(0, true, 0, WitherDamageSourceKind::Generic),
        WitherDamageResult::Accepted {
            schedule_block_destroy: true
        }
    );

    // WitherBoss#customServerAiStep heals 10 every 10 ticks while invulnerable,
    // then 1 every 20 ticks after the spawn sequence completes.
    assert_eq!(wither_ai_tick(10, 220, 0, true, 100.0).heal_amount, 10.0);
    assert_eq!(wither_ai_tick(11, 220, 0, true, 100.0).heal_amount, 0.0);
    assert_eq!(wither_ai_tick(20, 0, 0, true, 150.0).heal_amount, 1.0);
    assert_eq!(wither_ai_tick(21, 0, 0, true, 150.0).heal_amount, 0.0);
}

#[test]
fn wither_head_target_assignment_and_skull_delay_rules_match_java() {
    assert_eq!(
        wither_main_head_sync(Some(42)),
        WitherMainHeadSync {
            alternative_target_head: 0,
            target_entity_id: 42
        }
    );
    assert_eq!(wither_main_head_sync(None).target_entity_id, 0);
    assert_eq!(WITHER_BLUE_SKULL_CHANCE, 0.001);

    assert_eq!(
        wither_alt_head_tick_action(WitherAltHeadTickContext {
            head: 1,
            tick_count: 9,
            next_head_update: 10,
            difficulty_allows_idle_attack: true,
            idle_head_updates: 16,
            random_0_to_9: 3,
            nearby_targets_available: true,
            ..WitherAltHeadTickContext::default()
        }),
        WitherHeadTickAction::Wait
    );
    assert_eq!(
        wither_alt_head_tick_action(WitherAltHeadTickContext {
            head: 1,
            tick_count: 10,
            next_head_update: 10,
            difficulty_allows_idle_attack: true,
            idle_head_updates: 16,
            random_0_to_9: 3,
            ..WitherAltHeadTickContext::default()
        }),
        WitherHeadTickAction::FireIdleBlueSkull {
            head: 1,
            next_update_delay: 13,
            reset_idle_updates: true
        }
    );
    assert_eq!(
        wither_alt_head_tick_action(WitherAltHeadTickContext {
            head: 2,
            tick_count: 20,
            next_head_update: 20,
            difficulty_allows_idle_attack: true,
            random_0_to_9: 7,
            alternative_target_entity_id: Some(99),
            current_target_valid: true,
            current_target_distance_sqr: WITHER_HEAD_TARGET_RANGE_SQUARED,
            current_target_line_of_sight: true,
            ..WitherAltHeadTickContext::default()
        }),
        WitherHeadTickAction::FireAtCurrentTarget {
            head: 2,
            dangerous: false,
            next_update_delay: 47,
            reset_idle_updates: true
        }
    );
    assert_eq!(
        wither_alt_head_tick_action(WitherAltHeadTickContext {
            head: 2,
            tick_count: 20,
            next_head_update: 20,
            difficulty_allows_idle_attack: true,
            random_0_to_9: 7,
            alternative_target_entity_id: Some(99),
            current_target_valid: true,
            current_target_distance_sqr: WITHER_HEAD_TARGET_RANGE_SQUARED + 1.0,
            current_target_line_of_sight: true,
            ..WitherAltHeadTickContext::default()
        }),
        WitherHeadTickAction::ClearInvalidTarget { head: 2 }
    );
    assert_eq!(
        wither_alt_head_tick_action(WitherAltHeadTickContext {
            head: 2,
            tick_count: 20,
            next_head_update: 20,
            difficulty_allows_idle_attack: true,
            random_0_to_9: 7,
            nearby_targets_available: true,
            ..WitherAltHeadTickContext::default()
        }),
        WitherHeadTickAction::AcquireNearbyTarget { head: 2 }
    );
}

#[test]
fn boss_fight_checklist_surface_is_covered_by_source_families() {
    let covered: Vec<&str> = BOSS_FIGHT_COVERAGE
        .iter()
        .flat_map(|entry| entry.covered_rules.iter().copied())
        .collect();
    for expected in [
        "bossbar",
        "dragon phases",
        "end fight state",
        "gateway constants",
        "end crystals",
        "invulnerability",
        "spawn explosion",
        "block destruction",
        "ranged skulls",
        "damage immunity",
    ] {
        assert!(covered.contains(&expected), "missing {expected}");
    }
}
