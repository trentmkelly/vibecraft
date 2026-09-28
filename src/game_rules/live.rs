//! The live, server-wide [`GameRules`] store shared by every connection and the tick thread.
//!
//! Java keeps one `GameRules` on the overworld (`MinecraftServer.getGlobalGameRules`) and
//! fans every `GameRules.set` out through `MinecraftServer.onGameRuleChanged`, which sends
//! packets to *all* players. VibeCraft sessions run on separate threads with no cross-player
//! send path, so each `set` is appended to a bounded change log and every session replays
//! the entries it has not seen yet on its own tick (the same pull model used for weather).

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::{GameRuleDefinition, GameRuleError, GameRuleSync, GameRuleValue, GameRules};

/// Maximum retained change-log entries; sessions poll every tick so this is ample.
const CHANGE_LOG_CAPACITY: usize = 512;

/// One `GameRules.set` call, replayed per session as `MinecraftServer.onGameRuleChanged`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRuleChange {
    /// Monotonic sequence number (first change is `1`).
    pub seq: u64,
    /// Short rule id (`keep_inventory`).
    pub rule: &'static str,
    pub value: GameRuleValue,
    /// `ServerGamePacketListenerImpl.broadcastGameRuleChangeToOperators`: every operator
    /// receives the `commands.gamerule.set` system message (set via `ServerboundSetGameRulePacket`).
    pub announce_to_operators: bool,
    /// `false` when `GameRules.set` ignored the call because the rule is disabled by the
    /// world's feature flags (it only logs); the operator announcement still happens.
    pub applied: bool,
}

/// Shared handle used by the status runtime.
pub type SharedGameRules = Arc<Mutex<LiveGameRules>>;

/// [`GameRules`] plus the change log and persistence dirty flag.
#[derive(Debug, Clone)]
pub struct LiveGameRules {
    rules: GameRules,
    next_seq: u64,
    changes: VecDeque<GameRuleChange>,
    dirty: bool,
}

impl LiveGameRules {
    pub fn new(rules: GameRules) -> Self {
        Self {
            rules,
            next_seq: 1,
            changes: VecDeque::new(),
            dirty: false,
        }
    }

    pub fn shared(rules: GameRules) -> SharedGameRules {
        Arc::new(Mutex::new(Self::new(rules)))
    }

    /// Read access to the current values.
    pub fn rules(&self) -> &GameRules {
        &self.rules
    }

    pub fn get(&self, name: &str) -> Option<GameRuleValue> {
        self.rules.get(name)
    }

    /// Current boolean value of an enabled rule (`GameRules.get`); disabled rules read `false`.
    pub fn bool(&self, name: &str) -> bool {
        matches!(self.rules.get(name), Some(GameRuleValue::Bool(true)))
    }

    /// `GameRules.set`: parses `raw` with the rule's argument type, stores it, and records the
    /// change so every session replays `onGameRuleChanged`.
    pub fn set(
        &mut self,
        name: &str,
        raw: &str,
        announce_to_operators: bool,
    ) -> Result<GameRuleSync, GameRuleError> {
        let sync = self.rules.set(name, raw)?;
        let short = sync.rule.strip_prefix("minecraft:").unwrap_or(&sync.rule);
        let definition = super::game_rule_definition(short).ok_or(GameRuleError::UnknownRule)?;
        let value = self.rules.get(short).ok_or(GameRuleError::UnknownRule)?;
        self.record(definition, value, announce_to_operators, true);
        Ok(sync)
    }

    /// Typed `GameRules.set` (used by the `show_advancement_messages` startup override).
    pub fn set_value(
        &mut self,
        definition: &'static GameRuleDefinition,
        value: GameRuleValue,
    ) -> GameRuleSync {
        let sync = self.rules.set_value(definition, value);
        self.record(definition, value, false, true);
        sync
    }

    /// `ServerGamePacketListenerImpl.setGameRuleValue`: `GameRules.set` followed by the
    /// unconditional operator announcement. A rule that is not enabled is left untouched
    /// (`GameRules.set` only logs a warning) but is still announced.
    pub fn set_from_client(&mut self, definition: &'static GameRuleDefinition, value: GameRuleValue) {
        if self.rules.get(definition.name).is_some() {
            self.set_value(definition, value);
            if let Some(last) = self.changes.back_mut() {
                last.announce_to_operators = true;
            }
        } else {
            self.record(definition, value, true, false);
        }
    }

    fn record(
        &mut self,
        definition: &'static GameRuleDefinition,
        value: GameRuleValue,
        announce: bool,
        applied: bool,
    ) {
        self.dirty = applied || self.dirty;
        if self.changes.len() == CHANGE_LOG_CAPACITY {
            self.changes.pop_front();
        }
        self.changes.push_back(GameRuleChange {
            seq: self.next_seq,
            rule: definition.name,
            value,
            announce_to_operators: announce,
            applied,
        });
        self.next_seq += 1;
    }

    /// Sequence number a newly joined session starts from: it sees only later changes.
    pub fn current_seq(&self) -> u64 {
        self.next_seq - 1
    }

    /// Changes newer than `after_seq`, oldest first.
    pub fn changes_since(&self, after_seq: u64) -> Vec<GameRuleChange> {
        self.changes
            .iter()
            .filter(|change| change.seq > after_seq)
            .cloned()
            .collect()
    }

    /// Returns and clears the "needs saving" flag (`SavedData.isDirty`).
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }
}
