//! The online players' [`PlayerAdvancements`], keyed by UUID.
//!
//! Java reaches a player's advancements through `ServerPlayer.getAdvancements()` from any
//! code that has the player (commands, triggers). VibeCraft's players live on their own
//! connection threads, so the advancement state is shared through this registry: the
//! session inserts it on join and removes it on quit, and `/advancement` looks players up
//! by UUID.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use super::PlayerAdvancements;

/// A shared handle to one player's advancements.
pub type SharedPlayerAdvancements = Arc<Mutex<PlayerAdvancements>>;

/// Online players' advancements.
#[derive(Default)]
pub struct AdvancementRegistry {
    players: Mutex<HashMap<String, SharedPlayerAdvancements>>,
}

impl AdvancementRegistry {
    /// The process-wide registry every session and command shares.
    pub fn global() -> &'static AdvancementRegistry {
        static GLOBAL: OnceLock<AdvancementRegistry> = OnceLock::new();
        GLOBAL.get_or_init(AdvancementRegistry::default)
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, SharedPlayerAdvancements>> {
        // A poisoned map only means another session panicked mid-insert.
        self.players.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Registers (or replaces) the advancements of the player with `uuid`.
    pub fn insert(&self, uuid: &str, advancements: PlayerAdvancements) -> SharedPlayerAdvancements {
        let shared = Arc::new(Mutex::new(advancements));
        self.lock().insert(uuid.to_string(), Arc::clone(&shared));
        shared
    }

    /// The advancements of the online player with `uuid`.
    pub fn get(&self, uuid: &str) -> Option<SharedPlayerAdvancements> {
        self.lock().get(uuid).cloned()
    }

    /// Forgets the advancements only if `expected` is still the registered handle (a
    /// login that replaced this session registered its own).
    pub fn remove_if_same(&self, uuid: &str, expected: &SharedPlayerAdvancements) {
        let mut players = self.lock();
        if players
            .get(uuid)
            .is_some_and(|current| Arc::ptr_eq(current, expected))
        {
            players.remove(uuid);
        }
    }
}

/// Locks a player's advancements, recovering from a poisoned lock.
pub fn lock_advancements(shared: &SharedPlayerAdvancements) -> MutexGuard<'_, PlayerAdvancements> {
    shared.lock().unwrap_or_else(|e| e.into_inner())
}
