use super::*;

#[test]
fn sculk_sensor_block_entity_tracks_vibration_phase_frequency_and_power() {
    assert_eq!(SculkSensorBlockEntity::LISTENER_RADIUS, 8);
    assert_eq!(SculkSensorBlockEntity::DEFAULT_LAST_VIBRATION_FREQUENCY, 0);
    assert_sculk_sensor_immediate_activation_and_cooldown();
    assert_sculk_sensor_delayed_vibration_and_persistence();
}

fn assert_sculk_sensor_immediate_activation_and_cooldown() {
    let mut sensor = SculkSensorBlockEntity::new();
    assert!(sensor.can_receive_vibration("minecraft:step", true));
    assert!(!sensor.can_receive_vibration("minecraft:unknown", true));
    assert!(!sensor.can_receive_vibration("minecraft:step", false));

    assert_eq!(
        sensor.receive_vibration("minecraft:block_place", 3.2),
        Some(SculkSensorTickResult::Activate {
            frequency: 13,
            redstone: 9,
        })
    );
    assert_eq!(sensor.last_vibration_frequency, 13);
    assert_eq!(sensor.power, 9);
    assert_eq!(sensor.phase, SculkSensorPhase::VibrationDone);
    assert!(!sensor.can_receive_vibration("minecraft:step", true));
    for _ in 0..SculkSensorBlockEntity::ACTIVE_TICKS - 1 {
        assert_eq!(sensor.tick(0), SculkSensorTickResult::None);
    }
    assert_eq!(sensor.tick(0), SculkSensorTickResult::Cooldown);
    assert_eq!(sensor.phase, SculkSensorPhase::Cooldown);
    for _ in 0..SculkSensorBlockEntity::COOLDOWN_TICKS - 1 {
        assert_eq!(sensor.tick(0), SculkSensorTickResult::None);
    }
    assert_eq!(sensor.tick(0), SculkSensorTickResult::Deactivate);
    assert_eq!(sensor.phase, SculkSensorPhase::Listening);
    assert_eq!(sensor.power, 0);
}

fn assert_sculk_sensor_delayed_vibration_and_persistence() {
    let mut delayed = SculkSensorBlockEntity::new();
    let Some(event) = crate::game_event::game_event_by_id("minecraft:entity_damage") else {
        panic!("minecraft:entity_damage should be registered for sculk sensor coverage");
    };
    assert!(delayed.queue_vibration(
        VibrationInfo {
            event,
            distance: 2.9,
            pos: crate::entity_physics::Vec3::ZERO,
            source_entity: Some("zombie".to_string()),
            projectile_owner: None,
        },
        5,
    ));
    assert_eq!(delayed.tick(5), SculkSensorTickResult::None);
    assert_eq!(
        delayed.tick(6),
        SculkSensorTickResult::Particle {
            travel_time_in_ticks: 2
        }
    );
    assert_eq!(delayed.phase, SculkSensorPhase::Ticking);
    assert_eq!(delayed.tick(7), SculkSensorTickResult::None);
    assert_eq!(
        delayed.tick(8),
        SculkSensorTickResult::Activate {
            frequency: 7,
            redstone: 15,
        }
    );
    assert_eq!(delayed.last_vibration_frequency, 7);

    let saved = delayed.save_additional();
    let loaded = SculkSensorBlockEntity::load_additional(&saved);
    assert_eq!(
        loaded.last_vibration_frequency,
        delayed.last_vibration_frequency
    );
    assert_eq!(loaded.phase, delayed.phase);
    assert_eq!(loaded.power, delayed.power);
    assert_eq!(loaded.active_ticks, delayed.active_ticks);
    assert_eq!(delayed.get_update_tag(), saved);
}

#[test]
fn calibrated_sculk_sensor_filters_vibrations_by_back_signal() {
    assert_eq!(CalibratedSculkSensorBlockEntity::LISTENER_RADIUS, 16);

    let mut sensor = CalibratedSculkSensorBlockEntity::new(13);
    assert!(sensor.can_receive_vibration("minecraft:block_place", true));
    assert!(!sensor.can_receive_vibration("minecraft:explode", true));
    assert_eq!(sensor.receive_vibration("minecraft:explode", 4.0), None);
    assert_eq!(
        sensor.receive_vibration("minecraft:block_place", 4.0),
        Some(SculkSensorTickResult::Activate {
            frequency: 13,
            redstone: 12,
        })
    );
    assert_eq!(sensor.sensor.listener_radius, 16);
    assert_eq!(sensor.sensor.last_vibration_frequency, 13);
    assert_eq!(sensor.sensor.power, 12);

    let saved = sensor.save_additional();
    let loaded = CalibratedSculkSensorBlockEntity::load_additional(&saved);
    assert_eq!(loaded.back_signal, 13);
    assert_eq!(loaded.sensor.listener_radius, 16);
    assert_eq!(loaded.sensor.last_vibration_frequency, 13);
    assert_eq!(loaded.sensor.power, 12);

    let mut unfiltered = CalibratedSculkSensorBlockEntity::new(0);
    assert_eq!(
        unfiltered.receive_vibration("minecraft:explode", 4.0),
        Some(SculkSensorTickResult::Activate {
            frequency: 15,
            redstone: 12,
        })
    );
    unfiltered.set_back_signal(99);
    assert_eq!(unfiltered.back_signal, 15);
}

#[test]
fn sculk_catalyst_block_entity_queues_charge_and_pulses_on_mob_death() {
    assert_eq!(SculkCatalystBlockEntity::LISTENER_RADIUS, 8);
    assert_eq!(SculkCatalystBlockEntity::PULSE_TICKS, 8);
    assert_eq!(SculkCatalystBlockEntity::MAX_CURSORS, 32);
    assert_eq!(SculkCatalystBlockEntity::MAX_CHARGE, 1000);

    let mut catalyst = SculkCatalystBlockEntity::new();
    assert_eq!(
        catalyst.handle_entity_die(BlockPos { x: 3, y: 64, z: -2 }, 2300, true, false,),
        SculkCatalystEventResult::Bloom { pulse_ticks: 8 }
    );
    assert_eq!(catalyst.pulse_ticks, 8);
    assert_eq!(
        catalyst.cursors,
        vec![
            SculkChargeCursor::new(BlockPos { x: 3, y: 65, z: -2 }, 1000),
            SculkChargeCursor::new(BlockPos { x: 3, y: 65, z: -2 }, 1000),
            SculkChargeCursor::new(BlockPos { x: 3, y: 65, z: -2 }, 300),
        ]
    );

    let saved = catalyst.save_additional();
    assert_eq!(SculkCatalystBlockEntity::load_additional(&saved), catalyst);
    catalyst.tick(BlockPos { x: 0, y: 64, z: 0 });
    assert_eq!(catalyst.pulse_ticks, 7);
    assert_eq!(catalyst.cursors[0].decay_delay, 0);
    catalyst.tick(BlockPos { x: 0, y: 64, z: 0 });
    assert_eq!(catalyst.cursors[0].charge, 999);
    assert_eq!(catalyst.cursors[0].decay_delay, 1);

    let mut ignored = SculkCatalystBlockEntity::new();
    assert_eq!(
        ignored.handle_entity_die(BlockPos { x: 0, y: 0, z: 0 }, 5, true, true),
        SculkCatalystEventResult::Ignored
    );
    assert!(ignored.cursors.is_empty());
    assert_eq!(ignored.pulse_ticks, 0);

    ignored.add_cursors(BlockPos { x: 0, y: 0, z: 0 }, 40_000);
    assert_eq!(ignored.cursors.len(), 32);
    assert!(ignored.cursors.iter().all(|cursor| cursor.charge == 1000));
    ignored.cursors[0].pos = BlockPos {
        x: 2000,
        y: 0,
        z: 0,
    };
    ignored.tick(BlockPos { x: 0, y: 0, z: 0 });
    assert_eq!(ignored.cursors.len(), 31);
}

#[test]
fn beehive_block_entity_persists_occupants_releases_and_increments_honey_like_java() {
    assert_eq!(BeehiveBlockEntity::MAX_OCCUPANTS, 3);
    assert_eq!(BeehiveBlockEntity::MIN_OCCUPATION_TICKS_NECTAR, 2400);
    assert_eq!(BeehiveBlockEntity::MIN_OCCUPATION_TICKS_NECTARLESS, 600);
    assert_eq!(BeehiveBlockEntity::MIN_TICKS_BEFORE_REENTERING_HIVE, 400);
    assert_eq!(BeehiveBlockEntity::MAX_HONEY_LEVEL, 5);
    assert_eq!(BeehiveBlockEntity::WORK_SOUND_CHANCE, 0.005);

    let mut hive = populated_beehive();
    assert_beehive_save_load_and_blocked_ticks(&mut hive);
    assert_beehive_normal_release_increments_honey(&mut hive);
    assert_beehive_honey_cap_emergency_and_sedated_release(&mut hive);
}

fn populated_beehive() -> BeehiveBlockEntity {
    let mut hive = BeehiveBlockEntity::new();
    assert!(hive.is_empty());
    assert!(hive.add_occupant(
        BeehiveOccupant::bee(600, false),
        Some(BlockPos { x: 2, y: 70, z: -3 })
    ));
    assert!(hive.add_occupant(BeehiveOccupant::bee(2400, true), None));
    assert!(hive.add_occupant(BeehiveOccupant::bee(2401, true), None));
    assert!(!hive.add_occupant(BeehiveOccupant::bee(0, false), None));
    assert!(hive.is_full());
    assert_eq!(hive.occupant_count(), 3);
    assert_eq!(hive.saved_flower_pos, Some(BlockPos { x: 2, y: 70, z: -3 }));
    hive
}

fn assert_beehive_save_load_and_blocked_ticks(hive: &mut BeehiveBlockEntity) {
    let saved = hive.save_additional();
    let loaded = BeehiveBlockEntity::load_additional(&saved);
    assert_eq!(loaded, *hive);

    let blocked = hive.tick(true, false, false);
    assert!(blocked.is_empty());
    assert_eq!(hive.occupant_count(), 3);
    assert_eq!(hive.occupants[0].ticks_in_hive, 601);
    assert_eq!(hive.occupants[1].ticks_in_hive, 2401);
    assert_eq!(hive.occupants[2].ticks_in_hive, 2402);

    let released = hive.tick(false, true, false);
    assert!(released.is_empty());
    assert_eq!(hive.occupant_count(), 3);
}

fn assert_beehive_normal_release_increments_honey(hive: &mut BeehiveBlockEntity) {
    let released = hive.tick(false, false, false);
    assert_eq!(
        released,
        vec![
            BeeReleaseEvent {
                entity_type: "minecraft:bee".to_string(),
                status: BeeReleaseStatus::BeeReleased,
                honey_level: 0,
                stay_out_of_hive_ticks: 0,
            },
            BeeReleaseEvent {
                entity_type: "minecraft:bee".to_string(),
                status: BeeReleaseStatus::HoneyDelivered,
                honey_level: 1,
                stay_out_of_hive_ticks: 0,
            },
            BeeReleaseEvent {
                entity_type: "minecraft:bee".to_string(),
                status: BeeReleaseStatus::HoneyDelivered,
                honey_level: 2,
                stay_out_of_hive_ticks: 0,
            },
        ]
    );
    assert!(hive.is_empty());
}

fn assert_beehive_honey_cap_emergency_and_sedated_release(hive: &mut BeehiveBlockEntity) {
    hive.honey_level = 4;
    assert!(hive.add_occupant(BeehiveOccupant::bee(2401, true), None));
    let released = hive.tick(false, false, true);
    assert_eq!(released[0].honey_level, 5);
    assert_eq!(hive.honey_level, 5);

    assert!(hive.add_occupant(BeehiveOccupant::bee(0, false), None));
    assert!(hive.add_occupant(BeehiveOccupant::bee(0, false), None));
    let emergency = hive.on_fire_nearby();
    assert_eq!(emergency.len(), 2);
    assert!(emergency
        .iter()
        .all(|event| event.status == BeeReleaseStatus::Emergency));
    assert!(hive.is_empty());

    assert!(hive.add_occupant(BeehiveOccupant::bee(0, false), None));
    let sedated = hive.empty_all_living_from_hive(BeeReleaseStatus::Emergency, true);
    assert_eq!(sedated[0].stay_out_of_hive_ticks, 400);
}

#[test]
fn creaking_heart_block_entity_tracks_state_protector_and_output_like_java() {
    assert_eq!(CreakingHeartBlockEntity::PLAYER_DETECTION_RANGE, 32);
    assert_eq!(CreakingHeartBlockEntity::CREAKING_ROAMING_RADIUS, 32);
    assert_eq!(CreakingHeartBlockEntity::DISTANCE_CREAKING_TOO_FAR, 34.0);
    assert_eq!(CreakingHeartBlockEntity::SPAWN_RANGE_XZ, 16);
    assert_eq!(CreakingHeartBlockEntity::SPAWN_RANGE_Y, 8);
    assert_eq!(CreakingHeartBlockEntity::ATTEMPTS_PER_SPAWN, 5);
    assert_eq!(CreakingHeartBlockEntity::UPDATE_TICKS, 20);
    assert_eq!(CreakingHeartBlockEntity::UPDATE_TICKS_VARIANCE, 5);
    assert_eq!(CreakingHeartBlockEntity::HURT_CALL_TOTAL_TICKS, 100);
    assert_eq!(CreakingHeartBlockEntity::HURT_CALL_INTERVAL, 10);
    assert_eq!(CreakingHeartBlockEntity::HURT_CALL_PARTICLE_TICKS, 50);
    assert_eq!(CreakingHeartBlockEntity::MAX_RESIN_DEPTH, 2);
    assert_eq!(CreakingHeartBlockEntity::MAX_RESIN_COUNT, 64);
    assert_eq!(CreakingHeartBlockEntity::TICKS_GRACE_PERIOD, 30);

    let mut heart = CreakingHeartBlockEntity::new();
    heart.ticker = -1;
    let actions = heart.server_tick(CreakingHeartTickContext {
        has_required_logs: true,
        creaking_active: true,
        spawning_monsters: true,
        player_nearby: true,
        next_ticker_offset: 4,
        ..CreakingHeartTickContext::default()
    });
    assert_eq!(
        actions,
        vec![
            CreakingHeartAction::StateChanged(CreakingHeartStateModel::Awake),
            CreakingHeartAction::SpawnProtector {
                attempts: 5,
                range_xz: 16,
                range_y: 8,
            },
        ]
    );
    assert_eq!(heart.ticker, 24);
    assert_eq!(heart.state, CreakingHeartStateModel::Awake);

    heart.on_protector_spawned("00000000-0000-0000-0000-000000000001".to_string());
    assert_eq!(heart.compute_analog_output_signal(Some(0.0)), 15);
    assert_eq!(heart.compute_analog_output_signal(Some(16.0)), 8);
    assert_eq!(heart.compute_analog_output_signal(Some(32.0)), 0);
    assert_eq!(heart.compute_analog_output_signal(Some(64.0)), 0);

    let saved = heart.save_additional();
    assert_eq!(
        CreakingHeartBlockEntity::load_additional(&saved).creaking_uuid,
        heart.creaking_uuid
    );

    let hurt = heart.creaking_hurt(true, 3);
    assert_eq!(
        hurt,
        CreakingHeartAction::HurtPulse {
            total_ticks: 100,
            particle_ticks: 50,
            resin_clumps: 3,
        }
    );
    assert_eq!(heart.emitter_ticks, 100);
    assert_eq!(heart.creaking_hurt(true, 2), CreakingHeartAction::None);
    heart.server_tick(CreakingHeartTickContext {
        has_required_logs: true,
        creaking_active: true,
        spawning_monsters: true,
        player_nearby: true,
        protector_resolved: true,
        protector_distance: Some(4.0),
        ..CreakingHeartTickContext::default()
    });
    assert_eq!(heart.emitter_ticks, 99);

    heart.ticker = -1;
    let actions = heart.server_tick(CreakingHeartTickContext {
        has_required_logs: true,
        spawning_monsters: true,
        player_nearby: true,
        protector_resolved: true,
        protector_distance: Some(35.0),
        ..CreakingHeartTickContext::default()
    });
    assert!(actions.contains(&CreakingHeartAction::StateChanged(
        CreakingHeartStateModel::Dormant
    )));
    assert!(actions.contains(&CreakingHeartAction::RemoveProtector));
    assert!(heart.creaking_uuid.is_none());

    heart.state = CreakingHeartStateModel::Dormant;
    heart.ticker = -1;
    let actions = heart.server_tick(CreakingHeartTickContext {
        creaking_active: true,
        spawning_monsters: true,
        player_nearby: true,
        ..CreakingHeartTickContext::default()
    });
    assert_eq!(
        actions,
        vec![CreakingHeartAction::StateChanged(
            CreakingHeartStateModel::Uprooted
        )]
    );

    let mut unresolved = CreakingHeartBlockEntity::load_additional(&Tag::Compound(vec![(
        "creaking".to_string(),
        Tag::String("00000000-0000-0000-0000-000000000002".to_string()),
    )]));
    unresolved.ticks_existed = 29;
    unresolved.ticker = -1;
    assert!(unresolved
        .server_tick(CreakingHeartTickContext {
            has_required_logs: true,
            creaking_active: true,
            spawning_monsters: true,
            player_nearby: true,
            ..CreakingHeartTickContext::default()
        })
        .contains(&CreakingHeartAction::RemoveProtector));
    assert!(unresolved.creaking_uuid.is_none());
}

#[test]
fn sculk_shrieker_block_entity_tracks_warning_shriek_and_warden_response() {
    assert_eq!(SculkShriekerBlockEntity::LISTENER_RADIUS, 8);
    assert_eq!(SculkShriekerBlockEntity::WARNING_SOUND_RADIUS, 10);
    assert_eq!(SculkShriekerBlockEntity::SHRIEKING_TICKS, 90);
    assert_eq!(SculkShriekerBlockEntity::DARKNESS_RADIUS, 40);
    assert_eq!(SculkShriekerBlockEntity::WARDEN_SUMMON_WARNING_LEVEL, 4);
    assert_eq!(SculkShriekerBlockEntity::WARDEN_SPAWN_ATTEMPTS, 20);
    assert_eq!(SculkShriekerBlockEntity::WARDEN_SPAWN_RANGE_XZ, 5);
    assert_eq!(SculkShriekerBlockEntity::WARDEN_SPAWN_RANGE_Y, 6);

    let mut shrieker = SculkShriekerBlockEntity::new(true);
    assert!(shrieker.can_receive_vibration(false, true));
    assert!(!shrieker.can_receive_vibration(false, false));
    assert!(!shrieker.can_receive_vibration(true, true));
    assert_eq!(
        shrieker.try_shriek(false, true, Some(1), false),
        SculkShriekResult::Ignored
    );
    assert_eq!(
        shrieker.try_shriek(true, true, None, false),
        SculkShriekResult::Ignored
    );

    assert_eq!(
        shrieker.try_shriek(true, true, Some(3), false),
        SculkShriekResult::ReplySound {
            warning_level: 3,
            darkness_radius: 40,
        }
    );
    assert_eq!(shrieker.warning_level, 3);
    assert_eq!(shrieker.shrieking_ticks, 90);
    assert_eq!(
        shrieker.try_shriek(true, true, Some(4), true),
        SculkShriekResult::Ignored
    );
    assert_eq!(shrieker.tick(), SculkShriekResult::Ignored);
    assert_eq!(shrieker.shrieking_ticks, 89);

    shrieker.shrieking_ticks = 0;
    assert_eq!(
        shrieker.try_shriek(true, true, Some(4), true),
        SculkShriekResult::SummonWarden {
            warning_level: 4,
            attempts: 20,
            range_xz: 5,
            range_y: 6,
            darkness_radius: 40,
        }
    );

    let saved = shrieker.save_additional();
    let loaded = SculkShriekerBlockEntity::load_additional(&saved);
    assert_eq!(loaded.warning_level, 4);
    assert_eq!(loaded.shrieking_ticks, 0);
    assert!(!loaded.can_summon);

    let mut disabled = SculkShriekerBlockEntity::new(false);
    assert_eq!(
        disabled.try_shriek(true, false, None, false),
        SculkShriekResult::Shriek { warning_level: 0 }
    );
    assert_eq!(disabled.shrieking_ticks, 90);
}

#[test]
fn bell_block_entity_tracks_ring_resonation_and_raider_glow_like_java() {
    assert_eq!(BellBlockEntity::DURATION, 50);
    assert_eq!(BellBlockEntity::GLOW_DURATION, 60);
    assert_eq!(BellBlockEntity::MIN_TICKS_BETWEEN_SEARCHES, 60);
    assert_eq!(BellBlockEntity::MAX_RESONATION_TICKS, 40);
    assert_eq!(BellBlockEntity::TICKS_BEFORE_RESONATION, 5);
    assert_eq!(BellBlockEntity::SEARCH_RADIUS, 48.0);
    assert_eq!(BellBlockEntity::HEAR_BELL_RADIUS, 32.0);
    assert_eq!(BellBlockEntity::HIGHLIGHT_RAIDERS_RADIUS, 48.0);

    let mut bell = BellBlockEntity::new();
    let block_event = bell.on_hit(Direction::North);
    assert_eq!(
        block_event,
        BellBlockEvent {
            event_id: BellBlockEntity::EVENT_RING,
            event_param: 2,
        }
    );
    assert!(bell.shaking);
    assert_eq!(bell.click_direction, Some(Direction::North));

    bell.ticks = 12;
    assert_eq!(bell.on_hit(Direction::East).event_param, 5);
    assert_eq!(bell.ticks, 0);
    assert_eq!(bell.click_direction, Some(Direction::East));

    assert!(bell.trigger_event(1, 3, 100, 4, 1, 2));
    assert_eq!(bell.click_direction, Some(Direction::South));
    assert_eq!(bell.last_ring_timestamp, 100);
    assert_eq!(bell.heard_bell_entities, 4);
    assert_eq!(bell.nearby_raiders_within_hear_radius, 1);
    assert_eq!(bell.nearby_raiders_within_highlight_radius, 2);
    assert_eq!(bell.ticks, 0);
    assert!(bell.shaking);
    assert!(!bell.trigger_event(99, 0, 100, 0, 0, 0));

    for _ in 0..4 {
        assert_eq!(
            bell.tick(),
            BellTickEffects {
                play_resonate_sound: false,
                glowing_raiders: 0,
            }
        );
    }
    assert_eq!(bell.ticks, 4);
    assert_eq!(
        bell.tick(),
        BellTickEffects {
            play_resonate_sound: true,
            glowing_raiders: 0,
        }
    );
    assert!(bell.resonating);
    assert_eq!(bell.resonation_ticks, 1);

    for _ in 0..39 {
        let effects = bell.tick();
        assert!(!effects.play_resonate_sound);
        assert_eq!(effects.glowing_raiders, 0);
    }
    assert_eq!(bell.resonation_ticks, 40);
    assert_eq!(
        bell.tick(),
        BellTickEffects {
            play_resonate_sound: false,
            glowing_raiders: 2,
        }
    );
    assert!(!bell.resonating);

    while bell.shaking {
        bell.tick();
    }
    assert_eq!(bell.ticks, 0);
    assert_eq!(bell.save_additional(), Tag::Compound(vec![]));
    assert_eq!(bell.get_update_tag(), Tag::Compound(vec![]));

    let mut cached = bell.clone();
    cached.update_entities(120, 7, 3, 5);
    assert_eq!(cached.heard_bell_entities, 4);
    cached.update_entities(161, 7, 3, 5);
    assert_eq!(cached.heard_bell_entities, 7);
    assert_eq!(cached.nearby_raiders_within_highlight_radius, 5);
}

#[test]
fn save_modes_match_metadata_and_custom_data_boundaries() {
    let mut entity =
        BlockEntity::new(BlockEntityTypeId::Sign, pos(), "minecraft:oak_sign").unwrap();
    entity
        .custom_data
        .insert("front_text".to_string(), Tag::String("hello".to_string()));
    entity.components.insert(
        "minecraft:custom_name".to_string(),
        Tag::String("\"Name\"".to_string()),
    );

    assert_eq!(
        entity.save_custom_only(),
        Tag::Compound(vec![(
            "front_text".to_string(),
            Tag::String("hello".to_string())
        )])
    );
    assert!(
        matches!(entity.save_without_metadata(), Tag::Compound(values) if values.iter().any(|(k, _)| k == "components") && values.iter().all(|(k, _)| k != "id"))
    );
    assert!(
        matches!(entity.save_with_id(), Tag::Compound(values) if values.iter().any(|(k, v)| k == "id" && *v == Tag::String("sign".to_string())) && values.iter().all(|(k, _)| k != "x"))
    );
    assert!(
        matches!(entity.save_with_full_metadata(), Tag::Compound(values) if values.iter().any(|(k, v)| k == "x" && *v == Tag::Int(18)))
    );
}
