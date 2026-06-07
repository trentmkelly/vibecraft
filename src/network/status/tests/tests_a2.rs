use super::super::*;
use super::*;

#[test]
pub fn duplicated_registry_manifest_ids_remain_in_sync_across_tables() {
    assert_trim_registry_ids_match_presentation_and_models();
    assert_instrument_registry_ids_match_presentation();
    assert_damage_and_biome_registry_ids_match_models();
    assert_painting_jukebox_and_banner_ids_match_presentation();
}

fn assert_trim_registry_ids_match_presentation_and_models() {
    let status_trim_materials: Vec<String> = TRIM_MATERIALS
        .iter()
        .map(|entry| format!("minecraft:{}", entry.id))
        .collect();
    let presentation_trim_materials: Vec<String> = presentation_data::TRIM_MATERIALS
        .iter()
        .map(|material| material.id.to_string())
        .collect();
    let model_trim_materials: Vec<String> = equipment_trim::TRIM_MATERIALS
        .iter()
        .map(|material| material.id.to_string())
        .collect();

    assert_eq!(status_trim_materials, presentation_trim_materials);
    assert_eq!(status_trim_materials, model_trim_materials);

    let status_trim_patterns: Vec<String> = TRIM_PATTERNS
        .iter()
        .map(|id| format!("minecraft:{id}"))
        .collect();
    let presentation_trim_patterns: Vec<String> = presentation_data::TRIM_PATTERNS
        .iter()
        .map(|pattern| pattern.id.to_string())
        .collect();
    let model_trim_patterns: Vec<String> = equipment_trim::TRIM_PATTERNS
        .iter()
        .map(|pattern| format!("minecraft:{}", pattern.id))
        .collect();

    assert_eq!(status_trim_patterns, presentation_trim_patterns);
    assert_eq!(status_trim_patterns, model_trim_patterns);
}

fn assert_instrument_registry_ids_match_presentation() {
    let status_instruments: BTreeSet<String> = INSTRUMENTS
        .iter()
        .map(|instrument| format!("minecraft:{}", instrument.id))
        .collect();
    let presentation_instruments: BTreeSet<String> = presentation_data::INSTRUMENTS
        .iter()
        .map(|instrument| instrument.id.to_string())
        .collect();

    assert_eq!(status_instruments, presentation_instruments);
}

fn assert_damage_and_biome_registry_ids_match_models() {
    let status_damage_types: BTreeSet<String> = DAMAGE_TYPES
        .iter()
        .map(|id| format!("minecraft:{id}"))
        .collect();
    let presentation_damage_types: BTreeSet<String> = presentation_data::DAMAGE_TYPES
        .iter()
        .map(|entry| entry.id.to_string())
        .collect();
    let model_damage_types: BTreeSet<String> = damage_type::BUILTIN_DAMAGE_TYPES
        .iter()
        .map(|entry| entry.id.to_string())
        .collect();

    assert!(status_damage_types.is_superset(&presentation_damage_types));
    assert!(status_damage_types.is_superset(&model_damage_types));

    let status_biomes: BTreeSet<String> = BIOMES
        .iter()
        .map(|id| format!("minecraft:{}", id))
        .collect();
    let model_biomes: BTreeSet<String> = biome::BUILTIN_BIOMES
        .iter()
        .map(|biome| biome.id.to_string())
        .collect();
    assert_eq!(status_biomes, model_biomes);
}

fn assert_painting_jukebox_and_banner_ids_match_presentation() {
    let status_paintings =
        status_registry_entry_ids_ordered(write_vanilla_painting_variant_registry_packet);
    let presentation_paintings: Vec<String> = presentation_data::PAINTING_VARIANTS
        .iter()
        .map(|painting| painting.id.to_string())
        .collect();
    assert_eq!(status_paintings, presentation_paintings);

    let status_jukebox_songs: BTreeSet<String> = JUKEBOX_SONGS
        .iter()
        .map(|song| format!("minecraft:{}", song.id))
        .collect();
    let presentation_jukebox_songs: BTreeSet<String> = presentation_data::JUKEBOX_SONGS
        .iter()
        .map(|song| song.id.to_string())
        .collect();
    assert!(presentation_jukebox_songs.is_subset(&status_jukebox_songs));

    let status_banner_patterns: BTreeSet<String> = BANNER_PATTERNS
        .iter()
        .map(|id| format!("minecraft:{}", id))
        .collect();
    let presentation_banner_patterns: BTreeSet<String> = presentation_data::BANNER_PATTERNS
        .iter()
        .map(|pattern| pattern.id.to_string())
        .collect();
    assert_eq!(status_banner_patterns, presentation_banner_patterns);
    let status_tag_names: BTreeSet<String> = BANNER_PATTERN_TAGS
        .iter()
        .map(|(tag, _)| tag.to_string())
        .collect();
    let expected_banner_pattern_tags: BTreeSet<String> = [
        "minecraft:no_item_required",
        "minecraft:pattern_item/flower",
        "minecraft:pattern_item/creeper",
        "minecraft:pattern_item/skull",
        "minecraft:pattern_item/mojang",
        "minecraft:pattern_item/globe",
        "minecraft:pattern_item/piglin",
        "minecraft:pattern_item/flow",
        "minecraft:pattern_item/guster",
        "minecraft:pattern_item/field_masoned",
        "minecraft:pattern_item/bordure_indented",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(status_tag_names, expected_banner_pattern_tags);
}

#[test]
pub fn synchronized_registry_closure_notes_match_writer_payloads() {
    for entry in SYNCHRONIZED_REGISTRY_MANIFEST {
        assert!(!entry.java_network_shape.is_empty());

        let registry_id = status_registry_id(entry.write_packet);
        assert_eq!(registry_id, entry.registry_id);

        let entries = status_registry_entry_ids_ordered(entry.write_packet);
        assert_eq!(entries.len(), entry.expected_entry_count);
    }
}

#[test]
pub fn synced_registry_entry_orders_match_official_transcript_fixtures() {
    assert_animal_sound_registry_order();
    assert_painting_registry_order();
    assert_damage_type_registry_order();
    assert_banner_and_jukebox_registry_order();
    assert_instrument_chat_trim_and_animal_registry_order();
}

fn assert_animal_sound_registry_order() {
    assert_registry_order(
        write_vanilla_cat_variant_registry_packet,
        &[
            "all_black",
            "black",
            "british_shorthair",
            "calico",
            "jellie",
            "persian",
            "ragdoll",
            "red",
            "siamese",
            "tabby",
            "white",
        ],
    );
    assert_registry_order(
        write_vanilla_cat_sound_variant_registry_packet,
        &["classic", "royal"],
    );
    assert_registry_order(
        write_vanilla_chicken_sound_variant_registry_packet,
        &["classic", "picky"],
    );
    assert_registry_order(
        write_vanilla_cow_sound_variant_registry_packet,
        &["classic", "moody"],
    );
    assert_registry_order(
        write_vanilla_pig_sound_variant_registry_packet,
        &["big", "classic", "mini"],
    );
    assert_registry_order(
        write_vanilla_wolf_sound_variant_registry_packet,
        &["angry", "big", "classic", "cute", "grumpy", "puglin", "sad"],
    );
}

fn assert_painting_registry_order() {
    assert_registry_order(
        write_vanilla_painting_variant_registry_packet,
        &[
            "alban",
            "aztec",
            "aztec2",
            "backyard",
            "baroque",
            "bomb",
            "bouquet",
            "burning_skull",
            "bust",
            "cavebird",
            "changing",
            "cotan",
            "courbet",
            "creebet",
            "dennis",
            "donkey_kong",
            "earth",
            "endboss",
            "fern",
            "fighters",
            "finding",
            "fire",
            "graham",
            "humble",
            "kebab",
            "lowmist",
            "match",
            "meditative",
            "orb",
            "owlemons",
            "passage",
            "pigscene",
            "plant",
            "pointer",
            "pond",
            "pool",
            "prairie_ride",
            "sea",
            "skeleton",
            "skull_and_roses",
            "stage",
            "sunflowers",
            "sunset",
            "tides",
            "unpacked",
            "void",
            "wanderer",
            "wasteland",
            "water",
            "wind",
            "wither",
        ],
    );
}

fn assert_damage_type_registry_order() {
    assert_registry_order(
        write_minimal_damage_type_registry_packet,
        &[
            "arrow",
            "bad_respawn_point",
            "cactus",
            "campfire",
            "cramming",
            "dragon_breath",
            "drown",
            "dry_out",
            "ender_pearl",
            "explosion",
            "fall",
            "falling_anvil",
            "falling_block",
            "falling_stalactite",
            "fireball",
            "fireworks",
            "fly_into_wall",
            "freeze",
            "generic",
            "generic_kill",
            "hot_floor",
            "in_fire",
            "in_wall",
            "indirect_magic",
            "lava",
            "lightning_bolt",
            "mace_smash",
            "magic",
            "mob_attack",
            "mob_attack_no_aggro",
            "mob_projectile",
            "on_fire",
            "out_of_world",
            "outside_border",
            "player_attack",
            "player_explosion",
            "sonic_boom",
            "spear",
            "spit",
            "stalagmite",
            "starve",
            "sting",
            "sweet_berry_bush",
            "thorns",
            "thrown",
            "trident",
            "unattributed_fireball",
            "wind_charge",
            "wither",
            "wither_skull",
        ],
    );
}

fn assert_banner_and_jukebox_registry_order() {
    assert_registry_order(
        write_vanilla_banner_pattern_registry_packet,
        &[
            "base",
            "border",
            "bricks",
            "circle",
            "creeper",
            "cross",
            "curly_border",
            "diagonal_left",
            "diagonal_right",
            "diagonal_up_left",
            "diagonal_up_right",
            "flow",
            "flower",
            "globe",
            "gradient",
            "gradient_up",
            "guster",
            "half_horizontal",
            "half_horizontal_bottom",
            "half_vertical",
            "half_vertical_right",
            "mojang",
            "piglin",
            "rhombus",
            "skull",
            "small_stripes",
            "square_bottom_left",
            "square_bottom_right",
            "square_top_left",
            "square_top_right",
            "straight_cross",
            "stripe_bottom",
            "stripe_center",
            "stripe_downleft",
            "stripe_downright",
            "stripe_left",
            "stripe_middle",
            "stripe_right",
            "stripe_top",
            "triangle_bottom",
            "triangle_top",
            "triangles_bottom",
            "triangles_top",
        ],
    );
    assert_registry_order(
        write_vanilla_jukebox_song_registry_packet,
        &[
            "11",
            "13",
            "5",
            "blocks",
            "cat",
            "chirp",
            "creator",
            "creator_music_box",
            "far",
            "lava_chicken",
            "mall",
            "mellohi",
            "otherside",
            "pigstep",
            "precipice",
            "relic",
            "stal",
            "strad",
            "tears",
            "wait",
            "ward",
        ],
    );
}

fn assert_instrument_chat_trim_and_animal_registry_order() {
    assert_registry_order(
        write_vanilla_instrument_registry_packet,
        &[
            "admire_goat_horn",
            "call_goat_horn",
            "dream_goat_horn",
            "feel_goat_horn",
            "ponder_goat_horn",
            "seek_goat_horn",
            "sing_goat_horn",
            "yearn_goat_horn",
        ],
    );
    assert_registry_order(
        write_vanilla_chat_type_registry_packet,
        &[
            "chat",
            "emote_command",
            "msg_command_incoming",
            "msg_command_outgoing",
            "say_command",
            "team_msg_command_incoming",
            "team_msg_command_outgoing",
        ],
    );
    assert_registry_order(
        write_minimal_trim_material_registry_packet,
        &[
            "quartz",
            "iron",
            "netherite",
            "redstone",
            "copper",
            "gold",
            "emerald",
            "diamond",
            "lapis",
            "amethyst",
            "resin",
        ],
    );
    assert_registry_order(
        write_vanilla_trim_pattern_registry_packet,
        &[
            "sentry",
            "dune",
            "coast",
            "wild",
            "ward",
            "eye",
            "vex",
            "tide",
            "snout",
            "rib",
            "spire",
            "wayfinder",
            "shaper",
            "silence",
            "raiser",
            "host",
            "flow",
            "bolt",
        ],
    );
    assert_registry_order(
        write_vanilla_wolf_variant_registry_packet,
        &[
            "ashen", "black", "chestnut", "pale", "rusty", "snowy", "spotted", "striped", "woods",
        ],
    );
    assert_registry_order(
        write_vanilla_pig_variant_registry_packet,
        &["cold", "temperate", "warm"],
    );
    assert_registry_order(
        write_vanilla_frog_variant_registry_packet,
        &["cold", "temperate", "warm"],
    );
    assert_registry_order(
        write_vanilla_cow_variant_registry_packet,
        &["cold", "temperate", "warm"],
    );
    assert_registry_order(
        write_vanilla_chicken_variant_registry_packet,
        &["cold", "temperate", "warm"],
    );
}

#[test]
pub fn chat_type_registry_payloads_include_vanilla_routes() {
    assert_eq!(
        registry_element_count(write_vanilla_chat_type_registry_packet),
        7
    );
    let outgoing = CHAT_TYPES
        .iter()
        .find(|chat_type| chat_type.id == "msg_command_outgoing")
        .expect("outgoing direct message chat type should be sent");
    let tag = chat_type_nbt(outgoing);

    let chat = compound_field(&tag, "chat");
    assert!(matches!(
        field_value(chat, "translation_key"),
        Some(Tag::String(value)) if value == "commands.message.display.outgoing"
    ));
    assert_string_list(field_value(chat, "parameters"), &["target", "content"]);

    let narration = compound_field(&tag, "narration");
    assert!(matches!(
        field_value(narration, "translation_key"),
        Some(Tag::String(value)) if value == "chat.type.text.narrate"
    ));
    assert_string_list(field_value(narration, "parameters"), &["sender", "content"]);
}

#[test]
pub fn biome_registry_payloads_include_full_vanilla_id_set_with_plains() {
    assert_eq!(BIOMES.len(), 65);
    assert!(BIOMES.contains(&"plains"));
    assert!(BIOMES.contains(&"the_void"));
    assert!(BIOMES.contains(&"end_barrens"));
    assert_eq!(BIOMES[0], "badlands");
    assert_eq!(BIOMES[40], "plains");
    assert_eq!(BIOMES[64], "wooded_badlands");
    assert_eq!(
        registry_element_count(write_minimal_biome_registry_packet),
        BIOMES.len() as i32
    );

    let plains = vanilla_baseline_biome_nbt("plains");
    assert!(matches!(
        field_value(&plains, "has_precipitation"),
        Some(Tag::Byte(1))
    ));
    assert!(matches!(
        field_value(&plains, "temperature"),
        Some(Tag::Float(value)) if (*value - 0.8).abs() < f32::EPSILON
    ));
    let effects = compound_field(&plains, "effects");
    assert!(matches!(
        field_value(effects, "water_color"),
        Some(Tag::Int(4_159_204))
    ));
}

#[test]
pub fn biome_network_codec_fixture_covers_required_fields_for_every_emitted_biome() {
    for biome in BIOMES {
        let tag = vanilla_baseline_biome_nbt(biome);
        assert!(matches!(
            field_value(&tag, "has_precipitation"),
            Some(Tag::Byte(0 | 1))
        ));
        assert!(matches!(
            field_value(&tag, "temperature"),
            Some(Tag::Float(_))
        ));
        assert!(matches!(field_value(&tag, "downfall"), Some(Tag::Float(_))));
        assert!(matches!(
            field_value(&tag, "effects"),
            Some(Tag::Compound(_))
        ));

        let effects = compound_field(&tag, "effects");
        assert!(matches!(
            field_value(effects, "water_color"),
            Some(Tag::Int(_))
        ));
    }
}

#[test]
pub fn visible_spawn_terrain_uses_deterministic_rolling_grass_layers() {
    let words = visible_spawn_palette_words();
    let columns = visible_spawn_test_columns();

    assert!(
        columns.max_height > 95,
        "spawn terrain should be visibly non-flat"
    );
    assert!(visible_spawn_terrain_block_count(0, 0, 8) > 4000);
    assert!(visible_spawn_terrain_block_count(0, 0, 9) > 512);
    assert!(visible_spawn_terrain_block_count(0, 0, 10) > 0);
    assert_ne!(words.iter().filter(|word| **word != 0).count(), 0);
    assert_high_visible_spawn_column(&words, columns.high_column);
    assert_outcrop_visible_spawn_column(&words, columns.outcrop_column);
    assert_featured_visible_spawn_column(&words, columns.featured_column);
    assert!(
        visible_spawn_terrain_height(0, 0, columns.ridge_column.0, columns.ridge_column.1) >= 96
    );
}

fn visible_spawn_palette_words() -> Vec<u64> {
    let mut payload = Vec::new();
    write_visible_spawn_terrain_block_state_container(&mut payload, 0, 0, 9).unwrap();
    let mut input = Cursor::new(payload);

    let mut bits = [0_u8; 1];
    input.read_exact(&mut bits).unwrap();
    assert_eq!(bits[0], 4);
    assert_eq!(read_var_i32(&mut input).unwrap(), 11);
    assert_eq!(read_var_i32(&mut input).unwrap(), 0);
    assert_eq!(read_var_i32(&mut input).unwrap(), STONE_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), GRANITE_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), DIORITE_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), ANDESITE_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), BEDROCK_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), DIRT_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), GRASS_BLOCK_STATE_ID);
    assert_eq!(
        read_var_i32(&mut input).unwrap(),
        SHORT_GRASS_BLOCK_STATE_ID
    );
    assert_eq!(read_var_i32(&mut input).unwrap(), DANDELION_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), POPPY_BLOCK_STATE_ID);

    let mut raw = Vec::new();
    input.read_to_end(&mut raw).unwrap();
    assert_eq!(raw.len(), 2048);
    let words = raw
        .chunks_exact(8)
        .map(|chunk| {
            u64::from_be_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
            ])
        })
        .collect::<Vec<_>>();
    words
}

struct VisibleSpawnTestColumns {
    high_column: (usize, usize),
    featured_column: (usize, usize, i32),
    outcrop_column: (usize, usize),
    ridge_column: (usize, usize),
    max_height: i32,
}

fn visible_spawn_test_columns() -> VisibleSpawnTestColumns {
    let high_column = (0..16)
        .flat_map(|z| (0..16).map(move |x| (x, z)))
        .find(|(x, z)| {
            (90..96).contains(&visible_spawn_terrain_height(0, 0, *x, *z))
                && visible_spawn_surface_top_block_id(0, 0, *x, *z) == GRASS_BLOCK_STATE_ID
        })
        .expect("spawn chunk should contain a high visible hill top");
    let featured_column = (0..16)
        .flat_map(|z| (0..16).map(move |x| (x, z)))
        .find_map(|(x, z)| {
            (visible_spawn_surface_top_block_id(0, 0, x, z) == GRASS_BLOCK_STATE_ID
                && (80..95).contains(&visible_spawn_terrain_height(0, 0, x, z)))
            .then(|| visible_spawn_surface_feature_id(0, 0, x, z).map(|id| (x, z, id)))
            .flatten()
        })
        .expect("spawn chunk should contain visible surface vegetation");
    let outcrop_column = (0..16)
        .flat_map(|z| (0..16).map(move |x| (x, z)))
        .find(|(x, z)| {
            visible_spawn_surface_top_block_id(0, 0, *x, *z) != GRASS_BLOCK_STATE_ID
                && (80..96).contains(&visible_spawn_terrain_height(0, 0, *x, *z))
        })
        .expect("spawn chunk should contain a visible non-grass outcrop");
    let max_height = (0..16)
        .flat_map(|z| (0..16).map(move |x| visible_spawn_terrain_height(0, 0, x, z)))
        .max()
        .expect("spawn chunk should contain terrain columns");
    let ridge_column = (0..16)
        .flat_map(|z| (0..16).map(move |x| (x, z)))
        .find(|(x, z)| visible_spawn_terrain_height(0, 0, *x, *z) == max_height)
        .expect("spawn chunk should contain a visible ridge");
    VisibleSpawnTestColumns {
        high_column,
        featured_column,
        outcrop_column,
        ridge_column,
        max_height,
    }
}

fn assert_high_visible_spawn_column(words: &[u64], high_column: (usize, usize)) {
    let high_local_y =
        (visible_spawn_terrain_height(0, 0, high_column.0, high_column.1) - 80) as usize;
    assert_eq!(
        palette_index_at(words, high_column.0, high_local_y - 1, high_column.1),
        6
    );
    assert_eq!(
        palette_index_at(words, high_column.0, high_local_y, high_column.1),
        7
    );
}

fn assert_outcrop_visible_spawn_column(words: &[u64], outcrop_column: (usize, usize)) {
    let expected_outcrop_palette =
        match visible_spawn_surface_top_block_id(0, 0, outcrop_column.0, outcrop_column.1) {
            STONE_BLOCK_STATE_ID => 1,
            GRANITE_BLOCK_STATE_ID => 2,
            DIORITE_BLOCK_STATE_ID => 3,
            ANDESITE_BLOCK_STATE_ID => 4,
            DIRT_BLOCK_STATE_ID => 6,
            _ => unreachable!("outcrop column must be non-grass"),
        };
    assert_eq!(
        palette_index_at(
            words,
            outcrop_column.0,
            (visible_spawn_terrain_height(0, 0, outcrop_column.0, outcrop_column.1) - 80) as usize,
            outcrop_column.1
        ),
        expected_outcrop_palette
    );
}

fn assert_featured_visible_spawn_column(words: &[u64], featured_column: (usize, usize, i32)) {
    let feature_y = (visible_spawn_terrain_height(0, 0, featured_column.0, featured_column.1) + 1
        - 80) as usize;
    let expected_feature_palette = match featured_column.2 {
        SHORT_GRASS_BLOCK_STATE_ID => 8,
        DANDELION_BLOCK_STATE_ID => 9,
        POPPY_BLOCK_STATE_ID => 10,
        _ => unreachable!("feature id must be in the emitted palette"),
    };
    assert_eq!(
        palette_index_at(words, featured_column.0, feature_y, featured_column.1),
        expected_feature_palette
    );
}

#[test]
pub fn all_air_persisted_chunks_are_not_reused_for_spawn_terrain() {
    let empty = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    assert!(!chunk_has_non_air_blocks(&empty));

    let generated =
        crate::worldgen::generate_overworld_chunk_for_preset(ChunkPos { x: 0, z: 0 }, "normal")
            .expect("normal preset should generate visible terrain");
    assert!(chunk_has_non_air_blocks(&generated));
}

#[test]
pub fn wolf_sound_variant_uses_per_variant_adult_and_generic_baby_sounds() {
    // angry: adult ambient/death/… use the wolf_angry prefix, but adult step is the
    // generic entity.wolf.step; baby is the shared entity.baby_wolf.* set
    // (SoundEvents.registerWolfSoundVariants).
    let angry = wolf_sound_variant_nbt("angry");
    let adult = compound_field(&angry, "adult_sounds");
    assert_eq!(
        field_value(adult, "ambient_sound"),
        Some(&Tag::String("minecraft:entity.wolf_angry.ambient".to_string()))
    );
    assert_eq!(
        field_value(adult, "whine_sound"),
        Some(&Tag::String("minecraft:entity.wolf_angry.whine".to_string()))
    );
    assert_eq!(
        field_value(adult, "step_sound"),
        Some(&Tag::String("minecraft:entity.wolf.step".to_string()))
    );
    let baby = compound_field(&angry, "baby_sounds");
    assert_eq!(
        field_value(baby, "ambient_sound"),
        Some(&Tag::String("minecraft:entity.baby_wolf.ambient".to_string()))
    );
    assert_eq!(
        field_value(baby, "step_sound"),
        Some(&Tag::String("minecraft:entity.baby_wolf.step".to_string()))
    );

    // classic uses the bare `wolf` prefix for adult sounds.
    let classic = wolf_sound_variant_nbt("classic");
    assert_eq!(
        field_value(compound_field(&classic, "adult_sounds"), "ambient_sound"),
        Some(&Tag::String("minecraft:entity.wolf.ambient".to_string()))
    );
}

#[test]
pub fn animal_sound_variants_use_per_variant_prefixes_and_generic_baby_sounds() {
    // cow is flat: moody → every sound uses the cow_moody prefix.
    let moody = cow_sound_variant_nbt("moody");
    assert_eq!(
        field_value(&moody, "ambient_sound"),
        Some(&Tag::String("minecraft:entity.cow_moody.ambient".to_string()))
    );
    assert_eq!(
        field_value(&moody, "step_sound"),
        Some(&Tag::String("minecraft:entity.cow_moody.step".to_string()))
    );

    // pig mini: adult eat is prefixed, step is the generic entity.pig.step, baby is
    // the shared entity.baby_pig.* set.
    let mini = pig_sound_variant_nbt("mini");
    let adult = compound_field(&mini, "adult_sounds");
    assert_eq!(
        field_value(adult, "eat_sound"),
        Some(&Tag::String("minecraft:entity.pig_mini.eat".to_string()))
    );
    assert_eq!(
        field_value(adult, "step_sound"),
        Some(&Tag::String("minecraft:entity.pig.step".to_string()))
    );
    assert_eq!(
        field_value(compound_field(&mini, "baby_sounds"), "eat_sound"),
        Some(&Tag::String("minecraft:entity.baby_pig.eat".to_string()))
    );

    // cat royal: adult uses cat_royal prefix, baby uses the generic baby_cat set.
    let royal = cat_sound_variant_nbt("royal");
    assert_eq!(
        field_value(compound_field(&royal, "adult_sounds"), "purreow_sound"),
        Some(&Tag::String("minecraft:entity.cat_royal.purreow".to_string()))
    );
    assert_eq!(
        field_value(compound_field(&royal, "baby_sounds"), "purreow_sound"),
        Some(&Tag::String("minecraft:entity.baby_cat.purreow".to_string()))
    );

    // chicken picky: adult ambient prefixed, step generic, baby generic.
    let picky = chicken_sound_variant_nbt("picky");
    assert_eq!(
        field_value(compound_field(&picky, "adult_sounds"), "ambient_sound"),
        Some(&Tag::String("minecraft:entity.chicken_picky.ambient".to_string()))
    );
    assert_eq!(
        field_value(compound_field(&picky, "adult_sounds"), "step_sound"),
        Some(&Tag::String("minecraft:entity.chicken.step".to_string()))
    );
    assert_eq!(
        field_value(compound_field(&picky, "baby_sounds"), "ambient_sound"),
        Some(&Tag::String("minecraft:entity.baby_chicken.ambient".to_string()))
    );
}


/// Parse a registry-data packet into a map of entry id → NBT payload Tag.
fn registry_entries_with_nbt(
    write_packet: fn(&mut Vec<u8>) -> std::io::Result<()>,
) -> std::collections::BTreeMap<String, Tag> {
    let mut payload = Vec::new();
    write_packet(&mut payload).unwrap();
    let mut cursor = Cursor::new(payload);
    let _registry = crate::network::codec::read_identifier(&mut cursor).unwrap();
    let count = read_var_i32(&mut cursor).unwrap();
    let mut out = std::collections::BTreeMap::new();
    for _ in 0..count {
        let id = crate::network::codec::read_identifier(&mut cursor)
            .unwrap()
            .to_string();
        let mut present = [0u8; 1];
        cursor.read_exact(&mut present).unwrap();
        let mut tag_id = [0u8; 1];
        cursor.read_exact(&mut tag_id).unwrap();
        let tag = Tag::read_payload(tag_id[0], &mut cursor).unwrap();
        out.insert(id, tag);
    }
    out
}

#[test]
pub fn damage_type_registry_carries_full_per_type_vanilla_data() {
    let entries = registry_entries_with_nbt(write_minimal_damage_type_registry_packet);

    // in_fire: effects=burning, exhaustion=0.1, message_id=inFire, scaling present.
    let in_fire = &entries["minecraft:in_fire"];
    assert_eq!(
        field_value(in_fire, "message_id"),
        Some(&Tag::String("inFire".to_string()))
    );
    assert_eq!(
        field_value(in_fire, "scaling"),
        Some(&Tag::String("when_caused_by_living_non_player".to_string()))
    );
    assert_eq!(field_value(in_fire, "exhaustion"), Some(&Tag::Float(0.1)));
    assert_eq!(
        field_value(in_fire, "effects"),
        Some(&Tag::String("burning".to_string()))
    );

    // arrow: HURT effects → the optional field is omitted.
    let arrow = &entries["minecraft:arrow"];
    assert_eq!(
        field_value(arrow, "message_id"),
        Some(&Tag::String("arrow".to_string()))
    );
    assert!(field_value(arrow, "effects").is_none());
    assert!(field_value(arrow, "death_message_type").is_none());

    // fall: non-default death_message_type is written.
    let fall = &entries["minecraft:fall"];
    assert_eq!(
        field_value(fall, "death_message_type"),
        Some(&Tag::String("fall_variants".to_string()))
    );
    assert!(field_value(fall, "effects").is_none());
}

#[test]
pub fn painting_variant_registry_carries_dimensions_and_title_author() {
    let entries = registry_entries_with_nbt(write_vanilla_painting_variant_registry_packet);

    // kebab: 1x1, both title (yellow) and author (gray) translate components.
    let kebab = &entries["minecraft:kebab"];
    assert_eq!(field_value(kebab, "width"), Some(&Tag::Int(1)));
    assert_eq!(field_value(kebab, "height"), Some(&Tag::Int(1)));
    let title = compound_field(kebab, "title");
    assert_eq!(
        field_value(title, "translate"),
        Some(&Tag::String("painting.minecraft.kebab.title".to_string()))
    );
    assert_eq!(
        field_value(title, "color"),
        Some(&Tag::String("yellow".to_string()))
    );
    let author = compound_field(kebab, "author");
    assert_eq!(
        field_value(author, "color"),
        Some(&Tag::String("gray".to_string()))
    );

    // pigscene: 4x4.
    let pigscene = &entries["minecraft:pigscene"];
    assert_eq!(field_value(pigscene, "width"), Some(&Tag::Int(4)));
    assert_eq!(field_value(pigscene, "height"), Some(&Tag::Int(4)));

    // earth: 2x2, has a title but NO author (matching the vanilla JSON).
    let earth = &entries["minecraft:earth"];
    assert_eq!(field_value(earth, "width"), Some(&Tag::Int(2)));
    assert!(field_value(earth, "title").is_some());
    assert!(field_value(earth, "author").is_none());
}
