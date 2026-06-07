use super::super::*;
use super::*;

#[test]
fn trial_spawner_state_machine_configs_rewards_and_nbt_like_java() {
    let mut spawner = configured_trial_spawner();
    assert_trial_spawner_waiting_and_activation(&mut spawner);
    assert_trial_spawner_spawn_and_reward_ejection(&mut spawner);
    assert_trial_spawner_cooldown_and_ominous_transition(&mut spawner);
    assert_trial_spawner_override_update_tag_and_save_load(&mut spawner);
}

fn configured_trial_spawner() -> TrialSpawnerBlockEntity {
    let mut spawner = TrialSpawnerBlockEntity::default();
    spawner.config.normal_config.spawn_potentials = vec![SpawnDataModel::new("minecraft:zombie")];
    spawner.config.ominous_config.spawn_potentials = vec![SpawnDataModel::new("minecraft:breeze")];
    spawner.config.normal_config.total_mobs = 2.0;
    spawner.config.normal_config.simultaneous_mobs = 1.0;
    spawner.config.normal_config.ticks_between_spawn = 5;
    spawner.config.target_cooldown_length = 100;
    spawner
}

fn trial_spawner_context(
    game_time: i64,
    detected_player_count: usize,
    spawn_success: bool,
    apply_ominous: bool,
    roll: usize,
) -> TrialSpawnerTickContext {
    TrialSpawnerTickContext {
        game_time,
        can_spawn_in_level: true,
        detected_player_count,
        current_mobs_alive: 0,
        spawn_success,
        apply_ominous,
        roll,
    }
}

fn assert_trial_spawner_waiting_and_activation(spawner: &mut TrialSpawnerBlockEntity) {
    assert_eq!(spawner.state, TrialSpawnerStateModel::Inactive);
    assert_eq!(spawner.config.required_player_range, 14);
    assert_eq!(TrialSpawnerStateModel::Active.light_level(), 8);
    assert_eq!(
        spawner.tick_server(trial_spawner_context(0, 0, false, false, 0)),
        TrialSpawnerTickResult::StateChanged(TrialSpawnerStateModel::WaitingForPlayers)
    );

    assert_eq!(
        spawner.tick_server(trial_spawner_context(1, 2, false, false, 0)),
        TrialSpawnerTickResult::DetectedPlayers(2)
    );
    assert_eq!(spawner.state, TrialSpawnerStateModel::Active);
    assert_eq!(spawner.next_mob_spawns_at, 41);
    assert_eq!(spawner.active_config().target_total_mobs(1), 4);
    assert_eq!(spawner.active_config().target_simultaneous_mobs(1), 2);
}

fn assert_trial_spawner_spawn_and_reward_ejection(spawner: &mut TrialSpawnerBlockEntity) {
    assert_eq!(
        spawner.tick_server(trial_spawner_context(41, 2, true, false, 0)),
        TrialSpawnerTickResult::SpawnMob {
            entity_id: "minecraft:zombie".to_string(),
        }
    );
    assert_eq!(spawner.total_mobs_spawned, 1);
    assert_eq!(spawner.current_mobs.len(), 1);
    assert_eq!(spawner.next_mob_spawns_at, 46);

    spawner.total_mobs_spawned = spawner.active_config().target_total_mobs(1);
    spawner.current_mobs.clear();
    assert_eq!(
        spawner.tick_server(trial_spawner_context(47, 2, false, false, 0)),
        TrialSpawnerTickResult::ReadyForRewards
    );
    assert_eq!(
        spawner.state,
        TrialSpawnerStateModel::WaitingForRewardEjection
    );
    assert_eq!(spawner.cooldown_ends_at, 147);

    assert_eq!(
        spawner.tick_server(trial_spawner_context(87, 2, false, false, 0)),
        TrialSpawnerTickResult::StateChanged(TrialSpawnerStateModel::EjectingReward)
    );
    assert_eq!(
        spawner.tick_server(trial_spawner_context(107, 2, false, false, 1)),
        TrialSpawnerTickResult::EjectedReward {
            loot_table: "minecraft:spawners/trial_chamber/key".to_string(),
            remaining_players: 1,
        }
    );
    assert_eq!(
        spawner.tick_server(trial_spawner_context(137, 2, false, false, 0)),
        TrialSpawnerTickResult::EjectedReward {
            loot_table: "minecraft:spawners/trial_chamber/key".to_string(),
            remaining_players: 0,
        }
    );
}

fn assert_trial_spawner_cooldown_and_ominous_transition(spawner: &mut TrialSpawnerBlockEntity) {
    assert_eq!(
        spawner.tick_server(trial_spawner_context(167, 0, false, false, 0)),
        TrialSpawnerTickResult::StateChanged(TrialSpawnerStateModel::Cooldown)
    );
    assert_eq!(
        spawner.tick_server(trial_spawner_context(180, 0, false, false, 0)),
        TrialSpawnerTickResult::CooldownFinished
    );
    assert_eq!(spawner.state, TrialSpawnerStateModel::WaitingForPlayers);

    assert_eq!(
        spawner.tick_server(trial_spawner_context(200, 1, false, true, 0)),
        TrialSpawnerTickResult::BecameOminous
    );
    assert!(spawner.is_ominous);
    assert_eq!(spawner.next_mob_spawns_at, 240);
    assert_eq!(spawner.cooldown_ends_at, 360);
}

fn assert_trial_spawner_override_update_tag_and_save_load(spawner: &mut TrialSpawnerBlockEntity) {
    spawner.override_entity_to_spawn("minecraft:husk");
    assert_eq!(spawner.state, TrialSpawnerStateModel::Inactive);
    assert_eq!(
        spawner.config.normal_config.spawn_potentials[0].entity_id(),
        Some("minecraft:husk")
    );
    assert_eq!(
        spawner.config.ominous_config.spawn_potentials[0].entity_id(),
        Some("minecraft:husk")
    );

    spawner.state = TrialSpawnerStateModel::Active;
    spawner.next_mob_spawns_at = 500;
    spawner.next_spawn_data = Some(SpawnDataModel::new("minecraft:husk"));
    let update_tag = spawner.update_tag();
    assert!(compound_entries(&update_tag)
        .unwrap()
        .iter()
        .any(|(name, _)| name == "next_mob_spawns_at"));
    assert!(compound_entries(&update_tag)
        .unwrap()
        .iter()
        .any(|(name, _)| name == "spawn_data"));

    let saved = spawner.save_additional();
    assert_eq!(TrialSpawnerBlockEntity::load_additional(&saved), *spawner);
}

#[test]
fn vault_block_entity_tracks_key_unlock_ejection_shared_update_and_nbt_like_java() {
    let mut vault = VaultBlockEntity::default();
    assert_vault_defaults_and_inactive_key_insert(&mut vault);
    assert_vault_activation_wrong_key_and_unlock(&mut vault);
    assert_vault_ejects_rewards_and_resets(&mut vault);
    assert_vault_rewarded_player_cap(&mut vault);
    assert_vault_display_update_tag_and_persistence(&mut vault);
}

fn assert_vault_defaults_and_inactive_key_insert(vault: &mut VaultBlockEntity) {
    assert_eq!(vault.state, VaultStateModel::Inactive);
    assert_eq!(vault.state.light_level(), 6);
    assert_eq!(vault.config.activation_range, 4.0);
    assert_eq!(vault.config.deactivation_range, 4.5);
    assert_eq!(vault.config.key_item.item_id, "minecraft:trial_key");

    assert_eq!(
        vault.try_insert_key(
            "player-a",
            &stack("minecraft:trial_key", 1),
            vec![stack("minecraft:diamond", 1)],
            0,
        ),
        VaultInsertResult::IgnoredInactive
    );
}

fn assert_vault_activation_wrong_key_and_unlock(vault: &mut VaultBlockEntity) {
    assert_eq!(
        vault.tick_server(20, &["player-a".to_string()], None),
        VaultTickResult::StateChanged(VaultStateModel::Active)
    );
    assert_eq!(
        vault.connected_particles_range,
        vault.config.activation_range
    );
    assert!(vault.connected_players.contains("player-a"));

    assert_eq!(
        vault.try_insert_key(
            "player-a",
            &stack("minecraft:stick", 1),
            vec![stack("minecraft:diamond", 1)],
            21,
        ),
        VaultInsertResult::WrongKey {
            expected: "minecraft:trial_key".to_string(),
        }
    );
    assert_eq!(vault.last_insert_fail_timestamp, 21);

    assert_eq!(
        vault.try_insert_key(
            "player-a",
            &stack("minecraft:trial_key", 1),
            vec![stack("minecraft:emerald", 2), stack("minecraft:diamond", 1)],
            22,
        ),
        VaultInsertResult::Unlocking { items_to_eject: 2 }
    );
    assert_eq!(vault.state, VaultStateModel::Unlocking);
    assert_eq!(vault.state_updating_resumes_at, 36);
    assert_eq!(vault.display_item, Some(stack("minecraft:diamond", 1)));
    assert!(vault.rewarded_players.contains("player-a"));

    assert_eq!(
        vault.try_insert_key(
            "player-a",
            &stack("minecraft:trial_key", 1),
            vec![stack("minecraft:gold_ingot", 1)],
            37,
        ),
        VaultInsertResult::AlreadyRewarded
    );
}

fn assert_vault_ejects_rewards_and_resets(vault: &mut VaultBlockEntity) {
    assert_eq!(
        vault.tick_server(35, &["player-a".to_string()], None),
        VaultTickResult::Waiting
    );
    assert_eq!(
        vault.tick_server(36, &["player-a".to_string()], None),
        VaultTickResult::StateChanged(VaultStateModel::Ejecting)
    );
    assert_eq!(
        vault.tick_server(56, &["player-a".to_string()], None),
        VaultTickResult::EjectedItem(stack("minecraft:diamond", 1))
    );
    assert_eq!(vault.display_item, Some(stack("minecraft:emerald", 2)));
    assert_eq!(
        vault.tick_server(76, &["player-a".to_string()], None),
        VaultTickResult::EjectedItem(stack("minecraft:emerald", 2))
    );
    assert_eq!(
        vault.tick_server(96, &["player-a".to_string()], None),
        VaultTickResult::EjectionFinished
    );
    assert_eq!(vault.state, VaultStateModel::Inactive);
}

fn assert_vault_rewarded_player_cap(vault: &mut VaultBlockEntity) {
    for index in 0..130 {
        vault.add_rewarded_player(format!("player-{index:03}"));
    }
    assert_eq!(
        vault.rewarded_players.len(),
        VaultBlockEntity::MAX_REWARDED_PLAYERS
    );
    assert!(!vault.rewarded_players.contains("player-000"));
    assert!(vault.rewarded_players.contains("player-129"));
}

fn assert_vault_display_update_tag_and_persistence(vault: &mut VaultBlockEntity) {
    vault.state = VaultStateModel::Active;
    assert_eq!(
        vault.tick_server(
            120,
            &["player-new".to_string()],
            Some(stack("minecraft:apple", 1)),
        ),
        VaultTickResult::DisplayItemCycled(Some(stack("minecraft:apple", 1)))
    );
    vault.tick_client();
    assert_eq!(vault.previous_spin, 0.0);
    assert_eq!(vault.current_spin, 10.0);

    let update_tag = vault.get_update_tag();
    let update_entries = compound_entries(&update_tag).unwrap();
    assert!(update_entries.iter().any(|(name, _)| name == "shared_data"));
    assert!(!update_entries
        .iter()
        .any(|(name, _)| name == "server_data" || name == "config"));

    let saved = vault.save_additional();
    let loaded = VaultBlockEntity::load_additional(&saved);
    assert_eq!(loaded.state, vault.state);
    assert_eq!(loaded.is_ominous, vault.is_ominous);
    assert_eq!(loaded.config, vault.config);
    assert_eq!(loaded.rewarded_players, vault.rewarded_players);
    assert_eq!(loaded.connected_players, vault.connected_players);
    assert_eq!(loaded.display_item, vault.display_item);
    assert_eq!(loaded.items_to_eject, vault.items_to_eject);
    assert_eq!(loaded.total_ejections_needed, vault.total_ejections_needed);
    assert_eq!(
        loaded.state_updating_resumes_at,
        vault.state_updating_resumes_at
    );
    assert_eq!(
        loaded.connected_particles_range,
        vault.connected_particles_range
    );
    assert_eq!(loaded.last_insert_fail_timestamp, 0);
    assert_eq!(loaded.current_spin, 0.0);
    assert_eq!(loaded.previous_spin, 0.0);
}

#[test]
fn furnace_family_ticks_fuel_recipes_xp_sided_slots_and_speed_like_java() {
    let fuels = FuelValues::vanilla();
    let (smelting, blasting, smoking) = furnace_test_recipes();
    assert_smelting_furnace_ticks_fuel_xp_and_persists(&fuels, &smelting);
    assert_invalid_fuel_does_not_start(&fuels, &smelting);
    assert_blast_furnace_uses_java_speed(&fuels, &blasting);
    let smoker = assert_smoker_uses_java_speed(&fuels, &smoking);
    assert_furnace_sided_slots_and_container_helpers(&fuels, &smoker);
}

fn furnace_test_recipes() -> (
    FurnaceCookingRecipe,
    FurnaceCookingRecipe,
    FurnaceCookingRecipe,
) {
    (
        FurnaceCookingRecipe::new(
            "minecraft:iron_ingot_from_smelting_raw_iron",
            "smelting",
            "minecraft:raw_iron",
            "minecraft:iron_ingot",
            200,
            700,
        ),
        FurnaceCookingRecipe::new(
            "minecraft:iron_ingot_from_blasting_raw_iron",
            "blasting",
            "minecraft:raw_iron",
            "minecraft:iron_ingot",
            100,
            700,
        ),
        FurnaceCookingRecipe::new(
            "minecraft:cooked_beef_from_smoking",
            "smoking",
            "minecraft:beef",
            "minecraft:cooked_beef",
            100,
            350,
        ),
    )
}

fn set_furnace_stack(
    furnace: &mut AbstractFurnaceBlockEntity,
    slot: usize,
    item_id: &str,
    recipe: &FurnaceCookingRecipe,
) {
    furnace.set_item(slot, Some(stack(item_id, 1)), Some(recipe));
}

fn assert_smelting_furnace_ticks_fuel_xp_and_persists(
    fuels: &FuelValues,
    smelting: &FurnaceCookingRecipe,
) {
    let mut furnace = AbstractFurnaceBlockEntity::furnace();
    assert_eq!(furnace.kind.recipe_type(), "smelting");
    assert_eq!(furnace.kind.default_cooking_time(), 200);
    set_furnace_stack(
        &mut furnace,
        AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
        "minecraft:raw_iron",
        smelting,
    );
    set_furnace_stack(
        &mut furnace,
        AbstractFurnaceBlockEntity::FUEL_SLOT,
        "minecraft:coal",
        smelting,
    );
    assert_eq!(
        furnace.server_tick(fuels, Some(smelting)),
        FurnaceTickResult::LitChanged { lit: true }
    );
    assert_eq!(furnace.lit_time_remaining, 1600);
    assert_eq!(furnace.lit_total_time, 1600);
    assert_eq!(furnace.cooking_time_spent, 1);
    assert!(furnace.items[AbstractFurnaceBlockEntity::FUEL_SLOT].is_none());

    for _ in 1..199 {
        furnace.server_tick(fuels, Some(smelting));
    }
    assert_eq!(
        furnace.server_tick(fuels, Some(smelting)),
        FurnaceTickResult::Burned { output_count: 1 }
    );
    assert!(furnace.items[AbstractFurnaceBlockEntity::INGREDIENT_SLOT].is_none());
    assert_eq!(
        furnace.items[AbstractFurnaceBlockEntity::RESULT_SLOT],
        Some(stack("minecraft:iron_ingot", 1))
    );
    assert_eq!(
        furnace
            .recipes_used
            .get("minecraft:iron_ingot_from_smelting_raw_iron"),
        Some(&(1, 700))
    );
    assert_eq!(furnace.xp_to_award_and_clear(0.0), 1);
    assert!(furnace.recipes_used.is_empty());

    let saved = furnace.save_additional();
    let loaded =
        AbstractFurnaceBlockEntity::load_additional(FurnaceBlockEntityKind::Furnace, &saved);
    assert_eq!(loaded, furnace);
}

fn assert_invalid_fuel_does_not_start(fuels: &FuelValues, smelting: &FurnaceCookingRecipe) {
    let mut invalid_fuel = AbstractFurnaceBlockEntity::furnace();
    set_furnace_stack(
        &mut invalid_fuel,
        AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
        "minecraft:raw_iron",
        smelting,
    );
    set_furnace_stack(
        &mut invalid_fuel,
        AbstractFurnaceBlockEntity::FUEL_SLOT,
        "minecraft:stone",
        smelting,
    );
    assert_eq!(
        invalid_fuel.server_tick(fuels, Some(smelting)),
        FurnaceTickResult::Idle
    );
    assert_eq!(invalid_fuel.cooking_time_spent, 0);
}

fn assert_blast_furnace_uses_java_speed(fuels: &FuelValues, blasting: &FurnaceCookingRecipe) {
    let mut blast = AbstractFurnaceBlockEntity::blast_furnace();
    assert_eq!(blast.kind.recipe_type(), "blasting");
    assert_eq!(blast.kind.default_cooking_time(), 100);
    set_furnace_stack(
        &mut blast,
        AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
        "minecraft:raw_iron",
        blasting,
    );
    set_furnace_stack(
        &mut blast,
        AbstractFurnaceBlockEntity::FUEL_SLOT,
        "minecraft:coal",
        blasting,
    );
    assert_eq!(
        blast.server_tick(fuels, Some(blasting)),
        FurnaceTickResult::LitChanged { lit: true }
    );
    assert_eq!(blast.lit_total_time, 800);
    for _ in 1..99 {
        blast.server_tick(fuels, Some(blasting));
    }
    assert_eq!(
        blast.server_tick(fuels, Some(blasting)),
        FurnaceTickResult::Burned { output_count: 1 }
    );
}

fn assert_smoker_uses_java_speed(
    fuels: &FuelValues,
    smoking: &FurnaceCookingRecipe,
) -> AbstractFurnaceBlockEntity {
    let mut smoker = AbstractFurnaceBlockEntity::smoker();
    set_furnace_stack(
        &mut smoker,
        AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
        "minecraft:beef",
        smoking,
    );
    set_furnace_stack(
        &mut smoker,
        AbstractFurnaceBlockEntity::FUEL_SLOT,
        "minecraft:coal",
        smoking,
    );
    assert_eq!(
        smoker.server_tick(fuels, Some(smoking)),
        FurnaceTickResult::LitChanged { lit: true }
    );
    assert_eq!(smoker.kind.recipe_type(), "smoking");
    assert_eq!(smoker.lit_total_time, 800);
    smoker
}

fn assert_furnace_sided_slots_and_container_helpers(
    fuels: &FuelValues,
    smoker: &AbstractFurnaceBlockEntity,
) {
    assert_eq!(
        AbstractFurnaceBlockEntity::get_slots_for_face(Direction::Up),
        &[AbstractFurnaceBlockEntity::INGREDIENT_SLOT]
    );
    assert_eq!(
        AbstractFurnaceBlockEntity::get_slots_for_face(Direction::Down),
        &[
            AbstractFurnaceBlockEntity::RESULT_SLOT,
            AbstractFurnaceBlockEntity::FUEL_SLOT,
        ]
    );
    assert!(smoker.can_take_item_through_face(
        AbstractFurnaceBlockEntity::FUEL_SLOT,
        "minecraft:bucket",
        Direction::Down
    ));
    assert!(!smoker.can_take_item_through_face(
        AbstractFurnaceBlockEntity::FUEL_SLOT,
        "minecraft:coal",
        Direction::Down
    ));
    assert!(!smoker.can_place_item(
        AbstractFurnaceBlockEntity::RESULT_SLOT,
        &stack("minecraft:iron_ingot", 1),
        fuels,
    ));
    assert_eq!(
        smoker.max_stack_size(
            AbstractFurnaceBlockEntity::FUEL_SLOT,
            &stack("minecraft:bucket", 16),
        ),
        1
    );
    assert!(smoker.comparator_output() > 0);
}

#[test]
fn container_block_entities_track_loot_openers_lids_redstone_and_hopper_like_java() {
    assert_chest_loot_lock_lid_and_persistence();
    assert_list_backed_container_defaults_match_java();
    assert_trapped_chest_signal_tracks_openers();
    assert_barrel_open_state_tracks_viewers();
    assert_shulker_box_animation_and_sided_insertion();
    assert_dispenser_and_dropper_activation_slots();
    assert_hopper_slots_cooldown_push_pull_and_persistence();
}

fn assert_chest_loot_lock_lid_and_persistence() {
    let mut chest = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Chest);
    chest.custom_name = Some("Supply Cache".to_string());
    chest.lock_key = Some("brass_key".to_string());
    chest.loot_table = Some("minecraft:chests/simple_dungeon".to_string());
    chest.loot_table_seed = 42;

    assert_eq!(chest.kind.size(), 27);
    assert_eq!(chest.kind.menu_type(), "generic_9x3");
    assert_eq!(chest.kind.default_name(), "container.chest");
    assert!(!chest.can_open(None, false));
    assert_eq!(chest.create_menu(Some("brass_key"), true), None);
    assert_eq!(
        chest.create_menu(Some("brass_key"), false),
        Some("generic_9x3")
    );
    assert!(chest.loot_table.is_none());
    assert_eq!(chest.loot_table_seed, 0);

    assert!(chest.set_item(0, Some(stack("minecraft:apple", 32))));
    assert!(chest.comparator_output() > 0);
    assert_eq!(chest.merged_chest_access_size(false), 27);
    assert_eq!(chest.merged_chest_access_size(true), 54);

    assert_chest_lid_interpolates_like_java(&mut chest);

    let loaded_chest = ContainerBlockEntityModel::load_additional(
        ContainerBlockEntityKind::Chest,
        &chest.save_additional(),
    );
    assert_eq!(loaded_chest.custom_name.as_deref(), Some("Supply Cache"));
    assert_eq!(loaded_chest.lock_key.as_deref(), Some("brass_key"));
    assert_eq!(loaded_chest.items[0], Some(stack("minecraft:apple", 32)));
}

fn assert_list_backed_container_defaults_match_java() {
    let mut chest = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Chest);
    assert_eq!(chest.container_size(), 27);
    assert_eq!(chest.count(), 0);
    assert!(chest.is_empty());
    assert_eq!(chest.get_item(0), None);
    assert!(!chest.content_changed);

    assert!(chest.set_item_no_update(0, Some(stack("minecraft:apple", 80))));
    assert_eq!(chest.get_item(0), Some(&stack("minecraft:apple", 64)));
    assert_eq!(chest.count(), 1);
    assert!(!chest.content_changed);
    assert!(!chest.can_place_item(0, &stack("minecraft:apple", 1)));
    assert!(chest.can_place_item(1, &stack("minecraft:air", 0)));

    assert_eq!(chest.remove_item(0, 16), Some(stack("minecraft:apple", 16)));
    assert_eq!(chest.get_item(0), Some(&stack("minecraft:apple", 48)));
    assert!(chest.content_changed);
    chest.content_changed = false;
    assert_eq!(
        chest.remove_item_no_update(0),
        Some(stack("minecraft:apple", 48))
    );
    assert_eq!(chest.get_item(0), None);
    assert!(!chest.content_changed);

    assert!(chest.set_item(1, Some(stack("minecraft:stone", 1))));
    assert!(chest.content_changed);
    chest.clear_content();
    assert_eq!(chest.container_size(), 27);
    assert!(chest.is_empty());
}

fn assert_chest_lid_interpolates_like_java(chest: &mut ContainerBlockEntityModel) {
    chest.start_open();
    chest.tick_lid();
    assert!((chest.chest_lid.previous_openness() - 0.0).abs() < 0.001);
    assert!((chest.lid_progress - 0.1).abs() < 0.001);
    assert!((chest.chest_lid_openness(0.5) - 0.05).abs() < 0.001);
    for _ in 0..10 {
        chest.tick_lid();
    }
    assert!((chest.lid_progress - 1.0).abs() < 0.001);

    chest.stop_open();
    chest.tick_lid();
    assert!((chest.chest_lid.previous_openness() - 1.0).abs() < 0.001);
    assert!((chest.lid_progress - 0.9).abs() < 0.001);
    assert!((chest.chest_lid_openness(0.5) - 0.95).abs() < 0.001);
    for _ in 0..10 {
        chest.tick_lid();
    }
    assert!((chest.lid_progress - 0.0).abs() < 0.001);
}

fn assert_trapped_chest_signal_tracks_openers() {
    let mut trapped = ContainerBlockEntityModel::new(ContainerBlockEntityKind::TrappedChest);
    trapped.world_position = BlockPos {
        x: 12,
        y: 70,
        z: -4,
    };
    trapped.facing = Direction::North;
    for _ in 0..20 {
        trapped.start_open();
    }
    assert_eq!(trapped.trapped_chest_signal(), 15);
    assert_eq!(
        trapped.trapped_chest_signal_open_count(0, 20),
        Some(TrappedChestOpenCountEffect {
            update_positions: vec![
                BlockPos {
                    x: 12,
                    y: 70,
                    z: -4,
                },
                BlockPos {
                    x: 12,
                    y: 69,
                    z: -4,
                },
            ],
            source_block: "minecraft:trapped_chest",
            orientation_facing: Direction::South,
            orientation_up: Direction::Up,
        })
    );
    assert_eq!(trapped.trapped_chest_signal_open_count(20, 20), None);
    for _ in 0..6 {
        trapped.stop_open();
    }
    assert_eq!(trapped.trapped_chest_signal(), 14);
}

fn assert_barrel_open_state_tracks_viewers() {
    let mut barrel = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Barrel);
    barrel.start_open();
    barrel.tick_lid();
    assert!(barrel.barrel_is_open());
    assert_eq!(barrel.lid_progress, 0.0);
    barrel.stop_open();
    assert!(!barrel.barrel_is_open());
}

fn assert_shulker_box_animation_and_sided_insertion() {
    let mut shulker = ContainerBlockEntityModel::new(ContainerBlockEntityKind::ShulkerBox);
    shulker.shulker_color = Some(DyeColor::Purple);
    assert_eq!(shulker.kind.size(), 27);
    assert!(shulker.shulker_is_closed());
    shulker.start_open();
    assert_eq!(shulker.shulker_status, ShulkerBoxAnimationStatus::Opening);
    for _ in 0..ContainerBlockEntityModel::SHULKER_OPENING_TICK_LENGTH {
        shulker.tick_lid();
    }
    assert_eq!(shulker.shulker_status, ShulkerBoxAnimationStatus::Opened);
    assert!(!shulker.can_place_through_face(0, "minecraft:white_shulker_box", Direction::Up));
    assert!(shulker.can_place_through_face(0, "minecraft:diamond", Direction::Up));
    shulker.stop_open();
    for _ in 0..ContainerBlockEntityModel::SHULKER_OPENING_TICK_LENGTH {
        shulker.tick_lid();
    }
    assert!(shulker.shulker_is_closed());
}

fn assert_dispenser_and_dropper_activation_slots() {
    let mut dispenser = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Dispenser);
    assert_eq!(dispenser.kind.size(), 9);
    assert_eq!(dispenser.kind.menu_type(), "generic_3x3");
    assert_eq!(dispenser.kind.default_name(), "container.dispenser");
    dispenser.set_item(1, Some(stack("minecraft:arrow", 1)));
    dispenser.set_item(5, Some(stack("minecraft:egg", 1)));
    assert_eq!(
        dispenser.activate_once(&[0, 1]),
        ContainerActivation::Dispense { slot: 1 }
    );

    let mut dropper = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Dropper);
    assert_eq!(dropper.kind.size(), 9);
    assert_eq!(dropper.kind.menu_type(), "generic_3x3");
    assert_eq!(dropper.kind.default_name(), "container.dropper");
    dropper.set_item(2, Some(stack("minecraft:cobblestone", 1)));
    assert_eq!(
        dropper.activate_once(&[0]),
        ContainerActivation::Drop { slot: 2 }
    );
}

fn assert_hopper_slots_cooldown_push_pull_and_persistence() {
    let mut hopper = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Hopper);
    hopper.world_position = BlockPos {
        x: -3,
        y: 64,
        z: 9,
    };
    assert_eq!(hopper.kind.size(), 5);
    assert_eq!(
        ContainerBlockEntityModel::hopper_suck_aabb(),
        crate::collision_shape::Aabb::new(0.0, 11.0 / 16.0, 0.0, 1.0, 2.0, 1.0)
    );
    assert_eq!(hopper.hopper_level_x(), -2.5);
    assert_eq!(hopper.hopper_level_y(), 64.5);
    assert_eq!(hopper.hopper_level_z(), 9.5);
    assert!(hopper.hopper_is_grid_aligned());
    assert_eq!(
        hopper.transfer_cooldown,
        ContainerBlockEntityModel::HOPPER_NO_COOLDOWN
    );
    hopper.set_item(0, Some(stack("minecraft:iron_ingot", 1)));
    assert_eq!(
        hopper.hopper_slots_for_face(Direction::Down),
        vec![0, 1, 2, 3, 4]
    );
    assert!(hopper.hopper_can_place_item(1, &stack("minecraft:gold_ingot", 1), Direction::Up,));
    assert!(hopper.hopper_can_take_item(0, Direction::Down));
    assert_eq!(
        hopper.hopper_tick(true, true, true),
        ContainerActivation::Push { from_slot: 0 }
    );
    assert_eq!(
        hopper.transfer_cooldown,
        ContainerBlockEntityModel::HOPPER_MOVE_ITEM_SPEED
    );
    assert_eq!(
        hopper.hopper_tick(true, true, true),
        ContainerActivation::None
    );

    let mut pulling_hopper = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Hopper);
    pulling_hopper.transfer_cooldown = 0;
    assert_eq!(
        pulling_hopper.hopper_tick(true, false, true),
        ContainerActivation::Pull { to_slot: 0 }
    );
    let loaded_hopper = ContainerBlockEntityModel::load_additional(
        ContainerBlockEntityKind::Hopper,
        &pulling_hopper.save_additional(),
    );
    assert_eq!(
        loaded_hopper.transfer_cooldown,
        ContainerBlockEntityModel::HOPPER_MOVE_ITEM_SPEED
    );
}

#[test]
fn structure_block_entity_round_trips_bounds_and_render_box() {
    let mut structure = StructureBlockEntity::new(StructureBlockMode::Save);
    assert!(!structure.has_structure_name());
    assert_eq!(structure.structure_name(), "");
    structure.set_structure_name(Some("minecraft:village/plains/houses/plains_small_house_1"));
    structure.author = "Builder".to_string();
    structure.metadata = "data".to_string();
    structure.set_structure_pos(BlockPos {
        x: 99,
        y: -99,
        z: 7,
    });
    structure.set_structure_size((50, -2, 12));
    structure.mirror = StructureMirror::LeftRight;
    structure.rotation = StructureRotation::Clockwise90;
    structure.ignore_entities = false;
    structure.strict = true;
    structure.powered = true;
    structure.show_air = true;
    structure.show_bounding_box = false;
    structure.integrity = 0.65;
    structure.seed = 12345;

    assert_eq!(
        structure.structure_pos,
        BlockPos {
            x: 48,
            y: -48,
            z: 7
        }
    );
    assert_eq!(structure.structure_size, (48, 0, 12));
    assert_eq!(
        structure.render_mode(),
        StructureRenderMode::BoxAndInvisibleBlocks
    );
    assert_eq!(
        structure.renderable_box(),
        StructureRenderableBox {
            min: BlockPos {
                x: 48,
                y: -48,
                z: 7
            },
            max: BlockPos {
                x: 60,
                y: -48,
                z: 55
            },
        }
    );
    let renderable = structure.renderable_box();
    assert_eq!(renderable.local_pos(), renderable.min);
    assert_eq!(renderable.size(), (12, 0, 48));
    assert_eq!(
        StructureRenderableBox::from_corners(60, -48, 55, 48, -48, 7),
        renderable
    );

    let saved = structure.save_additional();
    assert_eq!(StructureBlockEntity::load_additional(&saved), structure);
    assert_eq!(structure.get_update_tag(), saved);

    let loaded = StructureBlockEntity::load_additional(&Tag::Compound(vec![
        ("name".to_string(), Tag::String(String::new())),
        ("posX".to_string(), Tag::Int(-99)),
        ("posY".to_string(), Tag::Int(2)),
        ("posZ".to_string(), Tag::Int(99)),
        ("sizeX".to_string(), Tag::Int(-1)),
        ("sizeY".to_string(), Tag::Int(64)),
        ("sizeZ".to_string(), Tag::Int(9)),
        (
            "rotation".to_string(),
            Tag::String("CLOCKWISE_180".to_string()),
        ),
        ("mirror".to_string(), Tag::String("FRONT_BACK".to_string())),
        ("mode".to_string(), Tag::String("LOAD".to_string())),
        ("showboundingbox".to_string(), Tag::Byte(0)),
    ]));
    assert_eq!(loaded.structure_name, None);
    assert_eq!(
        loaded.structure_pos,
        BlockPos {
            x: -48,
            y: 2,
            z: 48
        }
    );
    assert_eq!(loaded.structure_size, (0, 48, 9));
    assert_eq!(loaded.rotation, StructureRotation::Clockwise180);
    assert_eq!(loaded.mirror, StructureMirror::FrontBack);
    assert_eq!(loaded.mode, StructureBlockMode::Load);
    assert_eq!(loaded.render_mode(), StructureRenderMode::None);
    assert!(loaded.ignore_entities);
    assert_eq!(loaded.integrity, 1.0);
}

#[test]
fn banner_block_entity_tracks_color_patterns_and_update_tag_shape() {
    let mut banner =
        BannerBlockEntity::from_block_state("minecraft:light_blue_wall_banner").unwrap();
    assert_eq!(banner.base_color, DyeColor::LightBlue);
    assert!(banner.add_pattern("minecraft:stripe_bottom", DyeColor::Red));
    assert!(banner.add_pattern("minecraft:flower", DyeColor::Yellow));
    assert!(banner.add_pattern("minecraft:creeper", DyeColor::Green));
    assert!(banner.add_pattern("minecraft:skull", DyeColor::Black));
    assert!(banner.add_pattern("minecraft:mojang", DyeColor::Purple));
    assert!(banner.add_pattern("minecraft:globe", DyeColor::White));
    assert!(!banner.add_pattern("minecraft:extra", DyeColor::Orange));
    banner.custom_name = Some("{\"text\":\"Marker\"}".to_string());

    let saved = banner.save_additional();
    assert_eq!(
        saved,
        Tag::Compound(vec![
            (
                "patterns".to_string(),
                Tag::List(vec![
                    Tag::Compound(vec![
                        (
                            "pattern".to_string(),
                            Tag::String("minecraft:stripe_bottom".to_string())
                        ),
                        ("color".to_string(), Tag::String("red".to_string())),
                    ]),
                    Tag::Compound(vec![
                        (
                            "pattern".to_string(),
                            Tag::String("minecraft:flower".to_string())
                        ),
                        ("color".to_string(), Tag::String("yellow".to_string())),
                    ]),
                    Tag::Compound(vec![
                        (
                            "pattern".to_string(),
                            Tag::String("minecraft:creeper".to_string())
                        ),
                        ("color".to_string(), Tag::String("green".to_string())),
                    ]),
                    Tag::Compound(vec![
                        (
                            "pattern".to_string(),
                            Tag::String("minecraft:skull".to_string())
                        ),
                        ("color".to_string(), Tag::String("black".to_string())),
                    ]),
                    Tag::Compound(vec![
                        (
                            "pattern".to_string(),
                            Tag::String("minecraft:mojang".to_string())
                        ),
                        ("color".to_string(), Tag::String("purple".to_string())),
                    ]),
                    Tag::Compound(vec![
                        (
                            "pattern".to_string(),
                            Tag::String("minecraft:globe".to_string())
                        ),
                        ("color".to_string(), Tag::String("white".to_string())),
                    ]),
                ])
            ),
            (
                "CustomName".to_string(),
                Tag::String("{\"text\":\"Marker\"}".to_string())
            ),
        ])
    );

    let loaded =
        BannerBlockEntity::load_additional("minecraft:light_blue_wall_banner", &saved).unwrap();
    assert_eq!(loaded, banner);
    assert_eq!(BannerBlockEntity::from_block_state("minecraft:stone"), None);

    let mut entity = BlockEntity::new(
        BlockEntityTypeId::Banner,
        pos(),
        "minecraft:light_blue_banner",
    )
    .unwrap();
    entity.custom_data.insert(
        "patterns".to_string(),
        match saved {
            Tag::Compound(fields) => fields
                .into_iter()
                .find(|(key, _)| key == "patterns")
                .map(|(_, value)| value)
                .unwrap(),
            _ => unreachable!(),
        },
    );
    assert!(
        matches!(entity.get_update_tag(), Tag::Compound(fields) if fields.iter().any(|(key, _)| key == "patterns") && fields.iter().all(|(key, _)| key != "id"))
    );
}

#[test]
fn furnace_cooking_recipe_lookup_derives_from_loaded_cooking_recipes() {
    use crate::recipe_system::{
        CookingKind, IngredientSpec, ItemAmount, RecipeHolder, RecipeKind, RecipeMap,
    };

    // raw_iron smelts to iron_ingot at the default 200 ticks, 0.7 XP; the same
    // ingredient also has a blasting recipe at 100 ticks.
    let recipes = RecipeMap::create(vec![
        RecipeHolder {
            id: "minecraft:iron_ingot_from_smelting_raw_iron",
            recipe: RecipeKind::Cooking {
                kind: CookingKind::Smelting,
                ingredient: IngredientSpec::Item("minecraft:raw_iron"),
                result: ItemAmount::one("minecraft:iron_ingot"),
                experience_millis: 700,
                cooking_time: None, // -> default 200
                category: crate::recipe_system::CookingBookCategory::Misc,
            },
        },
        RecipeHolder {
            id: "minecraft:iron_ingot_from_blasting_raw_iron",
            recipe: RecipeKind::Cooking {
                kind: CookingKind::Blasting,
                ingredient: IngredientSpec::Item("minecraft:raw_iron"),
                result: ItemAmount::one("minecraft:iron_ingot"),
                experience_millis: 700,
                cooking_time: None, // -> default 100
                category: crate::recipe_system::CookingBookCategory::Misc,
            },
        },
    ]);

    let smelting = FurnaceCookingRecipe::lookup(&recipes, "smelting", "minecraft:raw_iron")
        .expect("smelting recipe should be found");
    assert_eq!(smelting.result.item_id, "minecraft:iron_ingot");
    assert_eq!(smelting.cooking_time, 200, "smelting default cook time");
    assert_eq!(smelting.experience_millis, 700);

    let blasting = FurnaceCookingRecipe::lookup(&recipes, "blasting", "minecraft:raw_iron")
        .expect("blasting recipe should be found");
    assert_eq!(blasting.cooking_time, 100, "blasting default cook time");

    // No recipe for an unrelated input, and a smoker has no matching recipe here.
    assert!(FurnaceCookingRecipe::lookup(&recipes, "smelting", "minecraft:stone").is_none());
    assert!(FurnaceCookingRecipe::lookup(&recipes, "smoking", "minecraft:raw_iron").is_none());
}
