#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatDamageKind {
    Melee,
    Projectile,
    Explosion,
    Fall,
    Drowning,
    Fire,
    Freezing,
    Void,
    Suffocation,
    Cactus,
    SweetBerryBush,
    Dripstone,
    WorldBorder,
    Magic,
    Starvation,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageMitigation {
    pub armor: f32,
    pub armor_toughness: f32,
    pub enchantment_protection: f32,
    pub absorption: f32,
    pub shield_blocking: bool,
    pub bypasses_armor: bool,
    pub bypasses_enchantments: bool,
    pub bypasses_shield: bool,
}

impl Default for DamageMitigation {
    fn default() -> Self {
        Self {
            armor: 0.0,
            armor_toughness: 0.0,
            enchantment_protection: 0.0,
            absorption: 0.0,
            shield_blocking: false,
            bypasses_armor: false,
            bypasses_enchantments: false,
            bypasses_shield: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageOutcome {
    pub incoming: f32,
    pub after_shield: f32,
    pub after_armor: f32,
    pub after_enchantments: f32,
    pub absorbed: f32,
    pub health_damage: f32,
    pub remaining_absorption: f32,
}

pub fn apply_damage_pipeline(incoming: f32, mitigation: DamageMitigation) -> DamageOutcome {
    let incoming = incoming.max(0.0);
    let after_shield = if mitigation.shield_blocking && !mitigation.bypasses_shield {
        0.0
    } else {
        incoming
    };
    let after_armor = if mitigation.bypasses_armor {
        after_shield
    } else {
        damage_after_armor(after_shield, mitigation.armor, mitigation.armor_toughness)
    };
    let after_enchantments = if mitigation.bypasses_enchantments {
        after_armor
    } else {
        damage_after_magic_absorb(after_armor, mitigation.enchantment_protection)
    };
    let absorbed = after_enchantments.min(mitigation.absorption.max(0.0));
    let health_damage = after_enchantments - absorbed;

    DamageOutcome {
        incoming,
        after_shield,
        after_armor,
        after_enchantments,
        absorbed,
        health_damage,
        remaining_absorption: mitigation.absorption.max(0.0) - absorbed,
    }
}

pub fn damage_after_armor(damage: f32, armor: f32, armor_toughness: f32) -> f32 {
    let toughness = 2.0 + armor_toughness / 4.0;
    let effective_armor = (armor - damage / toughness).clamp(armor * 0.2, 20.0);
    damage * (1.0 - effective_armor / 25.0)
}

pub fn damage_after_magic_absorb(damage: f32, protection: f32) -> f32 {
    damage * (1.0 - protection.clamp(0.0, 20.0) / 25.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvulnerabilityFrame {
    pub invulnerable_ticks: i32,
    pub last_damage: f32,
    pub bypasses_cooldown: bool,
}

impl InvulnerabilityFrame {
    pub fn accepted_damage(self, incoming: f32) -> Option<f32> {
        if self.invulnerable_ticks > 10 && !self.bypasses_cooldown {
            let delta = incoming - self.last_damage;
            (delta > 0.0).then_some(delta)
        } else {
            Some(incoming)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackContext {
    pub base_damage: f32,
    pub enchantment_bonus: f32,
    pub attack_strength_scale: f32,
    pub sprinting: bool,
    pub target_on_ground: bool,
    pub attacker_on_ground: bool,
    pub attacker_falling: bool,
    pub attacker_in_water: bool,
    pub attacker_blind: bool,
    pub using_sweep_weapon: bool,
    pub sweeping_edge_level: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackPlan {
    pub total_damage: f32,
    pub sweeping_damage: f32,
    pub critical: bool,
    pub sweeping: bool,
    pub knockback: f32,
    pub thorns_reflection: bool,
    pub particle: Option<&'static str>,
}

pub fn plan_player_attack(context: AttackContext, thorns_level: u8) -> AttackPlan {
    let full_strength = context.attack_strength_scale > 0.9;
    let knockback_attack = full_strength && context.sprinting;
    let critical = full_strength
        && context.attacker_falling
        && !context.attacker_on_ground
        && !context.attacker_in_water
        && !context.attacker_blind;
    let sweeping = full_strength
        && !critical
        && !knockback_attack
        && context.attacker_on_ground
        && context.using_sweep_weapon;
    let base = context.base_damage
        * (0.2 + context.attack_strength_scale * context.attack_strength_scale * 0.8);
    let mut total_damage = base + context.enchantment_bonus * context.attack_strength_scale;
    if critical {
        total_damage *= 1.5;
    }
    let sweeping_damage = if sweeping {
        1.0 + context.base_damage * sweeping_damage_ratio(context.sweeping_edge_level)
    } else {
        0.0
    };

    AttackPlan {
        total_damage,
        sweeping_damage,
        critical,
        sweeping,
        knockback: if knockback_attack { 0.5 } else { 0.0 },
        thorns_reflection: thorns_level > 0,
        particle: if critical {
            Some("minecraft:crit")
        } else if sweeping {
            Some("minecraft:sweep_attack")
        } else {
            None
        },
    }
}

pub fn sweeping_damage_ratio(level: u8) -> f32 {
    if level == 0 {
        0.0
    } else {
        1.0 - 1.0 / (level as f32 + 1.0)
    }
}

pub fn projectile_damage(speed: f64, base_damage: f64, critical: bool) -> i32 {
    let mut damage = (speed * base_damage).ceil() as i32;
    if critical {
        damage += (damage + 1) / 2;
    }
    damage.max(0)
}

pub fn explosion_damage(distance_fraction: f32, exposure: f32, power: f32) -> f32 {
    let impact = (1.0 - distance_fraction.clamp(0.0, 1.0)) * exposure.clamp(0.0, 1.0);
    ((impact * impact + impact) / 2.0 * 7.0 * power * 2.0 + 1.0).floor()
}

pub fn fall_damage(fall_distance: f32, safe_distance: f32, multiplier: f32) -> f32 {
    ((fall_distance - safe_distance).ceil() * multiplier).max(0.0)
}

pub fn environmental_damage(kind: CombatDamageKind) -> Option<f32> {
    Some(match kind {
        CombatDamageKind::Drowning => 2.0,
        CombatDamageKind::Freezing => 1.0,
        CombatDamageKind::Void => 4.0,
        CombatDamageKind::Cactus | CombatDamageKind::SweetBerryBush => 1.0,
        CombatDamageKind::WorldBorder => return None,
        CombatDamageKind::Fire => 1.0,
        CombatDamageKind::Suffocation => 1.0,
        CombatDamageKind::Magic => 1.0,
        CombatDamageKind::Starvation => 1.0,
        CombatDamageKind::Dripstone => 6.0,
        CombatDamageKind::Command => return None,
        CombatDamageKind::Melee
        | CombatDamageKind::Projectile
        | CombatDamageKind::Explosion
        | CombatDamageKind::Fall => return None,
    })
}

pub fn command_damage(amount: f32) -> Option<f32> {
    (amount >= 0.0).then_some(amount)
}

// ---------------------------------------------------------------------------
// Shield blocking (DataComponents.BLOCKS_ATTACKS)
//
// 1:1 port of net.minecraft.world.item.component.BlocksAttacks and the blocking
// resolution in LivingEntity.applyItemBlocking. As of 26.1.2 shield blocking is
// fully data-driven: a held item carries a BlocksAttacks component describing a
// set of angle-gated damage reductions, an item-damage function, a block delay,
// a disable cooldown, and a set of bypassing damage types. The vanilla shield is
// just one configuration of this component.
//
// TODO: wire apply_item_blocking into the live LivingEntity hurt pipeline once
// item data-component decoding and active-use-item tracking (getItemBlockingWith)
// are available; today the hurt path is a test-only model.
// ---------------------------------------------------------------------------

/// `BlocksAttacks.DamageReduction`: a single angle-gated reduction.
#[derive(Debug, Clone, PartialEq)]
pub struct DamageReduction {
    /// Half-width of the blocking arc in degrees (`horizontal_blocking_angle`).
    pub horizontal_blocking_angle: f32,
    /// Damage type ids this reduction applies to. `None` matches every type.
    pub damage_types: Option<Vec<&'static str>>,
    pub base: f32,
    pub factor: f32,
}

impl DamageReduction {
    /// `BlocksAttacks.DamageReduction.resolve`.
    pub fn resolve(&self, source_type: &str, dealt_damage: f32, angle: f64) -> f32 {
        let outside_arc =
            angle > (std::f64::consts::PI / 180.0) * self.horizontal_blocking_angle as f64;
        let wrong_type = self
            .damage_types
            .as_ref()
            .is_some_and(|types| !types.contains(&source_type));
        if outside_arc || wrong_type {
            0.0
        } else {
            (self.base + self.factor * dealt_damage).clamp(0.0, dealt_damage)
        }
    }
}

/// `BlocksAttacks.ItemDamageFunction`: how much durability the blocking item
/// loses for a blocked hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemDamageFunction {
    pub threshold: f32,
    pub base: f32,
    pub factor: f32,
}

impl ItemDamageFunction {
    /// `BlocksAttacks.ItemDamageFunction.DEFAULT`.
    pub const DEFAULT: Self = Self {
        threshold: 1.0,
        base: 0.0,
        factor: 1.0,
    };

    /// `BlocksAttacks.ItemDamageFunction.apply`.
    pub fn apply(&self, dealt_damage: f32) -> i32 {
        if dealt_damage < self.threshold {
            0
        } else {
            (self.base + self.factor * dealt_damage).floor() as i32
        }
    }
}

/// `net.minecraft.world.item.component.BlocksAttacks`.
#[derive(Debug, Clone, PartialEq)]
pub struct BlocksAttacks {
    pub block_delay_seconds: f32,
    pub disable_cooldown_scale: f32,
    pub damage_reductions: Vec<DamageReduction>,
    pub item_damage: ItemDamageFunction,
    /// Damage type ids that bypass this block entirely (`bypassed_by`). `None`
    /// means nothing bypasses.
    pub bypassed_by: Option<Vec<&'static str>>,
}

impl BlocksAttacks {
    /// `BlocksAttacks.blockDelayTicks`.
    pub fn block_delay_ticks(&self) -> i32 {
        (self.block_delay_seconds * 20.0).round() as i32
    }

    /// `BlocksAttacks.disableBlockingForTicks`.
    pub fn disable_blocking_for_ticks(&self, base_seconds: f32) -> i32 {
        let seconds = base_seconds * self.disable_cooldown_scale;
        if seconds > 0.0 {
            (seconds * 20.0).round() as i32
        } else {
            0
        }
    }

    /// `BlocksAttacks.resolveBlockedDamage`.
    pub fn resolve_blocked_damage(&self, source_type: &str, dealt_damage: f32, angle: f64) -> f32 {
        let blocked: f32 = self
            .damage_reductions
            .iter()
            .map(|reduction| reduction.resolve(source_type, dealt_damage, angle))
            .sum();
        blocked.clamp(0.0, dealt_damage)
    }

    /// Whether `source_type` is in the component's `bypassed_by` set.
    pub fn is_bypassed_by(&self, source_type: &str) -> bool {
        self.bypassed_by
            .as_ref()
            .is_some_and(|types| types.contains(&source_type))
    }

    /// The vanilla shield (`Items.SHIELD`) component: 0.25s block delay, full
    /// disable scaling, one 90°/base 0/factor 1 reduction (blocks 100% in the
    /// front arc), item damage `(threshold 3, base 1, factor 1)`, and bypassed
    /// by the `minecraft:bypasses_shield` damage type tag.
    pub fn vanilla_shield() -> Self {
        Self {
            block_delay_seconds: 0.25,
            disable_cooldown_scale: 1.0,
            damage_reductions: vec![DamageReduction {
                horizontal_blocking_angle: 90.0,
                damage_types: None,
                base: 0.0,
                factor: 1.0,
            }],
            item_damage: ItemDamageFunction {
                threshold: 3.0,
                base: 1.0,
                factor: 1.0,
            },
            bypassed_by: Some(BYPASSES_SHIELD.to_vec()),
        }
    }
}

/// Membership of the `minecraft:bypasses_shield` damage type tag (which nests
/// `minecraft:bypasses_armor`).
pub const BYPASSES_SHIELD: &[&str] = &[
    // #minecraft:bypasses_armor
    "minecraft:on_fire",
    "minecraft:in_wall",
    "minecraft:cramming",
    "minecraft:drown",
    "minecraft:fly_into_wall",
    "minecraft:generic",
    "minecraft:wither",
    "minecraft:dragon_breath",
    "minecraft:starve",
    "minecraft:fall",
    "minecraft:ender_pearl",
    "minecraft:freeze",
    "minecraft:stalagmite",
    "minecraft:magic",
    "minecraft:indirect_magic",
    "minecraft:out_of_world",
    "minecraft:generic_kill",
    "minecraft:sonic_boom",
    "minecraft:outside_border",
    // bypasses_shield-specific entries
    "minecraft:cactus",
    "minecraft:campfire",
    "minecraft:dry_out",
    "minecraft:falling_anvil",
    "minecraft:falling_stalactite",
    "minecraft:hot_floor",
    "minecraft:in_fire",
    "minecraft:lava",
    "minecraft:lightning_bolt",
    "minecraft:sweet_berry_bush",
];

/// 1:1 port of `LivingEntity.applyItemBlocking`: the damage blocked by an item
/// carrying `component`. `angle` is the blocking angle from [`blocking_angle`],
/// and `piercing_arrow` is true when the direct entity is a piercing arrow.
pub fn apply_item_blocking(
    component: &BlocksAttacks,
    source_type: &str,
    damage: f32,
    angle: f64,
    piercing_arrow: bool,
) -> f32 {
    if damage <= 0.0 {
        return 0.0;
    }
    if component.is_bypassed_by(source_type) {
        return 0.0;
    }
    if piercing_arrow {
        return 0.0;
    }
    component.resolve_blocked_damage(source_type, damage, angle)
}

/// The blocking angle used by `applyItemBlocking`: `acos` of the dot product of
/// the head's horizontal view vector and the normalized horizontal vector to the
/// damage source. Returns `PI` when the source has no position (an unblockable
/// "everywhere" hit). `head_yaw_deg` is `getYHeadRot`.
pub fn blocking_angle(
    head_yaw_deg: f32,
    source_pos: Option<[f64; 3]>,
    entity_pos: [f64; 3],
) -> f64 {
    let Some(source_pos) = source_pos else {
        return std::f64::consts::PI;
    };
    // Entity.calculateViewVector(0, yHeadRot) with pitch 0: (-sin yaw, 0, cos yaw).
    let yaw = (head_yaw_deg as f64).to_radians();
    let view = [-yaw.sin(), 0.0, yaw.cos()];
    let dx = source_pos[0] - entity_pos[0];
    let dz = source_pos[2] - entity_pos[2];
    let len = (dx * dx + dz * dz).sqrt();
    if len == 0.0 {
        // normalize() of a zero vector yields NaN in vanilla too; treat the
        // degenerate "source on top of the entity" case as a head-on hit.
        return 0.0;
    }
    let dot = (dx / len) * view[0] + (dz / len) * view[2];
    dot.clamp(-1.0, 1.0).acos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armor_toughness_enchantments_shields_and_absorption_follow_vanilla_order() {
        let blocked = apply_damage_pipeline(
            12.0,
            DamageMitigation {
                shield_blocking: true,
                armor: 20.0,
                armor_toughness: 8.0,
                enchantment_protection: 10.0,
                absorption: 4.0,
                ..DamageMitigation::default()
            },
        );
        assert_eq!(blocked.health_damage, 0.0);

        let outcome = apply_damage_pipeline(
            12.0,
            DamageMitigation {
                armor: 20.0,
                armor_toughness: 8.0,
                enchantment_protection: 10.0,
                absorption: 4.0,
                ..DamageMitigation::default()
            },
        );
        assert!(outcome.after_armor < outcome.after_shield);
        assert!(outcome.after_enchantments < outcome.after_armor);
        assert_eq!(outcome.absorbed, outcome.after_enchantments);
        assert!(outcome.remaining_absorption > 0.0);
        assert_eq!(outcome.health_damage, 0.0);
    }

    #[test]
    fn invulnerability_frames_accept_only_larger_damage_delta_unless_bypassed() {
        let frame = InvulnerabilityFrame {
            invulnerable_ticks: 20,
            last_damage: 6.0,
            bypasses_cooldown: false,
        };
        assert_eq!(frame.accepted_damage(4.0), None);
        assert_eq!(frame.accepted_damage(9.0), Some(3.0));
        assert_eq!(
            InvulnerabilityFrame {
                bypasses_cooldown: true,
                ..frame
            }
            .accepted_damage(4.0),
            Some(4.0)
        );
    }

    #[test]
    fn attack_plan_covers_knockback_critical_sweeping_and_thorns() {
        let critical = plan_player_attack(
            AttackContext {
                base_damage: 8.0,
                enchantment_bonus: 2.0,
                attack_strength_scale: 1.0,
                sprinting: false,
                target_on_ground: true,
                attacker_on_ground: false,
                attacker_falling: true,
                attacker_in_water: false,
                attacker_blind: false,
                using_sweep_weapon: true,
                sweeping_edge_level: 3,
            },
            1,
        );
        assert!(critical.critical);
        assert!(!critical.sweeping);
        assert!(critical.thorns_reflection);
        assert_eq!(critical.total_damage, 15.0);
        assert_eq!(critical.sweeping_damage, 0.0);
        assert_eq!(critical.particle, Some("minecraft:crit"));

        let sweep = plan_player_attack(
            AttackContext {
                attacker_on_ground: true,
                attacker_falling: false,
                sprinting: false,
                ..AttackContext {
                    base_damage: 6.0,
                    enchantment_bonus: 0.0,
                    attack_strength_scale: 1.0,
                    sprinting: false,
                    target_on_ground: true,
                    attacker_on_ground: true,
                    attacker_falling: false,
                    attacker_in_water: false,
                    attacker_blind: false,
                    using_sweep_weapon: true,
                    sweeping_edge_level: 2,
                }
            },
            0,
        );
        assert!(sweep.sweeping);
        assert_eq!(sweep.sweeping_damage, 5.0);
        assert!((sweeping_damage_ratio(2) - 2.0 / 3.0).abs() < f32::EPSILON);
        assert_eq!(sweep.particle, Some("minecraft:sweep_attack"));

        let knockback = plan_player_attack(
            AttackContext {
                sprinting: true,
                using_sweep_weapon: true,
                ..AttackContext {
                    base_damage: 6.0,
                    enchantment_bonus: 0.0,
                    attack_strength_scale: 1.0,
                    sprinting: true,
                    target_on_ground: true,
                    attacker_on_ground: true,
                    attacker_falling: false,
                    attacker_in_water: false,
                    attacker_blind: false,
                    using_sweep_weapon: true,
                    sweeping_edge_level: 2,
                }
            },
            0,
        );
        assert_eq!(knockback.knockback, 0.5);
        assert!(!knockback.sweeping);
    }

    #[test]
    fn projectile_explosion_fall_and_command_damage_use_vanilla_shapes() {
        assert_eq!(projectile_damage(3.1, 2.0, false), 7);
        assert_eq!(projectile_damage(3.1, 2.0, true), 11);
        assert_eq!(explosion_damage(0.0, 1.0, 4.0), 57.0);
        assert_eq!(fall_damage(7.2, 3.0, 1.0), 5.0);
        assert_eq!(command_damage(0.0), Some(0.0));
        assert_eq!(command_damage(-1.0), None);
    }

    #[test]
    fn environmental_damage_sources_cover_required_survival_and_border_cases() {
        let covered = [
            CombatDamageKind::Melee,
            CombatDamageKind::Projectile,
            CombatDamageKind::Explosion,
            CombatDamageKind::Fall,
            CombatDamageKind::Drowning,
            CombatDamageKind::Fire,
            CombatDamageKind::Freezing,
            CombatDamageKind::Void,
            CombatDamageKind::Suffocation,
            CombatDamageKind::Cactus,
            CombatDamageKind::SweetBerryBush,
            CombatDamageKind::Dripstone,
            CombatDamageKind::WorldBorder,
            CombatDamageKind::Magic,
            CombatDamageKind::Starvation,
            CombatDamageKind::Command,
        ];
        assert_eq!(covered.len(), 16);
        assert_eq!(environmental_damage(CombatDamageKind::Drowning), Some(2.0));
        assert_eq!(environmental_damage(CombatDamageKind::Freezing), Some(1.0));
        assert_eq!(environmental_damage(CombatDamageKind::Void), Some(4.0));
        assert_eq!(environmental_damage(CombatDamageKind::WorldBorder), None);
        assert_eq!(environmental_damage(CombatDamageKind::Command), None);
    }

    #[test]
    fn vanilla_shield_blocks_attacks_component_matches_java() {
        let shield = BlocksAttacks::vanilla_shield();

        // 0.25s delay → 5 ticks; disable scale 1.0 → cooldown seconds×20.
        assert_eq!(shield.block_delay_ticks(), 5);
        assert_eq!(shield.disable_blocking_for_ticks(5.0), 100);
        assert_eq!(shield.disable_blocking_for_ticks(0.0), 0);

        // Item durability function: floor(1 + 1·damage) once damage ≥ 3.
        assert_eq!(shield.item_damage.apply(2.9), 0);
        assert_eq!(shield.item_damage.apply(3.0), 4);
        assert_eq!(shield.item_damage.apply(7.5), 8);

        // Head-on hit (angle 0) within the 90° arc → 100% blocked (factor 1).
        assert_eq!(
            apply_item_blocking(&shield, "minecraft:player_attack", 8.0, 0.0, false),
            8.0
        );
        // Exactly on the 90° arc edge is still blocked; just past it is not.
        let edge = std::f64::consts::FRAC_PI_2;
        assert_eq!(
            apply_item_blocking(&shield, "minecraft:player_attack", 8.0, edge, false),
            8.0
        );
        assert_eq!(
            apply_item_blocking(&shield, "minecraft:player_attack", 8.0, edge + 0.01, false),
            0.0
        );

        // Projectiles are blocked too, but a piercing arrow ignores the shield.
        assert_eq!(
            apply_item_blocking(&shield, "minecraft:arrow", 6.0, 0.0, false),
            6.0
        );
        assert_eq!(
            apply_item_blocking(&shield, "minecraft:arrow", 6.0, 0.0, true),
            0.0
        );

        // Bypassing damage types (lava, in_fire, lightning, magic, …) are never
        // blocked, even head-on.
        for bypass in ["minecraft:lava", "minecraft:in_fire", "minecraft:magic"] {
            assert_eq!(
                apply_item_blocking(&shield, bypass, 8.0, 0.0, false),
                0.0,
                "{bypass}"
            );
            assert!(shield.is_bypassed_by(bypass));
        }
        assert!(!shield.is_bypassed_by("minecraft:player_attack"));

        // Zero/negative damage blocks nothing.
        assert_eq!(
            apply_item_blocking(&shield, "minecraft:player_attack", 0.0, 0.0, false),
            0.0
        );
    }

    #[test]
    fn blocking_angle_matches_view_vector_geometry() {
        // Facing +Z (yaw 0); a source straight ahead in +Z → angle 0.
        let angle = blocking_angle(0.0, Some([0.0, 0.0, 5.0]), [0.0, 0.0, 0.0]);
        assert!(angle.abs() < 1e-6, "{angle}");
        // Source directly behind (−Z) → angle π.
        let behind = blocking_angle(0.0, Some([0.0, 0.0, -5.0]), [0.0, 0.0, 0.0]);
        assert!((behind - std::f64::consts::PI).abs() < 1e-6, "{behind}");
        // Source to the side (+X) → angle π/2.
        let side = blocking_angle(0.0, Some([5.0, 0.0, 0.0]), [0.0, 0.0, 0.0]);
        assert!((side - std::f64::consts::FRAC_PI_2).abs() < 1e-6, "{side}");
        // No source position → π (an "everywhere" hit that cannot be blocked).
        assert_eq!(
            blocking_angle(0.0, None, [0.0, 0.0, 0.0]),
            std::f64::consts::PI
        );
    }

    #[test]
    fn damage_reduction_partial_factor_and_type_filter() {
        // A reduction that only blocks half of arrow damage within a 60° arc.
        let reduction = DamageReduction {
            horizontal_blocking_angle: 60.0,
            damage_types: Some(vec!["minecraft:arrow"]),
            base: 0.0,
            factor: 0.5,
        };
        // In-arc arrow → half blocked.
        assert_eq!(reduction.resolve("minecraft:arrow", 10.0, 0.0), 5.0);
        // Wrong damage type → nothing blocked even in-arc.
        assert_eq!(reduction.resolve("minecraft:player_attack", 10.0, 0.0), 0.0);
        // Out of arc (70° > 60°) → nothing blocked.
        let wide = 70.0f64.to_radians();
        assert_eq!(reduction.resolve("minecraft:arrow", 10.0, wide), 0.0);
        // base + factor is clamped to the dealt damage.
        let capped = DamageReduction {
            horizontal_blocking_angle: 90.0,
            damage_types: None,
            base: 100.0,
            factor: 0.0,
        };
        assert_eq!(capped.resolve("minecraft:generic", 4.0, 0.0), 4.0);
    }

    #[test]
    fn armor_mitigation_parity_iron_chestplate_vs_full_diamond() {
        // Iron chestplate: 8 armor points, 0 toughness
        // Full diamond: 20 armor points, 8 toughness
        // Test with representative damage values: 10, 20 HP

        let iron_10 = damage_after_armor(10.0, 8.0, 0.0);
        let diamond_10 = damage_after_armor(10.0, 20.0, 8.0);

        // Iron: effective_armor = clamp(8 - 10/2, 8*0.2, 20) = clamp(3, 1.6, 20) = 3
        // damage = 10 * (1 - 3/25) = 10 * 0.88 = 8.8
        assert!((iron_10 - 8.8).abs() < 0.01, "iron 10hp: {iron_10}");

        // Diamond: effective_armor = clamp(20 - 10/(2+8/4), 20*0.2, 20) = clamp(20-2.5, 4, 20) = clamp(17.5, 4, 20) = 17.5
        // damage = 10 * (1 - 17.5/25) = 10 * 0.3 = 3.0
        assert!(
            (diamond_10 - 3.0).abs() < 0.01,
            "diamond 10hp: {diamond_10}"
        );

        let iron_20 = damage_after_armor(20.0, 8.0, 0.0);
        // Iron: effective_armor = clamp(8 - 20/2, 1.6, 20) = clamp(-2, 1.6, 20) = 1.6
        // damage = 20 * (1 - 1.6/25) = 20 * 0.936 = 18.72
        assert!((iron_20 - 18.72).abs() < 0.01, "iron 20hp: {iron_20}");
    }
}
