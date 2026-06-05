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
// Shield blocking
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShieldBlockState {
    pub blocking: bool,
    pub block_cooldown: i32, // 5-tick cooldown after strong hit (100 HP)
    pub look_yaw: f32,       // player's horizontal look angle for arc check
}

impl Default for ShieldBlockState {
    fn default() -> Self {
        Self {
            blocking: false,
            block_cooldown: 0,
            look_yaw: 0.0,
        }
    }
}

impl ShieldBlockState {
    pub fn is_blocking_effectively(&self) -> bool {
        self.blocking && self.block_cooldown <= 0
    }

    pub fn tick_cooldown(&mut self) {
        if self.block_cooldown > 0 {
            self.block_cooldown -= 1;
        }
    }
}

/// Compute the fraction of damage blocked by a shield.
/// `attack_yaw` is the direction the attack comes from (degrees).
/// The shield blocks within a 180-degree arc in front of the player.
/// For projectiles: 100% blocked if in arc.
/// For melee: 100% blocked if damage < strong_hit_threshold AND in arc.
/// After blocking a strong hit (>= strong_hit_threshold), apply 5-tick cooldown.
pub fn shield_block_result(
    state: &mut ShieldBlockState,
    incoming_damage: f32,
    is_projectile: bool,
    attack_yaw: f32,
    strong_hit_threshold: f32,
) -> f32 {
    if !state.is_blocking_effectively() {
        return incoming_damage;
    }

    // Check arc: attack must be within 180 degrees of shield face (opposite of look)
    let angle_diff = normalize_angle(attack_yaw - state.look_yaw);
    if angle_diff.abs() > 90.0 {
        // Attack from behind — shield does not block
        return incoming_damage;
    }

    if is_projectile {
        // Projectiles blocked completely
        0.0
    } else {
        // Melee: blocked if damage < strong_hit_threshold
        if incoming_damage >= strong_hit_threshold {
            // Strong hit disables shield temporarily
            state.block_cooldown = 5;
            incoming_damage
        } else {
            0.0
        }
    }
}

fn normalize_angle(mut angle: f32) -> f32 {
    while angle > 180.0 {
        angle -= 360.0;
    }
    while angle < -180.0 {
        angle += 360.0;
    }
    angle
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
    fn shield_blocks_projectile_in_arc_and_strong_hit_disables_shield() {
        let mut shield = ShieldBlockState {
            blocking: true,
            block_cooldown: 0,
            look_yaw: 0.0,
        };

        // Projectile from front (angle 0) → fully blocked
        let remaining = shield_block_result(&mut shield, 5.0, true, 0.0, 8.0);
        assert_eq!(remaining, 0.0);

        // Projectile from behind (angle 180) → not blocked
        let remaining = shield_block_result(&mut shield, 5.0, true, 180.0, 8.0);
        assert_eq!(remaining, 5.0);

        // Melee weak hit from front → blocked
        let remaining = shield_block_result(&mut shield, 3.0, false, 0.0, 8.0);
        assert_eq!(remaining, 0.0);
        assert_eq!(shield.block_cooldown, 0);

        // Melee strong hit → not blocked, sets cooldown
        let remaining = shield_block_result(&mut shield, 10.0, false, 0.0, 8.0);
        assert_eq!(remaining, 10.0);
        assert_eq!(shield.block_cooldown, 5);

        // During cooldown: shield disabled
        assert!(!shield.is_blocking_effectively());
        shield.tick_cooldown();
        assert_eq!(shield.block_cooldown, 4);
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
