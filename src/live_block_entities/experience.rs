//! Experience that block entities award, waiting for the world ticker to spawn
//! the orbs.
//!
//! Java's `AbstractFurnaceBlockEntity.awardUsedRecipesAndPopExperience` calls
//! `ExperienceOrb.award(level, player.position(), amount)` straight from the
//! menu's result slot. VibeCraft's menus run inside per-connection sessions that
//! do not own the world's entity store, so the session records the award here
//! and the world ticker (which owns the entity store and the packet bus) spawns
//! the orbs on the same tick.

use std::sync::Mutex;

/// One `ExperienceOrb.award(level, pos, amount)` call.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExperienceAward {
    pub pos: (f64, f64, f64),
    pub amount: i32,
}

/// The awards recorded since the ticker last drained them.
#[derive(Debug, Default)]
pub struct ExperienceAwards {
    pending: Mutex<Vec<ExperienceAward>>,
}

impl ExperienceAwards {
    /// Records an award; non-positive amounts spawn nothing and are dropped.
    pub fn award(&self, pos: (f64, f64, f64), amount: i32) {
        if amount > 0 {
            self.pending
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(ExperienceAward { pos, amount });
        }
    }

    /// Takes every pending award.
    pub fn drain(&self) -> Vec<ExperienceAward> {
        std::mem::take(&mut *self.pending.lock().unwrap_or_else(|e| e.into_inner()))
    }
}
