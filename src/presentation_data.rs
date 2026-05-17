#![allow(dead_code)]

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

pub const SOUND_EVENTS: &[SoundEventDef] = &[
    sound("minecraft:entity.player.levelup"),
    sound("minecraft:entity.player.death"),
    sound("minecraft:entity.item.pickup"),
    sound("minecraft:block.note_block.harp"),
    sound("minecraft:block.vault.activate"),
    sound("minecraft:item.bundle.insert"),
    sound("minecraft:item.brush.brushing.generic"),
    sound("minecraft:entity.fishing_bobber.retrieve"),
    sound("minecraft:music_disc.pigstep"),
    sound("minecraft:ui.button.click"),
];

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
    Item,
    Vibration,
    Trail,
    SculkCharge,
}

pub const PARTICLES: &[ParticleDef] = &[
    particle(
        "minecraft:angry_villager",
        false,
        ParticleOptionShape::Simple,
    ),
    particle("minecraft:block", false, ParticleOptionShape::Block),
    particle("minecraft:block_marker", true, ParticleOptionShape::Block),
    particle("minecraft:dust", false, ParticleOptionShape::Dust),
    particle(
        "minecraft:dust_color_transition",
        false,
        ParticleOptionShape::DustColorTransition,
    ),
    particle("minecraft:entity_effect", false, ParticleOptionShape::Color),
    particle("minecraft:item", false, ParticleOptionShape::Item),
    particle("minecraft:vibration", true, ParticleOptionShape::Vibration),
    particle("minecraft:trail", false, ParticleOptionShape::Trail),
    particle(
        "minecraft:sculk_charge",
        true,
        ParticleOptionShape::SculkCharge,
    ),
    particle("minecraft:sonic_boom", true, ParticleOptionShape::Simple),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintingVariantDef {
    pub id: &'static str,
    pub width: u8,
    pub height: u8,
    pub has_author: bool,
}

pub const PAINTING_VARIANTS: &[PaintingVariantDef] = &[
    painting("minecraft:kebab", 1, 1, true),
    painting("minecraft:wanderer", 1, 2, true),
    painting("minecraft:pool", 2, 1, true),
    painting("minecraft:wither", 2, 2, false),
    painting("minecraft:earth", 2, 2, false),
    painting("minecraft:fighters", 4, 2, true),
    painting("minecraft:skeleton", 4, 3, true),
    painting("minecraft:burning_skull", 4, 4, true),
    painting("minecraft:backyard", 3, 4, true),
    painting("minecraft:dennis", 3, 3, true),
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

pub const BANNER_PATTERNS: &[BannerPatternDef] = &[
    banner("minecraft:base", "block.minecraft.banner.base"),
    banner(
        "minecraft:square_bottom_left",
        "block.minecraft.banner.square_bottom_left",
    ),
    banner(
        "minecraft:stripe_center",
        "block.minecraft.banner.stripe_center",
    ),
    banner("minecraft:cross", "block.minecraft.banner.cross"),
    banner(
        "minecraft:straight_cross",
        "block.minecraft.banner.straight_cross",
    ),
    banner("minecraft:gradient", "block.minecraft.banner.gradient"),
    banner("minecraft:globe", "block.minecraft.banner.globe"),
    banner("minecraft:creeper", "block.minecraft.banner.creeper"),
    banner("minecraft:piglin", "block.minecraft.banner.piglin"),
    banner("minecraft:flow", "block.minecraft.banner.flow"),
    banner("minecraft:guster", "block.minecraft.banner.guster"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimMaterialDef {
    pub id: &'static str,
    pub color: u32,
    pub asset_group: &'static str,
}

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
    fn particles_cover_simple_and_data_backed_option_shapes() {
        assert!(PARTICLES.iter().any(|particle| {
            particle.id == "minecraft:block" && particle.option_shape == ParticleOptionShape::Block
        }));
        assert!(PARTICLES.iter().any(|particle| {
            particle.id == "minecraft:dust"
                && particle.option_shape == ParticleOptionShape::Dust
                && !particle.override_limiter
        }));
        assert!(PARTICLES.iter().any(|particle| {
            particle.id == "minecraft:vibration"
                && particle.option_shape == ParticleOptionShape::Vibration
                && particle.override_limiter
        }));
    }

    #[test]
    fn painting_variants_keep_sizes_and_title_author_keys() {
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
    }

    #[test]
    fn banner_patterns_include_vanilla_translation_keys_and_new_patterns() {
        assert!(BANNER_PATTERNS.iter().any(|pattern| {
            pattern.id == "minecraft:base"
                && pattern.translation_key == "block.minecraft.banner.base"
        }));
        assert!(BANNER_PATTERNS
            .iter()
            .any(|pattern| pattern.id == "minecraft:flow"));
        assert!(BANNER_PATTERNS
            .iter()
            .any(|pattern| pattern.id == "minecraft:guster"));
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
