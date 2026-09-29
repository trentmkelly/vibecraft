//! Contents of the registry and tag payloads generated from the vanilla data.
//!
//! Expectations come from the Java data files and codecs; entry orders that were
//! captured from the official server are in `order_fixtures`.

use super::{builtin, compound, entry_map, field, full_entries, registries, string};
use crate::registry::Identifier;
use crate::registry_pipeline::registry_data::unloaded_worldgen_registries;
use crate::registry_pipeline::sync::serialize_tags_to_network;
use crate::resource_registry_data_loader::SYNCHRONIZED_REGISTRIES;
use crate::storage::nbt::Tag;

/// Registry id and element count of every synchronised registry that the vanilla
/// data pack defines (`RegistryDataLoader.SYNCHRONIZED_REGISTRIES`).
const SYNCHRONIZED_COUNTS: &[(&str, usize)] = &[
    ("minecraft:worldgen/biome", 65),
    ("minecraft:chat_type", 7),
    ("minecraft:trim_pattern", 18),
    ("minecraft:trim_material", 11),
    ("minecraft:wolf_variant", 9),
    ("minecraft:wolf_sound_variant", 7),
    ("minecraft:pig_variant", 3),
    ("minecraft:pig_sound_variant", 3),
    ("minecraft:frog_variant", 3),
    ("minecraft:cat_variant", 11),
    ("minecraft:cat_sound_variant", 2),
    ("minecraft:cow_sound_variant", 2),
    ("minecraft:cow_variant", 3),
    ("minecraft:chicken_sound_variant", 2),
    ("minecraft:chicken_variant", 3),
    ("minecraft:zombie_nautilus_variant", 2),
    ("minecraft:painting_variant", 51),
    ("minecraft:dimension_type", 4),
    ("minecraft:damage_type", 50),
    ("minecraft:banner_pattern", 43),
    ("minecraft:enchantment", 43),
    ("minecraft:jukebox_song", 21),
    ("minecraft:instrument", 8),
    ("minecraft:test_environment", 1),
    ("minecraft:test_instance", 1),
    ("minecraft:dialog", 3),
    ("minecraft:world_clock", 2),
    ("minecraft:timeline", 4),
];

#[test]
fn every_synchronized_registry_is_sent_in_java_order_with_all_vanilla_elements() {
    let packets = crate::registry_pipeline::sync::pack_registries(registries(), builtin(), &[])
        .expect("pack registries");
    let sent: Vec<(String, usize)> = packets
        .iter()
        .map(|packet| (packet.registry.to_string(), packet.entries.len()))
        .collect();
    let expected: Vec<(String, usize)> = SYNCHRONIZED_COUNTS
        .iter()
        .map(|(id, count)| (id.to_string(), *count))
        .collect();
    assert_eq!(sent, expected);
    // The table this is checked against is the Java SYNCHRONIZED_REGISTRIES list.
    assert_eq!(
        SYNCHRONIZED_REGISTRIES
            .iter()
            .map(|data| data.key)
            .collect::<Vec<_>>(),
        SYNCHRONIZED_COUNTS
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>()
    );
}

#[test]
fn unloaded_worldgen_registries_are_exactly_the_ones_without_ported_codecs() {
    // None of these is in SYNCHRONIZED_REGISTRIES, so clients never receive them.
    let unloaded = unloaded_worldgen_registries();
    for key in &unloaded {
        assert!(
            !SYNCHRONIZED_REGISTRIES.iter().any(|data| data.key == *key),
            "{key} is synchronised but has no codec"
        );
    }
    assert!(unloaded.contains(&"minecraft:worldgen/structure_set"));
    assert!(!unloaded.contains(&"minecraft:worldgen/configured_feature"));
    assert!(!unloaded.contains(&"minecraft:worldgen/placed_feature"));
    assert!(!unloaded.contains(&"minecraft:worldgen/biome"));
}

#[test]
fn chat_type_registry_payloads_include_vanilla_routes() {
    let entries = entry_map("minecraft:chat_type");
    // Incoming direct messages carry a gray italic style; narration has none.
    let incoming = &entries["minecraft:msg_command_incoming"];
    let incoming_style = compound(compound(incoming, "chat"), "style");
    assert_eq!(field(incoming_style, "color"), Some(&string("gray")));
    assert_eq!(field(incoming_style, "italic"), Some(&Tag::Byte(1)));
    assert!(field(compound(incoming, "narration"), "style").is_none());
    let outgoing = &entries["minecraft:msg_command_outgoing"];

    let chat = compound(outgoing, "chat");
    assert_eq!(
        field(chat, "translation_key"),
        Some(&string("commands.message.display.outgoing"))
    );
    assert_eq!(
        field(chat, "parameters"),
        Some(&Tag::List(vec![string("target"), string("content")]))
    );
    let style = compound(chat, "style");
    assert_eq!(field(style, "color"), Some(&string("gray")));
    assert_eq!(field(style, "italic"), Some(&Tag::Byte(1)));

    let narration = compound(outgoing, "narration");
    assert_eq!(
        field(narration, "translation_key"),
        Some(&string("chat.type.text.narrate"))
    );
    assert_eq!(
        field(narration, "parameters"),
        Some(&Tag::List(vec![string("sender"), string("content")]))
    );
    assert!(field(narration, "style").is_none());
}

#[test]
fn biome_registry_includes_full_vanilla_id_set_and_network_fields() {
    let entries = full_entries("minecraft:worldgen/biome");
    assert_eq!(entries.len(), 65);
    assert_eq!(entries[0].0, "minecraft:badlands");
    assert_eq!(entries[40].0, "minecraft:plains");
    assert_eq!(entries[64].0, "minecraft:wooded_badlands");

    let plains = &entries[40].1;
    assert_eq!(field(plains, "has_precipitation"), Some(&Tag::Byte(1)));
    assert_eq!(field(plains, "temperature"), Some(&Tag::Float(0.8)));
    assert_eq!(field(plains, "downfall"), Some(&Tag::Float(0.4)));
    // STRING_RGB_COLOR encodes as a lowercase hex string.
    assert_eq!(
        field(compound(plains, "effects"), "water_color"),
        Some(&string("#3f76e4"))
    );

    // Biome.NETWORK_CODEC carries neither generation nor spawn settings.
    for (id, biome) in &entries {
        for stripped in ["carvers", "features", "spawners", "spawn_costs"] {
            assert!(field(biome, stripped).is_none(), "{id} leaks {stripped}");
        }
        assert!(
            matches!(field(biome, "has_precipitation"), Some(Tag::Byte(0 | 1))),
            "{id}"
        );
        assert!(
            matches!(field(biome, "temperature"), Some(Tag::Float(_))),
            "{id}"
        );
        assert!(
            matches!(field(biome, "downfall"), Some(Tag::Float(_))),
            "{id}"
        );
        assert!(
            matches!(
                field(compound(biome, "effects"), "water_color"),
                Some(Tag::String(_))
            ),
            "{id}"
        );
    }
}

#[test]
fn biome_network_attributes_keep_only_syncable_attributes() {
    let swamp = &entry_map("minecraft:worldgen/biome")["minecraft:swamp"];
    let attributes = compound(swamp, "attributes");
    // `visual/sky_color` and the fog attributes are syncable; the swamp's
    // `gameplay/increased_fire_burnout` is not, so it must not be sent.
    assert_eq!(
        field(attributes, "minecraft:visual/sky_color"),
        Some(&string("#78a7ff"))
    );
    assert!(field(attributes, "minecraft:gameplay/increased_fire_burnout").is_none());
    // A non-override modifier keeps its `{modifier, argument}` form.
    let fog_end = compound(attributes, "minecraft:visual/water_fog_end_distance");
    assert_eq!(field(fog_end, "modifier"), Some(&string("multiply")));
    assert_eq!(field(fog_end, "argument"), Some(&Tag::Float(0.85)));
}

#[test]
fn wolf_sound_variant_uses_per_variant_adult_and_generic_baby_sounds() {
    // angry: adult ambient/death/... use the wolf_angry prefix, but adult step is the
    // generic entity.wolf.step; baby is the shared entity.baby_wolf.* set.
    let entries = entry_map("minecraft:wolf_sound_variant");
    let angry = &entries["minecraft:angry"];
    let adult = compound(angry, "adult_sounds");
    assert_eq!(
        field(adult, "ambient_sound"),
        Some(&string("minecraft:entity.wolf_angry.ambient"))
    );
    assert_eq!(
        field(adult, "whine_sound"),
        Some(&string("minecraft:entity.wolf_angry.whine"))
    );
    assert_eq!(
        field(adult, "step_sound"),
        Some(&string("minecraft:entity.wolf.step"))
    );
    let baby = compound(angry, "baby_sounds");
    assert_eq!(
        field(baby, "ambient_sound"),
        Some(&string("minecraft:entity.baby_wolf.ambient"))
    );
    assert_eq!(
        field(baby, "step_sound"),
        Some(&string("minecraft:entity.baby_wolf.step"))
    );
    assert_eq!(
        field(
            compound(&entries["minecraft:classic"], "adult_sounds"),
            "ambient_sound"
        ),
        Some(&string("minecraft:entity.wolf.ambient"))
    );
}

#[test]
fn animal_sound_variants_use_per_variant_prefixes_and_generic_baby_sounds() {
    let cow = entry_map("minecraft:cow_sound_variant");
    let moody = &cow["minecraft:moody"];
    assert_eq!(
        field(moody, "ambient_sound"),
        Some(&string("minecraft:entity.cow_moody.ambient"))
    );
    assert_eq!(
        field(moody, "step_sound"),
        Some(&string("minecraft:entity.cow_moody.step"))
    );

    let pig = entry_map("minecraft:pig_sound_variant");
    let mini = &pig["minecraft:mini"];
    assert_eq!(
        field(compound(mini, "adult_sounds"), "eat_sound"),
        Some(&string("minecraft:entity.pig_mini.eat"))
    );
    assert_eq!(
        field(compound(mini, "adult_sounds"), "step_sound"),
        Some(&string("minecraft:entity.pig.step"))
    );
    assert_eq!(
        field(compound(mini, "baby_sounds"), "eat_sound"),
        Some(&string("minecraft:entity.baby_pig.eat"))
    );

    let cat = entry_map("minecraft:cat_sound_variant");
    let royal = &cat["minecraft:royal"];
    assert_eq!(
        field(compound(royal, "adult_sounds"), "purreow_sound"),
        Some(&string("minecraft:entity.cat_royal.purreow"))
    );
    assert_eq!(
        field(compound(royal, "baby_sounds"), "purreow_sound"),
        Some(&string("minecraft:entity.baby_cat.purreow"))
    );

    let chicken = entry_map("minecraft:chicken_sound_variant");
    let picky = &chicken["minecraft:picky"];
    assert_eq!(
        field(compound(picky, "adult_sounds"), "ambient_sound"),
        Some(&string("minecraft:entity.chicken_picky.ambient"))
    );
    assert_eq!(
        field(compound(picky, "adult_sounds"), "step_sound"),
        Some(&string("minecraft:entity.chicken.step"))
    );
    assert_eq!(
        field(compound(picky, "baby_sounds"), "ambient_sound"),
        Some(&string("minecraft:entity.baby_chicken.ambient"))
    );
}

#[test]
fn model_and_texture_variants_omit_the_default_model_and_drop_spawn_conditions() {
    let pigs = entry_map("minecraft:pig_variant");
    // `ModelAndTexture.codec` is `optionalFieldOf("model", NORMAL)`: written only
    // when it differs from the default.
    assert_eq!(
        field(&pigs["minecraft:cold"], "model"),
        Some(&string("cold"))
    );
    assert!(field(&pigs["minecraft:temperate"], "model").is_none());
    assert_eq!(
        field(&pigs["minecraft:cold"], "asset_id"),
        Some(&string("minecraft:entity/pig/pig_cold"))
    );
    for (id, pig) in &pigs {
        assert!(field(pig, "spawn_conditions").is_none(), "{id}");
    }
    for (id, wolf) in entry_map("minecraft:wolf_variant") {
        assert!(field(&wolf, "spawn_conditions").is_none(), "{id}");
        assert!(field(&wolf, "baby_assets").is_some(), "{id}");
    }
}

#[test]
fn damage_type_registry_carries_full_per_type_vanilla_data() {
    let entries = entry_map("minecraft:damage_type");

    let in_fire = &entries["minecraft:in_fire"];
    assert_eq!(field(in_fire, "message_id"), Some(&string("inFire")));
    assert_eq!(
        field(in_fire, "scaling"),
        Some(&string("when_caused_by_living_non_player"))
    );
    assert_eq!(field(in_fire, "exhaustion"), Some(&Tag::Float(0.1)));
    assert_eq!(field(in_fire, "effects"), Some(&string("burning")));

    // arrow: HURT effects and DEFAULT death message are the defaults, so omitted.
    let arrow = &entries["minecraft:arrow"];
    assert_eq!(field(arrow, "message_id"), Some(&string("arrow")));
    assert!(field(arrow, "effects").is_none());
    assert!(field(arrow, "death_message_type").is_none());

    let fall = &entries["minecraft:fall"];
    assert_eq!(
        field(fall, "death_message_type"),
        Some(&string("fall_variants"))
    );
    assert!(field(fall, "effects").is_none());
}

#[test]
fn painting_variant_registry_carries_dimensions_and_title_author() {
    let entries = entry_map("minecraft:painting_variant");

    let kebab = &entries["minecraft:kebab"];
    assert_eq!(field(kebab, "width"), Some(&Tag::Int(1)));
    assert_eq!(field(kebab, "height"), Some(&Tag::Int(1)));
    let title = compound(kebab, "title");
    assert_eq!(
        field(title, "translate"),
        Some(&string("painting.minecraft.kebab.title"))
    );
    assert_eq!(field(title, "color"), Some(&string("yellow")));
    assert_eq!(
        field(compound(kebab, "author"), "color"),
        Some(&string("gray"))
    );

    let pigscene = &entries["minecraft:pigscene"];
    assert_eq!(field(pigscene, "width"), Some(&Tag::Int(4)));
    assert_eq!(field(pigscene, "height"), Some(&Tag::Int(4)));

    // earth has a title but no author in the vanilla JSON.
    let earth = &entries["minecraft:earth"];
    assert_eq!(field(earth, "width"), Some(&Tag::Int(2)));
    assert!(field(earth, "title").is_some());
    assert!(field(earth, "author").is_none());
}

#[test]
fn jukebox_song_registry_payloads_include_disc_13_component_data() {
    let entries = entry_map("minecraft:jukebox_song");
    assert_eq!(entries.len(), 21);
    let thirteen = &entries["minecraft:13"];
    assert_eq!(
        field(thirteen, "sound_event"),
        Some(&string("minecraft:music_disc.13"))
    );
    assert_eq!(
        field(compound(thirteen, "description"), "translate"),
        Some(&string("jukebox_song.minecraft.13"))
    );
    assert_eq!(
        field(thirteen, "length_in_seconds"),
        Some(&Tag::Float(178.0))
    );
    assert_eq!(field(thirteen, "comparator_output"), Some(&Tag::Int(1)));
}

#[test]
fn trim_registries_use_asset_names_and_colored_descriptions() {
    let materials = entry_map("minecraft:trim_material");
    let amethyst = &materials["minecraft:amethyst"];
    assert_eq!(field(amethyst, "asset_name"), Some(&string("amethyst")));
    assert_eq!(
        field(compound(amethyst, "description"), "color"),
        Some(&string("#9A5CC6"))
    );
    // Materials with darker armor variants carry `override_armor_assets`.
    let iron = &materials["minecraft:iron"];
    assert_eq!(
        field(compound(iron, "override_armor_assets"), "minecraft:iron"),
        Some(&string("iron_darker"))
    );
    assert!(field(amethyst, "override_armor_assets").is_none());

    let coast = &entry_map("minecraft:trim_pattern")["minecraft:coast"];
    assert_eq!(field(coast, "asset_id"), Some(&string("minecraft:coast")));
    // `decal` uses `.orElse(false)`, which the encoder always writes.
    assert_eq!(field(coast, "decal"), Some(&Tag::Byte(0)));
}

#[test]
fn dimension_type_overworld_carries_syncable_audio_attributes() {
    let overworld = &entry_map("minecraft:dimension_type")["minecraft:overworld"];
    let attributes = compound(overworld, "attributes");

    let mood = compound(
        compound(attributes, "minecraft:audio/ambient_sounds"),
        "mood",
    );
    assert_eq!(
        field(mood, "sound"),
        Some(&string("minecraft:ambient.cave"))
    );
    assert_eq!(field(mood, "tick_delay"), Some(&Tag::Int(6000)));
    assert_eq!(field(mood, "block_search_extent"), Some(&Tag::Int(8)));
    assert_eq!(field(mood, "offset"), Some(&Tag::Double(2.0)));

    let music = compound(attributes, "minecraft:audio/background_music");
    let default = compound(music, "default");
    assert_eq!(
        field(default, "sound"),
        Some(&string("minecraft:music.game"))
    );
    assert_eq!(field(default, "min_delay"), Some(&Tag::Int(12000)));
    assert_eq!(field(default, "max_delay"), Some(&Tag::Int(24000)));
    assert_eq!(
        field(compound(music, "creative"), "sound"),
        Some(&string("minecraft:music.creative"))
    );
}

#[test]
fn overworld_dimension_type_has_timelines_clock_and_attributes() {
    // These fields are required for the client to render a non-black sky.
    let overworld = &entry_map("minecraft:dimension_type")["minecraft:overworld"];
    assert_eq!(
        field(overworld, "timelines"),
        Some(&string("#minecraft:in_overworld"))
    );
    assert_eq!(
        field(overworld, "default_clock"),
        Some(&string("minecraft:overworld"))
    );
    assert_eq!(
        field(overworld, "coordinate_scale"),
        Some(&Tag::Double(1.0))
    );
    assert_eq!(field(overworld, "min_y"), Some(&Tag::Int(-64)));
    assert_eq!(field(overworld, "height"), Some(&Tag::Int(384)));
    // skybox/cardinal_light/has_fixed_time use `optionalFieldOf(.., default)`.
    assert!(field(overworld, "skybox").is_none());
    assert!(field(overworld, "cardinal_light").is_none());
    assert!(field(overworld, "has_fixed_time").is_none());
    // A constant IntProvider is written as a bare int.
    let nether = &entry_map("minecraft:dimension_type")["minecraft:the_nether"];
    assert_eq!(
        field(nether, "monster_spawn_light_level"),
        Some(&Tag::Int(7))
    );
    assert_eq!(field(nether, "skybox"), Some(&string("none")));
    assert_eq!(field(nether, "cardinal_light"), Some(&string("nether")));
    assert_eq!(field(nether, "has_fixed_time"), Some(&Tag::Byte(1)));
    assert_eq!(
        field(compound(overworld, "monster_spawn_light_level"), "type"),
        Some(&string("minecraft:uniform"))
    );

    let attributes = compound(overworld, "attributes");
    assert_eq!(
        field(attributes, "minecraft:visual/sky_color"),
        Some(&string("#78a7ff"))
    );
    assert_eq!(
        field(attributes, "minecraft:visual/fog_color"),
        Some(&string("#c0d8ff"))
    );
    assert_eq!(
        field(attributes, "minecraft:visual/cloud_color"),
        Some(&string("#ccffffff"))
    );
    assert!(matches!(
        field(attributes, "minecraft:visual/cloud_height"),
        Some(Tag::Float(_))
    ));
    assert_eq!(
        field(attributes, "minecraft:visual/ambient_light_color"),
        Some(&string("#0a0a0a"))
    );
    // gameplay/bed_rule is not syncable.
    assert!(field(attributes, "minecraft:gameplay/bed_rule").is_none());
}

#[test]
fn timeline_registry_is_sorted_by_resource_file() {
    // RegistryDataLoader registers elements in sorted resource-file order, so the
    // ids the tag payload references are day=0, early_game=1, moon=2,
    // villager_schedule=3.
    let ids: Vec<String> = full_entries("minecraft:timeline")
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    assert_eq!(
        ids,
        [
            "minecraft:day",
            "minecraft:early_game",
            "minecraft:moon",
            "minecraft:villager_schedule"
        ]
    );
}

#[test]
fn day_timeline_contains_syncable_tracks_and_omits_non_syncable() {
    let day = &entry_map("minecraft:timeline")["minecraft:day"];
    assert_eq!(field(day, "clock"), Some(&string("minecraft:overworld")));
    assert_eq!(field(day, "period_ticks"), Some(&Tag::Int(24000)));
    let tracks = compound(day, "tracks");

    for syncable in [
        "sun_angle",
        "moon_angle",
        "star_angle",
        "fog_color",
        "sky_color",
        "sky_light_color",
        "sky_light_factor",
        "star_brightness",
        "cloud_color",
        "sunrise_sunset_color",
    ] {
        assert!(
            field(tracks, &format!("minecraft:visual/{syncable}")).is_some(),
            "{syncable}"
        );
    }
    assert!(field(tracks, "minecraft:gameplay/sky_light_level").is_some());
    assert!(field(tracks, "minecraft:audio/firefly_bush_sounds").is_some());
    assert!(field(tracks, "minecraft:gameplay/creaking_active").is_some());
    for filtered in ["monsters_burn", "bees_stay_in_hive", "eyeblossom_open"] {
        assert!(
            field(tracks, &format!("minecraft:gameplay/{filtered}")).is_none(),
            "{filtered} is not syncable"
        );
    }
    // time markers: a bare int, or {ticks, show_in_commands: true}.
    let markers = compound(day, "time_markers");
    assert_eq!(
        field(markers, "minecraft:wake_up_from_sleep"),
        Some(&Tag::Int(0))
    );
    assert_eq!(
        field(compound(markers, "minecraft:noon"), "ticks"),
        Some(&Tag::Int(6000))
    );
}

#[test]
fn day_timeline_tracks_use_cubic_bezier_and_opaque_argb_ints() {
    let day = &entry_map("minecraft:timeline")["minecraft:day"];
    let tracks = compound(day, "tracks");
    let sun_angle = compound(tracks, "minecraft:visual/sun_angle");
    let ease = compound(sun_angle, "ease");
    assert!(matches!(field(ease, "cubic_bezier"), Some(Tag::List(points)) if points.len() == 4));

    // ArgbModifier: an opaque colour is written as a plain int (0xFFFFFFFF == -1).
    let cloud_color = compound(tracks, "minecraft:visual/cloud_color");
    assert_eq!(field(cloud_color, "modifier"), Some(&string("multiply")));
    let Some(Tag::List(keyframes)) = field(cloud_color, "keyframes") else {
        panic!("cloud_color keyframes must be a list");
    };
    assert_eq!(field(&keyframes[0], "value"), Some(&Tag::Int(-1)));
}

#[test]
fn moon_villager_and_early_game_timelines_are_filtered_by_syncability() {
    let timelines = entry_map("minecraft:timeline");

    let moon = &timelines["minecraft:moon"];
    assert_eq!(field(moon, "period_ticks"), Some(&Tag::Int(192000)));
    let tracks = compound(moon, "tracks");
    assert!(field(tracks, "minecraft:gameplay/surface_slime_spawn_chance").is_none());
    let Some(Tag::List(keyframes)) =
        field(compound(tracks, "minecraft:visual/moon_phase"), "keyframes")
    else {
        panic!("moon_phase keyframes must be a list");
    };
    assert_eq!(keyframes.len(), 8);
    assert_eq!(field(&keyframes[0], "value"), Some(&string("full_moon")));
    assert_eq!(field(&keyframes[4], "value"), Some(&string("new_moon")));

    // All villager_schedule tracks are non-syncable, so `tracks` (equal to the
    // empty-map default) is not written.
    let villager = &timelines["minecraft:villager_schedule"];
    assert_eq!(field(villager, "period_ticks"), Some(&Tag::Int(24000)));
    assert!(field(villager, "tracks").is_none());

    let early_game = &timelines["minecraft:early_game"];
    assert!(field(early_game, "period_ticks").is_none());
    assert!(field(early_game, "tracks").is_none());
}

#[test]
fn damage_type_and_banner_pattern_tags_match_vanilla_set() {
    let damage = registries()
        .lookup(&Identifier::parse("minecraft:damage_type").expect("id"))
        .expect("damage_type registry");
    // 33 vanilla damage_type tags; `bypasses_cooldown` is a code-only TagKey with no
    // data file and is not synced.
    assert_eq!(damage.tags().len(), 33);
    let is_fire = Identifier::parse("minecraft:is_fire").expect("id");
    assert!(damage.tags().contains_key(&is_fire));
    assert!(!damage
        .tags()
        .contains_key(&Identifier::parse("minecraft:bypasses_cooldown").expect("id")));

    let banner = registries()
        .lookup(&Identifier::parse("minecraft:banner_pattern").expect("id"))
        .expect("banner_pattern registry");
    let no_item = &banner.tags()[&Identifier::parse("minecraft:no_item_required").expect("id")];
    // `no_item_required` excludes base/bricks/curly_border, which vanilla omits.
    assert_eq!(no_item.len(), 32);
    for excluded in ["base", "bricks", "curly_border"] {
        let id = banner
            .id_of(&Identifier::parse(excluded).expect("id"))
            .expect("banner pattern id");
        assert!(!no_item.contains(&id), "{excluded} should be excluded");
    }
    // The 10 pattern_item/* tags plus no_item_required.
    assert_eq!(banner.tags().len(), 11);
}

#[test]
fn timeline_tags_are_expanded_with_nested_tags_first() {
    // in_overworld.json is [#universal, day, moon, early_game] and universal.json is
    // [villager_schedule]; nested tags are expanded in place.
    let timeline = registries()
        .lookup(&Identifier::parse("minecraft:timeline").expect("id"))
        .expect("timeline registry");
    let id = |name: &str| {
        timeline
            .id_of(&Identifier::parse(name).expect("id"))
            .expect("timeline")
    };
    let in_overworld = &timeline.tags()[&Identifier::parse("minecraft:in_overworld").expect("id")];
    assert_eq!(
        in_overworld,
        &vec![
            id("villager_schedule"),
            id("day"),
            id("moon"),
            id("early_game")
        ]
    );
    let universal = &timeline.tags()[&Identifier::parse("minecraft:universal").expect("id")];
    assert_eq!(universal, &vec![id("villager_schedule")]);
}

#[test]
fn tag_payload_covers_static_and_networked_registries_with_stable_ids() {
    let payload = serialize_tags_to_network(registries());
    let registry_ids: Vec<String> = payload.iter().map(|(id, _)| id.to_string()).collect();
    for expected in [
        "minecraft:banner_pattern",
        "minecraft:damage_type",
        "minecraft:timeline",
        "minecraft:worldgen/biome",
        "minecraft:block",
        "minecraft:item",
        "minecraft:entity_type",
        "minecraft:fluid",
        "minecraft:game_event",
    ] {
        assert!(
            registry_ids.iter().any(|id| id == expected),
            "{expected} tags missing"
        );
    }
    // Registries that are neither synchronised nor static (trades) are not sent.
    assert!(!registry_ids
        .iter()
        .any(|id| id == "minecraft:villager_trade"));

    let block = &payload
        .iter()
        .find(|(id, _)| id.to_string() == "minecraft:block")
        .expect("block tags")
        .1;
    let pickaxe = block
        .iter()
        .find(|(tag, _)| tag.to_string() == "minecraft:mineable/pickaxe")
        .expect("mineable/pickaxe");
    // The client needs #mineable/pickaxe to animate pickaxes at tool speed, and the
    // ids must agree with the block registry ids used elsewhere on the wire.
    let stone = crate::block_states::block_registry_network_id("minecraft:stone").expect("stone");
    assert!(pickaxe.1.contains(&stone));
}

#[test]
fn item_and_entity_tags_resolve_against_static_registry_ids() {
    let item = registries()
        .lookup(&Identifier::parse("minecraft:item").expect("id"))
        .expect("item registry");
    let planks = &item.tags()[&Identifier::parse("minecraft:planks").expect("id")];
    let oak = item
        .id_of(&Identifier::parse("minecraft:oak_planks").expect("id"))
        .expect("oak_planks");
    assert!(planks.contains(&oak));

    let entity = registries()
        .lookup(&Identifier::parse("minecraft:entity_type").expect("id"))
        .expect("entity_type registry");
    assert!(entity
        .tags()
        .contains_key(&Identifier::parse("minecraft:skeletons").expect("id")));
}

#[test]
fn synced_tag_registries_include_required_names_and_indices() {
    let tags = |registry: &str| {
        registries()
            .lookup(&Identifier::parse(registry).expect("id"))
            .unwrap_or_else(|| panic!("{registry} missing"))
    };
    let damage = tags("minecraft:damage_type");
    let tag = |registry: &crate::registry_pipeline::store::MappedRegistry, name: &str| {
        registry.tags()[&Identifier::parse(name).expect("id")].clone()
    };
    // Ids are positions in the alphabetical damage type registry.
    assert_eq!(
        tag(damage, "minecraft:is_fire"),
        vec![21, 3, 31, 24, 20, 46, 14]
    );
    let bypasses_shield = tag(damage, "minecraft:bypasses_shield");
    assert!(bypasses_shield.contains(&11));
    assert!(bypasses_shield.contains(&13));

    let banner = tags("minecraft:banner_pattern");
    let index = |name: &str| {
        banner
            .id_of(&Identifier::parse(name).expect("id"))
            .expect("pattern")
    };
    assert_eq!(
        tag(banner, "minecraft:pattern_item/flower"),
        vec![index("flower")]
    );
    assert_eq!(
        tag(banner, "minecraft:pattern_item/field_masoned"),
        vec![index("bricks")]
    );
    assert_eq!(
        tag(banner, "minecraft:pattern_item/bordure_indented"),
        vec![index("curly_border")]
    );
}
