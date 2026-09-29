//! Live player death flow.
//!
//! Java references:
//! * `ServerPlayer.die(DamageSource)` — death message, `ClientboundPlayerCombatKillPacket`,
//!   loot/equipment/experience drops, `lastDeathLocation`.
//! * `LivingEntity.dropAllDeathLoot` / `dropExperience` and `Player.dropEquipment` /
//!   `destroyVanishingCursedItems` / `getBaseExperienceReward`.
//! * `Inventory.dropAll` and `LivingEntity.createItemStackToDrop(randomly = true)`.
//! * `CombatTracker.recordDamage` — every health-reducing source records a
//!   [`CombatEntry`] so the death message can be built when health reaches zero.
//!
//! Damage sources call [`record_player_damage`]; the per-tick lifecycle calls
//! [`tick_player_death`], which runs [`die`] exactly once per death (Java sets
//! `LivingEntity.dead`, modelled by [`PlayerCombatState::dead`]).

use std::collections::BTreeMap;

use super::player_damage::{hurt_server, HurtOrigin};
use super::*;
use crate::chat_component::{ClickEvent, Component, ComponentArgument, HoverEvent, Style};
use crate::combat_tracker::{CombatEntry, CombatTracker, DeathMessageSource, FallLocation};
use crate::damage_type::{builtin_damage_type, DamageEntityRef, DamageSource};
use crate::network::codec::component_json_to_network_tag;
use crate::network::play::{
    ClientboundPlayerCombatKillPacket, CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID,
};
use crate::xp_orb_entity::XpOrbRandom;

/// `Player.getEyeHeight()` for the standing pose (`EntityType.PLAYER.eyeHeight(1.62)`).
const PLAYER_EYE_HEIGHT: f64 = 1.62;
/// `ItemEntity.setPickUpDelay(40)` applied by `LivingEntity.createItemStackToDrop`.
const DEATH_DROP_PICKUP_DELAY: i32 = 40;
/// `LivingEntity.tick`: `lastHurtByMob` is forgotten after this many ticks.
const LAST_HURT_BY_MOB_MEMORY_TICKS: i32 = 100;
/// `Player.getBaseExperienceReward`: `Math.min(experienceLevel * 7, 100)`.
const MAX_DEATH_EXPERIENCE: i32 = 100;
/// Enchantment carrying `EnchantmentEffectComponents.PREVENT_EQUIPMENT_DROP`.
const VANISHING_CURSE: &str = "minecraft:vanishing_curse";

/// Per-player combat/death bookkeeping that Java keeps on `LivingEntity`/`Player`.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct PlayerCombatState {
    /// `LivingEntity.combatTracker`.
    pub tracker: CombatTracker,
    /// `Entity.tickCount`, advanced once per player tick.
    pub tick_count: i32,
    /// `LivingEntity.dead`: set by [`die`] so a death is processed exactly once.
    pub dead: bool,
    /// `Player.takeXpDelay` — ticks until the next orb may be collected.
    pub take_xp_delay: i32,
    /// `LivingEntity.lastHurtByMob` as `(entity id, tickCount when hurt)`.
    pub last_hurt_by_mob: Option<(i32, i32)>,
    /// Registry names (`minecraft:zombie`) of entities that hurt the player, so the death
    /// message can name them even after they left the world.
    pub attacker_types: BTreeMap<i32, String>,
    /// Hurt cooldown, absorption and the queued hurt packets, see [`super::player_damage`].
    pub hurt: super::player_damage::HurtState,
    /// Fire / freeze bookkeeping, see [`super::player_environment`].
    pub environment: super::player_environment::EnvironmentState,
}

/// `DamageSources.<id>()` for a source without entities.
pub(super) fn simple_damage_source(id: &str) -> DamageSource {
    let damage_type = builtin_damage_type(id)
        .unwrap_or_else(|| panic!("unknown builtin damage type {id}"));
    DamageSource::simple(damage_type)
}

/// `DamageSources.mobAttack(mob)` — the attacker is a living, non-player entity.
pub(super) fn mob_attack_damage_source(entity_id: i32) -> DamageSource {
    crate::damage_type::mob_attack_source(DamageEntityRef::living_mob(entity_id))
}

/// `LivingEntity.actuallyHurt` → `CombatTracker.recordDamage(source, damage)`, plus the
/// `lastHurtByMob` bookkeeping `hurtServer` performs for living attackers.
///
/// `attacker_type` is the registry name of the causing entity, when there is one.
pub(super) fn record_player_damage(
    state: &mut PlaySessionState,
    source: DamageSource,
    damage: f32,
    attacker_type: Option<&str>,
) {
    let combat = &mut state.combat;
    let is_alive = state.health > 0.0;
    // FallLocation.getCurrentFallLocation: only "in water" is known without climbable tracking.
    let fall_location = state.in_water.then_some(FallLocation::Water);
    let entry = CombatEntry::new(source, damage, fall_location, state.fall_distance);
    combat.tracker.record_damage(combat.tick_count, is_alive, entry);
    note_hurt_by_entity(state, &source, attacker_type);
}

/// `LivingEntity.resolveMobResponsibleForDamage`: remembers a living attacker and its type so
/// the death message can name it. Runs for every accepted hit, even one fully absorbed.
pub(super) fn note_hurt_by_entity(
    state: &mut PlaySessionState,
    source: &DamageSource,
    attacker_type: Option<&str>,
) {
    let combat = &mut state.combat;
    if let (Some(entity), Some(kind)) = (source.causing_entity, attacker_type) {
        if entity.is_living {
            combat.last_hurt_by_mob = Some((entity.id, combat.tick_count));
        }
        combat.attacker_types.insert(entity.id, kind.to_string());
    }
}

/// `Entity.hurtServer` for an entity-less source, through the full
/// [`hurt_server`](super::player_damage::hurt_server) pipeline. Returns whether it was accepted.
pub(super) fn hurt_player(state: &mut PlaySessionState, damage_type: &str, damage: f32) -> bool {
    hurt_server(state, simple_damage_source(damage_type), damage, HurtOrigin::default())
}

/// `LivingEntity.kill(level)`: `hurtServer(genericKill, Float.MAX_VALUE)`. `generic_kill` is in
/// `bypasses_invulnerability`, so creative players die too.
pub(super) fn kill_player(state: &mut PlaySessionState) {
    hurt_player(state, "minecraft:generic_kill", f32::MAX);
}

/// Applies `/kill` to the executing player and syncs the new health (the death itself,
/// `ServerPlayer.die`, runs from the next lifecycle tick).
///
/// TODO(kick-kill-live-wiring): killing *other* players or mobs needs cross-player SEND
/// (STAGE 4 of [[project-live-player-registry-gap]]), and a bare `/kill` needs
/// `command_source_entity` seeded in `command_state_for_player`.
pub(super) fn apply_kill_command(
    stream: &mut ClientStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    profile: &NameAndId,
    command_state: &crate::command::ServerCommandState,
) -> io::Result<()> {
    let killed = command_state
        .killed_entities
        .iter()
        .any(|entity| entity.id == profile.name);
    if !killed || state.health <= 0.0 {
        return Ok(());
    }
    kill_player(state);
    write_play_state_health_packet(stream, compression, state)
}

/// `LivingEntity.hurtServer` for `DamageSources.mobAttack(mob)`; `mob_position` is the
/// attacker's position (`DamageSource.getSourcePosition()` of an entity source).
pub(super) fn hurt_player_by_mob(
    state: &mut PlaySessionState,
    mob_id: i32,
    mob_type: &str,
    mob_position: [f64; 3],
    damage: f32,
) -> bool {
    let origin = HurtOrigin { attacker_type: Some(mob_type), position: Some(mob_position) };
    hurt_server(state, mob_attack_damage_source(mob_id), damage, origin)
}

/// Collaborators [`tick_player_lifecycle`] needs from the running session.
pub(super) struct PlayerLifecycleContext<'a> {
    pub game_rules: &'a SharedGameRules,
    pub world_items: &'a Arc<Mutex<WorldItemEntities>>,
    pub world_mobs: &'a Arc<Mutex<LiveMobStore>>,
    /// Server-wide fan-out, used for what Java sends to every tracking player.
    pub bus: &'a WorldPacketBus,
    pub profile: &'a NameAndId,
}

/// Seed for cosmetic randomness (drop scatter, orb velocity); Java draws these from
/// `Entity.random`.
pub(super) fn live_random_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos() as u64)
}

/// Per-tick player lifecycle: collects experience orbs, advances the combat clock and
/// runs [`die`] when health has reached zero.
pub(super) fn tick_player_lifecycle(
    stream: &mut ClientStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    context: &PlayerLifecycleContext<'_>,
) -> io::Result<()> {
    state.combat.tick_count = state.combat.tick_count.wrapping_add(1);
    // Player.tick: takeXpDelay counts down before this tick's pickups.
    if state.combat.take_xp_delay > 0 {
        state.combat.take_xp_delay -= 1;
    }
    super::xp_orb_live::tick_xp_orbs(
        stream,
        compression,
        state,
        context.world_items,
        context.bus,
        context.profile,
    )?;
    super::player_damage::flush_hurt_packets(stream, compression, state)?;
    if state.health > 0.0 || state.combat.dead {
        return Ok(());
    }
    die(stream, compression, state, context, live_random_seed())
}

/// `ServerPlayer.die(DamageSource)`. The damage source is the combat tracker's last entry.
pub(super) fn die<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    context: &PlayerLifecycleContext<'_>,
    random_seed: u64,
) -> io::Result<()> {
    state.combat.dead = true;
    let (show_death_message, keep_inventory) = {
        let rules = lock_status_mutex(context.game_rules);
        (rules.bool("show_death_messages"), rules.bool("keep_inventory"))
    };

    if show_death_message {
        let message = death_message(state, context);
        write_combat_kill(writer, compression, &message.to_json())?;
        broadcast_system_message(context.bus, &message)?;
    } else {
        write_combat_kill(writer, compression, &Component::empty().to_json())?;
    }

    if state.game_mode != GameMode::Spectator {
        drop_all_death_loot(writer, compression, state, context, keep_inventory, random_seed)?;
    }

    // ServerPlayer.setLastDeathLocation(GlobalPos.of(dimension, blockPosition())).
    state.last_death_location = Some(PlayerGlobalPosData {
        dimension: "minecraft:overworld".to_string(),
        x: state.x.floor() as i32,
        y: state.y.floor() as i32,
        z: state.z.floor() as i32,
    });
    // CombatTracker.recheckStatus with the mob dead clears the recorded entries.
    let (tick_count, combat) = (state.combat.tick_count, &mut state.combat);
    combat.tracker.recheck_status(tick_count, false);
    Ok(())
}

/// `CombatTracker.getDeathMessage()` with live display names.
fn death_message(state: &PlaySessionState, context: &PlayerLifecycleContext<'_>) -> Component {
    let names = LiveDeathNames {
        victim: player_display_name(context.profile),
        combat: &state.combat,
        world_mobs: context.world_mobs,
    };
    state.combat.tracker.death_message(&names)
}

/// `Player.getDisplayName()`: the name decorated with a `/tell` click event, an entity
/// hover event and the name as insertion. Team colouring needs the scoreboard team of the
/// player, which is not tracked live.
fn player_display_name(profile: &NameAndId) -> Component {
    let name = Component::literal(profile.name.clone());
    let style = Style::empty()
        .with_click_event(ClickEvent::SuggestCommand(format!("/tell {} ", profile.name)))
        .with_hover_event(HoverEvent::Entity {
            entity_type: "minecraft:player".to_string(),
            uuid: profile.uuid.clone(),
            name: Some(Box::new(name.clone())),
        })
        .with_insertion(profile.name.clone());
    name.styled(style)
}

/// `Entity.getDisplayName()` for a non-player entity of `entity_type`.
fn entity_display_name(entity_type: &str) -> Component {
    let path = entity_type.strip_prefix("minecraft:").unwrap_or(entity_type);
    let key = format!("entity.minecraft.{}", path.replace('/', "."));
    Component::translatable(key, Vec::<ComponentArgument>::new())
}

/// Resolves the names `CombatTracker.getDeathMessage` reads from live entities.
struct LiveDeathNames<'a> {
    victim: Component,
    combat: &'a PlayerCombatState,
    world_mobs: &'a Arc<Mutex<LiveMobStore>>,
}

impl DeathMessageSource for LiveDeathNames<'_> {
    fn victim_name(&self) -> Component {
        self.victim.clone()
    }

    /// `LivingEntity.getKillCredit()` — the mob that hurt the player within the last
    /// 100 ticks and is still alive.
    fn kill_credit_name(&self) -> Option<Component> {
        let (id, hurt_at) = self.combat.last_hurt_by_mob?;
        if self.combat.tick_count - hurt_at > LAST_HURT_BY_MOB_MEMORY_TICKS {
            return None;
        }
        if !lock_status_mutex(self.world_mobs).contains(id) {
            return None;
        }
        self.combat.attacker_types.get(&id).map(|kind| entity_display_name(kind))
    }

    fn entity_name(&self, entity: DamageEntityRef) -> Component {
        self.combat
            .attacker_types
            .get(&entity.id)
            .map(|kind| entity_display_name(kind))
            .unwrap_or_else(Component::empty)
    }

    /// Live mobs carry no equipment, so no attacker ever holds a named item.
    fn held_item_name(&self, _entity: DamageEntityRef) -> Option<Component> {
        None
    }
}

fn write_combat_kill<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    message_json: &str,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID,
        |payload| {
            ClientboundPlayerCombatKillPacket {
                player_id: PLAYER_ENTITY_ID,
                message: message_json.to_string(),
            }
            .write(payload)
        },
    )
}

/// `PlayerList.broadcastSystemMessage(message, false)`. Scoreboard team death-message
/// visibility (`Team.Visibility`) is not modelled live, which matches the `team == null`
/// branch of `ServerPlayer.die`.
fn broadcast_system_message(bus: &WorldPacketBus, message: &Component) -> io::Result<()> {
    let mut frames = Vec::new();
    write_framed_packet_with_compression(
        &mut frames,
        CompressionState::disabled(),
        CLIENTBOUND_SYSTEM_CHAT_PACKET_ID,
        |payload| {
            ClientboundSystemChatPacket {
                content: component_json_to_network_tag(&message.to_json()),
                overlay: false,
            }
            .write(payload)
        },
    )?;
    bus.publish_frames(&frames)
}

/// `LivingEntity.dropAllDeathLoot`: equipment (`Player.dropEquipment`) then experience.
/// Players have no loot table, so `dropFromLootTable`/`dropCustomDeathLoot` add nothing.
fn drop_all_death_loot<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    context: &PlayerLifecycleContext<'_>,
    keep_inventory: bool,
    random_seed: u64,
) -> io::Result<()> {
    let mut random = XpOrbRandom::new(random_seed);
    drop_equipment(writer, compression, state, context, keep_inventory, &mut random)?;
    drop_experience(state, context, keep_inventory, &mut random)
}

/// `Player.dropEquipment`: unless `keepInventory`, destroy vanishing-cursed items and drop
/// everything else (`Inventory.dropAll`, each stack via `drop(stack, randomly = true)`).
fn drop_equipment<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    context: &PlayerLifecycleContext<'_>,
    keep_inventory: bool,
    random: &mut XpOrbRandom,
) -> io::Result<()> {
    if keep_inventory {
        return Ok(());
    }
    let times_changed_before = state.inventory_menu.player_inventory().times_changed();
    let drops = state
        .inventory_menu
        .player_inventory_mut()
        .death_drops(false, has_prevent_equipment_drop);
    let mut frames = Vec::new();
    for stack in drops {
        spawn_death_drop(&mut frames, context, state, &stack, random)?;
    }
    context.bus.publish_frames(&frames)?;
    write_pickup_inventory_sync(writer, compression, state, times_changed_before)
}

/// `EnchantmentHelper.has(stack, PREVENT_EQUIPMENT_DROP)` — only Curse of Vanishing has it.
fn has_prevent_equipment_drop(stack: &ItemStack) -> bool {
    matches!(
        stack.component("minecraft:enchantments"),
        Some(ItemComponent::Enchantments(enchantments)) if enchantments.contains_key(VANISHING_CURSE)
    )
}

/// `LivingEntity.createItemStackToDrop(stack, randomly = true, thrownFromHand = false)`
/// followed by `addFreshEntity`: spawns the item entity at `eyeY - 0.3` with a random
/// horizontal push and a 40 tick pickup delay.
fn spawn_death_drop(
    frames: &mut Vec<u8>,
    context: &PlayerLifecycleContext<'_>,
    state: &PlaySessionState,
    stack: &ItemStack,
    random: &mut XpOrbRandom,
) -> io::Result<()> {
    let Some(item_pid) = item_protocol_id(stack.item_id()) else {
        return Ok(());
    };
    let power = random.next_double() * 0.5;
    let direction = random.next_double() * std::f64::consts::TAU;
    let entity_id = lock_status_mutex(context.world_items).alloc_entity_id();
    let item = DroppedItem {
        entity_id,
        item: stack.item_id(),
        count: stack.count(),
        x: state.x,
        y: state.y + PLAYER_EYE_HEIGHT - 0.3,
        z: state.z,
        vel_x: -direction.sin() * power,
        vel_y: 0.2,
        vel_z: direction.cos() * power,
        pickup_delay: DEATH_DROP_PICKUP_DELAY,
        age: 0,
        target_uuid: None,
    };
    write_item_entity_spawn_packets(frames, CompressionState::disabled(), &item, item_pid)?;
    lock_status_mutex(context.world_items).entities.push(item);
    Ok(())
}

/// `LivingEntity.dropExperience`: players are `isAlwaysExperienceDropper`, so
/// `ExperienceOrb.award(level, position, getExperienceReward(...))` always runs.
fn drop_experience(
    state: &PlaySessionState,
    context: &PlayerLifecycleContext<'_>,
    keep_inventory: bool,
    random: &mut XpOrbRandom,
) -> io::Result<()> {
    let amount = death_experience_reward(state.xp_level, keep_inventory, state.game_mode);
    if amount <= 0 {
        return Ok(());
    }
    let spawned = lock_status_mutex(context.world_items).award_experience(
        (state.x, state.y, state.z),
        amount,
        random,
    );
    let mut frames = Vec::new();
    for orb in &spawned {
        super::xp_orb_live::write_xp_orb_spawn_packets(&mut frames, CompressionState::disabled(), orb)?;
    }
    context.bus.publish_frames(&frames)
}

/// `Player.getBaseExperienceReward`: nothing under `keepInventory` or in spectator mode,
/// otherwise `min(level * 7, 100)`.
pub(super) fn death_experience_reward(level: i32, keep_inventory: bool, game_mode: GameMode) -> i32 {
    if keep_inventory || game_mode == GameMode::Spectator {
        0
    } else {
        level.saturating_mul(7).min(MAX_DEATH_EXPERIENCE)
    }
}

#[cfg(test)]
mod tests;
