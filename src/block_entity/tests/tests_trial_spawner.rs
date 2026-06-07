use super::*;
use crate::block_entity::spawners::{
    TrialSpawnerFlameParticle, TrialSpawnerParticleEmission, TrialSpawnerTrackedMob,
};

#[test]
fn trial_spawner_wrapper_update_state_and_client_spin_match_java() {
    let mut spawner = TrialSpawnerBlockEntity::default();
    assert_trial_spawner_state_flags_and_client_spin(&mut spawner);
    assert_trial_spawner_java_events_particles_and_spawn_gate();
    assert_trial_spawner_mob_tracking_and_entity_override(&mut spawner);
    assert_trial_spawner_state_data_helpers_match_java(&mut spawner);
    assert_trial_spawner_update_packet_omits_components();
}

fn assert_trial_spawner_state_flags_and_client_spin(spawner: &mut TrialSpawnerBlockEntity) {
    assert_eq!(spawner.get_state(false), TrialSpawnerStateModel::Inactive);
    assert_eq!(spawner.get_state(true), TrialSpawnerStateModel::Inactive);
    assert_eq!(
        spawner.set_state(TrialSpawnerStateModel::WaitingForPlayers),
        TrialSpawnerBlockEntity::BLOCK_UPDATE_FLAGS
    );
    assert_eq!(
        spawner.mark_updated_flags(),
        TrialSpawnerBlockEntity::BLOCK_UPDATE_FLAGS
    );
    assert_eq!(
        spawner.get_state(true),
        TrialSpawnerStateModel::WaitingForPlayers
    );

    assert_eq!(
        TrialSpawnerStateModel::WaitingForPlayers.spinning_mob_speed(),
        200.0
    );
    assert_eq!(TrialSpawnerStateModel::Active.spinning_mob_speed(), 1000.0);
    assert!(TrialSpawnerStateModel::WaitingForPlayers.has_spinning_mob());
    assert!(TrialSpawnerStateModel::Active.is_capable_of_spawning());
    assert!(!TrialSpawnerStateModel::Cooldown.is_capable_of_spawning());
    assert_eq!(
        TrialSpawnerStateModel::Inactive.serialized_name(),
        "inactive"
    );
    assert_eq!(
        TrialSpawnerStateModel::WaitingForPlayers.particle_emission(),
        TrialSpawnerParticleEmission::SmallFlames
    );
    assert_eq!(
        TrialSpawnerStateModel::Active.particle_emission(),
        TrialSpawnerParticleEmission::FlamesAndSmoke
    );
    assert_eq!(
        TrialSpawnerStateModel::Cooldown.particle_emission(),
        TrialSpawnerParticleEmission::SmokeInsideAndTopFace
    );
    spawner.next_mob_spawns_at = 100;
    spawner.tick_client(0);
    assert_eq!(spawner.old_spin, 0.0);
    assert!((spawner.spin - (200.0 / 300.0)).abs() < f64::EPSILON);
    assert_eq!(
        TrialSpawnerBlockEntity::reward_ejection_position(pos()),
        (18.5, 65.2, 35.5)
    );
}

fn assert_trial_spawner_java_events_particles_and_spawn_gate() {
    assert_eq!(TrialSpawnerFlameParticle::Normal.encode(), 0);
    assert_eq!(TrialSpawnerFlameParticle::Ominous.encode(), 1);
    assert_eq!(
        TrialSpawnerFlameParticle::decode(1),
        TrialSpawnerFlameParticle::Ominous
    );
    assert_eq!(
        TrialSpawnerFlameParticle::decode(2),
        TrialSpawnerFlameParticle::Normal
    );
    assert_eq!(
        TrialSpawnerFlameParticle::Ominous.particle_id(),
        "minecraft:soul_fire_flame"
    );
    assert_eq!(TrialSpawnerBlockEntity::SPAWN_MOB_EVENT, 3011);
    assert_eq!(TrialSpawnerBlockEntity::SPAWN_MOB_AT_EVENT, 3012);
    assert_eq!(TrialSpawnerBlockEntity::EJECT_REWARD_EVENT, 3014);
    assert_eq!(TrialSpawnerBlockEntity::BECOME_OMINOUS_EVENT, 3020);
    assert_eq!(TrialSpawnerBlockEntity::SPAWN_PARTICLE_COUNT, 20);
    assert_eq!(TrialSpawnerBlockEntity::EJECT_ITEM_PARTICLE_COUNT, 20);
    assert_eq!(
        TrialSpawnerBlockEntity::DELAY_BEFORE_EJECT_AFTER_KILLING_LAST_MOB,
        40
    );
    assert_eq!(TrialSpawnerBlockEntity::TIME_BETWEEN_REWARD_EJECTIONS, 30);
    assert_eq!(
        TrialSpawnerBlockEntity::detect_player_particle_count(12),
        80
    );
    let config = TrialSpawnerConfigModel::default();
    assert_eq!(config.ticks_between_item_spawners(), 160);
    let with_spawning = config.with_spawning("minecraft:stray");
    assert_eq!(with_spawning.spawn_potentials.len(), 1);
    assert_eq!(
        with_spawning.spawn_potentials[0].entity_id(),
        Some("minecraft:stray")
    );
    assert_eq!(
        with_spawning.loot_tables_to_eject,
        config.loot_tables_to_eject
    );
    assert_eq!(
        with_spawning.items_to_drop_when_ominous,
        config.items_to_drop_when_ominous
    );
    assert!(TrialSpawnerBlockEntity::can_spawn_in_level(
        true, true, true, false
    ));
    assert!(!TrialSpawnerBlockEntity::can_spawn_in_level(
        true, false, true, true
    ));
    assert!(!TrialSpawnerBlockEntity::can_spawn_in_level(
        true, false, false, false
    ));
    assert!(!TrialSpawnerBlockEntity::can_spawn_in_level(
        false, true, false, true
    ));
}

fn assert_trial_spawner_mob_tracking_and_entity_override(spawner: &mut TrialSpawnerBlockEntity) {
    assert!(TrialSpawnerBlockEntity::should_mob_be_untracked(
        TrialSpawnerTrackedMob {
            exists: false,
            alive: true,
            same_dimension: true,
            distance_squared: 0,
        }
    ));
    assert!(!TrialSpawnerBlockEntity::should_mob_be_untracked(
        TrialSpawnerTrackedMob {
            exists: true,
            alive: true,
            same_dimension: true,
            distance_squared: TrialSpawnerBlockEntity::MAX_MOB_TRACKING_DISTANCE_SQR,
        }
    ));
    assert!(TrialSpawnerBlockEntity::should_mob_be_untracked(
        TrialSpawnerTrackedMob {
            exists: true,
            alive: true,
            same_dimension: true,
            distance_squared: TrialSpawnerBlockEntity::MAX_MOB_TRACKING_DISTANCE_SQR + 1,
        }
    ));

    assert!(!spawner.set_entity_id("minecraft:husk", false));
    assert!(spawner.config.normal_config.spawn_potentials.is_empty());
    assert!(spawner.set_entity_id("minecraft:husk", true));
    assert_eq!(
        spawner.config.normal_config.spawn_potentials[0].entity_id(),
        Some("minecraft:husk")
    );
}

fn assert_trial_spawner_state_data_helpers_match_java(spawner: &mut TrialSpawnerBlockEntity) {
    assert_eq!(TrialSpawnerBlockEntity::DELAY_BETWEEN_PLAYER_SCANS, 20);
    assert_eq!(
        TrialSpawnerBlockEntity::TRIAL_OMEN_PER_BAD_OMEN_LEVEL,
        18_000
    );
    assert_eq!(
        TrialSpawnerBlockEntity::trial_omen_duration_from_bad_omen_amplifier(2),
        54_000
    );
    assert_eq!(TrialSpawnerBlockEntity::count_additional_players(0), 0);
    assert_eq!(TrialSpawnerBlockEntity::count_additional_players(3), 2);

    let pos = pos();
    let packed = TrialSpawnerBlockEntity::block_pos_as_long(pos);
    assert!(!TrialSpawnerBlockEntity::is_player_scan_throttled(
        pos, -packed
    ));
    assert!(TrialSpawnerBlockEntity::is_player_scan_throttled(
        pos,
        -packed + 1
    ));

    let cooldown_ends_at = 147;
    assert!(TrialSpawnerBlockEntity::is_ready_to_open_shutter(
        87,
        cooldown_ends_at,
        40.0,
        100,
    ));
    assert!(!TrialSpawnerBlockEntity::is_ready_to_open_shutter(
        86,
        cooldown_ends_at,
        40.0,
        100,
    ));
    assert!(TrialSpawnerBlockEntity::is_ready_to_eject_items(
        107,
        cooldown_ends_at,
        30.0,
        100,
    ));
    assert!(!TrialSpawnerBlockEntity::is_ready_to_eject_items(
        108,
        cooldown_ends_at,
        30.0,
        100,
    ));
    assert!(TrialSpawnerBlockEntity::is_cooldown_finished(
        147,
        cooldown_ends_at
    ));
    assert_eq!(
        TrialSpawnerBlockEntity::low_resolution_position_seed(
            123,
            BlockPos {
                x: -1,
                y: 39,
                z: 60
            }
        ),
        123 + TrialSpawnerBlockEntity::block_pos_as_long(BlockPos { x: -1, y: 1, z: 2 })
    );

    spawner.current_mobs = vec!["mob-a".to_string()];
    spawner.next_mob_spawns_at = 10;
    spawner.config.normal_config.simultaneous_mobs = 2.0;
    assert!(spawner.is_ready_to_spawn_next_mob(10, 0));
    spawner.current_mobs.push("mob-b".to_string());
    assert!(!spawner.is_ready_to_spawn_next_mob(10, 0));

    spawner.detected_players = vec!["player-a".to_string()];
    spawner.total_mobs_spawned = 4;
    spawner.cooldown_ends_at = 80;
    spawner.next_mob_spawns_at = 40;
    spawner.reset_statistics();
    assert!(spawner.detected_players.is_empty());
    assert_eq!(spawner.total_mobs_spawned, 0);
    assert_eq!(spawner.cooldown_ends_at, 0);
    assert_eq!(spawner.next_mob_spawns_at, 0);

    spawner.current_mobs = vec!["mob-a".to_string()];
    spawner.next_spawn_data = Some(SpawnDataModel::new("minecraft:zombie"));
    spawner.reset_state_data();
    assert!(spawner.current_mobs.is_empty());
    assert_eq!(spawner.next_spawn_data, None);
}

fn assert_trial_spawner_update_packet_omits_components() {
    let mut entity = BlockEntity::new(
        BlockEntityTypeId::TrialSpawner,
        pos(),
        "minecraft:trial_spawner",
    )
    .unwrap();
    entity.custom_data.insert(
        "spawn_data".to_string(),
        SpawnDataModel::new("minecraft:husk").to_tag(),
    );
    entity.components.insert(
        "minecraft:custom_name".to_string(),
        Tag::String("Trial".to_string()),
    );
    assert!(
        matches!(entity.get_update_packet().tag, Tag::Compound(fields) if fields.iter().any(|(key, _)| key == "spawn_data") && fields.iter().all(|(key, _)| key != "components"))
    );
}
