//! The shared registry behind `ContainerOpenersCounter`: which players have a
//! container block's menu open.
//!
//! Java calls `Container.startOpen` when a menu is constructed and `stopOpen`
//! from `AbstractContainerMenu.removed`, on the server thread. VibeCraft's menus
//! live in per-connection sessions, so a session instead records a start with
//! [`ContainerOpeners::start_open`] and gets an [`OpenGuard`] back; dropping the
//! guard (menu closed, replaced by another menu, player respawned or
//! disconnected) records the stop. The world ticker drains those events once per
//! tick ([`ContainerOpeners::drain_events`]) and applies the counter effects in
//! [`super::open_effects`].
//!
//! The registry hangs off
//! [`GeneratedChunkCache`](crate::network::status::GeneratedChunkCache), the
//! world's block-entity store, so every session and the ticker already share it.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::block_update::BlockPos;

/// A menu start or stop, in the order the sessions performed them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpenEvent {
    /// `Container.startOpen(user)` with the user's `getContainerInteractionRange`.
    Start {
        pos: BlockPos,
        user: u64,
        range: f64,
    },
    /// `Container.stopOpen(user)`.
    Stop { pos: BlockPos, user: u64 },
}

/// A player currently holding a container open.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpenUser {
    pub user: u64,
    pub pos: BlockPos,
    pub range: f64,
}

#[derive(Debug, Default)]
struct Inner {
    users: Vec<OpenUser>,
    events: VecDeque<OpenEvent>,
}

/// See the module documentation.
#[derive(Debug, Default)]
pub struct ContainerOpeners {
    inner: Mutex<Inner>,
    next_user: AtomicU64,
}

impl ContainerOpeners {
    /// Records that a new user opened the container at `pos`.
    pub fn start_open(self: &Arc<Self>, pos: BlockPos, range: f64) -> OpenGuard {
        let user = self.next_user.fetch_add(1, Ordering::Relaxed);
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.users.push(OpenUser { user, pos, range });
        inner.events.push_back(OpenEvent::Start { pos, user, range });
        drop(inner);
        OpenGuard(Arc::new(GuardInner {
            openers: Arc::clone(self),
            user,
            pos,
        }))
    }

    fn stop_open(&self, user: u64, pos: BlockPos) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.users.retain(|open| open.user != user);
        inner.events.push_back(OpenEvent::Stop { pos, user });
    }

    /// The starts and stops recorded since the last call.
    pub fn drain_events(&self) -> Vec<OpenEvent> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.events.drain(..).collect()
    }

    /// Users currently holding the container at `pos` open
    /// (`ContainerOpenersCounter.getEntitiesWithContainerOpen`).
    pub fn users_at(&self, pos: BlockPos) -> Vec<OpenUser> {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner
            .users
            .iter()
            .filter(|open| open.pos == pos)
            .copied()
            .collect()
    }
}

#[derive(Debug)]
struct GuardInner {
    openers: Arc<ContainerOpeners>,
    user: u64,
    pos: BlockPos,
}

impl Drop for GuardInner {
    fn drop(&mut self) {
        self.openers.stop_open(self.user, self.pos);
    }
}

/// Keeps one user's open container registered; dropping the last clone is
/// `Container.stopOpen`.
#[derive(Debug, Clone)]
pub struct OpenGuard(Arc<GuardInner>);

impl PartialEq for OpenGuard {
    fn eq(&self, other: &Self) -> bool {
        self.0.user == other.0.user
    }
}
