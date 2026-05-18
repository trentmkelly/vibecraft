#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundEventDef {
    pub id: &'static str,
    pub fixed_range: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSource {
    Master,
    Music,
    Records,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
    Voice,
    Ui,
}

impl SoundSource {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Master => "master",
            Self::Music => "music",
            Self::Records => "record",
            Self::Weather => "weather",
            Self::Blocks => "block",
            Self::Hostile => "hostile",
            Self::Neutral => "neutral",
            Self::Players => "player",
            Self::Ambient => "ambient",
            Self::Voice => "voice",
            Self::Ui => "ui",
        }
    }
}

// Source: decompiled-server-26.1.2/net/minecraft/sounds/SoundSource.java
pub const SOUND_SOURCES: &[SoundSource] = &[
    SoundSource::Master,
    SoundSource::Music,
    SoundSource::Records,
    SoundSource::Weather,
    SoundSource::Blocks,
    SoundSource::Hostile,
    SoundSource::Neutral,
    SoundSource::Players,
    SoundSource::Ambient,
    SoundSource::Voice,
    SoundSource::Ui,
];

// Source: decompiled-server-26.1.2/net/minecraft/sounds/SoundEvents.java
const SOUND_EVENTS_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../decompiled-server-26.1.2/net/minecraft/sounds/SoundEvents.java"
));

pub const SOUND_EVENTS_COUNT_26_1_2: usize = 1902;

pub static SOUND_EVENTS: LazyLock<&'static [SoundEventDef]> = LazyLock::new(|| {
    Box::leak(load_sound_events().into_boxed_slice())
});

fn load_sound_events() -> Vec<SoundEventDef> {
    let mut events = Vec::with_capacity(SOUND_EVENTS_COUNT_26_1_2);
    let mut seen = HashSet::new();

    for line in SOUND_EVENTS_SOURCE.lines() {
        let line = line.trim_start();
        if line.contains("CAT_SOUNDS = registerCatSoundVariants()") {
            add_cat_sounds(&mut events, &mut seen);
            continue;
        }

        if line.contains("CHICKEN_SOUNDS = registerChickenSoundVariants()") {
            add_chicken_sounds(&mut events, &mut seen);
            continue;
        }

        if line.contains("COW_SOUNDS = registerCowSoundVariants()") {
            add_cow_sounds(&mut events, &mut seen);
            continue;
        }

        if line.contains("GOAT_HORN_SOUND_VARIANTS = registerGoatHornSoundVariants()") {
            add_goat_horn_variants(&mut events, &mut seen);
            continue;
        }

        if line.contains("PIG_SOUNDS = registerPigSoundVariants()") {
            add_pig_sounds(&mut events, &mut seen);
            continue;
        }

        if line.contains("WOLF_SOUNDS = registerWolfSoundVariants()") {
            add_wolf_sounds(&mut events, &mut seen);
            continue;
        }

        if !line.starts_with("public static final ") {
            continue;
        }

        if let Some(id) = parse_registered_sound_id(line) {
            add_sound_event(&mut events, &mut seen, id, None);
        }
    }

    events
}

fn parse_registered_sound_id(line: &str) -> Option<&str> {
    const FOR_HOLDER: &str = "registerForHolder(\"";
    const REGISTER: &str = "register(\"";

    if let Some(start) = line.find(FOR_HOLDER) {
        let remainder = &line[start + FOR_HOLDER.len()..];
        let end = remainder.find('\"')?;
        return Some(&remainder[..end]);
    }

    let Some(start) = line.find(REGISTER) else {
        return None;
    };
    let remainder = &line[start + REGISTER.len()..];
    let end = remainder.find('\"')?;
    Some(&remainder[..end])
}

fn add_sound_event(
    events: &mut Vec<SoundEventDef>,
    seen: &mut HashSet<&'static str>,
    id: &str,
    fixed_range: Option<u8>,
) {
    let full_id = Box::leak(format!("minecraft:{id}").into_boxed_str());
    if seen.insert(full_id) {
        events.push(SoundEventDef {
            id: full_id,
            fixed_range,
        });
    }
}

fn add_cat_sounds(events: &mut Vec<SoundEventDef>, seen: &mut HashSet<&'static str>) {
    for base in ["cat", "cat_royal"] {
        for event in [
            "ambient",
            "stray_ambient",
            "hiss",
            "hurt",
            "death",
            "eat",
            "beg_for_food",
            "purr",
            "purreow",
        ] {
            add_sound_event(events, seen, &format!("entity.{base}.{event}"), None);
        }
    }
}

fn add_chicken_sounds(events: &mut Vec<SoundEventDef>, seen: &mut HashSet<&'static str>) {
    for base in ["chicken", "chicken_picky"] {
        for event in ["ambient", "hurt", "death"] {
            add_sound_event(events, seen, &format!("entity.{base}.{event}"), None);
        }
    }
}

fn add_cow_sounds(events: &mut Vec<SoundEventDef>, seen: &mut HashSet<&'static str>) {
    for base in ["cow", "cow_moody"] {
        for event in ["ambient", "hurt", "death", "step"] {
            add_sound_event(events, seen, &format!("entity.{base}.{event}"), None);
        }
    }
}

fn add_pig_sounds(events: &mut Vec<SoundEventDef>, seen: &mut HashSet<&'static str>) {
    for base in ["pig", "pig_big", "pig_mini"] {
        for event in ["ambient", "hurt", "death", "eat"] {
            add_sound_event(events, seen, &format!("entity.{base}.{event}"), None);
        }
    }
}

fn add_goat_horn_variants(events: &mut Vec<SoundEventDef>, seen: &mut HashSet<&'static str>) {
    for index in 0..8 {
        add_sound_event(events, seen, &format!("item.goat_horn.sound.{index}"), None);
    }
}

fn add_wolf_sounds(events: &mut Vec<SoundEventDef>, seen: &mut HashSet<&'static str>) {
    for base in [
        "wolf",
        "wolf_puglin",
        "wolf_sad",
        "wolf_angry",
        "wolf_grumpy",
        "wolf_big",
        "wolf_cute",
    ] {
        for event in ["ambient", "death", "growl", "hurt", "pant", "whine"] {
            add_sound_event(events, seen, &format!("entity.{base}.{event}"), None);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleDef {
    pub id: &'static str,
    pub override_limiter: bool,
    pub option_shape: ParticleOptionShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleOptionShape {
    Simple,
    Block,
    Dust,
    DustColorTransition,
    Color,
    Power,
    Item,
    Vibration,
    Trail,
    SculkCharge,
    Shriek,
    Spell,
}

// Source: decompiled-server-26.1.2/net/minecraft/core/particles/ParticleTypes.java
pub const PARTICLES: &[ParticleDef] = &[
    particle(
        "minecraft:angry_villager",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:block", false, ParticleOptionShape::Block),
    particle("minecraft:block_marker", true, ParticleOptionShape::Block),
    particle("minecraft:bubble", false, ParticleOptionShape::Simple),
    particle("minecraft:cloud", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:copper_fire_flame",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:crit", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:damage_indicator",
        true,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:dragon_breath", false, ParticleOptionShape::Power),
    particle(
        "minecraft:dripping_lava",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:falling_lava", false, ParticleOptionShape::Simple),
    particle("minecraft:landing_lava", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:dripping_water",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:falling_water",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:dust", false, ParticleOptionShape::Dust),
    particle(
        "minecraft:dust_color_transition",
        false,
        ParticleOptionShape::DustColorTransition,
    ),
    particle("minecraft:effect", false, ParticleOptionShape::Spell),
    particle(
        "minecraft:elder_guardian",
        true,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:enchanted_hit",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:enchant", false, ParticleOptionShape::Simple),
    particle("minecraft:end_rod", false, ParticleOptionShape::Simple),
    particle("minecraft:entity_effect", false, ParticleOptionShape::Color),
    particle(
        "minecraft:explosion_emitter",
        true,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:explosion", true, ParticleOptionShape::Simple),
    particle("minecraft:gust", true, ParticleOptionShape::Simple),
    particle("minecraft:small_gust", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:gust_emitter_large",
        true,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:gust_emitter_small",
        true,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:sonic_boom", true, ParticleOptionShape::Simple),
    particle("minecraft:falling_dust", false, ParticleOptionShape::Block),
    particle("minecraft:firework", false, ParticleOptionShape::Simple),
    particle("minecraft:fishing", false, ParticleOptionShape::Simple),
    particle("minecraft:flame", false, ParticleOptionShape::Simple),
    particle("minecraft:infested", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:cherry_leaves",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:pale_oak_leaves",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:tinted_leaves", false, ParticleOptionShape::Color),
    particle("minecraft:sculk_soul", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:sculk_charge",
        true,
        ParticleOptionShape::SculkCharge,
    ),
    particle(
        "minecraft:sculk_charge_pop",
        true,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:soul_fire_flame",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:soul", false, ParticleOptionShape::Simple),
    particle("minecraft:flash", false, ParticleOptionShape::Color),
    particle(
        "minecraft:happy_villager",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:composter", false, ParticleOptionShape::Simple),
    particle("minecraft:heart", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:instant_effect",
        false,
        ParticleOptionShape::Spell,
    ),
    particle("minecraft:item", false, ParticleOptionShape::Item),
    particle("minecraft:vibration", true, ParticleOptionShape::Vibration),
    particle("minecraft:trail", false, ParticleOptionShape::Trail),
    particle(
        "minecraft:pause_mob_growth",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:reset_mob_growth",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:item_slime", false, ParticleOptionShape::Simple),
    particle("minecraft:item_cobweb", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:item_snowball",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:large_smoke", false, ParticleOptionShape::Simple),
    particle("minecraft:lava", false, ParticleOptionShape::Simple),
    particle("minecraft:mycelium", false, ParticleOptionShape::Simple),
    particle("minecraft:note", false, ParticleOptionShape::Simple),
    particle("minecraft:poof", true, ParticleOptionShape::Simple),
    particle("minecraft:portal", false, ParticleOptionShape::Simple),
    particle("minecraft:rain", false, ParticleOptionShape::Simple),
    particle("minecraft:smoke", false, ParticleOptionShape::Simple),
    particle("minecraft:white_smoke", false, ParticleOptionShape::Simple),
    particle("minecraft:sneeze", false, ParticleOptionShape::Simple),
    particle("minecraft:spit", true, ParticleOptionShape::Simple),
    particle("minecraft:squid_ink", true, ParticleOptionShape::Simple),
    particle("minecraft:sweep_attack", true, ParticleOptionShape::Simple),
    particle(
        "minecraft:totem_of_undying",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:underwater", false, ParticleOptionShape::Simple),
    particle("minecraft:splash", false, ParticleOptionShape::Simple),
    particle("minecraft:witch", false, ParticleOptionShape::Simple),
    particle("minecraft:bubble_pop", false, ParticleOptionShape::Simple),
    particle("minecraft:current_down", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:bubble_column_up",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:nautilus", false, ParticleOptionShape::Simple),
    particle("minecraft:dolphin", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:campfire_cosy_smoke",
        true,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:campfire_signal_smoke",
        true,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:dripping_honey",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:falling_honey",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:landing_honey",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:falling_nectar",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:falling_spore_blossom",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:ash", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:crimson_spore",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:warped_spore", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:spore_blossom_air",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:dripping_obsidian_tear",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:falling_obsidian_tear",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:landing_obsidian_tear",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:reverse_portal",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:white_ash", false, ParticleOptionShape::Simple),
    particle("minecraft:small_flame", false, ParticleOptionShape::Simple),
    particle("minecraft:snowflake", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:dripping_dripstone_lava",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:falling_dripstone_lava",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:dripping_dripstone_water",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:falling_dripstone_water",
        false,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:glow_squid_ink",
        true,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:glow", true, ParticleOptionShape::Simple),
    particle("minecraft:wax_on", true, ParticleOptionShape::Simple),
    particle("minecraft:wax_off", true, ParticleOptionShape::Simple),
    particle(
        "minecraft:electric_spark",
        true,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:scrape", true, ParticleOptionShape::Simple),
    particle("minecraft:shriek", false, ParticleOptionShape::Shriek),
    particle("minecraft:egg_crack", false, ParticleOptionShape::Simple),
    particle("minecraft:dust_plume", false, ParticleOptionShape::Simple),
    particle(
        "minecraft:trial_spawner_detection",
        true,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:trial_spawner_detection_ominous",
        true,
        ParticleOptionShape::Simple,
    ),
    particle(
        "minecraft:vault_connection",
        true,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:dust_pillar", false, ParticleOptionShape::Block),
    particle(
        "minecraft:ominous_spawning",
        true,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:raid_omen", false, ParticleOptionShape::Simple),
    particle("minecraft:trial_omen", false, ParticleOptionShape::Simple),
    particle("minecraft:block_crumble", false, ParticleOptionShape::Block),
    particle("minecraft:firefly", false, ParticleOptionShape::Simple),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintingVariantDef {
    pub id: &'static str,
    pub width: u8,
    pub height: u8,
    pub has_author: bool,
}

// Source: decompiled-server-26.1.2/data/minecraft/painting_variant/*.json
pub const PAINTING_VARIANTS: &[PaintingVariantDef] = &[
    painting("minecraft:alban", 1, 1, true),
    painting("minecraft:aztec", 1, 1, true),
    painting("minecraft:aztec2", 1, 1, true),
    painting("minecraft:backyard", 3, 4, true),
    painting("minecraft:baroque", 2, 2, true),
    painting("minecraft:bomb", 1, 1, true),
    painting("minecraft:bouquet", 3, 3, true),
    painting("minecraft:burning_skull", 4, 4, true),
    painting("minecraft:bust", 2, 2, true),
    painting("minecraft:cavebird", 3, 3, true),
    painting("minecraft:changing", 4, 2, true),
    painting("minecraft:cotan", 3, 3, true),
    painting("minecraft:courbet", 2, 1, true),
    painting("minecraft:creebet", 2, 1, true),
    painting("minecraft:dennis", 3, 3, true),
    painting("minecraft:donkey_kong", 4, 3, true),
    painting("minecraft:earth", 2, 2, false),
    painting("minecraft:endboss", 3, 3, true),
    painting("minecraft:fern", 3, 3, true),
    painting("minecraft:fighters", 4, 2, true),
    painting("minecraft:finding", 4, 2, true),
    painting("minecraft:fire", 2, 2, false),
    painting("minecraft:graham", 1, 2, true),
    painting("minecraft:humble", 2, 2, true),
    painting("minecraft:kebab", 1, 1, true),
    painting("minecraft:lowmist", 4, 2, true),
    painting("minecraft:match", 2, 2, true),
    painting("minecraft:meditative", 1, 1, true),
    painting("minecraft:orb", 4, 4, true),
    painting("minecraft:owlemons", 3, 3, true),
    painting("minecraft:passage", 4, 2, true),
    painting("minecraft:pigscene", 4, 4, true),
    painting("minecraft:plant", 1, 1, true),
    painting("minecraft:pointer", 4, 4, true),
    painting("minecraft:pond", 3, 4, true),
    painting("minecraft:pool", 2, 1, true),
    painting("minecraft:prairie_ride", 1, 2, true),
    painting("minecraft:sea", 2, 1, true),
    painting("minecraft:skeleton", 4, 3, true),
    painting("minecraft:skull_and_roses", 2, 2, true),
    painting("minecraft:stage", 2, 2, true),
    painting("minecraft:sunflowers", 3, 3, true),
    painting("minecraft:sunset", 2, 1, true),
    painting("minecraft:tides", 3, 3, true),
    painting("minecraft:unpacked", 4, 4, true),
    painting("minecraft:void", 2, 2, true),
    painting("minecraft:wanderer", 1, 2, true),
    painting("minecraft:wasteland", 1, 1, true),
    painting("minecraft:water", 2, 2, false),
    painting("minecraft:wind", 2, 2, false),
    painting("minecraft:wither", 2, 2, false),
];

impl PaintingVariantDef {
    pub fn title_key(self) -> String {
        language_key("painting", self.id, "title")
    }

    pub fn author_key(self) -> Option<String> {
        self.has_author
            .then(|| language_key("painting", self.id, "author"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BannerPatternDef {
    pub id: &'static str,
    pub translation_key: &'static str,
}

// Source: decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BannerPatterns.java
// and decompiled-server-26.1.2/data/minecraft/banner_pattern/*.json
pub const BANNER_PATTERNS: &[BannerPatternDef] = &[
    banner("minecraft:base", "block.minecraft.banner.base"),
    banner("minecraft:border", "block.minecraft.banner.border"),
    banner("minecraft:bricks", "block.minecraft.banner.bricks"),
    banner("minecraft:circle", "block.minecraft.banner.circle"),
    banner("minecraft:creeper", "block.minecraft.banner.creeper"),
    banner("minecraft:cross", "block.minecraft.banner.cross"),
    banner("minecraft:curly_border", "block.minecraft.banner.curly_border"),
    banner("minecraft:diagonal_left", "block.minecraft.banner.diagonal_left"),
    banner("minecraft:diagonal_right", "block.minecraft.banner.diagonal_right"),
    banner("minecraft:diagonal_up_left", "block.minecraft.banner.diagonal_up_left"),
    banner("minecraft:diagonal_up_right", "block.minecraft.banner.diagonal_up_right"),
    banner("minecraft:flow", "block.minecraft.banner.flow"),
    banner("minecraft:flower", "block.minecraft.banner.flower"),
    banner("minecraft:globe", "block.minecraft.banner.globe"),
    banner("minecraft:gradient", "block.minecraft.banner.gradient"),
    banner("minecraft:gradient_up", "block.minecraft.banner.gradient_up"),
    banner("minecraft:guster", "block.minecraft.banner.guster"),
    banner("minecraft:half_horizontal", "block.minecraft.banner.half_horizontal"),
    banner(
        "minecraft:half_horizontal_bottom",
        "block.minecraft.banner.half_horizontal_bottom",
    ),
    banner("minecraft:half_vertical", "block.minecraft.banner.half_vertical"),
    banner(
        "minecraft:half_vertical_right",
        "block.minecraft.banner.half_vertical_right",
    ),
    banner("minecraft:mojang", "block.minecraft.banner.mojang"),
    banner("minecraft:piglin", "block.minecraft.banner.piglin"),
    banner("minecraft:rhombus", "block.minecraft.banner.rhombus"),
    banner("minecraft:skull", "block.minecraft.banner.skull"),
    banner("minecraft:small_stripes", "block.minecraft.banner.small_stripes"),
    banner(
        "minecraft:square_bottom_left",
        "block.minecraft.banner.square_bottom_left",
    ),
    banner(
        "minecraft:square_bottom_right",
        "block.minecraft.banner.square_bottom_right",
    ),
    banner("minecraft:square_top_left", "block.minecraft.banner.square_top_left"),
    banner("minecraft:square_top_right", "block.minecraft.banner.square_top_right"),
    banner(
        "minecraft:straight_cross",
        "block.minecraft.banner.straight_cross",
    ),
    banner("minecraft:stripe_bottom", "block.minecraft.banner.stripe_bottom"),
    banner("minecraft:stripe_center", "block.minecraft.banner.stripe_center"),
    banner("minecraft:stripe_downleft", "block.minecraft.banner.stripe_downleft"),
    banner(
        "minecraft:stripe_downright",
        "block.minecraft.banner.stripe_downright",
    ),
    banner("minecraft:stripe_left", "block.minecraft.banner.stripe_left"),
    banner("minecraft:stripe_middle", "block.minecraft.banner.stripe_middle"),
    banner("minecraft:stripe_right", "block.minecraft.banner.stripe_right"),
    banner("minecraft:stripe_top", "block.minecraft.banner.stripe_top"),
    banner("minecraft:triangle_bottom", "block.minecraft.banner.triangle_bottom"),
    banner("minecraft:triangle_top", "block.minecraft.banner.triangle_top"),
    banner(
        "minecraft:triangles_bottom",
        "block.minecraft.banner.triangles_bottom",
    ),
    banner("minecraft:triangles_top", "block.minecraft.banner.triangles_top"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimMaterialDef {
    pub id: &'static str,
    pub color: u32,
    pub asset_group: &'static str,
}

// Source: decompiled-server-26.1.2/net/minecraft/world/item/equipment/trim/TrimMaterials.java
// and decompiled-server-26.1.2/data/minecraft/trim_material/*.json
pub const TRIM_MATERIALS: &[TrimMaterialDef] = &[
    trim_material("minecraft:quartz", 14931140, "quartz"),
    trim_material("minecraft:iron", 15527148, "iron"),
    trim_material("minecraft:netherite", 6445145, "netherite"),
    trim_material("minecraft:redstone", 9901575, "redstone"),
    trim_material("minecraft:copper", 11823181, "copper"),
    trim_material("minecraft:gold", 14594349, "gold"),
    trim_material("minecraft:emerald", 1155126, "emerald"),
    trim_material("minecraft:diamond", 7269586, "diamond"),
    trim_material("minecraft:lapis", 4288151, "lapis"),
    trim_material("minecraft:amethyst", 10116294, "amethyst"),
    trim_material("minecraft:resin", 16545810, "resin"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimPatternDef {
    pub id: &'static str,
    pub decal: bool,
}

// Source: decompiled-server-26.1.2/net/minecraft/world/item/equipment/trim/TrimPatterns.java
// and decompiled-server-26.1.2/data/minecraft/trim_pattern/*.json
pub const TRIM_PATTERNS: &[TrimPatternDef] = &[
    trim_pattern("minecraft:sentry"),
    trim_pattern("minecraft:dune"),
    trim_pattern("minecraft:coast"),
    trim_pattern("minecraft:wild"),
    trim_pattern("minecraft:ward"),
    trim_pattern("minecraft:eye"),
    trim_pattern("minecraft:vex"),
    trim_pattern("minecraft:tide"),
    trim_pattern("minecraft:snout"),
    trim_pattern("minecraft:rib"),
    trim_pattern("minecraft:spire"),
    trim_pattern("minecraft:wayfinder"),
    trim_pattern("minecraft:shaper"),
    trim_pattern("minecraft:silence"),
    trim_pattern("minecraft:raiser"),
    trim_pattern("minecraft:host"),
    trim_pattern("minecraft:flow"),
    trim_pattern("minecraft:bolt"),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstrumentDef {
    pub id: &'static str,
    pub sound_event: &'static str,
    pub use_duration_seconds: f32,
    pub range: f32,
}

// Source: decompiled-server-26.1.2/net/minecraft/world/item/JukeboxSongs.java
// decompiled-server-26.1.2/net/minecraft/world/item/InstrumentItem.java
// and data under decompiled-server-26.1.2/data/minecraft/{instrument,jukebox_song}
pub const INSTRUMENTS: &[InstrumentDef] = &[
    instrument(
        "minecraft:ponder_goat_horn",
        "minecraft:item.goat_horn.sound.0",
    ),
    instrument(
        "minecraft:sing_goat_horn",
        "minecraft:item.goat_horn.sound.1",
    ),
    instrument(
        "minecraft:seek_goat_horn",
        "minecraft:item.goat_horn.sound.2",
    ),
    instrument(
        "minecraft:feel_goat_horn",
        "minecraft:item.goat_horn.sound.3",
    ),
    instrument(
        "minecraft:admire_goat_horn",
        "minecraft:item.goat_horn.sound.4",
    ),
    instrument(
        "minecraft:call_goat_horn",
        "minecraft:item.goat_horn.sound.5",
    ),
    instrument(
        "minecraft:yearn_goat_horn",
        "minecraft:item.goat_horn.sound.6",
    ),
    instrument(
        "minecraft:dream_goat_horn",
        "minecraft:item.goat_horn.sound.7",
    ),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JukeboxSongDef {
    pub id: &'static str,
    pub sound_event: &'static str,
    pub comparator_output: u8,
    pub length_seconds: f32,
}

// Source: decompiled-server-26.1.2/net/minecraft/world/item/JukeboxSongs.java
// and decompiled-server-26.1.2/net/minecraft/world/item/InstrumentItem.java
// plus decompiled-server-26.1.2/data/minecraft/{instrument,jukebox_song}/*.json
pub const JUKEBOX_SONGS: &[JukeboxSongDef] = &[
    jukebox_song("minecraft:13", "minecraft:music_disc.13", 1, 178.0),
    jukebox_song("minecraft:cat", "minecraft:music_disc.cat", 2, 185.0),
    jukebox_song("minecraft:blocks", "minecraft:music_disc.blocks", 3, 345.0),
    jukebox_song("minecraft:chirp", "minecraft:music_disc.chirp", 4, 185.0),
    jukebox_song("minecraft:far", "minecraft:music_disc.far", 5, 174.0),
    jukebox_song("minecraft:mall", "minecraft:music_disc.mall", 6, 197.0),
    jukebox_song("minecraft:mellohi", "minecraft:music_disc.mellohi", 7, 96.0),
    jukebox_song("minecraft:stal", "minecraft:music_disc.stal", 8, 150.0),
    jukebox_song("minecraft:strad", "minecraft:music_disc.strad", 9, 188.0),
    jukebox_song("minecraft:ward", "minecraft:music_disc.ward", 10, 251.0),
    jukebox_song("minecraft:11", "minecraft:music_disc.11", 11, 71.0),
    jukebox_song("minecraft:wait", "minecraft:music_disc.wait", 12, 238.0),
    jukebox_song(
        "minecraft:pigstep",
        "minecraft:music_disc.pigstep",
        13,
        149.0,
    ),
    jukebox_song(
        "minecraft:otherside",
        "minecraft:music_disc.otherside",
        14,
        195.0,
    ),
    jukebox_song("minecraft:5", "minecraft:music_disc.5", 15, 178.0),
    jukebox_song("minecraft:relic", "minecraft:music_disc.relic", 14, 218.0),
    jukebox_song(
        "minecraft:precipice",
        "minecraft:music_disc.precipice",
        13,
        299.0,
    ),
    jukebox_song(
        "minecraft:creator",
        "minecraft:music_disc.creator",
        12,
        176.0,
    ),
    jukebox_song(
        "minecraft:creator_music_box",
        "minecraft:music_disc.creator_music_box",
        11,
        73.0,
    ),
    jukebox_song("minecraft:tears", "minecraft:music_disc.tears", 10, 175.0),
    jukebox_song("minecraft:lava_chicken", "minecraft:music_disc.lava_chicken", 9, 134.0),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageTypeDef {
    pub id: &'static str,
    pub message_id: &'static str,
    pub scaling: DamageScaling,
    pub exhaustion: f32,
    pub effects: DamageEffects,
    pub death_message_type: DeathMessageType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageScaling {
    Never,
    Always,
    WhenCausedByLivingNonPlayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageEffects {
    Hurt,
    Burning,
    Drowning,
    Freezing,
    Poking,
    Thorns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathMessageType {
    Default,
    FallVariants,
    IntentionalGameDesign,
}

// Source: decompiled-server-26.1.2/net/minecraft/world/damagesource/DamageType.java
// and data/minecraft/damage_type/*.json
pub const DAMAGE_TYPES: &[DamageTypeDef] = &[
    damage(
        "minecraft:in_fire",
        "inFire",
        0.1,
        DamageEffects::Burning,
        DeathMessageType::Default,
    ),
    damage(
        "minecraft:on_fire",
        "onFire",
        0.0,
        DamageEffects::Burning,
        DeathMessageType::Default,
    ),
    damage(
        "minecraft:lava",
        "lava",
        0.1,
        DamageEffects::Burning,
        DeathMessageType::Default,
    ),
    damage(
        "minecraft:drown",
        "drown",
        0.0,
        DamageEffects::Drowning,
        DeathMessageType::Default,
    ),
    damage(
        "minecraft:cactus",
        "cactus",
        0.1,
        DamageEffects::Hurt,
        DeathMessageType::Default,
    ),
    damage(
        "minecraft:fall",
        "fall",
        0.0,
        DamageEffects::Hurt,
        DeathMessageType::FallVariants,
    ),
    damage(
        "minecraft:out_of_world",
        "outOfWorld",
        0.0,
        DamageEffects::Hurt,
        DeathMessageType::Default,
    ),
    damage(
        "minecraft:generic",
        "generic",
        0.0,
        DamageEffects::Hurt,
        DeathMessageType::Default,
    ),
    damage(
        "minecraft:freeze",
        "freeze",
        0.0,
        DamageEffects::Freezing,
        DeathMessageType::Default,
    ),
    damage(
        "minecraft:player_attack",
        "player",
        0.1,
        DamageEffects::Hurt,
        DeathMessageType::Default,
    ),
    damage(
        "minecraft:arrow",
        "arrow",
        0.1,
        DamageEffects::Hurt,
        DeathMessageType::Default,
    ),
    damage(
        "minecraft:fireball",
        "fireball",
        0.1,
        DamageEffects::Burning,
        DeathMessageType::Default,
    ),
    damage(
        "minecraft:thorns",
        "thorns",
        0.1,
        DamageEffects::Thorns,
        DeathMessageType::Default,
    ),
    DamageTypeDef {
        id: "minecraft:explosion",
        message_id: "explosion",
        scaling: DamageScaling::Always,
        exhaustion: 0.1,
        effects: DamageEffects::Hurt,
        death_message_type: DeathMessageType::Default,
    },
    DamageTypeDef {
        id: "minecraft:bad_respawn_point",
        message_id: "badRespawnPoint",
        scaling: DamageScaling::Always,
        exhaustion: 0.1,
        effects: DamageEffects::Hurt,
        death_message_type: DeathMessageType::IntentionalGameDesign,
    },
    damage(
        "minecraft:mace_smash",
        "mace_smash",
        0.1,
        DamageEffects::Hurt,
        DeathMessageType::Default,
    ),
];

impl DamageTypeDef {
    pub fn death_message_key(
        self,
        has_attacker: bool,
        has_item: bool,
        fall_variant: Option<FallVariant>,
    ) -> &'static str {
        match self.death_message_type {
            DeathMessageType::IntentionalGameDesign => "death.attack.badRespawnPoint.message",
            DeathMessageType::FallVariants => match fall_variant.unwrap_or(FallVariant::Generic) {
                FallVariant::Ladder => "death.fell.accident.ladder",
                FallVariant::Vines => "death.fell.accident.vines",
                FallVariant::WeepingVines => "death.fell.accident.weeping_vines",
                FallVariant::TwistingVines => "death.fell.accident.twisting_vines",
                FallVariant::Scaffolding => "death.fell.accident.scaffolding",
                FallVariant::OtherClimbable => "death.fell.accident.other_climbable",
                FallVariant::Generic => "death.fell.accident.generic",
            },
            DeathMessageType::Default if has_attacker && has_item => "death.attack.generic.item",
            DeathMessageType::Default if has_attacker => "death.attack.generic.player",
            DeathMessageType::Default => "death.attack.generic",
        }
    }

    pub fn typed_death_message_key(self, has_attacker: bool, has_item: bool) -> String {
        match self.death_message_type {
            DeathMessageType::Default if has_attacker && has_item => {
                format!("death.attack.{}.item", self.message_id)
            }
            DeathMessageType::Default if has_attacker => {
                format!("death.attack.{}.player", self.message_id)
            }
            DeathMessageType::Default => format!("death.attack.{}", self.message_id),
            _ => self
                .death_message_key(has_attacker, has_item, None)
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallVariant {
    Ladder,
    Vines,
    WeepingVines,
    TwistingVines,
    Scaffolding,
    OtherClimbable,
    Generic,
}

pub fn find_sound_event(id: &str) -> Option<&'static SoundEventDef> {
    SOUND_EVENTS.iter().find(|event| event.id == id)
}

pub fn find_damage_type(id: &str) -> Option<&'static DamageTypeDef> {
    DAMAGE_TYPES.iter().find(|damage| damage.id == id)
}

pub fn language_key(prefix: &str, id: &str, suffix: &str) -> String {
    let short = id
        .strip_prefix("minecraft:")
        .unwrap_or(id)
        .replace('/', ".");
    format!("{prefix}.minecraft.{short}.{suffix}")
}

const fn sound(id: &'static str) -> SoundEventDef {
    SoundEventDef {
        id,
        fixed_range: None,
    }
}

const fn particle(
    id: &'static str,
    override_limiter: bool,
    option_shape: ParticleOptionShape,
) -> ParticleDef {
    ParticleDef {
        id,
        override_limiter,
        option_shape,
    }
}

const fn painting(id: &'static str, width: u8, height: u8, has_author: bool) -> PaintingVariantDef {
    PaintingVariantDef {
        id,
        width,
        height,
        has_author,
    }
}

const fn banner(id: &'static str, translation_key: &'static str) -> BannerPatternDef {
    BannerPatternDef {
        id,
        translation_key,
    }
}

const fn trim_material(id: &'static str, color: u32, asset_group: &'static str) -> TrimMaterialDef {
    TrimMaterialDef {
        id,
        color,
        asset_group,
    }
}

const fn trim_pattern(id: &'static str) -> TrimPatternDef {
    TrimPatternDef { id, decal: false }
}

const fn instrument(id: &'static str, sound_event: &'static str) -> InstrumentDef {
    InstrumentDef {
        id,
        sound_event,
        use_duration_seconds: 7.0,
        range: 256.0,
    }
}

const fn jukebox_song(
    id: &'static str,
    sound_event: &'static str,
    comparator_output: u8,
    length_seconds: f32,
) -> JukeboxSongDef {
    JukeboxSongDef {
        id,
        sound_event,
        comparator_output,
        length_seconds,
    }
}

const fn damage(
    id: &'static str,
    message_id: &'static str,
    exhaustion: f32,
    effects: DamageEffects,
    death_message_type: DeathMessageType,
) -> DamageTypeDef {
    DamageTypeDef {
        id,
        message_id,
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        exhaustion,
        effects,
        death_message_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn sound_sources_match_vanilla_serialized_names_and_sound_events_are_lookupable() {
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
        assert_eq!(
            find_sound_event("minecraft:music_disc.pigstep").unwrap().id,
            "minecraft:music_disc.pigstep"
        );
        assert!(find_sound_event("minecraft:missing").is_none());
    }

    #[test]
    fn sound_events_cover_referenced_vanilla_ids_without_duplicates() {
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

        let no_author = ["minecraft:earth", "minecraft:fire", "minecraft:water", "minecraft:wind", "minecraft:wither"];
        for id in no_author {
            let entry = PAINTING_VARIANTS.iter().find(|variant| variant.id == id).unwrap();
            assert!(!entry.has_author);
        }
    }

    #[test]
    fn banner_patterns_include_vanilla_translation_keys_and_new_patterns() {
        assert_eq!(BANNER_PATTERNS.len(), 43);
        assert_eq!(BANNER_PATTERNS.first().unwrap().id, "minecraft:base");
        assert_eq!(BANNER_PATTERNS.last().unwrap().id, "minecraft:triangles_top");
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
            BANNER_PATTERNS.iter().find(|pattern| pattern.id == "minecraft:flow").unwrap().translation_key,
            "block.minecraft.banner.flow"
        );
        assert_eq!(
            BANNER_PATTERNS.iter().find(|pattern| pattern.id == "minecraft:triangles_top").unwrap().translation_key,
            "block.minecraft.banner.triangles_top"
        );
        assert_eq!(
            BANNER_PATTERNS.iter().find(|pattern| pattern.id == "minecraft:diagonal_up_left").unwrap().translation_key,
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
        assert!(TRIM_MATERIALS.iter().any(|material| {
            material.id == "minecraft:resin"
                && material.color == 16545810
                && material.asset_group == "resin"
        }));
        assert_eq!(TRIM_PATTERNS.len(), 18);
        assert!(TRIM_PATTERNS.iter().all(|pattern| !pattern.decal));
        assert!(TRIM_PATTERNS
            .iter()
            .any(|pattern| pattern.id == "minecraft:bolt"));
    }

    #[test]
    fn instruments_and_jukebox_songs_preserve_sound_and_redstone_surfaces() {
        assert_eq!(INSTRUMENTS.len(), 8);
        assert!(INSTRUMENTS.iter().all(|instrument| {
            instrument.use_duration_seconds == 7.0 && instrument.range == 256.0
        }));
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
            JUKEBOX_SONGS.iter().find(|song| song.id == "minecraft:tears").unwrap(),
            &JukeboxSongDef {
                id: "minecraft:tears",
                sound_event: "minecraft:music_disc.tears",
                comparator_output: 10,
                length_seconds: 175.0,
            }
        );
        assert_eq!(
            JUKEBOX_SONGS.iter().find(|song| song.id == "minecraft:lava_chicken").unwrap(),
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
}
