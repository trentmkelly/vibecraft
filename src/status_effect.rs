#![allow(dead_code)]

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEffectInstance {
    pub id: &'static str,
    pub duration: i32,
    pub amplifier: u8,
    pub ambient: bool,
    pub visible: bool,
    pub show_icon: bool,
    pub hidden: Option<Box<StatusEffectInstance>>,
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
}
