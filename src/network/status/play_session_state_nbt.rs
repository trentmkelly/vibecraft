use super::*;

fn play_session_base_nbt_values(state: &PlaySessionState) -> Vec<(String, Tag)> {
    vec![
        ("DataVersion".to_string(), Tag::Int(4791)),
        (
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(state.x),
                Tag::Double(state.y),
                Tag::Double(state.z),
            ]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(state.yaw), Tag::Float(state.pitch)]),
        ),
        (
            "Motion".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(0.0), Tag::Double(0.0)]),
        ),
        ("OnGround".to_string(), Tag::Byte(i8::from(state.on_ground))),
        ("Air".to_string(), Tag::Short(state.air_supply as i16)),
        (
            "fall_distance".to_string(),
            Tag::Double(state.fall_distance as f64),
        ),
        ("Health".to_string(), Tag::Float(state.health)),
        ("foodLevel".to_string(), Tag::Int(state.food_level)),
        (
            "foodSaturationLevel".to_string(),
            Tag::Float(state.food_saturation),
        ),
        (
            "foodExhaustionLevel".to_string(),
            Tag::Float(state.food_exhaustion),
        ),
        ("foodTickTimer".to_string(), Tag::Int(state.food_tick_timer)),
        ("XpLevel".to_string(), Tag::Int(state.xp_level)),
        ("XpP".to_string(), Tag::Float(state.xp_progress)),
        ("XpTotal".to_string(), Tag::Int(state.xp_total)),
        ("XpSeed".to_string(), Tag::Int(state.xp_seed)),
        ("Score".to_string(), Tag::Int(state.score)),
        (
            "SelectedItemSlot".to_string(),
            Tag::Int(state.selected_slot),
        ),
        (
            "playerGameType".to_string(),
            Tag::Int(game_mode_legacy_id(state.game_mode)),
        ),
        (
            "Dimension".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        ),
        (
            "seenCredits".to_string(),
            Tag::Byte(i8::from(state.seen_credits)),
        ),
        (
            "recipeBook".to_string(),
            play_session_recipe_book_nbt(state),
        ),
        ("abilities".to_string(), play_session_abilities_nbt(state)),
        (
            "EnderItems".to_string(),
            Tag::List(state.ender_items.clone()),
        ),
        (
            "active_effects".to_string(),
            Tag::List(state.active_effects.clone()),
        ),
    ]
}

fn play_session_recipe_book_nbt(state: &PlaySessionState) -> Tag {
    Tag::Compound(vec![
        (
            "recipes".to_string(),
            Tag::List(
                state
                    .inventory_menu
                    .recipe_book_known_recipes()
                    .into_iter()
                    .map(|id| Tag::String(id.to_string()))
                    .collect(),
            ),
        ),
        (
            "toBeDisplayed".to_string(),
            Tag::List(
                state
                    .inventory_menu
                    .recipe_book_highlighted_recipes()
                    .into_iter()
                    .map(|id| Tag::String(id.to_string()))
                    .collect(),
            ),
        ),
        (
            "isGuiOpen".to_string(),
            Tag::Byte(i8::from(state.recipe_book_settings.crafting.open)),
        ),
        (
            "isFilteringCraftable".to_string(),
            Tag::Byte(i8::from(state.recipe_book_settings.crafting.filtering)),
        ),
        (
            "isFurnaceGuiOpen".to_string(),
            Tag::Byte(i8::from(state.recipe_book_settings.furnace.open)),
        ),
        (
            "isFurnaceFilteringCraftable".to_string(),
            Tag::Byte(i8::from(state.recipe_book_settings.furnace.filtering)),
        ),
        (
            "isBlastingFurnaceGuiOpen".to_string(),
            Tag::Byte(i8::from(state.recipe_book_settings.blast_furnace.open)),
        ),
        (
            "isBlastingFurnaceFilteringCraftable".to_string(),
            Tag::Byte(i8::from(state.recipe_book_settings.blast_furnace.filtering)),
        ),
        (
            "isSmokerGuiOpen".to_string(),
            Tag::Byte(i8::from(state.recipe_book_settings.smoker.open)),
        ),
        (
            "isSmokerFilteringCraftable".to_string(),
            Tag::Byte(i8::from(state.recipe_book_settings.smoker.filtering)),
        ),
    ])
}

fn play_session_abilities_nbt(state: &PlaySessionState) -> Tag {
    Tag::Compound(vec![
        (
            "invulnerable".to_string(),
            Tag::Byte(i8::from(state.abilities.invulnerable)),
        ),
        (
            "flying".to_string(),
            Tag::Byte(i8::from(state.abilities.flying)),
        ),
        (
            "mayfly".to_string(),
            Tag::Byte(i8::from(state.abilities.mayfly)),
        ),
        (
            "instabuild".to_string(),
            Tag::Byte(i8::from(state.abilities.instabuild)),
        ),
        (
            "mayBuild".to_string(),
            Tag::Byte(i8::from(state.abilities.may_build)),
        ),
        (
            "flySpeed".to_string(),
            Tag::Float(state.abilities.fly_speed),
        ),
        (
            "walkSpeed".to_string(),
            Tag::Float(state.abilities.walk_speed),
        ),
    ])
}

fn append_play_session_optional_nbt_values(
    values: &mut Vec<(String, Tag)>,
    state: &PlaySessionState,
) {
    if let Some(mode) = state.previous_game_mode {
        values.push((
            "previousPlayerGameType".to_string(),
            Tag::Int(game_mode_legacy_id(mode)),
        ));
    }
    if let Some(spawn) = &state.spawn {
        values.push(("SpawnX".to_string(), Tag::Int(spawn.x)));
        values.push(("SpawnY".to_string(), Tag::Int(spawn.y)));
        values.push(("SpawnZ".to_string(), Tag::Int(spawn.z)));
        values.push(("SpawnForced".to_string(), Tag::Byte(i8::from(spawn.forced))));
        values.push((
            "SpawnDimension".to_string(),
            Tag::String(spawn.dimension.clone()),
        ));
    }
    if let Some((x, y, z)) = state.entered_nether_position {
        values.push((
            "enteredNetherPosition".to_string(),
            Tag::Compound(vec![
                ("x".to_string(), Tag::Double(x)),
                ("y".to_string(), Tag::Double(y)),
                ("z".to_string(), Tag::Double(z)),
            ]),
        ));
    }
    if let Some(last_death) = &state.last_death_location {
        values.push((
            "LastDeathLocation".to_string(),
            Tag::Compound(vec![
                (
                    "dimension".to_string(),
                    Tag::String(last_death.dimension.clone()),
                ),
                (
                    "pos".to_string(),
                    Tag::List(vec![
                        Tag::Int(last_death.x),
                        Tag::Int(last_death.y),
                        Tag::Int(last_death.z),
                    ]),
                ),
            ]),
        ));
    }
    if let Some(root_vehicle) = &state.root_vehicle {
        values.push(("RootVehicle".to_string(), root_vehicle.clone()));
    }
}

fn play_session_inventory_nbt(state: &PlaySessionState) -> Tag {
    // Serialize the hotbar and main inventory (slots 0-35) as a TAG_List of TAG_Compound
    // entries, matching vanilla's player NBT format.
    // Java: ServerPlayer.addAdditionalSaveData() -> Inventory.save()
    Tag::List(
        state
            .inventory_menu
            .player_inventory()
            .saved_items()
            .into_iter()
            .map(|(slot, stack)| {
                Tag::Compound(vec![
                    ("Slot".to_string(), Tag::Byte(slot as i8)),
                    ("id".to_string(), Tag::String(stack.item_id().to_string())),
                    ("count".to_string(), Tag::Int(stack.count())),
                ])
            })
            .collect(),
    )
}

pub fn play_session_state_to_nbt(state: &PlaySessionState) -> Tag {
    let mut values = play_session_base_nbt_values(state);
    append_play_session_optional_nbt_values(&mut values, state);
    values.push(("Inventory".to_string(), play_session_inventory_nbt(state)));
    Tag::Compound(values)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LoadedPlayerPose {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
    fall_distance: f32,
    air_supply: i32,
    selected_slot: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LoadedPlayerVitals {
    health: f32,
    food_level: i32,
    food_saturation: f32,
    food_exhaustion: f32,
    food_tick_timer: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LoadedPlayerProgress {
    xp_progress: f32,
    xp_level: i32,
    xp_total: i32,
    xp_seed: i32,
    score: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LoadedPlayerModes {
    game_mode: GameMode,
    previous_game_mode: Option<GameMode>,
    seen_credits: bool,
}

struct LoadedPlayerExtraNbt {
    spawn: Option<PlayerSpawnData>,
    entered_nether_position: Option<(f64, f64, f64)>,
    last_death_location: Option<PlayerGlobalPosData>,
    root_vehicle: Option<Tag>,
    active_effects: Vec<Tag>,
    ender_items: Vec<Tag>,
}

fn load_player_pose_from_nbt(compound: &[(String, Tag)]) -> Option<LoadedPlayerPose> {
    let pos = compound_list(compound, "Pos")?;
    let rotation = compound_list(compound, "Rotation")?;
    let [Tag::Double(x), Tag::Double(y), Tag::Double(z)] = pos else {
        return None;
    };
    let [Tag::Float(yaw), Tag::Float(pitch)] = rotation else {
        return None;
    };
    let on_ground = match compound_tag(compound, "OnGround") {
        Some(Tag::Byte(value)) => *value != 0,
        _ => true,
    };
    let fall_distance = match compound_tag(compound, "fall_distance") {
        Some(Tag::Double(value)) => (*value as f32).max(0.0),
        Some(Tag::Float(value)) => value.max(0.0),
        _ => 0.0,
    };
    let air_supply = match compound_tag(compound, "Air") {
        Some(Tag::Short(value)) => {
            i32::from(*value).clamp(DROWN_AIR_SUPPLY_THRESHOLD, MAX_AIR_SUPPLY)
        }
        Some(Tag::Int(value)) => (*value).clamp(DROWN_AIR_SUPPLY_THRESHOLD, MAX_AIR_SUPPLY),
        _ => MAX_AIR_SUPPLY,
    };
    let selected_slot = match compound_tag(compound, "SelectedItemSlot") {
        Some(Tag::Int(value)) if (0..9).contains(value) => *value,
        _ => 0,
    };
    Some(LoadedPlayerPose {
        x: *x,
        y: *y,
        z: *z,
        yaw: *yaw,
        pitch: *pitch,
        on_ground,
        fall_distance,
        air_supply,
        selected_slot,
    })
}

fn load_player_vitals_from_nbt(compound: &[(String, Tag)]) -> LoadedPlayerVitals {
    let health = match compound_tag(compound, "Health") {
        Some(Tag::Float(value)) => value.clamp(0.0, 20.0),
        _ => 20.0,
    };
    let food_level = match compound_tag(compound, "foodLevel") {
        Some(Tag::Int(value)) => (*value).clamp(0, 20),
        _ => 20,
    };
    let food_saturation = match compound_tag(compound, "foodSaturationLevel") {
        Some(Tag::Float(value)) => value.clamp(0.0, food_level as f32),
        _ => 5.0,
    };
    let food_exhaustion = match compound_tag(compound, "foodExhaustionLevel") {
        Some(Tag::Float(value)) => value.max(0.0),
        _ => 0.0,
    };
    let food_tick_timer = match compound_tag(compound, "foodTickTimer") {
        Some(Tag::Int(value)) => (*value).max(0),
        _ => 0,
    };
    LoadedPlayerVitals {
        health,
        food_level,
        food_saturation,
        food_exhaustion,
        food_tick_timer,
    }
}

fn load_player_progress_from_nbt(compound: &[(String, Tag)]) -> LoadedPlayerProgress {
    let xp_progress = match compound_tag(compound, "XpP") {
        Some(Tag::Float(value)) => value.clamp(0.0, 1.0),
        _ => 0.0,
    };
    let xp_level = match compound_tag(compound, "XpLevel") {
        Some(Tag::Int(value)) => (*value).max(0),
        _ => 0,
    };
    let xp_total = match compound_tag(compound, "XpTotal") {
        Some(Tag::Int(value)) => (*value).max(0),
        _ => 0,
    };
    let xp_seed = match compound_tag(compound, "XpSeed") {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    };
    let score = match compound_tag(compound, "Score") {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    };
    LoadedPlayerProgress {
        xp_progress,
        xp_level,
        xp_total,
        xp_seed,
        score,
    }
}

fn load_player_modes_from_nbt(
    compound: &[(String, Tag)],
    default_game_mode: GameMode,
) -> LoadedPlayerModes {
    let game_mode = match compound_tag(compound, "playerGameType") {
        Some(Tag::Int(value)) => game_mode_from_legacy_id(*value),
        _ => default_game_mode,
    };
    let previous_game_mode = match compound_tag(compound, "previousPlayerGameType") {
        Some(Tag::Int(value)) if *value == -1 => None,
        Some(Tag::Int(value)) => Some(game_mode_from_legacy_id(*value)),
        _ => None,
    };
    let seen_credits =
        matches!(compound_tag(compound, "seenCredits"), Some(Tag::Byte(value)) if *value != 0);
    LoadedPlayerModes {
        game_mode,
        previous_game_mode,
        seen_credits,
    }
}

fn load_player_spawn_from_nbt(compound: &[(String, Tag)]) -> Option<PlayerSpawnData> {
    match (
        compound_tag(compound, "SpawnX"),
        compound_tag(compound, "SpawnY"),
        compound_tag(compound, "SpawnZ"),
    ) {
        (Some(Tag::Int(x)), Some(Tag::Int(y)), Some(Tag::Int(z))) => Some(PlayerSpawnData {
            dimension: match compound_tag(compound, "SpawnDimension") {
                Some(Tag::String(value)) => value.clone(),
                _ => "minecraft:overworld".to_string(),
            },
            x: *x,
            y: *y,
            z: *z,
            forced: matches!(
                compound_tag(compound, "SpawnForced"),
                Some(Tag::Byte(value)) if *value != 0
            ),
        }),
        _ => None,
    }
}

fn load_entered_nether_position_from_nbt(compound: &[(String, Tag)]) -> Option<(f64, f64, f64)> {
    match compound_tag(compound, "enteredNetherPosition") {
        Some(Tag::Compound(fields)) => match (
            compound_tag(fields, "x"),
            compound_tag(fields, "y"),
            compound_tag(fields, "z"),
        ) {
            (Some(Tag::Double(x)), Some(Tag::Double(y)), Some(Tag::Double(z))) => {
                Some((*x, *y, *z))
            }
            _ => None,
        },
        _ => None,
    }
}

fn load_last_death_location_from_nbt(compound: &[(String, Tag)]) -> Option<PlayerGlobalPosData> {
    match compound_tag(compound, "LastDeathLocation") {
        Some(Tag::Compound(fields)) => match (
            compound_tag(fields, "dimension"),
            compound_list(fields, "pos"),
        ) {
            (Some(Tag::String(dimension)), Some([Tag::Int(x), Tag::Int(y), Tag::Int(z)])) => {
                Some(PlayerGlobalPosData {
                    dimension: dimension.clone(),
                    x: *x,
                    y: *y,
                    z: *z,
                })
            }
            _ => None,
        },
        _ => None,
    }
}

fn compound_list_clone(compound: &[(String, Tag)], key: &str) -> Vec<Tag> {
    match compound_tag(compound, key) {
        Some(Tag::List(values)) => values.clone(),
        _ => Vec::new(),
    }
}

fn load_player_extra_nbt(compound: &[(String, Tag)]) -> LoadedPlayerExtraNbt {
    LoadedPlayerExtraNbt {
        spawn: load_player_spawn_from_nbt(compound),
        entered_nether_position: load_entered_nether_position_from_nbt(compound),
        last_death_location: load_last_death_location_from_nbt(compound),
        root_vehicle: compound_tag(compound, "RootVehicle").cloned(),
        active_effects: compound_list_clone(compound, "active_effects"),
        ender_items: compound_list_clone(compound, "EnderItems"),
    }
}

fn load_player_abilities_from_nbt(
    compound: &[(String, Tag)],
    game_mode: GameMode,
) -> PlayerNbtAbilities {
    let mut abilities = match compound_tag(compound, "abilities") {
        Some(Tag::Compound(fields)) => PlayerNbtAbilities {
            invulnerable: compound_bool_byte(fields, "invulnerable", false),
            flying: compound_bool_byte(fields, "flying", false),
            mayfly: compound_bool_byte(fields, "mayfly", false),
            instabuild: compound_bool_byte(fields, "instabuild", false),
            may_build: compound_bool_byte(fields, "mayBuild", true),
            fly_speed: compound_float(fields, "flySpeed", 0.05),
            walk_speed: compound_float(fields, "walkSpeed", 0.1),
        },
        _ => PlayerNbtAbilities::default_survival(),
    };
    abilities.apply_game_mode(game_mode);
    abilities
}

fn load_player_inventory_from_nbt(compound: &[(String, Tag)]) -> PlayerInventory {
    // Restore hotbar and main inventory (slots 0-35) from the TAG_List written by
    // play_session_state_to_nbt.
    // Java: ServerPlayer.readAdditionalSaveData() -> Inventory.load()
    let mut inventory = PlayerInventory::new();
    if let Some(Tag::List(items)) = compound_tag(compound, "Inventory") {
        let loaded: Vec<(usize, ItemStack)> = items
            .iter()
            .filter_map(loaded_inventory_item_from_nbt)
            .collect();
        if !loaded.is_empty() {
            inventory.load_items(&loaded);
        }
    }
    inventory
}

fn loaded_inventory_item_from_nbt(item_tag: &Tag) -> Option<(usize, ItemStack)> {
    let Tag::Compound(fields) = item_tag else {
        return None;
    };
    let slot = match compound_tag(fields, "Slot") {
        Some(Tag::Byte(value)) => *value as u8 as usize,
        _ => return None,
    };
    let id = match compound_tag(fields, "id") {
        Some(Tag::String(value)) => value.as_str(),
        _ => return None,
    };
    let count = match compound_tag(fields, "count") {
        Some(Tag::Int(value)) => *value,
        _ => 1,
    };
    let static_name = item_static_name(id)?;
    (slot < 36 && count > 0).then(|| (slot, ItemStack::new(static_name, count)))
}

pub fn play_session_state_from_nbt(
    tag: &Tag,
    default_game_mode: GameMode,
    recipes: &RecipeMap,
) -> Option<PlaySessionState> {
    let compound = match tag {
        Tag::Compound(values) => values,
        _ => return None,
    };
    let pose = load_player_pose_from_nbt(compound)?;
    let vitals = load_player_vitals_from_nbt(compound);
    let progress = load_player_progress_from_nbt(compound);
    let modes = load_player_modes_from_nbt(compound, default_game_mode);
    let extra = load_player_extra_nbt(compound);
    let abilities = load_player_abilities_from_nbt(compound, modes.game_mode);
    let inventory = load_player_inventory_from_nbt(compound);
    let (recipe_book_settings, known_recipes, highlighted_recipes) =
        load_recipe_book_from_nbt(compound, recipes);
    let mut inventory_menu = InventoryMenu::new(inventory, recipes.clone());
    inventory_menu.load_recipe_book(known_recipes, highlighted_recipes);
    Some(PlaySessionState {
        x: pose.x,
        y: pose.y,
        z: pose.z,
        yaw: pose.yaw,
        pitch: pose.pitch,
        on_ground: pose.on_ground,
        fall_distance: pose.fall_distance,
        selected_slot: pose.selected_slot,
        health: vitals.health,
        food_level: vitals.food_level,
        food_saturation: vitals.food_saturation,
        food_exhaustion: vitals.food_exhaustion,
        food_tick_timer: vitals.food_tick_timer,
        input_forward: false,
        input_backward: false,
        input_left: false,
        input_right: false,
        input_shift: false,
        input_sprinting: false,
        input_jumping: false,
        air_supply: pose.air_supply,
        in_water: false,
        eye_in_water: false,
        water_fluid_height: 0.0,
        water_velocity_x: 0.0,
        water_velocity_y: 0.0,
        water_velocity_z: 0.0,
        xp_progress: progress.xp_progress,
        xp_level: progress.xp_level,
        xp_total: progress.xp_total,
        xp_seed: progress.xp_seed,
        score: progress.score,
        game_mode: modes.game_mode,
        previous_game_mode: modes.previous_game_mode,
        spawn: extra.spawn,
        seen_credits: modes.seen_credits,
        entered_nether_position: extra.entered_nether_position,
        last_death_location: extra.last_death_location,
        root_vehicle: extra.root_vehicle,
        active_effects: extra.active_effects,
        ender_items: extra.ender_items,
        abilities,
        inventory_menu,
        carried_item: ItemStack::empty(),
        container_state_id: 0,
        recipe_book_settings,
    })
}
