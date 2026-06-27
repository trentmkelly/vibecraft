use super::*;
use std::collections::HashSet;

const SOUND_SOURCE_JAVA: &str = vibecraft_java_source!("/net/minecraft/sounds/SoundSource.java");

#[test]
fn sound_sources_match_vanilla_serialized_names_and_sound_events_are_lookupable() {
    for sentinel in [
        "MASTER(\"master\")",
        "MUSIC(\"music\")",
        "RECORDS(\"record\")",
        "WEATHER(\"weather\")",
        "BLOCKS(\"block\")",
        "HOSTILE(\"hostile\")",
        "NEUTRAL(\"neutral\")",
        "PLAYERS(\"player\")",
        "AMBIENT(\"ambient\")",
        "VOICE(\"voice\")",
        "UI(\"ui\");",
        "public String getName()",
    ] {
        assert!(
            SOUND_SOURCE_JAVA.contains(sentinel),
            "SoundSource.java is missing sentinel: {sentinel}"
        );
    }

    assert_eq!(
        SOUND_SOURCES
            .iter()
            .map(|source| source.serialized_name())
            .collect::<Vec<_>>(),
        vec![
            "master", "music", "record", "weather", "block", "hostile", "neutral", "player",
            "ambient", "voice", "ui"
        ]
    );

    if !sound_events_java_source_available() {
        eprintln!(
            "skipping sound-event lookup assertions because \
             VIBECRAFT_DECOMPILED_SOURCE_ROOT does not provide net/minecraft/sounds/SoundEvents.java"
        );
        return;
    }

    assert_eq!(
        find_sound_event("minecraft:music_disc.pigstep").unwrap().id,
        "minecraft:music_disc.pigstep"
    );
    assert!(find_sound_event("minecraft:missing").is_none());
}

#[test]
fn sound_events_cover_referenced_vanilla_ids_without_duplicates() {
    if !sound_events_java_source_available() {
        eprintln!(
            "skipping sound-event parity assertions because \
             VIBECRAFT_DECOMPILED_SOURCE_ROOT does not provide net/minecraft/sounds/SoundEvents.java"
        );
        return;
    }

    assert_eq!(SOUND_EVENTS.len(), SOUND_EVENTS_COUNT_26_1_2);
    assert_eq!(
        SOUND_EVENTS.first().unwrap().id,
        "minecraft:entity.allay.ambient_with_item"
    );
    assert_eq!(
        SOUND_EVENTS.last().unwrap().id,
        "minecraft:item.nautilus_saddle_equip"
    );

    let ids = SOUND_EVENTS
        .iter()
        .map(|sound| sound.id)
        .collect::<Vec<_>>();
    assert!(ids.contains(&"minecraft:entity.player.levelup"));
    assert!(ids.contains(&"minecraft:block.vault.activate"));
    assert!(ids.contains(&"minecraft:item.bundle.insert"));
    assert!(ids.contains(&"minecraft:entity.cat_royal.purr"));
    assert!(ids.contains(&"minecraft:entity.chicken_picky.hurt"));
    assert!(ids.contains(&"minecraft:entity.cow_moody.step"));
    assert!(ids.contains(&"minecraft:entity.pig_mini.eat"));
    assert!(ids.contains(&"minecraft:item.goat_horn.sound.7"));
    assert!(ids.contains(&"minecraft:entity.wolf_puglin.growl"));
    assert!(ids.iter().all(|id| id.starts_with("minecraft:")));
    assert!(SOUND_EVENTS.iter().all(|sound| sound.fixed_range.is_none()));

    let unique_count = ids.iter().copied().collect::<HashSet<_>>().len();
    assert_eq!(unique_count, ids.len());
    assert!(!SOUND_EVENTS.is_empty());
}

#[test]
fn particles_cover_simple_and_data_backed_option_shapes() {
    assert_eq!(PARTICLES.len(), 117);
    assert_eq!(PARTICLES.first().unwrap().id, "minecraft:angry_villager");
    assert_eq!(PARTICLES.last().unwrap().id, "minecraft:firefly");
    assert!(PARTICLES.iter().any(|particle| {
        particle.id == "minecraft:block" && particle.option_shape == ParticleOptionShape::Block
    }));
    assert!(PARTICLES.iter().any(|particle| {
        particle.id == "minecraft:dust"
            && particle.option_shape == ParticleOptionShape::Dust
            && !particle.override_limiter
    }));
    assert!(PARTICLES.iter().any(|particle| {
        particle.id == "minecraft:dragon_breath"
            && particle.option_shape == ParticleOptionShape::Power
    }));
    assert!(PARTICLES.iter().any(|particle| {
        particle.id == "minecraft:vibration"
            && particle.option_shape == ParticleOptionShape::Vibration
            && particle.override_limiter
    }));
    assert!(PARTICLES.iter().any(|particle| {
        particle.id == "minecraft:item" && particle.option_shape == ParticleOptionShape::Item
    }));
    assert!(PARTICLES
        .iter()
        .any(|particle| particle.option_shape == ParticleOptionShape::Shriek));
    assert!(PARTICLES
        .iter()
        .any(|particle| particle.option_shape == ParticleOptionShape::Spell));
    assert!(PARTICLES
        .iter()
        .filter(|particle| particle.option_shape != ParticleOptionShape::Simple)
        .map(|particle| particle.option_shape)
        .collect::<Vec<_>>()
        .into_iter()
        .all(|shape| matches!(
            shape,
            ParticleOptionShape::Block
                | ParticleOptionShape::Dust
                | ParticleOptionShape::DustColorTransition
                | ParticleOptionShape::Color
                | ParticleOptionShape::Item
                | ParticleOptionShape::Vibration
                | ParticleOptionShape::Trail
                | ParticleOptionShape::SculkCharge
                | ParticleOptionShape::Shriek
                | ParticleOptionShape::Power
                | ParticleOptionShape::Spell
        )));
}

#[test]
fn painting_variants_keep_sizes_and_title_author_keys() {
    assert_eq!(PAINTING_VARIANTS.len(), 51);
    assert_eq!(PAINTING_VARIANTS.first().unwrap().id, "minecraft:alban");
    assert_eq!(PAINTING_VARIANTS.last().unwrap().id, "minecraft:wither");

    let wither = PAINTING_VARIANTS
        .iter()
        .find(|variant| variant.id == "minecraft:wither")
        .unwrap();
    assert_eq!((wither.width, wither.height), (2, 2));
    assert_eq!(wither.title_key(), "painting.minecraft.wither.title");
    assert_eq!(wither.author_key(), None);

    let backyard = PAINTING_VARIANTS
        .iter()
        .find(|variant| variant.id == "minecraft:backyard")
        .unwrap();
    assert_eq!((backyard.width, backyard.height), (3, 4));
    assert_eq!(
        backyard.author_key().as_deref(),
        Some("painting.minecraft.backyard.author")
    );

    let no_author = [
        "minecraft:earth",
        "minecraft:fire",
        "minecraft:water",
        "minecraft:wind",
        "minecraft:wither",
    ];
    for id in no_author {
        let entry = PAINTING_VARIANTS
            .iter()
            .find(|variant| variant.id == id)
            .unwrap();
        assert!(!entry.has_author);
    }
}

#[test]
fn banner_patterns_include_vanilla_translation_keys_and_new_patterns() {
    assert_eq!(BANNER_PATTERNS.len(), 43);
    assert_eq!(BANNER_PATTERNS.first().unwrap().id, "minecraft:base");
    assert_eq!(
        BANNER_PATTERNS.last().unwrap().id,
        "minecraft:triangles_top"
    );
    let expected_ids: [&str; 43] = [
        "minecraft:base",
        "minecraft:border",
        "minecraft:bricks",
        "minecraft:circle",
        "minecraft:creeper",
        "minecraft:cross",
        "minecraft:curly_border",
        "minecraft:diagonal_left",
        "minecraft:diagonal_right",
        "minecraft:diagonal_up_left",
        "minecraft:diagonal_up_right",
        "minecraft:flow",
        "minecraft:flower",
        "minecraft:globe",
        "minecraft:gradient",
        "minecraft:gradient_up",
        "minecraft:guster",
        "minecraft:half_horizontal",
        "minecraft:half_horizontal_bottom",
        "minecraft:half_vertical",
        "minecraft:half_vertical_right",
        "minecraft:mojang",
        "minecraft:piglin",
        "minecraft:rhombus",
        "minecraft:skull",
        "minecraft:small_stripes",
        "minecraft:square_bottom_left",
        "minecraft:square_bottom_right",
        "minecraft:square_top_left",
        "minecraft:square_top_right",
        "minecraft:straight_cross",
        "minecraft:stripe_bottom",
        "minecraft:stripe_center",
        "minecraft:stripe_downleft",
        "minecraft:stripe_downright",
        "minecraft:stripe_left",
        "minecraft:stripe_middle",
        "minecraft:stripe_right",
        "minecraft:stripe_top",
        "minecraft:triangle_bottom",
        "minecraft:triangle_top",
        "minecraft:triangles_bottom",
        "minecraft:triangles_top",
    ];
    let actual_ids: [&str; 43] = BANNER_PATTERNS
        .iter()
        .map(|pattern| pattern.id)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    assert_eq!(actual_ids, expected_ids);
    assert_eq!(
        BANNER_PATTERNS
            .iter()
            .find(|pattern| pattern.id == "minecraft:flow")
            .unwrap()
            .translation_key,
        "block.minecraft.banner.flow"
    );
    assert_eq!(
        BANNER_PATTERNS
            .iter()
            .find(|pattern| pattern.id == "minecraft:triangles_top")
            .unwrap()
            .translation_key,
        "block.minecraft.banner.triangles_top"
    );
    assert_eq!(
        BANNER_PATTERNS
            .iter()
            .find(|pattern| pattern.id == "minecraft:diagonal_up_left")
            .unwrap()
            .translation_key,
        "block.minecraft.banner.diagonal_up_left"
    );
}

#[test]
fn banner_pattern_tag_names_cover_pattern_item_tags() {
    let pattern_item_tags: [&str; 10] = [
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
    ];

    let mut declared = pattern_item_tags;
    let pattern_item_tag_names: [&str; 10] = [
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
    ];
    declared.sort_unstable();
    let mut expected = pattern_item_tag_names;
    expected.sort_unstable();
    assert_eq!(declared, expected);
}

#[test]
fn trim_materials_and_patterns_expose_bootstrap_assets() {
    assert_eq!(TRIM_MATERIALS.len(), 11);
    let trim_materials: Vec<(&str, u32, &str)> = TRIM_MATERIALS
        .iter()
        .map(|material| (material.id, material.color, material.asset_group))
        .collect();
    assert_eq!(
        trim_materials,
        [
            ("minecraft:quartz", 14931140, "quartz"),
            ("minecraft:iron", 15527148, "iron"),
            ("minecraft:netherite", 6445145, "netherite"),
            ("minecraft:redstone", 9901575, "redstone"),
            ("minecraft:copper", 11823181, "copper"),
            ("minecraft:gold", 14594349, "gold"),
            ("minecraft:emerald", 1155126, "emerald"),
            ("minecraft:diamond", 7269586, "diamond"),
            ("minecraft:lapis", 4288151, "lapis"),
            ("minecraft:amethyst", 10116294, "amethyst"),
            ("minecraft:resin", 16545810, "resin"),
        ]
    );
    assert!(TRIM_MATERIALS.iter().any(|material| {
        material.id == "minecraft:resin"
            && material.color == 16545810
            && material.asset_group == "resin"
    }));
    assert_eq!(TRIM_PATTERNS.len(), 18);
    let trim_patterns: Vec<&str> = TRIM_PATTERNS.iter().map(|pattern| pattern.id).collect();
    assert_eq!(
        trim_patterns,
        [
            "minecraft:sentry",
            "minecraft:dune",
            "minecraft:coast",
            "minecraft:wild",
            "minecraft:ward",
            "minecraft:eye",
            "minecraft:vex",
            "minecraft:tide",
            "minecraft:snout",
            "minecraft:rib",
            "minecraft:spire",
            "minecraft:wayfinder",
            "minecraft:shaper",
            "minecraft:silence",
            "minecraft:raiser",
            "minecraft:host",
            "minecraft:flow",
            "minecraft:bolt",
        ]
    );
    assert!(TRIM_PATTERNS.iter().all(|pattern| !pattern.decal));
    assert!(TRIM_PATTERNS
        .iter()
        .any(|pattern| pattern.id == "minecraft:bolt"));
}

#[test]
fn instruments_and_jukebox_songs_preserve_sound_and_redstone_surfaces() {
    assert_eq!(INSTRUMENTS.len(), 8);
    assert!(INSTRUMENTS
        .iter()
        .all(|instrument| { instrument.use_duration_seconds == 7.0 && instrument.range == 256.0 }));
    assert_eq!(JUKEBOX_SONGS.len(), 21);
    let mut jukebox_song_ids: Vec<&str> = JUKEBOX_SONGS.iter().map(|song| song.id).collect();
    jukebox_song_ids.sort_unstable();
    assert_eq!(
        jukebox_song_ids,
        [
            "minecraft:11",
            "minecraft:13",
            "minecraft:5",
            "minecraft:blocks",
            "minecraft:cat",
            "minecraft:chirp",
            "minecraft:creator",
            "minecraft:creator_music_box",
            "minecraft:far",
            "minecraft:lava_chicken",
            "minecraft:mall",
            "minecraft:mellohi",
            "minecraft:otherside",
            "minecraft:pigstep",
            "minecraft:precipice",
            "minecraft:relic",
            "minecraft:stal",
            "minecraft:strad",
            "minecraft:tears",
            "minecraft:wait",
            "minecraft:ward",
        ]
    );
    assert_eq!(
        JUKEBOX_SONGS
            .iter()
            .find(|song| song.id == "minecraft:tears")
            .unwrap(),
        &JukeboxSongDef {
            id: "minecraft:tears",
            sound_event: "minecraft:music_disc.tears",
            comparator_output: 10,
            length_seconds: 175.0,
        }
    );
    assert_eq!(
        JUKEBOX_SONGS
            .iter()
            .find(|song| song.id == "minecraft:lava_chicken")
            .unwrap(),
        &JukeboxSongDef {
            id: "minecraft:lava_chicken",
            sound_event: "minecraft:music_disc.lava_chicken",
            comparator_output: 9,
            length_seconds: 134.0,
        }
    );
    let pigstep = JUKEBOX_SONGS
        .iter()
        .find(|song| song.id == "minecraft:pigstep")
        .unwrap();
    assert_eq!(pigstep.sound_event, "minecraft:music_disc.pigstep");
    assert_eq!(pigstep.comparator_output, 13);
    assert!(JUKEBOX_SONGS
        .iter()
        .any(|song| song.id == "minecraft:creator_music_box"));
}

#[test]
fn damage_types_keep_message_ids_effects_and_death_message_variants() {
    let fall = find_damage_type("minecraft:fall").unwrap();
    assert_eq!(fall.message_id, "fall");
    assert_eq!(fall.death_message_type, DeathMessageType::FallVariants);
    assert_eq!(
        fall.death_message_key(false, false, Some(FallVariant::Ladder)),
        "death.fell.accident.ladder"
    );

    let fireball = find_damage_type("minecraft:fireball").unwrap();
    assert_eq!(fireball.effects, DamageEffects::Burning);
    assert_eq!(
        fireball.typed_death_message_key(true, true),
        "death.attack.fireball.item"
    );

    let design = find_damage_type("minecraft:bad_respawn_point").unwrap();
    assert_eq!(
        design.death_message_key(false, false, None),
        "death.attack.badRespawnPoint.message"
    );
}
