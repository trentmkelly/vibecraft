#![allow(dead_code)]

use crate::storage::nbt::Tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectCategory {
    Beneficial,
    Harmful,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeOp {
    AddValue,
    AddMultipliedTotal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectAttributeModifier {
    pub attribute: &'static str,
    pub id: &'static str,
    pub amount_per_level: f64,
    pub operation: AttributeOp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusEffectDef {
    pub id: &'static str,
    pub category: EffectCategory,
    pub color: u32,
    pub instantaneous: bool,
    pub particle: &'static str,
    pub blend_in: i32,
    pub blend_out: i32,
    pub blend_out_advance: i32,
    pub sound_on_added: Option<&'static str>,
    pub attribute: Option<EffectAttributeModifier>,
}

const fn effect(id: &'static str, category: EffectCategory, color: u32) -> StatusEffectDef {
    StatusEffectDef {
        id,
        category,
        color,
        instantaneous: false,
        particle: "minecraft:entity_effect",
        blend_in: 0,
        blend_out: 0,
        blend_out_advance: 0,
        sound_on_added: None,
        attribute: None,
    }
}

const fn instant(id: &'static str, category: EffectCategory, color: u32) -> StatusEffectDef {
    StatusEffectDef {
        instantaneous: true,
        ..effect(id, category, color)
    }
}

const fn attr(
    mut base: StatusEffectDef,
    attribute: &'static str,
    id: &'static str,
    amount_per_level: f64,
    operation: AttributeOp,
) -> StatusEffectDef {
    base.attribute = Some(EffectAttributeModifier {
        attribute,
        id,
        amount_per_level,
        operation,
    });
    base
}

const fn blend(
    mut base: StatusEffectDef,
    in_ticks: i32,
    out_ticks: i32,
    advance_ticks: i32,
) -> StatusEffectDef {
    base.blend_in = in_ticks;
    base.blend_out = out_ticks;
    base.blend_out_advance = advance_ticks;
    base
}

const fn particle(mut base: StatusEffectDef, particle: &'static str) -> StatusEffectDef {
    base.particle = particle;
    base
}

const fn sound(mut base: StatusEffectDef, event: &'static str) -> StatusEffectDef {
    base.sound_on_added = Some(event);
    base
}

// Source: decompiled-server-26.1.2/net/minecraft/world/effect/MobEffects.java
pub const STATUS_EFFECTS: &[StatusEffectDef] = &[
    attr(
        effect("minecraft:speed", EffectCategory::Beneficial, 3402751),
        "minecraft:movement_speed",
        "minecraft:effect.speed",
        0.2,
        AttributeOp::AddMultipliedTotal,
    ),
    attr(
        effect("minecraft:slowness", EffectCategory::Harmful, 9154528),
        "minecraft:movement_speed",
        "minecraft:effect.slowness",
        -0.15,
        AttributeOp::AddMultipliedTotal,
    ),
    attr(
        effect("minecraft:haste", EffectCategory::Beneficial, 14270531),
        "minecraft:attack_speed",
        "minecraft:effect.haste",
        0.1,
        AttributeOp::AddMultipliedTotal,
    ),
    attr(
        effect("minecraft:mining_fatigue", EffectCategory::Harmful, 4866583),
        "minecraft:attack_speed",
        "minecraft:effect.mining_fatigue",
        -0.1,
        AttributeOp::AddMultipliedTotal,
    ),
    attr(
        effect("minecraft:strength", EffectCategory::Beneficial, 16762624),
        "minecraft:attack_damage",
        "minecraft:effect.strength",
        3.0,
        AttributeOp::AddValue,
    ),
    instant(
        "minecraft:instant_health",
        EffectCategory::Beneficial,
        16262179,
    ),
    instant(
        "minecraft:instant_damage",
        EffectCategory::Harmful,
        11101546,
    ),
    attr(
        effect("minecraft:jump_boost", EffectCategory::Beneficial, 16646020),
        "minecraft:safe_fall_distance",
        "minecraft:effect.jump_boost",
        1.0,
        AttributeOp::AddValue,
    ),
    blend(
        effect("minecraft:nausea", EffectCategory::Harmful, 5578058),
        150,
        20,
        60,
    ),
    effect(
        "minecraft:regeneration",
        EffectCategory::Beneficial,
        13458603,
    ),
    effect("minecraft:resistance", EffectCategory::Beneficial, 9520880),
    effect(
        "minecraft:fire_resistance",
        EffectCategory::Beneficial,
        16750848,
    ),
    effect(
        "minecraft:water_breathing",
        EffectCategory::Beneficial,
        10017472,
    ),
    attr(
        effect(
            "minecraft:invisibility",
            EffectCategory::Beneficial,
            16185078,
        ),
        "minecraft:waypoint_transmit_range",
        "minecraft:effect.waypoint_transmit_range_hide",
        -1.0,
        AttributeOp::AddMultipliedTotal,
    ),
    effect("minecraft:blindness", EffectCategory::Harmful, 2039587),
    effect(
        "minecraft:night_vision",
        EffectCategory::Beneficial,
        12779366,
    ),
    effect("minecraft:hunger", EffectCategory::Harmful, 5797459),
    attr(
        effect("minecraft:weakness", EffectCategory::Harmful, 4738376),
        "minecraft:attack_damage",
        "minecraft:effect.weakness",
        -4.0,
        AttributeOp::AddValue,
    ),
    effect("minecraft:poison", EffectCategory::Harmful, 8889187),
    effect("minecraft:wither", EffectCategory::Harmful, 7561558),
    attr(
        effect(
            "minecraft:health_boost",
            EffectCategory::Beneficial,
            16284963,
        ),
        "minecraft:max_health",
        "minecraft:effect.health_boost",
        4.0,
        AttributeOp::AddValue,
    ),
    attr(
        effect("minecraft:absorption", EffectCategory::Beneficial, 2445989),
        "minecraft:max_absorption",
        "minecraft:effect.absorption",
        4.0,
        AttributeOp::AddValue,
    ),
    instant("minecraft:saturation", EffectCategory::Beneficial, 16262179),
    effect("minecraft:glowing", EffectCategory::Neutral, 9740385),
    effect("minecraft:levitation", EffectCategory::Harmful, 13565951),
    attr(
        effect("minecraft:luck", EffectCategory::Beneficial, 5882118),
        "minecraft:luck",
        "minecraft:effect.luck",
        1.0,
        AttributeOp::AddValue,
    ),
    attr(
        effect("minecraft:unluck", EffectCategory::Harmful, 12624973),
        "minecraft:luck",
        "minecraft:effect.unluck",
        -1.0,
        AttributeOp::AddValue,
    ),
    effect(
        "minecraft:slow_falling",
        EffectCategory::Beneficial,
        15978425,
    ),
    effect(
        "minecraft:conduit_power",
        EffectCategory::Beneficial,
        1950417,
    ),
    effect(
        "minecraft:dolphins_grace",
        EffectCategory::Beneficial,
        8954814,
    ),
    sound(
        effect("minecraft:bad_omen", EffectCategory::Neutral, 745784),
        "minecraft:event.mob_effect.bad_omen",
    ),
    effect(
        "minecraft:hero_of_the_village",
        EffectCategory::Beneficial,
        4521796,
    ),
    blend(
        effect("minecraft:darkness", EffectCategory::Harmful, 2696993),
        22,
        22,
        22,
    ),
    sound(
        particle(
            effect("minecraft:trial_omen", EffectCategory::Neutral, 1484454),
            "minecraft:trial_omen",
        ),
        "minecraft:event.mob_effect.trial_omen",
    ),
    sound(
        particle(
            effect("minecraft:raid_omen", EffectCategory::Neutral, 14565464),
            "minecraft:raid_omen",
        ),
        "minecraft:event.mob_effect.raid_omen",
    ),
    effect("minecraft:wind_charged", EffectCategory::Harmful, 12438015),
    effect("minecraft:weaving", EffectCategory::Harmful, 7891290),
    effect("minecraft:oozing", EffectCategory::Harmful, 10092451),
    effect("minecraft:infested", EffectCategory::Harmful, 9214860),
    effect(
        "minecraft:breath_of_the_nautilus",
        EffectCategory::Beneficial,
        65518,
    ),
];

pub fn status_effect(id: &str) -> Option<&'static StatusEffectDef> {
    let id = if id.contains(':') {
        id.to_string()
    } else {
        format!("minecraft:{id}")
    };
    STATUS_EFFECTS.iter().find(|effect| effect.id == id)
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusEffectInstance {
    pub id: &'static str,
    pub duration: i32,
    pub amplifier: u8,
    pub ambient: bool,
    pub visible: bool,
    pub show_icon: bool,
    pub hidden: Option<Box<StatusEffectInstance>>,
    pub factor_calculation_data: Option<Tag>,
}

impl StatusEffectInstance {
    pub fn new(id: &'static str, duration: i32, amplifier: u8) -> Self {
        Self {
            id,
            duration,
            amplifier,
            ambient: false,
            visible: true,
            show_icon: true,
            hidden: None,
            factor_calculation_data: None,
        }
    }

    pub fn is_infinite(&self) -> bool {
        self.duration == -1
    }

    pub fn update(&mut self, incoming: StatusEffectInstance) -> bool {
        let mut changed = false;
        let incoming_ambient = incoming.ambient;
        let incoming_visible = incoming.visible;
        let incoming_show_icon = incoming.show_icon;
        if incoming.amplifier > self.amplifier {
            if incoming.is_shorter_than(self) {
                self.hidden = Some(Box::new(self.clone()));
            }
            self.amplifier = incoming.amplifier;
            self.duration = incoming.duration;
            changed = true;
        } else if incoming.is_longer_than(self) {
            if incoming.amplifier == self.amplifier {
                self.duration = incoming.duration;
                changed = true;
            } else if let Some(hidden) = &mut self.hidden {
                hidden.update(incoming);
            } else {
                self.hidden = Some(Box::new(incoming));
            }
        }

        if (!incoming_ambient && self.ambient) || changed {
            self.ambient = incoming_ambient;
            changed = true;
        }
        if incoming_visible != self.visible {
            self.visible = incoming_visible;
            changed = true;
        }
        if incoming_show_icon != self.show_icon {
            self.show_icon = incoming_show_icon;
            changed = true;
        }
        changed
    }

    pub fn tick(&mut self, health: f32, max_health: f32, inverted_heal_harm: bool) -> EffectTick {
        let tick_count = if self.is_infinite() { 0 } else { self.duration };
        let action = tick_action(
            self.id,
            tick_count,
            self.amplifier,
            health,
            max_health,
            inverted_heal_harm,
        );
        if let Some(hidden) = &mut self.hidden {
            hidden.tick_down_only();
        }
        if !self.is_infinite() && self.duration > 0 {
            self.duration -= 1;
        }
        if self.duration == 0 {
            if let Some(hidden) = self.hidden.take() {
                *self = *hidden;
                return EffectTick {
                    action,
                    expired: false,
                    revealed_hidden: true,
                };
            }
        }
        EffectTick {
            action,
            expired: !self.is_infinite() && self.duration <= 0,
            revealed_hidden: false,
        }
    }

    fn is_longer_than(&self, other: &StatusEffectInstance) -> bool {
        !other.is_infinite() && (self.duration > other.duration || self.is_infinite())
    }

    fn is_shorter_than(&self, other: &StatusEffectInstance) -> bool {
        !self.is_infinite() && (self.duration < other.duration || other.is_infinite())
    }

    fn tick_down_only(&mut self) {
        if let Some(hidden) = &mut self.hidden {
            hidden.tick_down_only();
        }
        if !self.is_infinite() && self.duration > 0 {
            self.duration -= 1;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectAction {
    Heal(f32),
    Damage { source: &'static str, amount: f32 },
    ExhaustFood(f32),
    Saturate { food: i32, saturation: f32 },
    SetAbsorptionAtLeast(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementEffect {
    pub jump_boost: f32,
    pub levitation_velocity: f32,
    pub slow_falling: bool,
}

pub fn break_speed_multiplier(id: &str, amplifier: u8) -> Option<f32> {
    let level = i32::from(amplifier) + 1;
    match id {
        "minecraft:haste" => Some(1.0 + 0.2 * level as f32),
        "minecraft:mining_fatigue" => Some(match amplifier {
            0 => 0.3,
            1 => 0.09,
            2 => 0.0027,
            _ => 0.00081,
        }),
        _ => None,
    }
}

pub fn movement_effect(id: &str, amplifier: u8) -> Option<MovementEffect> {
    match id {
        "minecraft:jump_boost" => Some(MovementEffect {
            jump_boost: 0.1 * (f32::from(amplifier) + 1.0),
            levitation_velocity: 0.0,
            slow_falling: false,
        }),
        "minecraft:levitation" => Some(MovementEffect {
            jump_boost: 0.0,
            levitation_velocity: 0.05 * (f32::from(amplifier) + 1.0),
            slow_falling: false,
        }),
        "minecraft:slow_falling" => Some(MovementEffect {
            jump_boost: 0.0,
            levitation_velocity: 0.0,
            slow_falling: true,
        }),
        _ => None,
    }
}

pub fn resistance_damage_multiplier(amplifier: u8) -> f32 {
    (1.0 - 0.2 * (f32::from(amplifier) + 1.0)).max(0.0)
}

pub fn prevents_fire_damage(effect_id: &str) -> bool {
    effect_id == "minecraft:fire_resistance"
}

pub fn prevents_drowning(effect_id: &str) -> bool {
    matches!(
        effect_id,
        "minecraft:water_breathing" | "minecraft:conduit_power"
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientVisualEffect {
    NightVision,
    Blindness { prevents_sprinting: bool },
    Nausea,
    Glowing,
    Darkness,
}

pub fn client_visual_effect(effect_id: &str) -> Option<ClientVisualEffect> {
    match effect_id {
        "minecraft:night_vision" => Some(ClientVisualEffect::NightVision),
        "minecraft:blindness" => Some(ClientVisualEffect::Blindness {
            prevents_sprinting: true,
        }),
        "minecraft:nausea" => Some(ClientVisualEffect::Nausea),
        "minecraft:glowing" => Some(ClientVisualEffect::Glowing),
        "minecraft:darkness" => Some(ClientVisualEffect::Darkness),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeathEffectAction {
    SpawnSilverfish { chance: f32 },
    SpawnSlimes { count: i32 },
    PlaceCobweb,
    WindBurstExplosion { radius: f32 },
}

pub fn death_or_hit_effect_action(effect_id: &str, amplifier: u8) -> Option<DeathEffectAction> {
    match effect_id {
        "minecraft:infested" => Some(DeathEffectAction::SpawnSilverfish {
            chance: 0.1 * (f32::from(amplifier) + 1.0),
        }),
        "minecraft:oozing" => Some(DeathEffectAction::SpawnSlimes {
            count: 2 + i32::from(amplifier),
        }),
        "minecraft:weaving" => Some(DeathEffectAction::PlaceCobweb),
        "minecraft:wind_charged" => Some(DeathEffectAction::WindBurstExplosion { radius: 3.0 }),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConduitPowerEffect {
    pub underwater_break_speed_multiplier: f32,
    pub prevents_drowning: bool,
    pub grants_night_vision: bool,
    pub attack_damage: f32,
}

pub fn conduit_power_effect(active: bool, hostile_nearby: bool) -> Option<ConduitPowerEffect> {
    if !active {
        return None;
    }
    Some(ConduitPowerEffect {
        underwater_break_speed_multiplier: 1.2,
        prevents_drowning: true,
        grants_night_vision: true,
        attack_damage: if hostile_nearby { 4.0 } else { 0.0 },
    })
}

pub fn dolphins_grace_swim_multiplier(amplifier: u8) -> f32 {
    1.0 + 0.96 * (f32::from(amplifier) + 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectTick {
    pub action: Option<EffectAction>,
    pub expired: bool,
    pub revealed_hidden: bool,
}

pub fn tick_action(
    id: &str,
    tick_count: i32,
    amplifier: u8,
    health: f32,
    max_health: f32,
    inverted_heal_harm: bool,
) -> Option<EffectAction> {
    let level = i32::from(amplifier);
    match id {
        "minecraft:regeneration"
            if should_tick_interval(tick_count, 50, level) && health < max_health =>
        {
            Some(EffectAction::Heal(1.0))
        }
        "minecraft:poison" if should_tick_interval(tick_count, 25, level) && health > 1.0 => {
            Some(EffectAction::Damage {
                source: "minecraft:magic",
                amount: 1.0,
            })
        }
        "minecraft:wither" if should_tick_interval(tick_count, 40, level) => {
            Some(EffectAction::Damage {
                source: "minecraft:wither",
                amount: 1.0,
            })
        }
        "minecraft:hunger" => Some(EffectAction::ExhaustFood(0.005 * f32::from(amplifier + 1))),
        "minecraft:absorption" => Some(EffectAction::SetAbsorptionAtLeast(
            4.0 * f32::from(amplifier + 1),
        )),
        "minecraft:saturation" => Some(EffectAction::Saturate {
            food: i32::from(amplifier) + 1,
            saturation: 1.0,
        }),
        "minecraft:instant_health" => {
            let amount = (4_i32 << level) as f32;
            if inverted_heal_harm {
                Some(EffectAction::Damage {
                    source: "minecraft:magic",
                    amount: (6_i32 << level) as f32,
                })
            } else {
                Some(EffectAction::Heal(amount))
            }
        }
        "minecraft:instant_damage" => {
            if inverted_heal_harm {
                Some(EffectAction::Heal((4_i32 << level) as f32))
            } else {
                Some(EffectAction::Damage {
                    source: "minecraft:magic",
                    amount: (6_i32 << level) as f32,
                })
            }
        }
        _ => None,
    }
}

/// 1:1 port of `HealOrHarmMobEffect.applyInstantenousEffect` for the
/// `instant_health`/`instant_damage` effects — the path taken when a potion is
/// consumed/splashed (as opposed to [`tick_action`], which mirrors
/// `applyEffectTick`). `scale` is the potion application scale (e.g. splash
/// distance falloff), and `has_source` switches harm from `magic` to
/// `indirect_magic` when a causing entity is present. Heal amounts use
/// `4 << amplifier` and harm amounts `6 << amplifier`, each scaled and rounded
/// as `(int)(scale * base + 0.5)`. Roles swap for undead (`inverted_heal_harm`).
pub fn apply_instantaneous_effect(
    id: &str,
    amplifier: u8,
    scale: f64,
    inverted_heal_harm: bool,
    has_source: bool,
) -> Option<EffectAction> {
    let level = i32::from(amplifier);
    let heal = || EffectAction::Heal(scaled_instant_amount(scale, 4_i32 << level));
    let harm = || EffectAction::Damage {
        source: if has_source {
            "minecraft:indirect_magic"
        } else {
            "minecraft:magic"
        },
        amount: scaled_instant_amount(scale, 6_i32 << level),
    };
    match id {
        // isHarm == false: heal a normal mob, harm an undead one.
        "minecraft:instant_health" => Some(if inverted_heal_harm { harm() } else { heal() }),
        // isHarm == true: harm a normal mob, heal an undead one.
        "minecraft:instant_damage" => Some(if inverted_heal_harm { heal() } else { harm() }),
        _ => None,
    }
}

/// `(int)(scale * base + 0.5)` from `applyInstantenousEffect`, as health points.
/// Java's `(int)` cast truncates toward zero, matching Rust's `as i32`.
fn scaled_instant_amount(scale: f64, base: i32) -> f32 {
    (scale * f64::from(base) + 0.5) as i32 as f32
}

fn should_tick_interval(tick_count: i32, base_interval: i32, amplifier: i32) -> bool {
    let interval = base_interval >> amplifier;
    interval <= 0 || tick_count % interval == 0
}

pub fn particle_alpha(ambient: bool) -> u8 {
    if ambient {
        38
    } else {
        255
    }
}

pub fn serialization_flags(ambient: bool, visible: bool, show_icon: bool) -> u8 {
    u8::from(ambient) | (u8::from(visible) << 1) | (u8::from(show_icon) << 2)
}

/// NBT-serializable representation of a status effect instance for playerdata.
///
/// Source: `MobEffectInstance.save(DataOutput)` / `MobEffectInstance.load(DataInput)`
/// from `decompiled-server-26.1.2/net/minecraft/world/effect/MobEffectInstance.java`
#[derive(Debug, Clone, PartialEq)]
pub struct StatusEffectNbt {
    pub id: String,
    pub amplifier: u8,
    pub duration: i32,
    pub ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
    pub hidden_effect: Option<Box<StatusEffectNbt>>,
    pub factor_calculation_data: Option<Tag>,
}

impl StatusEffectNbt {
    /// Serialize a `StatusEffectInstance` into its NBT form.
    pub fn from_instance(instance: &StatusEffectInstance) -> Self {
        Self {
            id: instance.id.to_string(),
            amplifier: instance.amplifier,
            duration: instance.duration,
            ambient: instance.ambient,
            show_particles: instance.visible,
            show_icon: instance.show_icon,
            hidden_effect: instance
                .hidden
                .as_ref()
                .map(|h| Box::new(Self::from_instance(h))),
            factor_calculation_data: instance.factor_calculation_data.clone(),
        }
    }

    /// Deserialize back into a `StatusEffectInstance`.
    ///
    /// Returns `None` if the effect ID is not recognized in the registry.
    pub fn to_instance(&self) -> Option<StatusEffectInstance> {
        // Look up the effect id in the static registry to get the &'static str
        let def = status_effect(&self.id)?;
        let mut instance = StatusEffectInstance {
            id: def.id,
            duration: self.duration,
            amplifier: self.amplifier,
            ambient: self.ambient,
            visible: self.show_particles,
            show_icon: self.show_icon,
            hidden: None,
            factor_calculation_data: self.factor_calculation_data.clone(),
        };
        if let Some(hidden_nbt) = &self.hidden_effect {
            instance.hidden = hidden_nbt.to_instance().map(Box::new);
        }
        Some(instance)
    }
}

/// Serialize a list of active effects to their NBT forms.
pub fn serialize_active_effects(instances: &[StatusEffectInstance]) -> Vec<StatusEffectNbt> {
    instances
        .iter()
        .map(StatusEffectNbt::from_instance)
        .collect()
}

/// Deserialize active effects from NBT, skipping unknown effect IDs.
pub fn deserialize_active_effects(nbt_list: &[StatusEffectNbt]) -> Vec<StatusEffectInstance> {
    nbt_list
        .iter()
        .filter_map(|nbt| nbt.to_instance())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_registry_matches_26_1_2_catalog_and_representative_metadata() {
        assert_eq!(STATUS_EFFECTS.len(), 40);
        assert_eq!(STATUS_EFFECTS[0].id, "minecraft:speed");
        assert_eq!(
            STATUS_EFFECTS.last().unwrap().id,
            "minecraft:breath_of_the_nautilus"
        );
        assert_eq!(
            status_effect("poison").unwrap().category,
            EffectCategory::Harmful
        );
        assert!(
            status_effect("minecraft:instant_health")
                .unwrap()
                .instantaneous
        );
        assert_eq!(status_effect("minecraft:darkness").unwrap().blend_in, 22);
        assert_eq!(
            status_effect("minecraft:trial_omen").unwrap().particle,
            "minecraft:trial_omen"
        );
    }

    #[test]
    fn attribute_modifiers_scale_with_amplifier_like_vanilla_templates() {
        let strength = status_effect("strength").unwrap().attribute.unwrap();
        assert_eq!(strength.attribute, "minecraft:attack_damage");
        assert_eq!(strength.amount_per_level * 2.0, 6.0);
        assert_eq!(strength.operation, AttributeOp::AddValue);

        let speed = status_effect("speed").unwrap().attribute.unwrap();
        assert_eq!(speed.operation, AttributeOp::AddMultipliedTotal);
        assert_eq!(speed.amount_per_level * 3.0, 0.6000000000000001);
    }

    #[test]
    fn ticking_effects_apply_vanilla_intervals_and_instant_effects_respect_inversion() {
        assert_eq!(
            tick_action("minecraft:regeneration", 50, 0, 10.0, 20.0, false),
            Some(EffectAction::Heal(1.0))
        );
        assert_eq!(
            tick_action("minecraft:regeneration", 49, 0, 10.0, 20.0, false),
            None
        );
        assert_eq!(
            tick_action("minecraft:poison", 25, 0, 2.0, 20.0, false),
            Some(EffectAction::Damage {
                source: "minecraft:magic",
                amount: 1.0
            })
        );
        assert_eq!(
            tick_action("minecraft:poison", 25, 0, 1.0, 20.0, false),
            None
        );
        assert_eq!(
            tick_action("minecraft:wither", 40, 0, 1.0, 20.0, false),
            Some(EffectAction::Damage {
                source: "minecraft:wither",
                amount: 1.0
            })
        );
        assert_eq!(
            tick_action("minecraft:hunger", 1, 1, 20.0, 20.0, false),
            Some(EffectAction::ExhaustFood(0.01))
        );
        assert_eq!(
            tick_action("minecraft:instant_damage", 1, 1, 20.0, 20.0, false),
            Some(EffectAction::Damage {
                source: "minecraft:magic",
                amount: 12.0
            })
        );
        assert_eq!(
            tick_action("minecraft:instant_damage", 1, 1, 20.0, 20.0, true),
            Some(EffectAction::Heal(8.0))
        );
    }

    #[test]
    fn instant_health_and_damage_applied_effect_scale_and_inversion_match_vanilla() {
        // Drinking an Instant Health potion (scale 1.0, amplifier 0) heals 4.
        assert_eq!(
            apply_instantaneous_effect("minecraft:instant_health", 0, 1.0, false, false),
            Some(EffectAction::Heal(4.0))
        );
        // Amplifier 1 doubles the base: 4 << 1 = 8.
        assert_eq!(
            apply_instantaneous_effect("minecraft:instant_health", 1, 1.0, false, false),
            Some(EffectAction::Heal(8.0))
        );
        // Splash falloff scale rounds as (int)(scale * base + 0.5):
        // 0.5 * 4 + 0.5 = 2.5 -> 2.
        assert_eq!(
            apply_instantaneous_effect("minecraft:instant_health", 0, 0.5, false, false),
            Some(EffectAction::Heal(2.0))
        );

        // Instant Damage on a normal mob with no source → magic, 6 << amp.
        assert_eq!(
            apply_instantaneous_effect("minecraft:instant_damage", 0, 1.0, false, false),
            Some(EffectAction::Damage {
                source: "minecraft:magic",
                amount: 6.0
            })
        );
        // With a causing entity present → indirect_magic.
        assert_eq!(
            apply_instantaneous_effect("minecraft:instant_damage", 1, 1.0, false, true),
            Some(EffectAction::Damage {
                source: "minecraft:indirect_magic",
                amount: 12.0
            })
        );

        // Undead invert the roles: Instant Health harms them, Instant Damage
        // heals them.
        assert_eq!(
            apply_instantaneous_effect("minecraft:instant_health", 0, 1.0, true, false),
            Some(EffectAction::Damage {
                source: "minecraft:magic",
                amount: 6.0
            })
        );
        assert_eq!(
            apply_instantaneous_effect("minecraft:instant_damage", 0, 1.0, true, false),
            Some(EffectAction::Heal(4.0))
        );

        // Non-instant effects are not handled by this path.
        assert_eq!(
            apply_instantaneous_effect("minecraft:regeneration", 0, 1.0, false, false),
            None
        );
    }

    #[test]
    fn non_damage_status_effect_behavior_helpers_cover_vanilla_tick_surfaces() {
        assert_eq!(break_speed_multiplier("minecraft:haste", 0), Some(1.2));
        assert_eq!(break_speed_multiplier("minecraft:haste", 2), Some(1.6));
        assert_eq!(
            break_speed_multiplier("minecraft:mining_fatigue", 0),
            Some(0.3)
        );
        assert_eq!(
            break_speed_multiplier("minecraft:mining_fatigue", 3),
            Some(0.00081)
        );

        assert_eq!(
            movement_effect("minecraft:jump_boost", 1),
            Some(MovementEffect {
                jump_boost: 0.2,
                levitation_velocity: 0.0,
                slow_falling: false,
            })
        );
        assert_eq!(
            movement_effect("minecraft:levitation", 1),
            Some(MovementEffect {
                jump_boost: 0.0,
                levitation_velocity: 0.1,
                slow_falling: false,
            })
        );
        assert_eq!(
            movement_effect("minecraft:slow_falling", 0),
            Some(MovementEffect {
                jump_boost: 0.0,
                levitation_velocity: 0.0,
                slow_falling: true,
            })
        );

        assert_eq!(resistance_damage_multiplier(0), 0.8);
        assert_eq!(resistance_damage_multiplier(4), 0.0);
        assert!(prevents_fire_damage("minecraft:fire_resistance"));
        assert!(prevents_drowning("minecraft:water_breathing"));
        assert!(prevents_drowning("minecraft:conduit_power"));
    }

    #[test]
    fn visual_and_death_status_effect_helpers_cover_client_and_spawn_hooks() {
        assert_eq!(
            client_visual_effect("minecraft:blindness"),
            Some(ClientVisualEffect::Blindness {
                prevents_sprinting: true
            })
        );
        assert_eq!(
            client_visual_effect("minecraft:night_vision"),
            Some(ClientVisualEffect::NightVision)
        );
        assert_eq!(
            client_visual_effect("minecraft:nausea"),
            Some(ClientVisualEffect::Nausea)
        );
        assert_eq!(
            client_visual_effect("minecraft:glowing"),
            Some(ClientVisualEffect::Glowing)
        );
        assert_eq!(
            client_visual_effect("minecraft:darkness"),
            Some(ClientVisualEffect::Darkness)
        );

        assert_eq!(
            death_or_hit_effect_action("minecraft:infested", 1),
            Some(DeathEffectAction::SpawnSilverfish { chance: 0.2 })
        );
        assert_eq!(
            death_or_hit_effect_action("minecraft:oozing", 2),
            Some(DeathEffectAction::SpawnSlimes { count: 4 })
        );
        assert_eq!(
            death_or_hit_effect_action("minecraft:weaving", 0),
            Some(DeathEffectAction::PlaceCobweb)
        );
        assert_eq!(
            death_or_hit_effect_action("minecraft:wind_charged", 0),
            Some(DeathEffectAction::WindBurstExplosion { radius: 3.0 })
        );
    }

    #[test]
    fn conduit_power_and_dolphins_grace_expose_underwater_effects() {
        assert_eq!(
            conduit_power_effect(true, true),
            Some(ConduitPowerEffect {
                underwater_break_speed_multiplier: 1.2,
                prevents_drowning: true,
                grants_night_vision: true,
                attack_damage: 4.0,
            })
        );
        assert_eq!(conduit_power_effect(false, true), None);
        assert_eq!(dolphins_grace_swim_multiplier(0), 1.96);
        assert_eq!(dolphins_grace_swim_multiplier(1), 2.92);
    }

    #[test]
    fn instance_update_duration_hidden_effect_and_expiration_match_vanilla_shape() {
        let mut current = StatusEffectInstance::new("minecraft:speed", 100, 0);
        let stronger_short = StatusEffectInstance::new("minecraft:speed", 20, 1);
        assert!(current.update(stronger_short));
        assert_eq!(current.amplifier, 1);
        assert_eq!(current.hidden.as_ref().unwrap().amplifier, 0);

        current.duration = 1;
        let tick = current.tick(20.0, 20.0, false);
        assert!(tick.revealed_hidden);
        assert!(!tick.expired);
        assert_eq!(current.amplifier, 0);
        assert_eq!(current.duration, 99);

        current.duration = 1;
        let expired = current.tick(20.0, 20.0, false);
        assert!(expired.expired);
    }

    #[test]
    fn particles_icons_flags_and_serialization_are_visible_to_clients() {
        assert_eq!(particle_alpha(false), 255);
        assert_eq!(particle_alpha(true), 38);
        assert_eq!(serialization_flags(false, true, true), 0b110);
        assert_eq!(serialization_flags(true, false, true), 0b101);
    }

    #[test]
    fn effect_nbt_serialization_round_trips_all_fields_including_hidden() {
        let mut original = StatusEffectInstance::new("minecraft:speed", 200, 1);
        original.ambient = true;
        original.visible = false;
        original.show_icon = false;
        original.factor_calculation_data = Some(Tag::Compound(vec![(
            "ticks_active".to_string(),
            Tag::Int(42),
        )]));
        original.hidden = Some(Box::new(StatusEffectInstance::new(
            "minecraft:speed",
            400,
            0,
        )));

        let nbt = StatusEffectNbt::from_instance(&original);
        assert_eq!(nbt.id, "minecraft:speed");
        assert_eq!(nbt.amplifier, 1);
        assert_eq!(nbt.duration, 200);
        assert!(nbt.ambient);
        assert!(!nbt.show_particles);
        assert!(!nbt.show_icon);
        assert!(nbt.hidden_effect.is_some());
        assert_eq!(
            nbt.factor_calculation_data,
            Some(Tag::Compound(vec![(
                "ticks_active".to_string(),
                Tag::Int(42),
            )]))
        );
        let hidden = nbt.hidden_effect.as_ref().unwrap();
        assert_eq!(hidden.amplifier, 0);
        assert_eq!(hidden.duration, 400);

        let restored = nbt.to_instance().unwrap();
        assert_eq!(restored.id, "minecraft:speed");
        assert_eq!(restored.amplifier, 1);
        assert_eq!(restored.duration, 200);
        assert!(restored.ambient);
        assert!(!restored.visible);
        assert!(!restored.show_icon);
        assert_eq!(
            restored.factor_calculation_data,
            Some(Tag::Compound(vec![(
                "ticks_active".to_string(),
                Tag::Int(42),
            )]))
        );
        let restored_hidden = restored.hidden.as_ref().unwrap();
        assert_eq!(restored_hidden.amplifier, 0);
        assert_eq!(restored_hidden.duration, 400);
    }

    #[test]
    fn serialize_and_deserialize_active_effects_list_skips_unknown_ids() {
        let effects = vec![
            StatusEffectInstance::new("minecraft:speed", 100, 0),
            StatusEffectInstance::new("minecraft:poison", 200, 1),
        ];
        let nbt_list = serialize_active_effects(&effects);
        assert_eq!(nbt_list.len(), 2);

        let restored = deserialize_active_effects(&nbt_list);
        assert_eq!(restored.len(), 2);
        assert_eq!(restored[0].id, "minecraft:speed");
        assert_eq!(restored[1].id, "minecraft:poison");

        // Unknown ID is skipped
        let unknown = StatusEffectNbt {
            id: "unknown:mystery".to_string(),
            amplifier: 0,
            duration: 100,
            ambient: false,
            show_particles: true,
            show_icon: true,
            hidden_effect: None,
            factor_calculation_data: None,
        };
        let partial = deserialize_active_effects(&[unknown]);
        assert!(partial.is_empty());
    }

    #[test]
    fn regeneration_tick_interval_per_amplifier_matches_vanilla() {
        // Regen I (amplifier=0): interval = 50 >> 0 = 50 ticks
        // tick_action at tick 50 → fires, at tick 49 → doesn't
        assert_eq!(
            tick_action("minecraft:regeneration", 50, 0, 10.0, 20.0, false),
            Some(EffectAction::Heal(1.0))
        );
        assert_eq!(
            tick_action("minecraft:regeneration", 49, 0, 10.0, 20.0, false),
            None
        );

        // Regen II (amplifier=1): interval = 50 >> 1 = 25 ticks
        assert_eq!(
            tick_action("minecraft:regeneration", 25, 1, 10.0, 20.0, false),
            Some(EffectAction::Heal(1.0))
        );
        assert_eq!(
            tick_action("minecraft:regeneration", 24, 1, 10.0, 20.0, false),
            None
        );

        // Regen V (amplifier=4): interval = 50 >> 4 = 3 ticks
        assert_eq!(
            tick_action("minecraft:regeneration", 3, 4, 10.0, 20.0, false),
            Some(EffectAction::Heal(1.0))
        );
        assert_eq!(
            tick_action("minecraft:regeneration", 2, 4, 10.0, 20.0, false),
            None
        );

        // Healing at full health → None
        assert_eq!(
            tick_action("minecraft:regeneration", 50, 0, 20.0, 20.0, false),
            None
        );
    }
}
