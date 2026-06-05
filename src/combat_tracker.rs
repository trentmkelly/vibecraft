#![allow(dead_code)]

use crate::chat_component::{ClickEvent, Component, ComponentArgument, HoverEvent, Style};
use crate::damage_type::{DamageEntityRef, DamageSource, DeathMessageType};

// ---------------------------------------------------------------------------
// CombatTracker
// ---------------------------------------------------------------------------

/// `CombatTracker.RESET_DAMAGE_STATUS_TIME` — ticks of no damage before the
/// non-combat damage status is cleared.
pub const RESET_DAMAGE_STATUS_TIME: i32 = 100;
/// `CombatTracker.RESET_COMBAT_STATUS_TIME` — ticks of no damage before the
/// in-combat status is cleared.
pub const RESET_COMBAT_STATUS_TIME: i32 = 300;

const INTENTIONAL_GAME_DESIGN_BUG_URL: &str = "https://bugs.mojang.com/browse/MCPE-28723";
const INTENTIONAL_GAME_DESIGN_BUG_ID: &str = "MCPE-28723";

/// 1:1 port of `net.minecraft.world.damagesource.FallLocation`. The language
/// key is always `death.fell.accident.<id>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallLocation {
    Generic,
    Ladder,
    Vines,
    WeepingVines,
    TwistingVines,
    Scaffolding,
    OtherClimbable,
    Water,
}

impl FallLocation {
    /// `FallLocation.languageKey()`.
    pub fn language_key(self) -> &'static str {
        match self {
            Self::Generic => "death.fell.accident.generic",
            Self::Ladder => "death.fell.accident.ladder",
            Self::Vines => "death.fell.accident.vines",
            Self::WeepingVines => "death.fell.accident.weeping_vines",
            Self::TwistingVines => "death.fell.accident.twisting_vines",
            Self::Scaffolding => "death.fell.accident.scaffolding",
            Self::OtherClimbable => "death.fell.accident.other_climbable",
            Self::Water => "death.fell.accident.water",
        }
    }
}

/// 1:1 port of `net.minecraft.world.damagesource.CombatEntry`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombatEntry {
    pub source: DamageSource,
    pub damage: f32,
    pub fall_location: Option<FallLocation>,
    pub fall_distance: f32,
}

impl CombatEntry {
    pub fn new(
        source: DamageSource,
        damage: f32,
        fall_location: Option<FallLocation>,
        fall_distance: f32,
    ) -> Self {
        Self {
            source,
            damage,
            fall_location,
            fall_distance,
        }
    }
}

/// Resolves the live entity data that `CombatTracker.getDeathMessage` reads
/// when assembling a death message. The damage model only stores entity ids
/// and flags, so the network layer supplies the display names (`Component`s in
/// vanilla) for the victim, kill credit, attacking entities, and any custom
/// item names. Implementations mirror `LivingEntity.getDisplayName`,
/// `LivingEntity.getKillCredit`, and `ItemStack.has(DataComponents.CUSTOM_NAME)`.
pub trait DeathMessageSource {
    /// `mob.getDisplayName()` for the dying entity.
    fn victim_name(&self) -> Component;
    /// `mob.getKillCredit().getDisplayName()`, or `None` when there is no
    /// recent kill credit.
    fn kill_credit_name(&self) -> Option<Component>;
    /// `entity.getDisplayName()` for the referenced combat entity.
    fn entity_name(&self, entity: DamageEntityRef) -> Component;
    /// The custom name of `entity`'s main-hand item — `Some` only when the
    /// entity is living and holds an item that has `DataComponents.CUSTOM_NAME`.
    fn held_item_name(&self, entity: DamageEntityRef) -> Option<Component>;
}

/// 1:1 port of `net.minecraft.world.damagesource.CombatTracker`. The owning
/// `LivingEntity` is not stored; its tick count and liveness are passed to the
/// status methods, and its display data is resolved through
/// [`DeathMessageSource`] when a death message is built.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CombatTracker {
    pub entries: Vec<CombatEntry>,
    pub last_damage_time: i32,
    pub combat_start_time: i32,
    pub combat_end_time: i32,
    pub in_combat: bool,
    pub taking_damage: bool,
}

impl CombatTracker {
    /// `CombatTracker.recordDamage(DamageSource, float)`. `tick_count` is the
    /// owning mob's `tickCount`; `is_alive` is `mob.isAlive()`.
    pub fn record_damage(
        &mut self,
        tick_count: i32,
        is_alive: bool,
        entry: CombatEntry,
    ) {
        self.recheck_status(tick_count, is_alive);
        let should_enter = should_enter_combat(&entry.source);
        self.entries.push(entry);
        self.last_damage_time = tick_count;
        self.taking_damage = true;
        if !self.in_combat && is_alive && should_enter {
            self.in_combat = true;
            self.combat_start_time = tick_count;
            self.combat_end_time = self.combat_start_time;
            // mob.onEnterCombat() — side effect handled by the caller.
        }
    }

    /// `CombatTracker.recheckStatus()`.
    pub fn recheck_status(&mut self, tick_count: i32, is_alive: bool) {
        let reset = if self.in_combat {
            RESET_COMBAT_STATUS_TIME
        } else {
            RESET_DAMAGE_STATUS_TIME
        };
        if self.taking_damage && (!is_alive || tick_count - self.last_damage_time > reset) {
            // wasInCombat → mob.onLeaveCombat() is the caller's responsibility.
            self.taking_damage = false;
            self.in_combat = false;
            self.combat_end_time = tick_count;
            self.entries.clear();
        }
    }

    /// `CombatTracker.getCombatDuration()`.
    pub fn combat_duration(&self, tick_count: i32) -> i32 {
        if self.in_combat {
            tick_count - self.combat_start_time
        } else {
            self.combat_end_time - self.combat_start_time
        }
    }

    /// `CombatTracker.getDeathMessage()`.
    pub fn death_message(&self, names: &impl DeathMessageSource) -> Component {
        let Some(killing_blow) = self.entries.last() else {
            return Component::translatable("death.attack.generic", vec![arg(names.victim_name())]);
        };
        let killing_source = &killing_blow.source;
        let knock_off = self.most_significant_fall();
        match killing_source.damage_type.death_message_type {
            DeathMessageType::FallVariants => match knock_off {
                Some(index) => self.fall_message(index, killing_source.causing_entity, names),
                None => localized_death_message(killing_source, names),
            },
            DeathMessageType::IntentionalGameDesign => {
                intentional_game_design_message(killing_source, names)
            }
            DeathMessageType::Default => localized_death_message(killing_source, names),
        }
    }

    /// `CombatTracker.getMostSignificantFall()`, returning the index of the
    /// chosen entry (the entry *before* the fall when one precedes it).
    fn most_significant_fall(&self) -> Option<usize> {
        let mut result: Option<usize> = None;
        let mut alternative: Option<usize> = None;
        let mut alt_damage = 0.0f32;
        let mut best_fall = 0.0f32;

        for (i, entry) in self.entries.iter().enumerate() {
            let source = &entry.source;
            let is_fake_fall = is_always_most_significant_fall(source);
            let fall_distance = if is_fake_fall {
                f32::MAX
            } else {
                entry.fall_distance
            };
            if (is_fall_source(source) || is_fake_fall)
                && fall_distance > 0.0
                && (result.is_none() || fall_distance > best_fall)
            {
                result = Some(if i > 0 { i - 1 } else { i });
                best_fall = fall_distance;
            }

            if entry.fall_location.is_some() && (alternative.is_none() || entry.damage > alt_damage) {
                alternative = Some(i);
                alt_damage = entry.damage;
            }
        }

        if best_fall > 5.0 && result.is_some() {
            result
        } else if alt_damage > 5.0 && alternative.is_some() {
            alternative
        } else {
            None
        }
    }

    /// `CombatTracker.getFallMessage(CombatEntry, Entity)`.
    fn fall_message(
        &self,
        knock_off_index: usize,
        killing_entity: Option<DamageEntityRef>,
        names: &impl DeathMessageSource,
    ) -> Component {
        let knock_off = &self.entries[knock_off_index];
        let knock_off_source = &knock_off.source;
        if !is_fall_source(knock_off_source) && !is_always_most_significant_fall(knock_off_source) {
            let killer_name = killing_entity.map(|entity| names.entity_name(entity));
            let attacker_entity = knock_off_source.causing_entity;
            let attacker_name = attacker_entity.map(|entity| names.entity_name(entity));
            if let Some(attacker_name) = &attacker_name {
                if Some(attacker_name) != killer_name.as_ref() {
                    return message_for_assisted_fall(
                        attacker_entity,
                        attacker_name.clone(),
                        "death.fell.assist.item",
                        "death.fell.assist",
                        names,
                    );
                }
            }
            match killer_name {
                Some(killer_name) => message_for_assisted_fall(
                    killing_entity,
                    killer_name,
                    "death.fell.finish.item",
                    "death.fell.finish",
                    names,
                ),
                None => {
                    Component::translatable("death.fell.killer", vec![arg(names.victim_name())])
                }
            }
        } else {
            let location = knock_off.fall_location.unwrap_or(FallLocation::Generic);
            Component::translatable(location.language_key(), vec![arg(names.victim_name())])
        }
    }

    /// `CombatTracker.recheckStatus`/`recordDamage` clear `entries`; this is the
    /// equivalent of resetting the tracker without an entity to query.
    pub fn reset(&mut self) {
        self.entries.clear();
        self.in_combat = false;
        self.taking_damage = false;
    }
}

/// `CombatTracker.shouldEnterCombat(DamageSource)`: the source's causing entity
/// is a living entity.
fn should_enter_combat(source: &DamageSource) -> bool {
    matches!(source.causing_entity, Some(entity) if entity.is_living)
}

/// Membership of the `minecraft:is_fall` damage type tag.
fn is_fall_source(source: &DamageSource) -> bool {
    matches!(
        source.damage_type.id,
        "minecraft:fall" | "minecraft:ender_pearl" | "minecraft:stalagmite"
    )
}

/// Membership of the `minecraft:always_most_significant_fall` damage type tag.
fn is_always_most_significant_fall(source: &DamageSource) -> bool {
    source.damage_type.id == "minecraft:out_of_world"
}

fn arg(component: Component) -> ComponentArgument {
    ComponentArgument::Component(Box::new(component))
}

/// 1:1 port of `DamageSource.getLocalizedDeathMessage(LivingEntity)`.
fn localized_death_message(source: &DamageSource, names: &impl DeathMessageSource) -> Component {
    let death_msg = format!("death.attack.{}", source.damage_type.message_id);
    if source.causing_entity.is_none() && source.direct_entity.is_none() {
        match names.kill_credit_name() {
            Some(credit) => Component::translatable(
                format!("{death_msg}.player"),
                vec![arg(names.victim_name()), arg(credit)],
            ),
            None => Component::translatable(death_msg, vec![arg(names.victim_name())]),
        }
    } else {
        // name = causingEntity == null ? directEntity : causingEntity
        let name = source
            .causing_entity
            .or(source.direct_entity)
            .map(|entity| names.entity_name(entity))
            .unwrap_or_else(Component::empty);
        // held = causingEntity instanceof LivingEntity ? mainHandItem : EMPTY,
        // and only contributes the ".item" variant when it has a custom name.
        let held = source
            .causing_entity
            .filter(|entity| entity.is_living)
            .and_then(|entity| names.held_item_name(entity));
        match held {
            Some(item) => Component::translatable(
                format!("{death_msg}.item"),
                vec![arg(names.victim_name()), arg(name), arg(item)],
            ),
            None => Component::translatable(death_msg, vec![arg(names.victim_name()), arg(name)]),
        }
    }
}

/// `CombatTracker.getMessageForAssistedFall(...)`.
fn message_for_assisted_fall(
    attacker: Option<DamageEntityRef>,
    attacker_name: Component,
    message_with_item: &str,
    message_without_item: &str,
    names: &impl DeathMessageSource,
) -> Component {
    let item = attacker
        .filter(|entity| entity.is_living)
        .and_then(|entity| names.held_item_name(entity));
    match item {
        Some(item) => Component::translatable(
            message_with_item.to_string(),
            vec![arg(names.victim_name()), arg(attacker_name), arg(item)],
        ),
        None => Component::translatable(
            message_without_item.to_string(),
            vec![arg(names.victim_name()), arg(attacker_name)],
        ),
    }
}

/// The `DeathMessageType.INTENTIONAL_GAME_DESIGN` branch of
/// `CombatTracker.getDeathMessage()`: a clickable bug-tracker link.
fn intentional_game_design_message(
    source: &DamageSource,
    names: &impl DeathMessageSource,
) -> Component {
    let death_msg = format!("death.attack.{}", source.msg_id());
    let link_style = Style::empty()
        .with_click_event(ClickEvent::OpenUrl(
            INTENTIONAL_GAME_DESIGN_BUG_URL.to_string(),
        ))
        .with_hover_event(HoverEvent::Text(Box::new(Component::literal(
            INTENTIONAL_GAME_DESIGN_BUG_ID,
        ))));
    let link = Component::translatable(
        "chat.square_brackets",
        vec![arg(Component::translatable(
            format!("{death_msg}.link"),
            Vec::new(),
        ))],
    )
    .styled(link_style);
    Component::translatable(
        format!("{death_msg}.message"),
        vec![arg(names.victim_name()), arg(link)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::chat_component::ComponentContent;
    use crate::damage_type::{
        arrow_source, bad_respawn_point_source, builtin_damage_type, explosion_source,
        fireball_source, mob_attack_source, player_attack_source, simple_source, DamageSource,
    };
    use std::collections::HashMap;

    /// Stub [`DeathMessageSource`] backed by lookup tables, standing in for the
    /// live entity queries vanilla performs during death-message assembly.
    #[derive(Default)]
    struct TestNames {
        victim: String,
        kill_credit: Option<String>,
        entity_names: HashMap<i32, String>,
        held_items: HashMap<i32, String>,
    }

    impl TestNames {
        fn victim(name: &str) -> Self {
            Self {
                victim: name.to_string(),
                ..Self::default()
            }
        }

        fn with_entity(mut self, id: i32, name: &str) -> Self {
            self.entity_names.insert(id, name.to_string());
            self
        }

        fn with_held_item(mut self, id: i32, name: &str) -> Self {
            self.held_items.insert(id, name.to_string());
            self
        }

        fn with_kill_credit(mut self, name: &str) -> Self {
            self.kill_credit = Some(name.to_string());
            self
        }
    }

    impl DeathMessageSource for TestNames {
        fn victim_name(&self) -> Component {
            Component::literal(self.victim.clone())
        }
        fn kill_credit_name(&self) -> Option<Component> {
            self.kill_credit.clone().map(Component::literal)
        }
        fn entity_name(&self, entity: DamageEntityRef) -> Component {
            Component::literal(
                self.entity_names
                    .get(&entity.id)
                    .cloned()
                    .unwrap_or_else(|| "Entity".to_string()),
            )
        }
        fn held_item_name(&self, entity: DamageEntityRef) -> Option<Component> {
            self.held_items.get(&entity.id).cloned().map(Component::literal)
        }
    }

    /// Returns `(key, [literal args])` for a translatable death-message component.
    fn parts(component: &Component) -> (String, Vec<String>) {
        let ComponentContent::Translatable { key, args, .. } = &component.content else {
            panic!("expected a translatable death message");
        };
        let literals = args
            .iter()
            .map(|arg| match arg {
                ComponentArgument::Component(inner) => match &inner.content {
                    ComponentContent::Literal(text) => text.clone(),
                    ComponentContent::Translatable { key, .. } => format!("<{key}>"),
                    _ => "<arg>".to_string(),
                },
                ComponentArgument::String(value) => value.clone(),
                _ => "<arg>".to_string(),
            })
            .collect();
        (key.clone(), literals)
    }

    fn source(id: &str) -> DamageSource {
        simple_source(id).expect("built-in damage type")
    }

    #[test]
    fn death_message_keys_match_vanilla_localization_for_all_sources() {
        let victim = TestNames::victim("Bob");

        // Environmental, no entity, no kill credit: base key with [victim].
        for (id, key) in [
            ("minecraft:in_fire", "death.attack.inFire"),
            ("minecraft:on_fire", "death.attack.onFire"),
            ("minecraft:in_wall", "death.attack.inWall"),
            ("minecraft:drown", "death.attack.drown"),
            ("minecraft:starve", "death.attack.starve"),
            ("minecraft:wither", "death.attack.wither"),
            ("minecraft:out_of_world", "death.attack.outOfWorld"),
            ("minecraft:cactus", "death.attack.cactus"),
            ("minecraft:lightning_bolt", "death.attack.lightningBolt"),
        ] {
            assert_eq!(
                parts(&localized_death_message(&source(id), &victim)),
                (key.to_string(), vec!["Bob".to_string()]),
                "{id}"
            );
        }

        // No direct/causing entity but kill credit present: the ".player"
        // variant carries [victim, killCredit].
        let credited = TestNames::victim("Bob").with_kill_credit("Alice");
        assert_eq!(
            parts(&localized_death_message(&source("minecraft:drown"), &credited)),
            (
                "death.attack.drown.player".to_string(),
                vec!["Bob".to_string(), "Alice".to_string()]
            )
        );

        // Direct attacker: base key with [victim, attacker], NO ".player".
        let zombie = DamageEntityRef::living_mob(7);
        let names = TestNames::victim("Bob").with_entity(7, "Zombie");
        assert_eq!(
            parts(&localized_death_message(&mob_attack_source(zombie), &names)),
            (
                "death.attack.mob".to_string(),
                vec!["Bob".to_string(), "Zombie".to_string()]
            )
        );

        let steve = DamageEntityRef::player(3, false);
        let names = TestNames::victim("Bob").with_entity(3, "Steve");
        assert_eq!(
            parts(&localized_death_message(&player_attack_source(steve), &names)),
            (
                "death.attack.player".to_string(),
                vec!["Bob".to_string(), "Steve".to_string()]
            )
        );

        // Attacker holding a custom-named item: the ".item" variant carries
        // [victim, attacker, item].
        let names = TestNames::victim("Bob")
            .with_entity(3, "Steve")
            .with_held_item(3, "Excalibur");
        assert_eq!(
            parts(&localized_death_message(&player_attack_source(steve), &names)),
            (
                "death.attack.player.item".to_string(),
                vec![
                    "Bob".to_string(),
                    "Steve".to_string(),
                    "Excalibur".to_string()
                ]
            )
        );

        // Indirect arrow: direct entity is the (non-living) arrow, causing
        // entity is the shooter → base key with [victim, shooter].
        let arrow = DamageEntityRef::non_living(11);
        let shooter = DamageEntityRef::player(3, false);
        let names = TestNames::victim("Bob").with_entity(3, "Steve");
        assert_eq!(
            parts(&localized_death_message(
                &arrow_source(arrow, Some(shooter)),
                &names
            )),
            (
                "death.attack.arrow".to_string(),
                vec!["Bob".to_string(), "Steve".to_string()]
            )
        );

        // Shooter-less arrow: causing entity is null but direct (arrow) is not,
        // so vanilla still uses the base key with the arrow's name — NEVER the
        // ".item" variant, because the arrow is not a living holder.
        let names = TestNames::victim("Bob").with_entity(11, "Arrow");
        let shooterless = DamageSource::indirect(
            builtin_damage_type("minecraft:arrow").expect("arrow type"),
            arrow,
            None,
        );
        assert_eq!(
            parts(&localized_death_message(&shooterless, &names)),
            (
                "death.attack.arrow".to_string(),
                vec!["Bob".to_string(), "Arrow".to_string()]
            )
        );

        // Fireball with a shooter: indirect source keyed by the fireball
        // message id with [victim, shooter].
        let fireball = DamageEntityRef::non_living(12);
        let names = TestNames::victim("Bob").with_entity(7, "Ghast");
        let ghast = DamageEntityRef::living_mob(7);
        assert_eq!(
            parts(&localized_death_message(
                &fireball_source(fireball, Some(ghast)),
                &names
            )),
            (
                "death.attack.fireball".to_string(),
                vec!["Bob".to_string(), "Ghast".to_string()]
            )
        );

        // TNT (primed, no living cause): the explosion damage type with [victim].
        assert_eq!(
            parts(&localized_death_message(&explosion_source(None, None), &victim)),
            ("death.attack.explosion".to_string(), vec!["Bob".to_string()])
        );

        // TNT lit by a player: the player_explosion type (message id
        // "explosion.player") with [victim, igniter].
        let names = TestNames::victim("Bob").with_entity(3, "Steve");
        assert_eq!(
            parts(&localized_death_message(
                &explosion_source(Some(steve), Some(steve)),
                &names
            )),
            (
                "death.attack.explosion.player".to_string(),
                vec!["Bob".to_string(), "Steve".to_string()]
            )
        );

        // Intentional game design (bad respawn point): the linked ".message"
        // key with [victim, <bracketed link>].
        let design = bad_respawn_point_source([0.0, 0.0, 0.0]);
        let (key, args) = parts(&intentional_game_design_message(&design, &victim));
        assert_eq!(key, "death.attack.badRespawnPoint.message");
        assert_eq!(args, vec!["Bob".to_string(), "<chat.square_brackets>".to_string()]);
    }

    fn entry(source: DamageSource, damage: f32) -> CombatEntry {
        CombatEntry::new(source, damage, None, 0.0)
    }

    fn fall_entry(damage: f32, fall_location: Option<FallLocation>, fall_distance: f32) -> CombatEntry {
        CombatEntry::new(source("minecraft:fall"), damage, fall_location, fall_distance)
    }

    #[test]
    fn combat_tracker_records_damage_and_enters_combat_only_for_living_attackers() {
        let mut tracker = CombatTracker::default();
        let zombie = DamageEntityRef::living_mob(7);

        // A living attacker enters combat and stamps the combat start time.
        tracker.record_damage(0, true, entry(mob_attack_source(zombie), 8.0));
        assert_eq!(tracker.entries.len(), 1);
        assert!(tracker.in_combat);
        assert!(tracker.taking_damage);
        assert_eq!(tracker.combat_start_time, 0);

        // Environmental damage alone never starts combat.
        let mut env = CombatTracker::default();
        env.record_damage(5, true, entry(source("minecraft:lava"), 4.0));
        assert!(!env.in_combat);
        assert!(env.taking_damage);
    }

    #[test]
    fn combat_tracker_recheck_status_clears_after_reset_windows() {
        // In combat: status persists until RESET_COMBAT_STATUS_TIME passes.
        let mut tracker = CombatTracker::default();
        let zombie = DamageEntityRef::living_mob(7);
        tracker.record_damage(100, true, entry(mob_attack_source(zombie), 8.0));

        tracker.recheck_status(100 + RESET_COMBAT_STATUS_TIME, true);
        assert!(tracker.in_combat, "still within the in-combat window");

        tracker.recheck_status(100 + RESET_COMBAT_STATUS_TIME + 1, true);
        assert!(!tracker.in_combat, "in-combat window elapsed");
        assert!(!tracker.taking_damage);
        assert!(tracker.entries.is_empty());
        assert_eq!(
            tracker.combat_duration(100 + RESET_COMBAT_STATUS_TIME + 1),
            RESET_COMBAT_STATUS_TIME + 1
        );

        // Out of combat: the shorter RESET_DAMAGE_STATUS_TIME window applies.
        let mut env = CombatTracker::default();
        env.record_damage(0, true, entry(source("minecraft:lava"), 4.0));
        env.recheck_status(RESET_DAMAGE_STATUS_TIME, true);
        assert!(env.taking_damage);
        env.recheck_status(RESET_DAMAGE_STATUS_TIME + 1, true);
        assert!(!env.taking_damage);

        // Death clears status immediately regardless of elapsed ticks.
        let mut dying = CombatTracker::default();
        dying.record_damage(0, true, entry(source("minecraft:lava"), 4.0));
        dying.recheck_status(1, false);
        assert!(!dying.taking_damage);
        assert!(dying.entries.is_empty());
    }

    #[test]
    fn combat_tracker_death_message_handles_empty_and_default_sources() {
        let names = TestNames::victim("Bob").with_entity(7, "Zombie");

        // No entries → generic.
        let empty = CombatTracker::default();
        assert_eq!(
            parts(&empty.death_message(&names)),
            ("death.attack.generic".to_string(), vec!["Bob".to_string()])
        );

        // Default killing blow → localized message for that source.
        let mut tracker = CombatTracker::default();
        let zombie = DamageEntityRef::living_mob(7);
        tracker.record_damage(0, true, entry(mob_attack_source(zombie), 8.0));
        assert_eq!(
            parts(&tracker.death_message(&names)),
            (
                "death.attack.mob".to_string(),
                vec!["Bob".to_string(), "Zombie".to_string()]
            )
        );
    }

    #[test]
    fn combat_tracker_fall_messages_match_vanilla_variants() {
        let names = TestNames::victim("Bob")
            .with_entity(3, "Steve")
            .with_entity(7, "Zombie");

        // Pure fatal fall (> 5 blocks) with no climbable context → generic.
        let mut pure = CombatTracker::default();
        pure.record_damage(0, true, fall_entry(8.0, None, 8.0));
        assert_eq!(
            parts(&pure.death_message(&names)),
            (
                "death.fell.accident.generic".to_string(),
                vec!["Bob".to_string()]
            )
        );

        // Fall off a ladder → the ladder fall-location key.
        let mut ladder = CombatTracker::default();
        ladder.record_damage(0, true, fall_entry(8.0, Some(FallLocation::Ladder), 8.0));
        assert_eq!(
            parts(&ladder.death_message(&names)),
            (
                "death.fell.accident.ladder".to_string(),
                vec!["Bob".to_string()]
            )
        );

        // Knocked off by a mob, then fatal fall → assist message naming the mob.
        let zombie = DamageEntityRef::living_mob(7);
        let mut assisted = CombatTracker::default();
        assisted.record_damage(0, true, entry(mob_attack_source(zombie), 4.0));
        assisted.record_damage(1, true, fall_entry(6.0, None, 10.0));
        assert_eq!(
            parts(&assisted.death_message(&names)),
            (
                "death.fell.assist".to_string(),
                vec!["Bob".to_string(), "Zombie".to_string()]
            )
        );

        // Significant non-fall blow (drown, > 5 damage) with only a tiny fall →
        // the alternative path with no attacker yields "death.fell.killer".
        let mut killer = CombatTracker::default();
        killer.record_damage(
            0,
            true,
            CombatEntry::new(source("minecraft:drown"), 6.0, Some(FallLocation::Water), 0.0),
        );
        killer.record_damage(1, true, fall_entry(2.0, None, 2.0));
        assert_eq!(
            parts(&killer.death_message(&names)),
            ("death.fell.killer".to_string(), vec!["Bob".to_string()])
        );
    }
}
