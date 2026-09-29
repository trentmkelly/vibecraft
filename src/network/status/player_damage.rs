//! Live damage pipeline of a player: `Player.hurtServer` -> `LivingEntity.hurtServer` ->
//! `Player.actuallyHurt`, plus the packets Java sends when a player is hurt.
//!
//! Java references:
//! * `Player.isInvulnerableTo` / `Player.hurtServer` — game-rule gated damage
//!   (`drowning_damage`, `fall_damage`, `fire_damage`, `freeze_damage`), creative
//!   invulnerability and difficulty scaling.
//! * `LivingEntity.hurtServer` — fire resistance, the 10 tick hurt cooldown with the
//!   damage-difference rule (`invulnerableTime` / `lastHurt`), helmet damage and the
//!   `ClientboundDamageEventPacket` / `ClientboundHurtAnimationPacket` broadcast.
//! * `LivingEntity.actuallyHurt` / `getDamageAfterArmorAbsorb` / `getDamageAfterMagicAbsorb`
//!   and `CombatRules` — armor, resistance, enchantment protection and absorption.
//! * `ServerPlayer.tick` — `invulnerableTime` counts down once per tick.
//!
//! Every damage source funnels through [`hurt_server`]. Packets are queued on
//! [`HurtState::pending_packets`] and flushed by [`flush_hurt_packets`] from the lifecycle
//! tick, because damage sources run without access to the connection.

use super::*;
use crate::armor_stats::{total_armor, ArmorSlot};
use crate::damage_type::{DamageEntityRef, DamageSource};
use crate::network::play::{
    ClientboundDamageEventPacket, ClientboundHurtAnimationPacket, CLIENTBOUND_DAMAGE_EVENT_PACKET_ID,
    CLIENTBOUND_ENTITY_EVENT_PACKET_ID, CLIENTBOUND_HURT_ANIMATION_PACKET_ID,
};
use crate::player_entity::Difficulty;
use crate::registry_pipeline::{registry_element_id, vanilla_registries};
use crate::status_effect::status_effect;

/// `LivingEntity.hurtServer`: hits during the first `20 - 10` ticks of the hurt cooldown only
/// deal the difference to the previous hit.
const HURT_COOLDOWN_THRESHOLD: f32 = 10.0;
/// `LivingEntity.hurtServer`: `invulnerableTime = 20` after a full hit.
const HURT_COOLDOWN_TICKS: i32 = 20;
/// `DamageTypeTags.DAMAGES_HELMET` scaling.
const HELMET_DAMAGE_SCALE: f32 = 0.75;
/// `CombatRules.getDamageAfterMagicAbsorb`: protection is clamped to `0..=20`.
const MAX_MAGIC_ARMOR: f32 = 20.0;
/// `Level.broadcastEntityEvent` id of a drowning bubble burst (`LivingEntity.baseTick`).
pub(super) const ENTITY_EVENT_DROWN: i8 = 67;
/// `Level.broadcastEntityEvent` id sent by `HoneyBlock.fallOn`.
pub(super) const ENTITY_EVENT_HONEY_SLIDE: i8 = 54;

/// Menu slots of the worn armor pieces (`InventoryMenu` slots 5..=8) with the armor slot
/// each represents and the `LivingEntity.entityEventForEquipmentBreak` id.
const ARMOR_MENU_SLOTS: [(usize, ArmorSlot, i8); 4] = [
    (5, ArmorSlot::Helmet, 49),
    (6, ArmorSlot::Chestplate, 50),
    (7, ArmorSlot::Leggings, 51),
    (8, ArmorSlot::Boots, 52),
];

/// Damage type tags the live pipeline consults (`DamageTypeTags`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DamageTag {
    BypassesArmor,
    BypassesCooldown,
    BypassesEffects,
    BypassesEnchantments,
    BypassesInvulnerability,
    BypassesResistance,
    DamagesHelmet,
    IsDrowning,
    IsExplosion,
    IsFall,
    IsFire,
    IsFreezing,
    IsProjectile,
    NoKnockback,
}

impl DamageTag {
    /// The tag's resource path under `minecraft:` in `data/minecraft/tags/damage_type`.
    fn path(self) -> &'static str {
        match self {
            Self::BypassesArmor => "minecraft:bypasses_armor",
            Self::BypassesCooldown => "minecraft:bypasses_cooldown",
            Self::BypassesEffects => "minecraft:bypasses_effects",
            Self::BypassesEnchantments => "minecraft:bypasses_enchantments",
            Self::BypassesInvulnerability => "minecraft:bypasses_invulnerability",
            Self::BypassesResistance => "minecraft:bypasses_resistance",
            Self::DamagesHelmet => "minecraft:damages_helmet",
            Self::IsDrowning => "minecraft:is_drowning",
            Self::IsExplosion => "minecraft:is_explosion",
            Self::IsFall => "minecraft:is_fall",
            Self::IsFire => "minecraft:is_fire",
            Self::IsFreezing => "minecraft:is_freezing",
            Self::IsProjectile => "minecraft:is_projectile",
            Self::NoKnockback => "minecraft:no_knockback",
        }
    }
}

/// `DamageSource.is(TagKey)`: membership of the source's damage type in the loaded
/// `damage_type` tag. A tag without a data file (`bypasses_cooldown` in vanilla) is empty.
pub(super) fn source_is(source: &DamageSource, tag: DamageTag) -> bool {
    let Ok(registries) = vanilla_registries() else {
        return false;
    };
    let (Ok(registry_key), Ok(tag_key), Ok(type_key)) = (
        Identifier::parse("minecraft:damage_type"),
        Identifier::parse(tag.path()),
        Identifier::parse(source.damage_type.id),
    ) else {
        return false;
    };
    let Some(registry) = registries.lookup(&registry_key) else {
        return false;
    };
    match (registry.id_of(&type_key), registry.tags().get(&tag_key)) {
        (Some(id), Some(members)) => members.contains(&id),
        _ => false,
    }
}

/// The world state `Player.isInvulnerableTo` and `Player.hurtServer` read, refreshed from the
/// live game rules and server difficulty at the start of every player tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DamageRules {
    /// `GameRules.DROWNING_DAMAGE`.
    pub drowning_damage: bool,
    /// `GameRules.FALL_DAMAGE`.
    pub fall_damage: bool,
    /// `GameRules.FIRE_DAMAGE`.
    pub fire_damage: bool,
    /// `GameRules.FREEZE_DAMAGE`.
    pub freeze_damage: bool,
    /// `GameRules.PVP` (`ServerPlayer.isPvpAllowed`).
    pub pvp: bool,
    /// `Level.getDifficulty()`.
    pub difficulty: Difficulty,
}

impl Default for DamageRules {
    /// Vanilla defaults: every damage rule on, `Normal` difficulty.
    fn default() -> Self {
        Self {
            drowning_damage: true,
            fall_damage: true,
            fire_damage: true,
            freeze_damage: true,
            pvp: true,
            difficulty: Difficulty::Normal,
        }
    }
}

/// A packet queued by the damage pipeline for the connection to send.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PendingHurtPacket {
    /// `Level.broadcastDamageEvent` -> `ClientboundDamageEventPacket`.
    DamageEvent(DamageSource),
    /// `ServerPlayer.indicateDamage` -> `ClientboundHurtAnimationPacket(hurtDir)`.
    HurtAnimation { yaw: f32 },
    /// `Level.broadcastEntityEvent`.
    EntityEvent(i8),
    /// Armor durability changed: resend the inventory menu contents.
    InventorySync,
}

/// Per-player hurt bookkeeping Java keeps on `LivingEntity`.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct HurtState {
    /// `LivingEntity.invulnerableTime`.
    pub invulnerable_time: i32,
    /// `LivingEntity.lastHurt`.
    pub last_hurt: f32,
    /// `LivingEntity.getAbsorptionAmount()`.
    pub absorption: f32,
    /// Game-rule/difficulty snapshot, see [`DamageRules`].
    pub rules: DamageRules,
    /// Packets waiting for [`flush_hurt_packets`].
    pub pending_packets: Vec<PendingHurtPacket>,
    /// Fall distance of a landing that still has to be resolved against the block below the
    /// player (`Block.fallOn`), see `player_environment::resolve_landing`.
    pub pending_landing: Option<f32>,
}

/// Who and where a hit came from, for the parts of the pipeline that are not in the
/// `DamageSource` itself.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct HurtOrigin<'a> {
    /// Registry name of the causing entity, for the death message.
    pub attacker_type: Option<&'a str>,
    /// `DamageSource.getSourcePosition()`, the knockback / hurt-animation direction.
    pub position: Option<[f64; 3]>,
}

/// Refreshes the per-tick [`DamageRules`] snapshot.
pub(super) fn refresh_damage_rules(
    state: &mut PlaySessionState,
    game_rules: &SharedGameRules,
    difficulty: Difficulty,
) {
    let rules = lock_status_mutex(game_rules);
    state.combat.hurt.rules = DamageRules {
        drowning_damage: rules.bool("drowning_damage"),
        fall_damage: rules.bool("fall_damage"),
        fire_damage: rules.bool("fire_damage"),
        freeze_damage: rules.bool("freeze_damage"),
        pvp: rules.bool("pvp"),
        difficulty,
    };
}

/// `Player.isInvulnerableTo` (the entity-level and enchantment-immunity parts do not apply to a
/// player: it is neither fire-immune nor fall-damage-immune, and no vanilla enchantment grants
/// `DAMAGE_IMMUNITY`).
fn is_invulnerable_to(state: &PlaySessionState, source: &DamageSource) -> bool {
    let rules = state.combat.hurt.rules;
    if source_is(source, DamageTag::IsDrowning) {
        !rules.drowning_damage
    } else if source_is(source, DamageTag::IsFall) {
        !rules.fall_damage
    } else if source_is(source, DamageTag::IsFire) {
        !rules.fire_damage
    } else if source_is(source, DamageTag::IsFreezing) {
        !rules.freeze_damage
    } else {
        false
    }
}

/// `LivingEntity.hasEffect` / `getEffect(..).getAmplifier()` on the persisted effect list.
pub(super) fn effect_amplifier(state: &PlaySessionState, effect_id: &str) -> Option<u8> {
    state.active_effects.iter().find_map(|effect| {
        let Tag::Compound(fields) = effect else {
            return None;
        };
        let field = |name: &str| fields.iter().find(|(key, _)| key == name).map(|(_, tag)| tag);
        let id = match field("id")? {
            Tag::String(id) => id,
            _ => return None,
        };
        // The persisted id may omit the namespace only through hand-edited data.
        if status_effect(id)?.id != effect_id {
            return None;
        }
        Some(match field("amplifier") {
            Some(Tag::Byte(amplifier)) => *amplifier as u8,
            _ => 0,
        })
    })
}

/// The stacks worn in the four armor slots, `(slot, break event id, stack)`.
pub(super) fn worn_armor(state: &PlaySessionState) -> Vec<(usize, ArmorSlot, i8, ItemStack)> {
    ARMOR_MENU_SLOTS
        .iter()
        .filter_map(|&(menu_slot, slot, break_event)| {
            let stack = state.inventory_menu.get_slot(menu_slot)?;
            (!stack.is_empty()).then_some((menu_slot, slot, break_event, stack))
        })
        .collect()
}

/// `(getArmorValue(), Attributes.ARMOR_TOUGHNESS)` from the worn armor.
fn armor_totals(state: &PlaySessionState) -> (f32, f32) {
    let worn = worn_armor(state);
    total_armor(worn.iter().map(|(_, slot, _, stack)| (*slot, stack.item_id())))
}

/// `EnchantmentHelper.getDamageProtection`: the summed `minecraft:damage_protection` effects of
/// the armor enchantments whose `damage_source_properties` requirements match `source`.
///
/// Values are the `linear(base, per_level_above_first)` of the vanilla enchantment definitions
/// (`data/minecraft/enchantment/{protection,fire_protection,blast_protection,
/// projectile_protection,feather_falling}.json`), all requiring `!bypasses_invulnerability`.
fn enchantment_protection(state: &PlaySessionState, source: &DamageSource) -> f32 {
    if source_is(source, DamageTag::BypassesInvulnerability) {
        return 0.0;
    }
    let mut total = 0.0;
    for (_, _, _, stack) in worn_armor(state) {
        let Some(ItemComponent::Enchantments(enchantments)) = stack.component("minecraft:enchantments")
        else {
            continue;
        };
        for (id, &level) in enchantments {
            let (per_level, required) = match id.as_str() {
                "minecraft:protection" => (1.0, None),
                "minecraft:fire_protection" => (2.0, Some(DamageTag::IsFire)),
                "minecraft:blast_protection" => (2.0, Some(DamageTag::IsExplosion)),
                "minecraft:projectile_protection" => (2.0, Some(DamageTag::IsProjectile)),
                "minecraft:feather_falling" => (3.0, Some(DamageTag::IsFall)),
                _ => continue,
            };
            if level > 0 && required.is_none_or(|tag| source_is(source, tag)) {
                total += per_level * level as f32;
            }
        }
    }
    total
}

/// `LivingEntity.getAttributeValue(Attributes.EXPLOSION_KNOCKBACK_RESISTANCE)`: the
/// `minecraft:attributes` effect of worn Blast Protection
/// (`data/minecraft/enchantment/blast_protection.json`: `linear(0.15, 0.15)` per level,
/// `add_value`), clamped to the attribute range `0..=1`.
pub(super) fn explosion_knockback_resistance(state: &PlaySessionState) -> f64 {
    let mut total = 0.0;
    for (_, _, _, stack) in worn_armor(state) {
        let Some(ItemComponent::Enchantments(enchantments)) = stack.component("minecraft:enchantments")
        else {
            continue;
        };
        if let Some(&level) = enchantments.get("minecraft:blast_protection") {
            if level > 0 {
                total += 0.15 + 0.15 * f64::from(level - 1);
            }
        }
    }
    total.clamp(0.0, 1.0)
}

/// `LivingEntity.getDamageAfterArmorAbsorb`.
fn damage_after_armor_absorb(state: &mut PlaySessionState, source: &DamageSource, damage: f32) -> f32 {
    if source_is(source, DamageTag::BypassesArmor) {
        return damage;
    }
    hurt_armor(state, source, damage);
    let (armor, toughness) = armor_totals(state);
    // CombatRules.getDamageAfterAbsorb; the weapon `armor_effectiveness` enchantments (Breach)
    // need the attacker's held item and are not modelled.
    crate::combat_damage::damage_after_armor(damage, armor, toughness)
}

/// `LivingEntity.getDamageAfterMagicAbsorb`: Resistance, then enchantment protection.
fn damage_after_magic_absorb(state: &PlaySessionState, source: &DamageSource, mut damage: f32) -> f32 {
    if source_is(source, DamageTag::BypassesEffects) {
        return damage;
    }
    if !source_is(source, DamageTag::BypassesResistance) {
        if let Some(amplifier) = effect_amplifier(state, "minecraft:resistance") {
            let absorb = 25 - (i32::from(amplifier) + 1) * 5;
            damage = (damage * absorb as f32 / 25.0).max(0.0);
        }
    }
    if damage <= 0.0 {
        return 0.0;
    }
    if source_is(source, DamageTag::BypassesEnchantments) {
        return damage;
    }
    let protection = enchantment_protection(state, source);
    if protection > 0.0 {
        damage *= 1.0 - protection.clamp(0.0, MAX_MAGIC_ARMOR) / 25.0;
    }
    damage
}

/// `LivingEntity.doHurtEquipment` over `slots` (`None` = all four armor slots): wears the
/// armor by `max(1, damage / 4)` and queues the break events / inventory resync.
fn hurt_equipment(state: &mut PlaySessionState, source: &DamageSource, damage: f32, only_helmet: bool) {
    if damage <= 0.0 {
        return;
    }
    let durability_damage = (damage / 4.0).max(1.0) as u32;
    let fire = source_is(source, DamageTag::IsFire);
    let mut changed = false;
    for (menu_slot, slot, break_event, mut stack) in worn_armor(state) {
        if only_helmet && slot != ArmorSlot::Helmet {
            continue;
        }
        // Equippable.damageOnHurt() holds for all armor; netherite is `damage_resistant: is_fire`.
        let fireproof = fire && stack.item_id().starts_with("minecraft:netherite_");
        if !stack.is_damageable_item() || fireproof {
            continue;
        }
        let broke = stack.hurt_and_break(durability_damage);
        let stored = if broke { ItemStack::empty() } else { stack };
        state.inventory_menu.set_slot(menu_slot, stored);
        changed = true;
        if broke {
            state.combat.hurt.pending_packets.push(PendingHurtPacket::EntityEvent(break_event));
        }
    }
    if changed {
        state.combat.hurt.pending_packets.push(PendingHurtPacket::InventorySync);
    }
}

/// `Player.hurtArmor`: feet, legs, chest and head.
fn hurt_armor(state: &mut PlaySessionState, source: &DamageSource, damage: f32) {
    hurt_equipment(state, source, damage, false);
}

/// `Player.hurtHelmet`.
fn hurt_helmet(state: &mut PlaySessionState, source: &DamageSource, damage: f32) {
    hurt_equipment(state, source, damage, true);
}

/// `LivingEntity.actuallyHurt` as overridden by `Player`.
fn actually_hurt(
    state: &mut PlaySessionState,
    source: DamageSource,
    damage: f32,
    origin: HurtOrigin<'_>,
) {
    if is_invulnerable_to(state, &source) {
        return;
    }
    let mut damage = damage_after_armor_absorb(state, &source, damage);
    damage = damage_after_magic_absorb(state, &source, damage);
    let before_absorption = damage;
    let absorption = state.combat.hurt.absorption;
    damage = (damage - absorption).max(0.0);
    state.combat.hurt.absorption = absorption - (before_absorption - damage);
    if damage == 0.0 {
        super::player_death::note_hurt_by_entity(state, &source, origin.attacker_type);
        return;
    }
    add_player_food_exhaustion(state, source.food_exhaustion());
    super::player_death::record_player_damage(state, source, damage, origin.attacker_type);
    state.health = (state.health - damage).max(0.0);
    state.combat.hurt.absorption = (state.combat.hurt.absorption - damage).max(0.0);
}

/// `Player.hurtServer` -> `LivingEntity.hurtServer` for a [`DamageSource`].
///
/// Returns whether the hit was accepted. Knockback velocity, shield blocking
/// (`TODO(shield-blocking)`), totem of undying and thorns are not modelled here.
pub(super) fn hurt_server(
    state: &mut PlaySessionState,
    source: DamageSource,
    mut damage: f32,
    origin: HurtOrigin<'_>,
) -> bool {
    if is_invulnerable_to(state, &source) {
        return false;
    }
    // `ServerPlayer.hurtServer`: a player-caused hit needs `canHarmPlayer`, which
    // requires PvP (team friendly-fire is not modelled).
    if !state.combat.hurt.rules.pvp && source.causing_entity.is_some_and(|entity| entity.is_player) {
        return false;
    }
    if state.abilities.invulnerable && !source_is(&source, DamageTag::BypassesInvulnerability) {
        return false;
    }
    if state.health <= 0.0 {
        return false;
    }
    if source.scales_with_difficulty() {
        damage = scale_for_difficulty(state.combat.hurt.rules.difficulty, damage);
    }
    if damage == 0.0 {
        return false;
    }
    if source_is(&source, DamageTag::IsFire) && effect_amplifier(state, "minecraft:fire_resistance").is_some() {
        return false;
    }
    damage = damage.max(0.0);
    if source_is(&source, DamageTag::DamagesHelmet) && !worn_helmet_is_empty(state) {
        hurt_helmet(state, &source, damage);
        damage *= HELMET_DAMAGE_SCALE;
    }
    if damage.is_nan() || damage.is_infinite() {
        damage = f32::MAX;
    }

    let hurt = &mut state.combat.hurt;
    let took_full_damage;
    if hurt.invulnerable_time as f32 > HURT_COOLDOWN_THRESHOLD
        && !source_is(&source, DamageTag::BypassesCooldown)
    {
        if damage <= hurt.last_hurt {
            return false;
        }
        let dealt = damage - hurt.last_hurt;
        hurt.last_hurt = damage;
        took_full_damage = false;
        actually_hurt(state, source, dealt, origin);
    } else {
        hurt.last_hurt = damage;
        hurt.invulnerable_time = HURT_COOLDOWN_TICKS;
        took_full_damage = true;
        actually_hurt(state, source, damage, origin);
    }

    if took_full_damage {
        queue_hurt_packets(state, source, origin);
    }
    true
}

/// `Player.hurtServer` difficulty scaling for `source.scalesWithDifficulty()`.
fn scale_for_difficulty(difficulty: Difficulty, damage: f32) -> f32 {
    match difficulty {
        Difficulty::Peaceful => 0.0,
        Difficulty::Easy => (damage / 2.0 + 1.0).min(damage),
        Difficulty::Normal => damage,
        Difficulty::Hard => damage * 3.0 / 2.0,
    }
}

fn worn_helmet_is_empty(state: &PlaySessionState) -> bool {
    state.inventory_menu.get_slot(5).is_none_or(|stack| stack.is_empty())
}

/// The `tookFullDamage` branch of `LivingEntity.hurtServer`: `broadcastDamageEvent` and, unless
/// the source has `NO_KNOCKBACK`, `ServerPlayer.indicateDamage`.
fn queue_hurt_packets(state: &mut PlaySessionState, source: DamageSource, origin: HurtOrigin<'_>) {
    state.combat.hurt.pending_packets.push(PendingHurtPacket::DamageEvent(source));
    if source_is(&source, DamageTag::NoKnockback) {
        return;
    }
    let (dx, dz) = origin
        .position
        .map_or((0.0, 0.0), |at| (at[0] - state.x, at[2] - state.z));
    let hurt_dir = dz.atan2(dx).to_degrees() as f32 - state.yaw;
    state
        .combat
        .hurt
        .pending_packets
        .push(PendingHurtPacket::HurtAnimation { yaw: hurt_dir });
}

/// `ServerPlayer.tick`: `invulnerableTime` counts down once per tick.
pub(super) fn tick_hurt_cooldown(state: &mut PlaySessionState) {
    if state.combat.hurt.invulnerable_time > 0 {
        state.combat.hurt.invulnerable_time -= 1;
    }
}

/// Queues `Level.broadcastEntityEvent(player, id)`.
pub(super) fn queue_entity_event(state: &mut PlaySessionState, event: i8) {
    state.combat.hurt.pending_packets.push(PendingHurtPacket::EntityEvent(event));
}

/// `DamageSource` entity id, `-1` when absent (`ClientboundDamageEventPacket` writes `id + 1`).
fn entity_id(entity: Option<DamageEntityRef>) -> i32 {
    entity.map_or(-1, |entity| entity.id)
}

/// Sends everything [`hurt_server`] queued. The player's own connection is the only tracker
/// modelled (`sendToTrackingPlayersAndSelf`).
///
/// TODO(tracking-broadcast): other players' connections need the per-observer entity id of this
/// player (live player registry gap).
pub(super) fn flush_hurt_packets<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
) -> io::Result<()> {
    let pending = std::mem::take(&mut state.combat.hurt.pending_packets);
    for packet in pending {
        match packet {
            PendingHurtPacket::DamageEvent(source) => {
                let Some(type_id) = registry_element_id("minecraft:damage_type", source.damage_type.id) else {
                    continue;
                };
                let packet = ClientboundDamageEventPacket {
                    entity_id: PLAYER_ENTITY_ID,
                    source_type_id: type_id as i32,
                    source_cause_id: entity_id(source.causing_entity),
                    source_direct_id: entity_id(source.direct_entity),
                    source_position: source
                        .source_position
                        .map(|[x, y, z]| Vec3 { x, y, z }),
                };
                write_framed_packet_with_compression(
                    writer,
                    compression,
                    CLIENTBOUND_DAMAGE_EVENT_PACKET_ID,
                    |payload| packet.write(payload),
                )?;
            }
            PendingHurtPacket::HurtAnimation { yaw } => write_framed_packet_with_compression(
                writer,
                compression,
                CLIENTBOUND_HURT_ANIMATION_PACKET_ID,
                |payload| ClientboundHurtAnimationPacket { id: PLAYER_ENTITY_ID, yaw }.write(payload),
            )?,
            PendingHurtPacket::EntityEvent(event_id) => write_framed_packet_with_compression(
                writer,
                compression,
                CLIENTBOUND_ENTITY_EVENT_PACKET_ID,
                |payload| {
                    ClientboundEntityEventPacket { entity_id: PLAYER_ENTITY_ID, event_id }
                        .write(payload)
                },
            )?,
            PendingHurtPacket::InventorySync => write_inventory_content_sync(writer, compression, state)?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod explosion_tests;
