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

pub fn find_damage_type(id: &str) -> Option<&'static DamageTypeDef> {
    DAMAGE_TYPES.iter().find(|damage| damage.id == id)
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
