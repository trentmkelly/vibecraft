use super::*;

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
    assert_eq!(VaultStateModel::Active.light_level(), 12);
    assert_eq!(vault.config.activation_range, 4.0);
    assert_eq!(vault.config.deactivation_range, 4.5);
    assert_eq!(vault.config.key_item.item_id, "minecraft:trial_key");
    assert_eq!(
        vault.config.loot_table,
        "minecraft:chests/trial_chambers/reward"
    );
    assert_eq!(vault.config.override_loot_table_to_display, None);
    assert_eq!(vault.config.validate(), Ok(()));
    assert!(VaultConfigModel {
        activation_range: 5.0,
        ..vault.config.clone()
    }
    .validate()
    .is_err());

    assert!(!vault.can_eject_reward());
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
        vault.config.deactivation_range
    );
    assert!(vault.connected_players.contains("player-a"));
    assert!(vault.can_eject_reward());

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
    assert_eq!(vault.ejection_progress(), 0.0);
    assert_eq!(
        vault.tick_server(56, &["player-a".to_string()], None),
        VaultTickResult::EjectedItem(stack("minecraft:diamond", 1))
    );
    assert_eq!(vault.display_item, Some(stack("minecraft:emerald", 2)));
    assert_eq!(vault.ejection_progress(), 1.0);
    assert_eq!(
        vault.tick_server(76, &["player-a".to_string()], None),
        VaultTickResult::EjectedItem(stack("minecraft:emerald", 2))
    );
    assert_eq!(
        vault.tick_server(96, &["player-a".to_string()], None),
        VaultTickResult::EjectionFinished
    );
    assert_eq!(vault.state, VaultStateModel::Inactive);
    assert_eq!(vault.state_updating_resumes_at, 116);
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
    vault.state_updating_resumes_at = 140;
    assert!(VaultBlockEntity::should_cycle_display_item(
        120,
        VaultStateModel::Active
    ));
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
    assert_eq!(vault.current_spin, VaultBlockEntity::CLIENT_ROTATION_SPEED);
    assert!(vault.display_active_effects());

    assert_eq!(
        VaultBlockEntity::keyhole_position(pos(), Direction::South),
        (18.5, 65.75, 36.0)
    );
    assert_eq!(
        VaultBlockEntity::random_pos_center_of_cage(pos(), 0.0, 0.5, 1.0),
        (18.4, 64.5, 35.6)
    );
    assert_eq!(
        VaultBlockEntity::random_pos_inside_cage(pos(), 0.25, 0.5, 0.75),
        (18.3, 64.5, 35.7)
    );

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
fn vault_display_cycle_does_not_skip_due_state_update_like_java() {
    let mut vault = VaultBlockEntity {
        state: VaultStateModel::Active,
        state_updating_resumes_at: 120,
        ..VaultBlockEntity::default()
    };

    assert_eq!(
        vault.tick_server(120, &[], Some(stack("minecraft:apple", 1))),
        VaultTickResult::StateChanged(VaultStateModel::Inactive)
    );
    assert_eq!(vault.display_item, None);
    assert_eq!(vault.state_updating_resumes_at, 140);
}
