use super::*;

/// Builds a track compound from modifier, keyframes, and optional ease.
/// Java: AttributeTrack.createCodec — modifier field absent for `override` (the default),
/// present for any other modifier. Ease field absent when LINEAR (the default).
pub fn timeline_track(modifier: Option<&str>, keyframes: Vec<Tag>, ease: Option<Tag>) -> Tag {
    let mut fields = Vec::new();
    if let Some(m) = modifier {
        fields.push(("modifier".to_string(), Tag::String(m.to_string())));
    }
    fields.push(("keyframes".to_string(), Tag::List(keyframes)));
    if let Some(e) = ease {
        fields.push(("ease".to_string(), e));
    }
    Tag::Compound(fields)
}

/// Builds the NBT compound for `minecraft:day` filtered to syncable tracks only.
/// Java: data/minecraft/timeline/day.json; Timeline.NETWORK_CODEC removes non-syncable tracks.
pub fn day_timeline_nbt() -> Tag {
    // symmetricCubicBezier(0.362, 0.241) — shared by sun, moon, and star angle tracks.
    let sym_bezier = cubic_bezier_ease(0.362, 0.241, 0.638, 0.759);

    // Celestial angle tracks: ANGLE_DEGREES type, override modifier → Codec.FLOAT keyframes.
    let sun_angle = timeline_track(
        None,
        vec![
            timeline_keyframe_f32(6000, 360.0),
            timeline_keyframe_f32(6000, 0.0),
        ],
        Some(sym_bezier.clone()),
    );
    let moon_angle = timeline_track(
        None,
        vec![
            timeline_keyframe_f32(6000, 540.0),
            timeline_keyframe_f32(6000, 180.0),
        ],
        Some(sym_bezier.clone()),
    );
    let star_angle = timeline_track(
        None,
        vec![
            timeline_keyframe_f32(6000, 360.0),
            timeline_keyframe_f32(6000, 0.0),
        ],
        Some(sym_bezier),
    );

    // RGB colour tracks: multiply modifier → RgbModifier.argumentCodec = STRING_RGB_COLOR.
    // Primary encoder of STRING_RGB_COLOR is hexColor(6) → Tag::String("#rrggbb").
    let fog_color = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_str(133, "#ffffff"),
            timeline_keyframe_str(11867, "#ffffff"),
            timeline_keyframe_str(13670, "#0f0f16"),
            timeline_keyframe_str(22330, "#0f0f16"),
        ],
        None,
    );
    let sky_color = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_str(133, "#ffffff"),
            timeline_keyframe_str(11867, "#ffffff"),
            timeline_keyframe_str(13670, "#000000"),
            timeline_keyframe_str(22330, "#000000"),
        ],
        None,
    );
    let sky_light_color = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_str(730, "#ffffff"),
            timeline_keyframe_str(11270, "#ffffff"),
            timeline_keyframe_str(13140, "#7a7aff"),
            timeline_keyframe_str(22860, "#7a7aff"),
        ],
        None,
    );

    // Float tracks: multiply/maximum modifier → FloatModifier.Simple.argumentCodec = Codec.FLOAT.
    let sky_light_factor = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_f32(730, 1.0),
            timeline_keyframe_f32(11270, 1.0),
            timeline_keyframe_f32(13140, 0.24),
            timeline_keyframe_f32(22860, 0.24),
        ],
        None,
    );
    // gameplay/sky_light_level is syncable (SKY_LIGHT_LEVEL has .notPositional().syncable()).
    let sky_light_level = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_f32(133, 1.0),
            timeline_keyframe_f32(11867, 1.0),
            timeline_keyframe_f32(13670, 0.266_666_68),
            timeline_keyframe_f32(22330, 0.266_666_68),
        ],
        None,
    );
    let star_brightness = timeline_track(
        Some("maximum"),
        vec![
            timeline_keyframe_f32(92, 0.037),
            timeline_keyframe_f32(627, 0.0),
            timeline_keyframe_f32(11373, 0.0),
            timeline_keyframe_f32(11732, 0.016),
            timeline_keyframe_f32(11959, 0.044),
            timeline_keyframe_f32(12399, 0.143),
            timeline_keyframe_f32(12729, 0.258),
            timeline_keyframe_f32(13228, 0.5),
            timeline_keyframe_f32(22772, 0.5),
            timeline_keyframe_f32(23032, 0.364),
            timeline_keyframe_f32(23356, 0.225),
            timeline_keyframe_f32(23758, 0.101),
        ],
        None,
    );

    // ARGB colour: multiply modifier → ArgbModifier.argumentCodec = Either<STRING_ARGB, Codec.INT>.
    // When alpha == 0xFF: Either.right → Codec.INT → Tag::Int.
    // -1 = 0xFFFFFFFF (white); -15132378 = 0xFF1A1A26 (night-tinted dark grey).
    let cloud_color = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_i32(133, -1),
            timeline_keyframe_i32(11867, -1),
            timeline_keyframe_i32(13670, -15132378),
            timeline_keyframe_i32(22330, -15132378),
        ],
        None,
    );

    // ARGB colour: override modifier → OverrideModifier.argumentCodec = STRING_ARGB_COLOR.
    // Primary encoder is hexColor(8) → Tag::String("#aarrggbb").
    let sunrise_sunset_color = timeline_track(
        None,
        vec![
            timeline_keyframe_str(71, "#5fefa333"),
            timeline_keyframe_str(310, "#29f5ba33"),
            timeline_keyframe_str(565, "#06fbd433"),
            timeline_keyframe_str(730, "#00ffe533"),
            timeline_keyframe_str(11270, "#00ffe533"),
            timeline_keyframe_str(11397, "#04fcd833"),
            timeline_keyframe_str(11522, "#0ff9cb33"),
            timeline_keyframe_str(11690, "#29f5ba33"),
            timeline_keyframe_str(11929, "#5fefa333"),
            timeline_keyframe_str(12243, "#b1e78733"),
            timeline_keyframe_str(12358, "#cce47e33"),
            timeline_keyframe_str(12512, "#e9e07233"),
            timeline_keyframe_str(12613, "#f6dd6b33"),
            timeline_keyframe_str(12732, "#feda6333"),
            timeline_keyframe_str(12841, "#fed75c33"),
            timeline_keyframe_str(13035, "#ecd25133"),
            timeline_keyframe_str(13252, "#c1cc4733"),
            timeline_keyframe_str(13775, "#36be3733"),
            timeline_keyframe_str(13888, "#1fbb3533"),
            timeline_keyframe_str(14039, "#09b73333"),
            timeline_keyframe_str(14192, "#00b33333"),
            timeline_keyframe_str(21807, "#00b23333"),
            timeline_keyframe_str(21961, "#09b73333"),
            timeline_keyframe_str(22112, "#1fbb3533"),
            timeline_keyframe_str(22225, "#36be3733"),
            timeline_keyframe_str(22748, "#c1cc4733"),
            timeline_keyframe_str(22965, "#ecd25133"),
            timeline_keyframe_str(23159, "#fed75c33"),
            timeline_keyframe_str(23272, "#feda6333"),
            timeline_keyframe_str(23488, "#e9e07233"),
            timeline_keyframe_str(23642, "#cce47e33"),
            timeline_keyframe_str(23757, "#b1e78733"),
        ],
        None,
    );

    // Boolean tracks: OR modifier → BooleanModifier.OR.argumentCodec = Codec.BOOL → Tag::Byte.
    let firefly_bush_sounds = timeline_track(
        Some("or"),
        vec![
            timeline_keyframe_bool(12600, true),
            timeline_keyframe_bool(23401, false),
        ],
        None,
    );
    let creaking_active = timeline_track(
        Some("or"),
        vec![
            timeline_keyframe_bool(12600, true),
            timeline_keyframe_bool(23401, false),
        ],
        None,
    );

    // Time markers are preserved by NETWORK_CODEC (filterSyncableTracks only touches tracks).
    // TimeMarkerInfo.CODEC: showInCommands=true → Compound{ticks, show_in_commands};
    //                       showInCommands=false → Tag::Int(ticks).
    let time_markers = Tag::Compound(vec![
        (
            "minecraft:day".to_string(),
            Tag::Compound(vec![
                ("ticks".to_string(), Tag::Int(1000)),
                ("show_in_commands".to_string(), Tag::Byte(1)),
            ]),
        ),
        (
            "minecraft:midnight".to_string(),
            Tag::Compound(vec![
                ("ticks".to_string(), Tag::Int(18000)),
                ("show_in_commands".to_string(), Tag::Byte(1)),
            ]),
        ),
        (
            "minecraft:night".to_string(),
            Tag::Compound(vec![
                ("ticks".to_string(), Tag::Int(13000)),
                ("show_in_commands".to_string(), Tag::Byte(1)),
            ]),
        ),
        (
            "minecraft:noon".to_string(),
            Tag::Compound(vec![
                ("ticks".to_string(), Tag::Int(6000)),
                ("show_in_commands".to_string(), Tag::Byte(1)),
            ]),
        ),
        // showInCommands=false → encoded as plain Tag::Int(ticks).
        ("minecraft:roll_village_siege".to_string(), Tag::Int(18000)),
        ("minecraft:wake_up_from_sleep".to_string(), Tag::Int(0)),
    ]);

    Tag::Compound(vec![
        (
            "clock".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        ),
        ("period_ticks".to_string(), Tag::Int(24000)),
        (
            "tracks".to_string(),
            Tag::Compound(vec![
                ("minecraft:visual/sun_angle".to_string(), sun_angle),
                ("minecraft:visual/moon_angle".to_string(), moon_angle),
                ("minecraft:visual/star_angle".to_string(), star_angle),
                ("minecraft:visual/fog_color".to_string(), fog_color),
                ("minecraft:visual/sky_color".to_string(), sky_color),
                (
                    "minecraft:visual/sky_light_color".to_string(),
                    sky_light_color,
                ),
                (
                    "minecraft:visual/sky_light_factor".to_string(),
                    sky_light_factor,
                ),
                (
                    "minecraft:visual/star_brightness".to_string(),
                    star_brightness,
                ),
                ("minecraft:visual/cloud_color".to_string(), cloud_color),
                (
                    "minecraft:visual/sunrise_sunset_color".to_string(),
                    sunrise_sunset_color,
                ),
                (
                    "minecraft:gameplay/sky_light_level".to_string(),
                    sky_light_level,
                ),
                (
                    "minecraft:audio/firefly_bush_sounds".to_string(),
                    firefly_bush_sounds,
                ),
                (
                    "minecraft:gameplay/creaking_active".to_string(),
                    creaking_active,
                ),
            ]),
        ),
        ("time_markers".to_string(), time_markers),
    ])
}

/// Builds the NBT compound for `minecraft:moon` filtered to syncable tracks only.
/// Java: data/minecraft/timeline/moon.json; surface_slime_spawn_chance is non-syncable and
/// filtered out. Only visual/moon_phase (MOON_PHASE type, syncable) survives.
pub fn moon_timeline_nbt() -> Tag {
    // MoonPhase.CODEC = StringRepresentable.fromEnum → encodes as Tag::String name.
    let moon_phase = timeline_track(
        None,
        vec![
            timeline_keyframe_str(0, "full_moon"),
            timeline_keyframe_str(24000, "waning_gibbous"),
            timeline_keyframe_str(48000, "third_quarter"),
            timeline_keyframe_str(72000, "waning_crescent"),
            timeline_keyframe_str(96000, "new_moon"),
            timeline_keyframe_str(120000, "waxing_crescent"),
            timeline_keyframe_str(144000, "first_quarter"),
            timeline_keyframe_str(168000, "waxing_gibbous"),
        ],
        None,
    );
    Tag::Compound(vec![
        (
            "clock".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        ),
        ("period_ticks".to_string(), Tag::Int(192000)),
        (
            "tracks".to_string(),
            Tag::Compound(vec![(
                "minecraft:visual/moon_phase".to_string(),
                moon_phase,
            )]),
        ),
    ])
}

/// Builds the NBT compound for `minecraft:villager_schedule`.
/// Java: data/minecraft/timeline/villager_schedule.json; both tracks (villager_activity,
/// baby_villager_activity) are non-syncable → filtered out. Tracks field absent (equals
/// default Map.of()); only clock and period_ticks remain.
pub fn villager_schedule_timeline_nbt() -> Tag {
    Tag::Compound(vec![
        (
            "clock".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        ),
        ("period_ticks".to_string(), Tag::Int(24000)),
    ])
}

/// Builds the NBT compound for `minecraft:early_game`.
/// Java: data/minecraft/timeline/early_game.json; can_pillager_patrol_spawn is non-syncable
/// → filtered out. No period_ticks in source data. Only the clock field remains.
pub fn early_game_timeline_nbt() -> Tag {
    Tag::Compound(vec![(
        "clock".to_string(),
        Tag::String("minecraft:overworld".to_string()),
    )])
}

/// Sends the `minecraft:timeline` registry during configuration.
///
/// The client uses timeline data to drive all sky rendering (sun/moon angles, sky colour,
/// fog colour, star brightness, etc.) via its Timeline evaluation system. Without this
/// registry the client's `timelines` field in the dimension type cannot be resolved and
/// all sky colours remain black (default EnvironmentAttribute values).
///
/// Entry IDs (used by the tags packet): day=0, moon=1, villager_schedule=2, early_game=3.
///
/// Java refs:
///   net/minecraft/resources/RegistryDataLoader.java:125,160 — TIMELINE in sync registry list
///   net/minecraft/world/timeline/Timeline.java:49 — NETWORK_CODEC = filterSyncableTracks
pub fn write_vanilla_timeline_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:timeline").unwrap())?;
    write_var_i32(writer, 4)?; // day=0, moon=1, villager_schedule=2, early_game=3
    let entries: &[(&str, fn() -> Tag)] = &[
        ("day", day_timeline_nbt),
        ("moon", moon_timeline_nbt),
        ("villager_schedule", villager_schedule_timeline_nbt),
        ("early_game", early_game_timeline_nbt),
    ];
    for (name, build_nbt) in entries {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{name}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &build_nbt())?;
    }
    Ok(())
}

pub fn write_vanilla_cat_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const CATS: &[&str] = &[
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
    ];
    write_variant_registry(writer, "minecraft:cat_variant", CATS, |cat| {
        animal_texture_variant_nbt("cat", &format!("cat_{cat}"), "normal")
    })
}

pub fn write_vanilla_chicken_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const CHICKENS: &[(&str, &str)] = &[
        ("cold", "cold"),
        ("temperate", "normal"),
        ("warm", "normal"),
    ];
    write_variant_registry(
        writer,
        "minecraft:chicken_variant",
        CHICKENS,
        |(id, model)| animal_texture_variant_nbt("chicken", &format!("chicken_{id}"), model),
    )
}

pub fn write_vanilla_cow_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const COWS: &[(&str, &str)] = &[("cold", "cold"), ("temperate", "normal"), ("warm", "warm")];
    write_variant_registry(writer, "minecraft:cow_variant", COWS, |(id, model)| {
        animal_texture_variant_nbt("cow", &format!("cow_{id}"), model)
    })
}

pub fn write_vanilla_frog_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const FROGS: &[&str] = &["cold", "temperate", "warm"];
    write_variant_registry(writer, "minecraft:frog_variant", FROGS, |frog| {
        single_texture_variant_nbt(&format!("minecraft:entity/frog/frog_{frog}"))
    })
}

pub fn write_vanilla_pig_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const PIGS: &[(&str, &str)] = &[
        ("cold", "cold"),
        ("temperate", "normal"),
        ("warm", "normal"),
    ];
    write_variant_registry(writer, "minecraft:pig_variant", PIGS, |(id, model)| {
        animal_texture_variant_nbt("pig", &format!("pig_{id}"), model)
    })
}

pub fn write_vanilla_wolf_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const WOLVES: &[(&str, &str)] = &[
        ("ashen", "wolf_ashen"),
        ("black", "wolf_black"),
        ("chestnut", "wolf_chestnut"),
        ("pale", "wolf"),
        ("rusty", "wolf_rusty"),
        ("snowy", "wolf_snowy"),
        ("spotted", "wolf_spotted"),
        ("striped", "wolf_striped"),
        ("woods", "wolf_woods"),
    ];
    write_variant_registry(writer, "minecraft:wolf_variant", WOLVES, |(_id, file)| {
        wolf_variant_nbt(file)
    })
}

pub fn write_vanilla_cat_sound_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const VARIANTS: &[&str] = &["classic", "royal"];
    write_variant_registry(writer, "minecraft:cat_sound_variant", VARIANTS, |_| {
        cat_sound_variant_nbt()
    })
}

pub fn write_vanilla_chicken_sound_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const VARIANTS: &[&str] = &["classic", "picky"];
    write_variant_registry(writer, "minecraft:chicken_sound_variant", VARIANTS, |_| {
        chicken_sound_variant_nbt()
    })
}

pub fn write_vanilla_cow_sound_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const VARIANTS: &[&str] = &["classic", "moody"];
    write_variant_registry(writer, "minecraft:cow_sound_variant", VARIANTS, |_| {
        cow_sound_variant_nbt()
    })
}

pub fn write_vanilla_pig_sound_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const VARIANTS: &[&str] = &["big", "classic", "mini"];
    write_variant_registry(writer, "minecraft:pig_sound_variant", VARIANTS, |_| {
        pig_sound_variant_nbt()
    })
}

pub fn write_vanilla_wolf_sound_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const VARIANTS: &[&str] = &["angry", "big", "classic", "cute", "grumpy", "puglin", "sad"];
    write_variant_registry(writer, "minecraft:wolf_sound_variant", VARIANTS, |_| {
        wolf_sound_variant_nbt()
    })
}

pub fn write_vanilla_zombie_nautilus_variant_registry_packet<W: Write>(
    writer: &mut W,
) -> io::Result<()> {
    const VARIANTS: &[&str] = &["temperate", "warm"];
    write_variant_registry(
        writer,
        "minecraft:zombie_nautilus_variant",
        VARIANTS,
        |id| zombie_nautilus_variant_nbt(id),
    )
}

pub fn write_vanilla_painting_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const PAINTINGS: &[&str] = &[
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
    ];
    write_variant_registry(writer, "minecraft:painting_variant", PAINTINGS, |id| {
        painting_variant_nbt(id)
    })
}

fn write_variant_registry<W, T, F>(
    writer: &mut W,
    registry: &str,
    entries: &[T],
    mut value: F,
) -> io::Result<()>
where
    W: Write,
    F: FnMut(&T) -> Tag,
    T: VariantRegistryElement,
{
    write_identifier(writer, &Identifier::parse(registry).unwrap())?;
    write_var_i32(writer, entries.len() as i32)?;
    for entry in entries {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{}", entry.id())).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &value(entry))?;
    }
    Ok(())
}

trait VariantRegistryElement {
    fn id(&self) -> &str;
}

impl VariantRegistryElement for &str {
    fn id(&self) -> &str {
        self
    }
}

impl VariantRegistryElement for (&str, &str) {
    fn id(&self) -> &str {
        self.0
    }
}

pub fn write_minimal_biome_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:worldgen/biome").unwrap(),
    )?;
    write_var_i32(writer, BIOMES.len() as i32)?;
    for biome in BIOMES {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{biome}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &vanilla_baseline_biome_nbt(biome))?;
    }
    Ok(())
}

pub fn vanilla_baseline_biome_nbt(biome: &str) -> Tag {
    let (has_precipitation, temperature, downfall, water_color) = match biome {
        "the_void" => (false, 0.5, 0.5, 4_159_204),
        "snowy_plains" | "ice_spikes" | "snowy_taiga" | "frozen_river" | "snowy_beach"
        | "frozen_ocean" | "deep_frozen_ocean" | "grove" | "snowy_slopes" | "frozen_peaks"
        | "jagged_peaks" => (true, 0.0, 0.5, 4_020_182),
        "desert" | "savanna" | "savanna_plateau" | "windswept_savanna" | "badlands"
        | "eroded_badlands" | "wooded_badlands" | "nether_wastes" | "warped_forest"
        | "crimson_forest" | "soul_sand_valley" | "basalt_deltas" => (false, 2.0, 0.0, 4_159_204),
        "warm_ocean" => (true, 0.5, 0.5, 4_446_778),
        "lukewarm_ocean" | "deep_lukewarm_ocean" => (true, 0.5, 0.5, 4_566_514),
        "cold_ocean" | "deep_cold_ocean" => (true, 0.5, 0.5, 4_020_182),
        "swamp" | "mangrove_swamp" => (true, 0.8, 0.9, 6_388_580),
        "the_end" | "end_highlands" | "end_midlands" | "small_end_islands" | "end_barrens" => {
            (false, 0.5, 0.5, 4_159_204)
        }
        _ => (true, 0.8, 0.4, 4_159_204),
    };

    Tag::Compound(vec![
        (
            "has_precipitation".to_string(),
            Tag::Byte(if has_precipitation { 1 } else { 0 }),
        ),
        ("temperature".to_string(), Tag::Float(temperature)),
        ("downfall".to_string(), Tag::Float(downfall)),
        (
            "effects".to_string(),
            Tag::Compound(vec![("water_color".to_string(), Tag::Int(water_color))]),
        ),
    ])
}

pub fn dimension_type_nbt(dimension_type: &str) -> Tag {
    match dimension_type {
        "overworld" => overworld_dimension_type_nbt(false),
        "overworld_caves" => overworld_dimension_type_nbt(true),
        "the_end" => fixed_dimension_type_nbt(
            true,
            false,
            true,
            1.0,
            0,
            256,
            256,
            "#minecraft:infiniburn_end",
            0.25,
            Tag::Int(15),
            0,
        ),
        "the_nether" => fixed_dimension_type_nbt(
            false,
            true,
            false,
            8.0,
            0,
            256,
            128,
            "#minecraft:infiniburn_nether",
            0.1,
            Tag::Int(7),
            15,
        ),
        _ => overworld_dimension_type_nbt(false),
    }
}

pub fn overworld_dimension_type_nbt(has_ceiling: bool) -> Tag {
    // `attributes` encodes via EnvironmentAttributeMap.NETWORK_CODEC (syncable only).
    // Each entry uses EnvironmentAttributeMap.Entry.createCodec: override modifier →
    // Either.left(value) → attribute.valueCodec() directly.
    //
    // Syncable visual attributes from data/minecraft/dimension_type/overworld.json:
    //   RGB_COLOR  → ExtraCodecs.STRING_RGB_COLOR   → hexColor(6) → Tag::String "#rrggbb"
    //   ARGB_COLOR → ExtraCodecs.STRING_ARGB_COLOR  → hexColor(8) → Tag::String "#aarrggbb"
    //   FLOAT      → Codec.FLOAT                    → Tag::Float
    //
    // Non-syncable gameplay/audio attributes (bed_rule, nether_portal_spawns_piglin,
    // respawn_anchor_works) are filtered out by NETWORK_CODEC and must be omitted here.
    // Audio attributes (ambient_sounds, background_music) are syncable but their complex
    // codec structs are not yet implemented; clients fall back to their EMPTY defaults.
    let attributes = Tag::Compound(vec![
        (
            "minecraft:visual/sky_color".to_string(),
            Tag::String("#78a7ff".to_string()),
        ),
        (
            "minecraft:visual/fog_color".to_string(),
            Tag::String("#c0d8ff".to_string()),
        ),
        (
            "minecraft:visual/cloud_color".to_string(),
            Tag::String("#ccffffff".to_string()),
        ),
        (
            "minecraft:visual/cloud_height".to_string(),
            Tag::Float(192.33),
        ),
        (
            "minecraft:visual/ambient_light_color".to_string(),
            Tag::String("#0a0a0a".to_string()),
        ),
    ]);

    Tag::Compound(vec![
        ("has_skylight".to_string(), Tag::Byte(1)),
        (
            "has_ceiling".to_string(),
            Tag::Byte(if has_ceiling { 1 } else { 0 }),
        ),
        ("has_ender_dragon_fight".to_string(), Tag::Byte(0)),
        ("coordinate_scale".to_string(), Tag::Double(1.0)),
        ("min_y".to_string(), Tag::Int(-64)),
        ("height".to_string(), Tag::Int(384)),
        ("logical_height".to_string(), Tag::Int(384)),
        (
            "infiniburn".to_string(),
            Tag::String("#minecraft:infiniburn_overworld".to_string()),
        ),
        ("ambient_light".to_string(), Tag::Float(0.0)),
        (
            "monster_spawn_light_level".to_string(),
            Tag::Compound(vec![
                (
                    "type".to_string(),
                    Tag::String("minecraft:uniform".to_string()),
                ),
                ("min_inclusive".to_string(), Tag::Int(0)),
                ("max_inclusive".to_string(), Tag::Int(7)),
            ]),
        ),
        ("monster_spawn_block_light_limit".to_string(), Tag::Int(0)),
        // `timelines`: HolderSet<Timeline> reference. The tag "#minecraft:in_overworld"
        // is resolved by the client using the timeline tags sent in the tags packet.
        // Java: DimensionType.NETWORK_CODEC — RegistryCodecs.homogeneousList(TIMELINE)
        //       → HolderSet.TagKey encodes as Tag::String("#<tag-id>").
        (
            "timelines".to_string(),
            Tag::String("#minecraft:in_overworld".to_string()),
        ),
        // `default_clock`: which world clock drives the timeline for this dimension.
        // Java: WorldClock.CODEC = RegistryFixedCodec → Tag::String("<registry-key>").
        (
            "default_clock".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        ),
        // `attributes`: static base values for syncable EnvironmentAttributes.
        // The timeline tracks multiply/add to these base values at runtime on the client.
        ("attributes".to_string(), attributes),
    ])
}

pub fn fixed_dimension_type_nbt(
    has_skylight: bool,
    has_ceiling: bool,
    has_ender_dragon_fight: bool,
    coordinate_scale: f64,
    min_y: i32,
    height: i32,
    logical_height: i32,
    infiniburn: &str,
    ambient_light: f32,
    monster_spawn_light_level: Tag,
    monster_spawn_block_light_limit: i32,
) -> Tag {
    Tag::Compound(vec![
        (
            "has_skylight".to_string(),
            Tag::Byte(if has_skylight { 1 } else { 0 }),
        ),
        (
            "has_ceiling".to_string(),
            Tag::Byte(if has_ceiling { 1 } else { 0 }),
        ),
        (
            "has_ender_dragon_fight".to_string(),
            Tag::Byte(if has_ender_dragon_fight { 1 } else { 0 }),
        ),
        (
            "coordinate_scale".to_string(),
            Tag::Double(coordinate_scale),
        ),
        ("min_y".to_string(), Tag::Int(min_y)),
        ("height".to_string(), Tag::Int(height)),
        ("logical_height".to_string(), Tag::Int(logical_height)),
        (
            "infiniburn".to_string(),
            Tag::String(infiniburn.to_string()),
        ),
        ("ambient_light".to_string(), Tag::Float(ambient_light)),
        (
            "monster_spawn_light_level".to_string(),
            monster_spawn_light_level,
        ),
        (
            "monster_spawn_block_light_limit".to_string(),
            Tag::Int(monster_spawn_block_light_limit),
        ),
    ])
}

pub fn trim_material_nbt(material: &TrimMaterialEntry) -> Tag {
    let mut fields = vec![
        (
            "asset_name".to_string(),
            Tag::String(material.asset_name.to_string()),
        ),
        (
            "description".to_string(),
            Tag::Compound(vec![
                (
                    "translate".to_string(),
                    Tag::String(format!("trim_material.minecraft.{}", material.id)),
                ),
                ("color".to_string(), Tag::String(material.color.to_string())),
            ]),
        ),
    ];

    if !material.overrides.is_empty() {
        fields.push((
            "override_armor_assets".to_string(),
            Tag::Compound(
                material
                    .overrides
                    .iter()
                    .map(|(asset, suffix)| {
                        ((*asset).to_string(), Tag::String((*suffix).to_string()))
                    })
                    .collect(),
            ),
        ));
    }

    Tag::Compound(fields)
}
