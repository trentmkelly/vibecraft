#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageScaling {
    Never,
    WhenCausedByLivingNonPlayer,
    Always,
}

impl DamageScaling {
    pub fn id(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::WhenCausedByLivingNonPlayer => "when_caused_by_living_non_player",
            Self::Always => "always",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Some(match id {
            "never" => Self::Never,
            "when_caused_by_living_non_player" => Self::WhenCausedByLivingNonPlayer,
            "always" => Self::Always,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageEffects {
    Hurt,
    Thorns,
    Drowning,
    Burning,
    Poking,
    Freezing,
}

impl DamageEffects {
    pub fn id(self) -> &'static str {
        match self {
            Self::Hurt => "hurt",
            Self::Thorns => "thorns",
            Self::Drowning => "drowning",
            Self::Burning => "burning",
            Self::Poking => "poking",
            Self::Freezing => "freezing",
        }
    }

    pub fn hurt_sound(self) -> &'static str {
        match self {
            Self::Hurt | Self::Thorns => "minecraft:entity.player.hurt",
            Self::Drowning => "minecraft:entity.player.hurt_drown",
            Self::Burning => "minecraft:entity.player.hurt_on_fire",
            Self::Poking => "minecraft:entity.player.hurt_sweet_berry_bush",
            Self::Freezing => "minecraft:entity.player.hurt_freeze",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Some(match id {
            "hurt" => Self::Hurt,
            "thorns" => Self::Thorns,
            "drowning" => Self::Drowning,
            "burning" => Self::Burning,
            "poking" => Self::Poking,
            "freezing" => Self::Freezing,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathMessageType {
    Default,
    FallVariants,
    IntentionalGameDesign,
}

impl DeathMessageType {
    pub fn id(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::FallVariants => "fall_variants",
            Self::IntentionalGameDesign => "intentional_game_design",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Some(match id {
            "default" => Self::Default,
            "fall_variants" => Self::FallVariants,
            "intentional_game_design" => Self::IntentionalGameDesign,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageTypeDef {
    pub id: &'static str,
    pub message_id: &'static str,
    pub scaling: DamageScaling,
    pub exhaustion: f32,
    pub effects: DamageEffects,
    pub death_message_type: DeathMessageType,
}

const fn damage_type(
    id: &'static str,
    message_id: &'static str,
    scaling: DamageScaling,
    exhaustion: f32,
    effects: DamageEffects,
    death_message_type: DeathMessageType,
) -> DamageTypeDef {
    DamageTypeDef {
        id,
        message_id,
        scaling,
        exhaustion,
        effects,
        death_message_type,
    }
}

const fn normal(id: &'static str, message_id: &'static str, exhaustion: f32) -> DamageTypeDef {
    damage_type(
        id,
        message_id,
        DamageScaling::WhenCausedByLivingNonPlayer,
        exhaustion,
        DamageEffects::Hurt,
        DeathMessageType::Default,
    )
}

const fn effect(
    id: &'static str,
    message_id: &'static str,
    exhaustion: f32,
    effects: DamageEffects,
) -> DamageTypeDef {
    damage_type(
        id,
        message_id,
        DamageScaling::WhenCausedByLivingNonPlayer,
        exhaustion,
        effects,
        DeathMessageType::Default,
    )
}

pub const BUILTIN_DAMAGE_TYPES: &[DamageTypeDef] = &[
    effect("minecraft:in_fire", "inFire", 0.1, DamageEffects::Burning),
    effect("minecraft:campfire", "inFire", 0.1, DamageEffects::Burning),
    normal("minecraft:lightning_bolt", "lightningBolt", 0.1),
    effect("minecraft:on_fire", "onFire", 0.0, DamageEffects::Burning),
    effect("minecraft:lava", "lava", 0.1, DamageEffects::Burning),
    effect(
        "minecraft:hot_floor",
        "hotFloor",
        0.1,
        DamageEffects::Burning,
    ),
    normal("minecraft:in_wall", "inWall", 0.0),
    normal("minecraft:cramming", "cramming", 0.0),
    effect("minecraft:drown", "drown", 0.0, DamageEffects::Drowning),
    normal("minecraft:starve", "starve", 0.0),
    normal("minecraft:cactus", "cactus", 0.1),
    damage_type(
        "minecraft:fall",
        "fall",
        DamageScaling::WhenCausedByLivingNonPlayer,
        0.0,
        DamageEffects::Hurt,
        DeathMessageType::FallVariants,
    ),
    damage_type(
        "minecraft:ender_pearl",
        "fall",
        DamageScaling::WhenCausedByLivingNonPlayer,
        0.0,
        DamageEffects::Hurt,
        DeathMessageType::FallVariants,
    ),
    normal("minecraft:fly_into_wall", "flyIntoWall", 0.0),
    normal("minecraft:out_of_world", "outOfWorld", 0.0),
    normal("minecraft:generic", "generic", 0.0),
    normal("minecraft:magic", "magic", 0.0),
    normal("minecraft:wither", "wither", 0.0),
    normal("minecraft:dragon_breath", "dragonBreath", 0.0),
    normal("minecraft:dry_out", "dryout", 0.1),
    effect(
        "minecraft:sweet_berry_bush",
        "sweetBerryBush",
        0.1,
        DamageEffects::Poking,
    ),
    effect("minecraft:freeze", "freeze", 0.0, DamageEffects::Freezing),
    normal("minecraft:stalagmite", "stalagmite", 0.0),
    normal("minecraft:falling_block", "fallingBlock", 0.1),
    normal("minecraft:falling_anvil", "anvil", 0.1),
    normal("minecraft:falling_stalactite", "fallingStalactite", 0.1),
    normal("minecraft:sting", "sting", 0.1),
    normal("minecraft:mob_attack", "mob", 0.1),
    normal("minecraft:mob_attack_no_aggro", "mob", 0.1),
    normal("minecraft:player_attack", "player", 0.1),
    normal("minecraft:spear", "spear", 0.1),
    normal("minecraft:arrow", "arrow", 0.1),
    normal("minecraft:trident", "trident", 0.1),
    normal("minecraft:mob_projectile", "mob", 0.1),
    normal("minecraft:spit", "mob", 0.1),
    normal("minecraft:wind_charge", "mob", 0.1),
    normal("minecraft:fireworks", "fireworks", 0.1),
    effect(
        "minecraft:unattributed_fireball",
        "onFire",
        0.1,
        DamageEffects::Burning,
    ),
    effect(
        "minecraft:fireball",
        "fireball",
        0.1,
        DamageEffects::Burning,
    ),
    normal("minecraft:wither_skull", "witherSkull", 0.1),
    normal("minecraft:thrown", "thrown", 0.1),
    normal("minecraft:indirect_magic", "indirectMagic", 0.0),
    effect("minecraft:thorns", "thorns", 0.1, DamageEffects::Thorns),
    damage_type(
        "minecraft:explosion",
        "explosion",
        DamageScaling::Always,
        0.1,
        DamageEffects::Hurt,
        DeathMessageType::Default,
    ),
    damage_type(
        "minecraft:player_explosion",
        "explosion.player",
        DamageScaling::Always,
        0.1,
        DamageEffects::Hurt,
        DeathMessageType::Default,
    ),
    damage_type(
        "minecraft:sonic_boom",
        "sonic_boom",
        DamageScaling::Always,
        0.0,
        DamageEffects::Hurt,
        DeathMessageType::Default,
    ),
    damage_type(
        "minecraft:bad_respawn_point",
        "badRespawnPoint",
        DamageScaling::Always,
        0.1,
        DamageEffects::Hurt,
        DeathMessageType::IntentionalGameDesign,
    ),
    normal("minecraft:outside_border", "outsideBorder", 0.0),
    normal("minecraft:generic_kill", "genericKill", 0.0),
    normal("minecraft:mace_smash", "mace_smash", 0.1),
];

pub fn builtin_damage_type(id: &str) -> Option<&'static DamageTypeDef> {
    let id = normalize_identifier(id);
    BUILTIN_DAMAGE_TYPES.iter().find(|damage| damage.id == id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageEntityRef {
    pub id: i32,
    pub is_living: bool,
    pub is_player: bool,
    pub is_creative_player: bool,
}

impl DamageEntityRef {
    pub const fn living_mob(id: i32) -> Self {
        Self {
            id,
            is_living: true,
            is_player: false,
            is_creative_player: false,
        }
    }

    pub const fn player(id: i32, is_creative_player: bool) -> Self {
        Self {
            id,
            is_living: true,
            is_player: true,
            is_creative_player,
        }
    }

    pub const fn non_living(id: i32) -> Self {
        Self {
            id,
            is_living: false,
            is_player: false,
            is_creative_player: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageSource {
    pub damage_type: &'static DamageTypeDef,
    pub direct_entity: Option<DamageEntityRef>,
    pub causing_entity: Option<DamageEntityRef>,
    pub source_position: Option<[f64; 3]>,
}

impl DamageSource {
    pub const fn simple(damage_type: &'static DamageTypeDef) -> Self {
        Self {
            damage_type,
            direct_entity: None,
            causing_entity: None,
            source_position: None,
        }
    }

    pub const fn caused_by(damage_type: &'static DamageTypeDef, entity: DamageEntityRef) -> Self {
        Self {
            damage_type,
            direct_entity: Some(entity),
            causing_entity: Some(entity),
            source_position: None,
        }
    }

    pub const fn indirect(
        damage_type: &'static DamageTypeDef,
        direct_entity: DamageEntityRef,
        causing_entity: Option<DamageEntityRef>,
    ) -> Self {
        Self {
            damage_type,
            direct_entity: Some(direct_entity),
            causing_entity,
            source_position: None,
        }
    }

    pub const fn positioned(
        damage_type: &'static DamageTypeDef,
        source_position: [f64; 3],
    ) -> Self {
        Self {
            damage_type,
            direct_entity: None,
            causing_entity: None,
            source_position: Some(source_position),
        }
    }

    pub fn is_direct(&self) -> bool {
        self.causing_entity == self.direct_entity
    }

    pub fn food_exhaustion(&self) -> f32 {
        self.damage_type.exhaustion
    }

    pub fn msg_id(&self) -> &'static str {
        self.damage_type.message_id
    }

    pub fn is_creative_player(&self) -> bool {
        self.causing_entity
            .map(|entity| entity.is_creative_player)
            .unwrap_or(false)
    }

    pub fn scales_with_difficulty(&self) -> bool {
        let caused_by = self.causing_entity;
        scaling_applies_to_difficulty(
            self.damage_type.scaling,
            caused_by.map(|entity| entity.is_living).unwrap_or(false),
            caused_by.map(|entity| entity.is_player).unwrap_or(false),
        )
    }
}

pub fn simple_source(id: &str) -> Option<DamageSource> {
    builtin_damage_type(id).map(DamageSource::simple)
}

pub fn mob_attack_source(mob: DamageEntityRef) -> DamageSource {
    DamageSource::caused_by(required_builtin("minecraft:mob_attack"), mob)
}

pub fn player_attack_source(player: DamageEntityRef) -> DamageSource {
    DamageSource::caused_by(required_builtin("minecraft:player_attack"), player)
}

pub fn arrow_source(arrow: DamageEntityRef, owner: Option<DamageEntityRef>) -> DamageSource {
    DamageSource::indirect(required_builtin("minecraft:arrow"), arrow, owner)
}

pub fn fireball_source(fireball: DamageEntityRef, owner: Option<DamageEntityRef>) -> DamageSource {
    if let Some(owner) = owner {
        DamageSource::indirect(
            required_builtin("minecraft:fireball"),
            fireball,
            Some(owner),
        )
    } else {
        DamageSource::caused_by(
            required_builtin("minecraft:unattributed_fireball"),
            fireball,
        )
    }
}

pub fn explosion_source(
    direct_entity: Option<DamageEntityRef>,
    causing_entity: Option<DamageEntityRef>,
) -> DamageSource {
    let id = if direct_entity.is_some() && causing_entity.is_some() {
        "minecraft:player_explosion"
    } else {
        "minecraft:explosion"
    };
    DamageSource {
        damage_type: required_builtin(id),
        direct_entity,
        causing_entity,
        source_position: None,
    }
}

pub fn bad_respawn_point_source(source_position: [f64; 3]) -> DamageSource {
    DamageSource::positioned(
        required_builtin("minecraft:bad_respawn_point"),
        source_position,
    )
}

fn required_builtin(id: &str) -> &'static DamageTypeDef {
    match builtin_damage_type(id) {
        Some(damage_type) => damage_type,
        None => panic!("built-in damage type is not registered: {id}"),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatapackDamageType {
    pub id: String,
    pub message_id: String,
    pub scaling: DamageScaling,
    pub exhaustion: f32,
    pub effects: DamageEffects,
    pub death_message_type: DeathMessageType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DamageTypeParseError {
    MissingField(&'static str),
    InvalidEnum(&'static str),
    InvalidFloat,
}

pub fn parse_datapack_damage_type(
    id: impl Into<String>,
    json: &str,
) -> Result<DatapackDamageType, DamageTypeParseError> {
    let message_id =
        json_string(json, "message_id").ok_or(DamageTypeParseError::MissingField("message_id"))?;
    let scaling = DamageScaling::parse(
        &json_string(json, "scaling").ok_or(DamageTypeParseError::MissingField("scaling"))?,
    )
    .ok_or(DamageTypeParseError::InvalidEnum("scaling"))?;
    let exhaustion =
        json_float(json, "exhaustion").ok_or(DamageTypeParseError::MissingField("exhaustion"))?;
    let effects = json_string(json, "effects")
        .map(|value| {
            DamageEffects::parse(&value).ok_or(DamageTypeParseError::InvalidEnum("effects"))
        })
        .transpose()?
        .unwrap_or(DamageEffects::Hurt);
    let death_message_type = json_string(json, "death_message_type")
        .map(|value| {
            DeathMessageType::parse(&value)
                .ok_or(DamageTypeParseError::InvalidEnum("death_message_type"))
        })
        .transpose()?
        .unwrap_or(DeathMessageType::Default);

    Ok(DatapackDamageType {
        id: normalize_identifier_owned(&id.into()),
        message_id,
        scaling,
        exhaustion,
        effects,
        death_message_type,
    })
}

pub fn death_message_translation_key(damage: &DamageTypeDef) -> String {
    format!("death.attack.{}", damage.message_id)
}

pub fn scaling_applies_to_difficulty(
    scaling: DamageScaling,
    caused_by_living: bool,
    caused_by_player: bool,
) -> bool {
    match scaling {
        DamageScaling::Never => false,
        DamageScaling::Always => true,
        DamageScaling::WhenCausedByLivingNonPlayer => caused_by_living && !caused_by_player,
    }
}

fn normalize_identifier(id: &str) -> String {
    normalize_identifier_owned(id)
}

fn normalize_identifier_owned(id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("minecraft:{id}")
    }
}

fn json_string(json: &str, key: &'static str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let after_key = json.split_once(&needle)?.1;
    let after_colon = after_key.split_once(':')?.1.trim_start();
    let value = after_colon.strip_prefix('"')?;
    let end = value.find('"')?;
    Some(value[..end].to_string())
}

fn json_float(json: &str, key: &'static str) -> Option<f32> {
    let needle = format!("\"{key}\"");
    let after_key = json.split_once(&needle)?.1;
    let after_colon = after_key.split_once(':')?.1.trim_start();
    let end = after_colon
        .find(|ch: char| !(ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+')))
        .unwrap_or(after_colon.len());
    after_colon[..end].parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_damage_type_registry_matches_decompiled_damage_types_surface() {
        assert_eq!(BUILTIN_DAMAGE_TYPES.len(), 50);
        assert_eq!(
            builtin_damage_type("generic").unwrap().message_id,
            "generic"
        );
        assert_eq!(
            builtin_damage_type("minecraft:fall")
                .unwrap()
                .death_message_type,
            DeathMessageType::FallVariants
        );
        assert_eq!(
            builtin_damage_type("minecraft:bad_respawn_point")
                .unwrap()
                .death_message_type,
            DeathMessageType::IntentionalGameDesign
        );
        assert_eq!(
            builtin_damage_type("minecraft:explosion").unwrap().scaling,
            DamageScaling::Always
        );
        assert_eq!(
            builtin_damage_type("minecraft:drown").unwrap().effects,
            DamageEffects::Drowning
        );
        assert_eq!(
            DamageEffects::Burning.hurt_sound(),
            "minecraft:entity.player.hurt_on_fire"
        );
    }

    #[test]
    fn datapack_damage_type_json_uses_same_defaults_as_vanilla_codec() {
        let parsed = parse_datapack_damage_type(
            "example:laser",
            r#"{
              "message_id": "laser",
              "scaling": "always",
              "exhaustion": 0.25
            }"#,
        )
        .unwrap();
        assert_eq!(parsed.id, "example:laser");
        assert_eq!(parsed.message_id, "laser");
        assert_eq!(parsed.scaling, DamageScaling::Always);
        assert_eq!(parsed.exhaustion, 0.25);
        assert_eq!(parsed.effects, DamageEffects::Hurt);
        assert_eq!(parsed.death_message_type, DeathMessageType::Default);

        let rich = parse_datapack_damage_type(
            "freezer",
            r#"{"message_id":"freeze","scaling":"never","exhaustion":0.0,"effects":"freezing","death_message_type":"fall_variants"}"#,
        )
        .unwrap();
        assert_eq!(rich.id, "minecraft:freezer");
        assert_eq!(rich.effects, DamageEffects::Freezing);
        assert_eq!(rich.death_message_type, DeathMessageType::FallVariants);
    }

    #[test]
    fn damage_type_scaling_and_translation_keys_follow_damage_source_contracts() {
        assert!(!scaling_applies_to_difficulty(
            DamageScaling::Never,
            true,
            false
        ));
        assert!(scaling_applies_to_difficulty(
            DamageScaling::Always,
            false,
            false
        ));
        assert!(scaling_applies_to_difficulty(
            DamageScaling::WhenCausedByLivingNonPlayer,
            true,
            false
        ));
        assert!(!scaling_applies_to_difficulty(
            DamageScaling::WhenCausedByLivingNonPlayer,
            true,
            true
        ));
        assert_eq!(
            death_message_translation_key(builtin_damage_type("player_attack").unwrap()),
            "death.attack.player"
        );
    }

    #[test]
    fn damage_source_factories_match_vanilla_direct_and_indirect_contracts() {
        let zombie = DamageEntityRef::living_mob(1);
        let player = DamageEntityRef::player(2, false);
        let creative = DamageEntityRef::player(3, true);
        let arrow = DamageEntityRef::non_living(4);
        let fireball = DamageEntityRef::non_living(5);

        let mob_attack = mob_attack_source(zombie);
        assert!(mob_attack.is_direct());
        assert!(mob_attack.scales_with_difficulty());
        assert_eq!(mob_attack.food_exhaustion(), 0.1);

        let player_attack = player_attack_source(player);
        assert!(player_attack.is_direct());
        assert!(!player_attack.scales_with_difficulty());

        let creative_attack = player_attack_source(creative);
        assert!(creative_attack.is_creative_player());

        let arrow_attack = arrow_source(arrow, Some(player));
        assert!(!arrow_attack.is_direct());
        assert_eq!(arrow_attack.msg_id(), "arrow");

        let unattributed = fireball_source(fireball, None);
        assert_eq!(
            unattributed.damage_type.id,
            "minecraft:unattributed_fireball"
        );
        assert!(unattributed.is_direct());

        let owned_fireball = fireball_source(fireball, Some(zombie));
        assert_eq!(owned_fireball.damage_type.id, "minecraft:fireball");
        assert!(!owned_fireball.is_direct());

        let explosion = explosion_source(None, None);
        assert_eq!(explosion.damage_type.id, "minecraft:explosion");
        assert!(explosion.scales_with_difficulty());

        let player_explosion = explosion_source(Some(fireball), Some(player));
        assert_eq!(
            player_explosion.damage_type.id,
            "minecraft:player_explosion"
        );

        let bad_respawn = bad_respawn_point_source([1.0, 64.0, -2.0]);
        assert_eq!(bad_respawn.source_position, Some([1.0, 64.0, -2.0]));
        assert_eq!(
            bad_respawn.damage_type.death_message_type,
            DeathMessageType::IntentionalGameDesign
        );
    }

    #[test]
    fn datapack_damage_type_rejects_missing_or_unknown_codec_fields() {
        assert_eq!(
            parse_datapack_damage_type("bad", r#"{"scaling":"always","exhaustion":0.1}"#),
            Err(DamageTypeParseError::MissingField("message_id"))
        );
        assert_eq!(
            parse_datapack_damage_type(
                "bad",
                r#"{"message_id":"bad","scaling":"sometimes","exhaustion":0.1}"#
            ),
            Err(DamageTypeParseError::InvalidEnum("scaling"))
        );
    }
}
