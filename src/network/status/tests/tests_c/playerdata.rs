use super::*;

#[test]
pub fn play_session_state_nbt_round_trip_preserves_hotbar_and_main_inventory() {
    // Slot 0 (hotbar), slot 9 (main inventory row 1), slot 35 (last main slot).
    // Java: Inventory saves slots 0-35 — all three must survive the round-trip.
    let original_items = &[
        ("minecraft:dirt", 64, 0usize),
        ("minecraft:stone", 32, 9),
        ("minecraft:sand", 16, 35),
    ];
    let state = session_state_with_inventory(original_items);
    let tag = play_session_state_to_nbt(&state);
    let restored =
        play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();

    let saved = restored.inventory_menu.player_inventory().saved_items();
    assert_eq!(saved.len(), 3, "expected exactly 3 items after round-trip");

    for (id, count, slot) in original_items {
        let found = saved.iter().find(|(s, _)| s == slot);
        let found = found.unwrap_or_else(|| panic!("slot {slot} missing after round-trip"));
        assert_eq!(found.1.item_id(), *id, "item id mismatch at slot {slot}");
        assert_eq!(found.1.count(), *count, "count mismatch at slot {slot}");
    }
}

#[test]
pub fn play_session_state_nbt_round_trip_ignores_out_of_range_slots() {
    // Slots >= 36 (armour, offhand, etc.) are outside the main inventory range
    // and must be silently dropped during deserialisation.
    // Java: Inventory.load() only writes to slots 0-35.
    let tag = Tag::Compound(vec![
        ("DataVersion".to_string(), Tag::Int(4791)),
        (
            "Pos".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(64.0), Tag::Double(0.0)]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(0.0), Tag::Float(0.0)]),
        ),
        (
            "Motion".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(0.0), Tag::Double(0.0)]),
        ),
        ("OnGround".to_string(), Tag::Byte(1)),
        ("Health".to_string(), Tag::Float(20.0)),
        ("foodLevel".to_string(), Tag::Int(20)),
        ("foodSaturationLevel".to_string(), Tag::Float(5.0)),
        ("XpLevel".to_string(), Tag::Int(0)),
        ("XpP".to_string(), Tag::Float(0.0)),
        ("XpTotal".to_string(), Tag::Int(0)),
        ("SelectedItemSlot".to_string(), Tag::Int(0)),
        ("playerGameType".to_string(), Tag::Int(0)),
        (
            "Inventory".to_string(),
            Tag::List(vec![
                // Valid slot
                Tag::Compound(vec![
                    ("Slot".to_string(), Tag::Byte(0)),
                    ("id".to_string(), Tag::String("minecraft:dirt".to_string())),
                    ("count".to_string(), Tag::Int(1)),
                ]),
                // Out-of-range slot (armour slot 100) — must be ignored
                Tag::Compound(vec![
                    ("Slot".to_string(), Tag::Byte(100u8 as i8)),
                    ("id".to_string(), Tag::String("minecraft:stone".to_string())),
                    ("count".to_string(), Tag::Int(1)),
                ]),
            ]),
        ),
    ]);
    let restored =
        play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();
    let saved = restored.inventory_menu.player_inventory().saved_items();
    assert_eq!(saved.len(), 1, "only the in-range slot should survive");
    assert_eq!(saved[0].0, 0);
    assert_eq!(saved[0].1.item_id(), "minecraft:dirt");
}

#[test]
pub fn play_session_state_from_nbt_clamps_vanilla_playerdata_bounds() {
    let tag = Tag::Compound(vec![
        ("DataVersion".to_string(), Tag::Int(4790)),
        (
            "Pos".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(64.0), Tag::Double(0.0)]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(0.0), Tag::Float(0.0)]),
        ),
        ("Health".to_string(), Tag::Float(200.0)),
        ("foodLevel".to_string(), Tag::Int(99)),
        ("foodSaturationLevel".to_string(), Tag::Float(99.0)),
        ("XpP".to_string(), Tag::Float(9.0)),
        ("XpLevel".to_string(), Tag::Int(-7)),
        ("XpTotal".to_string(), Tag::Int(-12)),
        ("SelectedItemSlot".to_string(), Tag::Int(99)),
        ("playerGameType".to_string(), Tag::Int(99)),
    ]);

    let restored =
        play_session_state_from_nbt(&tag, GameMode::Creative, &RecipeMap::default()).unwrap();
    assert_eq!(restored.health, 20.0);
    assert_eq!(restored.food_level, 20);
    assert_eq!(restored.food_saturation, 20.0);
    assert_eq!(restored.air_supply, crate::network::status::MAX_AIR_SUPPLY);
    assert_eq!(restored.xp_progress, 1.0);
    assert_eq!(restored.xp_level, 0);
    assert_eq!(restored.xp_total, 0);
    assert_eq!(restored.selected_slot, 0);
    assert_eq!(restored.game_mode, GameMode::Survival);
}

#[test]
pub fn play_session_state_nbt_round_trip_preserves_full_playerdata_surface() {
    let state = full_playerdata_surface_state();
    let tag = play_session_state_to_nbt(&state);

    assert_playerdata_surface_fields_present(&tag);
    let restored =
        play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();
    assert_playerdata_surface_round_trip(&state, &restored);
}

fn full_playerdata_surface_state() -> PlaySessionState {
    let mut state = session_state_with_inventory(&[("minecraft:stone", 5, 3)]);
    state.fall_distance = 6.25;
    state.food_exhaustion = 3.5;
    state.food_tick_timer = 72;
    state.air_supply = 123;
    state.xp_progress = 0.75;
    state.xp_level = 12;
    state.xp_total = 345;
    state.xp_seed = 98_765;
    state.score = 42;
    state.previous_game_mode = Some(GameMode::Adventure);
    state.spawn = Some(PlayerSpawnData {
        dimension: "minecraft:the_nether".to_string(),
        x: 11,
        y: 72,
        z: -13,
        forced: true,
    });
    state.seen_credits = true;
    state.entered_nether_position = Some((1.25, 64.0, -2.5));
    state.last_death_location = Some(PlayerGlobalPosData {
        dimension: "minecraft:overworld".to_string(),
        x: 3,
        y: 65,
        z: 4,
    });
    state.root_vehicle = Some(Tag::Compound(vec![(
        "Entity".to_string(),
        Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:boat".to_string()),
        )]),
    )]));
    state.active_effects = vec![Tag::Compound(vec![
        ("id".to_string(), Tag::String("minecraft:speed".to_string())),
        ("amplifier".to_string(), Tag::Int(1)),
    ])];
    state.ender_items = vec![Tag::Compound(vec![
        ("Slot".to_string(), Tag::Byte(0)),
        (
            "id".to_string(),
            Tag::String("minecraft:diamond".to_string()),
        ),
        ("count".to_string(), Tag::Int(2)),
    ])];
    state.abilities = PlayerNbtAbilities {
        invulnerable: true,
        flying: true,
        mayfly: true,
        instabuild: true,
        may_build: false,
        fly_speed: 0.08,
        walk_speed: 0.12,
    };
    state
}

fn assert_playerdata_surface_fields_present(tag: &Tag) {
    for field in [
        "Pos",
        "Rotation",
        "Motion",
        "Air",
        "fall_distance",
        "Health",
        "foodLevel",
        "foodSaturationLevel",
        "foodExhaustionLevel",
        "foodTickTimer",
        "XpP",
        "XpLevel",
        "XpTotal",
        "XpSeed",
        "Score",
        "SelectedItemSlot",
        "Inventory",
        "EnderItems",
        "playerGameType",
        "previousPlayerGameType",
        "SpawnX",
        "SpawnY",
        "SpawnZ",
        "SpawnForced",
        "SpawnDimension",
        "seenCredits",
        "recipeBook",
        "LastDeathLocation",
        "enteredNetherPosition",
        "RootVehicle",
        "abilities",
        "active_effects",
    ] {
        assert!(
            field_value(tag, field).is_some(),
            "{field} missing from player NBT"
        );
    }
}

fn assert_playerdata_surface_round_trip(state: &PlaySessionState, restored: &PlaySessionState) {
    assert_eq!(restored.fall_distance, 6.25);
    assert_eq!(restored.food_exhaustion, 3.5);
    assert_eq!(restored.food_tick_timer, 72);
    assert_eq!(restored.air_supply, 123);
    assert_eq!(restored.xp_seed, 98_765);
    assert_eq!(restored.score, 42);
    assert_eq!(restored.previous_game_mode, Some(GameMode::Adventure));
    assert_eq!(restored.spawn, state.spawn);
    assert!(restored.seen_credits);
    assert_eq!(
        restored.entered_nether_position,
        state.entered_nether_position
    );
    assert_eq!(restored.last_death_location, state.last_death_location);
    assert_eq!(restored.root_vehicle, state.root_vehicle);
    assert_eq!(restored.active_effects, state.active_effects);
    assert_eq!(restored.ender_items, state.ender_items);
    let mut expected_abilities = state.abilities;
    expected_abilities.apply_game_mode(state.game_mode);
    assert_eq!(restored.abilities, expected_abilities);
}

#[test]
pub fn play_session_state_nbt_inventory_tag_matches_vanilla_format() {
    // Verify the serialised TAG_List contains TAG_Compound entries with the
    // exact field names used by vanilla: "Slot" (TAG_Byte), "id" (TAG_String),
    // "count" (TAG_Int).  This is the wire format read back by the Java server
    // when loading player data.
    let state = session_state_with_inventory(&[("minecraft:stone", 5, 3)]);
    let tag = play_session_state_to_nbt(&state);
    let Tag::Compound(fields) = &tag else {
        panic!("expected compound tag");
    };
    let inventory_tag = fields
        .iter()
        .find_map(|(name, value)| (name == "Inventory").then_some(value))
        .expect("Inventory tag missing");
    let Tag::List(items) = inventory_tag else {
        panic!("Inventory must be a TAG_List");
    };
    assert_eq!(items.len(), 1);
    let Tag::Compound(item_fields) = &items[0] else {
        panic!("inventory entry must be TAG_Compound");
    };
    let slot = item_fields
        .iter()
        .find_map(|(n, v)| (n == "Slot").then_some(v))
        .expect("Slot missing");
    assert!(matches!(slot, Tag::Byte(3)), "Slot must be TAG_Byte(3)");
    let id = item_fields
        .iter()
        .find_map(|(n, v)| (n == "id").then_some(v))
        .expect("id missing");
    assert!(
        matches!(id, Tag::String(s) if s == "minecraft:stone"),
        "id must be TAG_String"
    );
    let count = item_fields
        .iter()
        .find_map(|(n, v)| (n == "count").then_some(v))
        .expect("count missing");
    assert!(matches!(count, Tag::Int(5)), "count must be TAG_Int(5)");
}

// ─── write_lp_vec3 ───────────────────────────────────────────────────────

#[test]
pub fn write_lp_vec3_zero_writes_single_zero_byte() {
    // Java: LpVec3.write — chessboard length below threshold → single 0x00 byte.
    let mut buf = Vec::new();
    write_lp_vec3(&mut buf, 0.0, 0.0, 0.0).unwrap();
    assert_eq!(buf, &[0u8]);
}

#[test]
pub fn write_lp_vec3_nonzero_writes_six_bytes_for_unit_scale() {
    // For velocity magnitude ≤ 1.0 the scale is 1 and isPartial=false → exactly 6 bytes.
    let mut buf = Vec::new();
    write_lp_vec3(&mut buf, 0.3, 0.1, -0.3).unwrap();
    assert_eq!(
        buf.len(),
        6,
        "scale=1 non-zero velocity should encode to 6 bytes"
    );
}
