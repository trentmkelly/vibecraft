//! `net.minecraft.server.PlayerAdvancements`: one player's advancement progress.
//!
//! Java keeps this object on `ServerPlayer`; the player's connection thread owns the
//! player state here, so the object lives in the [`registry`] where the command thread
//! can also reach it (`/advancement grant|revoke`). It is pure state: the side effects
//! Java performs on the `ServerPlayer` (granting rewards, broadcasting the completion
//! message) are queued as [`AdvancementEvent`]s for the owning session, and the packets
//! are returned from [`PlayerAdvancements::flush_dirty`] and
//! [`PlayerAdvancements::set_selected_tab`] for whoever holds the connection.

pub mod registry;
mod persistence;
#[cfg(test)]
mod tests;
pub mod triggers;
pub mod wire;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::advancement_criteria::AdvancementRequirementsModel;
use crate::advancement_progress::JavaAdvancementProgressModel;
use crate::advancement_system::{AdvancementDefinition, CriterionSpec};
use crate::game_rules::SharedGameRules;
use crate::log::{log_error, log_warn};
use crate::network::play::{
    AdvancementProgressData, ClientboundAdvancementsPacket, ClientboundSelectAdvancementsTabPacket,
};
use crate::registry::Identifier;
use crate::server_advancement_manager::ServerAdvancementManagerModel;
use crate::storage::world::WorldLayout;
use persistence::{encode_progress_file, parse_progress_file, LoadError};

/// `GameRules.SHOW_ADVANCEMENT_MESSAGES`.
const SHOW_ADVANCEMENT_MESSAGES: &str = "show_advancement_messages";

/// A side effect of an advancement change that needs the owning player's state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvancementEvent {
    /// `advancement.rewards().grant(player)`: experience, loot, recipes and function.
    GrantRewards(Identifier),
    /// `playerList.broadcastSystemMessage(type.createAnnouncement(holder, player), false)`.
    Announce(Identifier),
}

/// Everything needed to create a player's [`PlayerAdvancements`].
pub struct PlayerAdvancementsConfig {
    /// `ServerAdvancementManager` in force.
    pub manager: Arc<ServerAdvancementManagerModel>,
    /// The world the progress file lives in.
    pub layout: WorldLayout,
    /// The player's UUID (`<uuid>.json`).
    pub uuid: String,
    /// Read for `show_advancement_messages` when an advancement completes.
    pub game_rules: SharedGameRules,
}

/// `PlayerAdvancements`.
pub struct PlayerAdvancements {
    manager: Arc<ServerAdvancementManagerModel>,
    layout: WorldLayout,
    uuid: String,
    game_rules: SharedGameRules,
    /// `progress`: a `LinkedHashMap`, so `order` records insertion.
    progress: BTreeMap<Identifier, JavaAdvancementProgressModel>,
    order: Vec<Identifier>,
    visible: BTreeSet<Identifier>,
    progress_changed: BTreeSet<Identifier>,
    roots_to_update: BTreeSet<Identifier>,
    last_selected_tab: Option<Identifier>,
    is_first_packet: bool,
    /// The trigger listeners `CriterionTrigger.addPlayerListener` registers, per trigger
    /// type: `(advancement, criterion)` pairs still waiting to be obtained.
    listeners: BTreeMap<Identifier, BTreeSet<(Identifier, String)>>,
    events: Vec<AdvancementEvent>,
    /// Set when the file could not be read for a reason other than being absent, so
    /// [`Self::save`] does not overwrite data this server failed to understand.
    save_blocked: bool,
}

impl PlayerAdvancements {
    /// The constructor: `this.load(manager)`.
    pub fn new(config: PlayerAdvancementsConfig) -> Self {
        let mut advancements = Self {
            manager: config.manager,
            layout: config.layout,
            uuid: config.uuid,
            game_rules: config.game_rules,
            progress: BTreeMap::new(),
            order: Vec::new(),
            visible: BTreeSet::new(),
            progress_changed: BTreeSet::new(),
            roots_to_update: BTreeSet::new(),
            last_selected_tab: None,
            is_first_packet: true,
            listeners: BTreeMap::new(),
            events: Vec::new(),
            save_blocked: false,
        };
        advancements.load();
        advancements
    }

    /// The manager this progress is validated against.
    pub fn manager(&self) -> &Arc<ServerAdvancementManagerModel> {
        &self.manager
    }

    /// `reload(manager)`: forget everything and load again against the new manager.
    pub fn reload(&mut self, manager: Arc<ServerAdvancementManagerModel>) {
        self.stop_listening();
        self.progress.clear();
        self.order.clear();
        self.visible.clear();
        self.roots_to_update.clear();
        self.progress_changed.clear();
        self.is_first_packet = true;
        self.last_selected_tab = None;
        self.manager = manager;
        self.load();
    }

    /// `stopListening()`: remove every trigger listener.
    pub fn stop_listening(&mut self) {
        self.listeners.clear();
    }

    /// Drains the queued player-side effects, oldest first.
    pub fn take_events(&mut self) -> Vec<AdvancementEvent> {
        std::mem::take(&mut self.events)
    }

    fn definition(&self, id: &Identifier) -> Option<&AdvancementDefinition> {
        self.manager.get(id).map(|holder| holder.value())
    }

    /// `checkForAutomaticTriggers`: an advancement without criteria completes on load.
    fn check_for_automatic_triggers(&mut self) {
        let automatic: Vec<Identifier> = self
            .manager
            .get_all_advancements()
            .filter(|holder| holder.value().criteria.is_empty())
            .map(|holder| holder.value().id.clone())
            .collect();
        for id in automatic {
            self.award(&id, "");
            self.events.push(AdvancementEvent::GrantRewards(id));
        }
    }

    /// `load(manager)`: read the file, then start the automatic advancements and the
    /// trigger listeners.
    fn load(&mut self) {
        let path = self.layout.advancements_file(&self.uuid);
        if path.is_file() {
            match std::fs::read_to_string(&path) {
                Ok(json) => match parse_progress_file(&json) {
                    Ok(entries) => self.apply_from(entries),
                    Err(LoadError::Parse(err)) => log_error(&format!(
                        "Couldn't parse player advancements in {}: {err}",
                        path.display()
                    )),
                    Err(LoadError::NeedsDataFix { found }) => {
                        self.save_blocked = true;
                        log_error(&format!(
                            "Couldn't parse player advancements in {}: DataVersion {found} needs an upgrade this server cannot perform",
                            path.display()
                        ));
                    }
                },
                Err(err) => {
                    self.save_blocked = true;
                    log_error(&format!(
                        "Couldn't access player advancements in {}: {err}",
                        path.display()
                    ));
                }
            }
        }
        self.check_for_automatic_triggers();
        self.register_all_listeners();
    }

    /// `applyFrom`: start the saved progress of every advancement that still exists.
    fn apply_from(&mut self, entries: persistence::ProgressEntries) {
        for (id, progress) in entries {
            if self.definition(&id).is_none() {
                log_warn(&format!(
                    "Ignored advancement '{id}' in progress file {} - it doesn't exist anymore?",
                    self.layout.advancements_file(&self.uuid).display()
                ));
                continue;
            }
            self.start_progress(&id, progress);
            self.progress_changed.insert(id.clone());
            self.mark_for_visibility_update(&id);
        }
    }

    /// `save()`: write the progress of every advancement that has any.
    pub fn save(&self) {
        if self.save_blocked {
            return;
        }
        let entries = self.order.iter().filter_map(|id| {
            let progress = self.progress.get(id)?;
            progress.has_progress().then_some((id, progress))
        });
        let json = encode_progress_file(entries);
        let path = self.layout.advancements_file(&self.uuid);
        if let Err(err) = self.layout.save_json_sidecar(path.clone(), &json) {
            log_error(&format!(
                "Couldn't save player advancements to {}: {err}",
                path.display()
            ));
        }
    }

    /// `award(holder, criterion)`: obtain `criterion`; returns whether it was newly obtained.
    pub fn award(&mut self, id: &Identifier, criterion: &str) -> bool {
        self.award_at(id, criterion, now_epoch_millis())
    }

    /// [`Self::award`] with an explicit obtained time.
    pub fn award_at(&mut self, id: &Identifier, criterion: &str, epoch_millis: i64) -> bool {
        if self.definition(id).is_none() {
            return false;
        }
        let mut result = false;
        self.ensure_progress(id);
        let (was_done, granted, now_done) = {
            let progress = self.progress_mut(id);
            let was_done = progress.is_done();
            let granted = progress.grant_progress_at_epoch_millis(criterion, epoch_millis);
            (was_done, granted, progress.is_done())
        };
        if granted {
            self.unregister_listeners(id);
            self.progress_changed.insert(id.clone());
            result = true;
            if !was_done && now_done {
                self.on_completed(id);
            }
        }
        if !was_done && now_done {
            self.mark_for_visibility_update(id);
        }
        result
    }

    /// The `!wasDone && isDone` block of `award`: rewards, then the announcement.
    fn on_completed(&mut self, id: &Identifier) {
        self.events.push(AdvancementEvent::GrantRewards(id.clone()));
        let announces = self
            .definition(id)
            .and_then(|definition| definition.display.as_ref())
            .is_some_and(|display| display.announce_chat);
        if announces && self.show_advancement_messages() {
            self.events.push(AdvancementEvent::Announce(id.clone()));
        }
    }

    fn show_advancement_messages(&self) -> bool {
        self.game_rules
            .lock()
            .map(|rules| rules.bool(SHOW_ADVANCEMENT_MESSAGES))
            .unwrap_or_else(|poisoned| poisoned.into_inner().bool(SHOW_ADVANCEMENT_MESSAGES))
    }

    /// `revoke(holder, criterion)`: un-obtain `criterion`; returns whether it changed.
    pub fn revoke(&mut self, id: &Identifier, criterion: &str) -> bool {
        if self.definition(id).is_none() {
            return false;
        }
        let mut result = false;
        self.ensure_progress(id);
        let (was_done, revoked, now_done) = {
            let progress = self.progress_mut(id);
            let was_done = progress.is_done();
            let revoked = progress.revoke_progress(criterion);
            (was_done, revoked, progress.is_done())
        };
        if revoked {
            self.register_listeners(id);
            self.progress_changed.insert(id.clone());
            result = true;
        }
        if was_done && !now_done {
            self.mark_for_visibility_update(id);
        }
        result
    }

    fn mark_for_visibility_update(&mut self, id: &Identifier) {
        if let Some(node) = self.manager.tree().get(id) {
            self.roots_to_update
                .insert(node.root(self.manager.tree()).id().clone());
        }
    }

    /// `getOrStartProgress(holder)` for an advancement known to the manager.
    pub fn get_or_start_progress(&mut self, id: &Identifier) -> Option<&JavaAdvancementProgressModel> {
        self.definition(id)?;
        self.ensure_progress(id);
        self.progress.get(id)
    }

    fn ensure_progress(&mut self, id: &Identifier) {
        if !self.progress.contains_key(id) {
            self.start_progress(id, JavaAdvancementProgressModel::new());
        }
    }

    fn progress_mut(&mut self, id: &Identifier) -> &mut JavaAdvancementProgressModel {
        self.progress
            .get_mut(id)
            .unwrap_or_else(|| panic!("progress of {id} was started by ensure_progress"))
    }

    /// `startProgress(holder, progress)`.
    fn start_progress(&mut self, id: &Identifier, mut progress: JavaAdvancementProgressModel) {
        if let Some(definition) = self.definition(id) {
            progress.update(AdvancementRequirementsModel::new(definition.requirements.clone()));
        }
        if self.progress.insert(id.clone(), progress).is_none() {
            self.order.push(id.clone());
        }
    }

    fn register_all_listeners(&mut self) {
        let ids: Vec<Identifier> = self
            .manager
            .get_all_advancements()
            .map(|holder| holder.value().id.clone())
            .collect();
        for id in ids {
            self.register_listeners(&id);
        }
    }

    /// `registerListeners(holder)`: listen for every criterion not yet obtained.
    fn register_listeners(&mut self, id: &Identifier) {
        let Some(progress) = self.get_or_start_progress(id) else {
            return;
        };
        if progress.is_done() {
            return;
        }
        let pending: Vec<String> = progress.get_remaining_criteria();
        let Some(definition) = self.definition(id) else {
            return;
        };
        let registrations: Vec<(Identifier, String)> = pending
            .into_iter()
            .filter_map(|name| {
                let spec = definition.criterion_specs.get(&name)?;
                Some((spec.trigger.clone(), name))
            })
            .collect();
        for (trigger, name) in registrations {
            self.listeners
                .entry(trigger)
                .or_default()
                .insert((id.clone(), name));
        }
    }

    /// `unregisterListeners(holder)`: drop listeners of obtained criteria (all of them
    /// once the advancement is done).
    fn unregister_listeners(&mut self, id: &Identifier) {
        let Some(progress) = self.progress.get(id) else {
            return;
        };
        let advancement_done = progress.is_done();
        let Some(definition) = self.definition(id) else {
            return;
        };
        let removals: Vec<(Identifier, String)> = definition
            .criterion_specs
            .iter()
            .filter(|(name, _)| {
                progress
                    .get_criterion(name)
                    .is_some_and(|criterion| criterion.is_done() || advancement_done)
            })
            .map(|(name, spec)| (spec.trigger.clone(), name.clone()))
            .collect();
        for (trigger, name) in removals {
            if let Some(listeners) = self.listeners.get_mut(&trigger) {
                listeners.remove(&(id.clone(), name));
            }
        }
    }

    /// `SimpleCriterionTrigger.trigger(player, predicate)`: awards every waiting
    /// criterion of `trigger` whose condition `matches`.
    pub fn trigger(&mut self, trigger: &Identifier, matches: impl Fn(&CriterionSpec) -> bool) {
        let candidates: Vec<(Identifier, String)> = match self.listeners.get(trigger) {
            Some(listeners) => listeners.iter().cloned().collect(),
            None => return,
        };
        let matched: Vec<(Identifier, String)> = candidates
            .into_iter()
            .filter(|(id, name)| {
                self.definition(id)
                    .and_then(|definition| definition.criterion_specs.get(name))
                    .is_some_and(&matches)
            })
            .collect();
        for (id, name) in matched {
            self.award(&id, &name);
        }
    }

    /// `flushDirty(player, showAdvancements)`: the packet describing what changed since
    /// the last flush, if anything did. Every flush after the first is incremental.
    pub fn flush_dirty(&mut self, show_advancements: bool) -> Option<ClientboundAdvancementsPacket> {
        let mut packet = None;
        if self.is_first_packet
            || !self.roots_to_update.is_empty()
            || !self.progress_changed.is_empty()
        {
            let mut added = BTreeSet::new();
            let mut removed = BTreeSet::new();
            for root in std::mem::take(&mut self.roots_to_update) {
                self.update_tree_visibility(&root, &mut added, &mut removed);
            }
            let changed = std::mem::take(&mut self.progress_changed);
            let progress: Vec<(Identifier, AdvancementProgressData)> = changed
                .into_iter()
                .filter(|id| self.visible.contains(id))
                .filter_map(|id| {
                    let data = self.progress.get(&id)?.to_network_data();
                    Some((id, data))
                })
                .collect();
            if !progress.is_empty() || !added.is_empty() || !removed.is_empty() {
                packet = Some(ClientboundAdvancementsPacket {
                    reset: self.is_first_packet,
                    added: self.holder_data(&added),
                    removed: removed.into_iter().collect(),
                    progress,
                    show_advancements,
                });
            }
        }
        self.is_first_packet = false;
        packet
    }

    /// The wire form of `added`; an advancement that cannot be encoded is left out
    /// with a warning (its progress is still tracked).
    fn holder_data(&self, added: &BTreeSet<Identifier>) -> Vec<crate::network::play::AdvancementHolderData> {
        added
            .iter()
            .filter_map(|id| {
                let definition = self.definition(id)?;
                match wire::holder_data(definition) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        log_warn(&format!("Couldn't send advancement {id}: {err}"));
                        None
                    }
                }
            })
            .collect()
    }

    /// `updateTreeVisibility(root, added, removed)`.
    fn update_tree_visibility(
        &mut self,
        root: &Identifier,
        added: &mut BTreeSet<Identifier>,
        removed: &mut BTreeSet<Identifier>,
    ) {
        let decisions = self.evaluate_visibility(root);
        for (id, should_be_visible) in decisions {
            if should_be_visible {
                if self.visible.insert(id.clone()) {
                    added.insert(id.clone());
                    if self.progress.contains_key(&id) {
                        self.progress_changed.insert(id);
                    }
                }
            } else if self.visible.remove(&id) {
                removed.insert(id);
            }
        }
    }

    /// `AdvancementVisibilityEvaluator.evaluateVisibility` over `root`'s tree, with
    /// `getOrStartProgress(node).isDone()` as the completion test.
    fn evaluate_visibility(&mut self, root: &Identifier) -> Vec<(Identifier, bool)> {
        use crate::advancement_visibility_evaluator::{
            evaluate_advancement_visibility, AdvancementVisibilityDisplay,
            AdvancementVisibilityTree,
        };
        let ids = self.subtree_parents_first(root);
        let mut tree = AdvancementVisibilityTree::default();
        for (id, parent) in &ids {
            let display = self
                .definition(id)
                .and_then(|definition| definition.display.as_ref())
                .map(|display| AdvancementVisibilityDisplay {
                    hidden: display.hidden,
                });
            tree.insert(id.clone(), parent.clone(), display);
        }
        let done: BTreeSet<Identifier> = ids
            .iter()
            .filter(|(id, _)| {
                self.get_or_start_progress(id)
                    .is_some_and(JavaAdvancementProgressModel::is_done)
            })
            .map(|(id, _)| id.clone())
            .collect();
        evaluate_advancement_visibility(&tree, root, &done)
    }

    /// Every node under `root` (inclusive) as `(id, parent)`, parents before children.
    fn subtree_parents_first(&self, root: &Identifier) -> Vec<(Identifier, Option<Identifier>)> {
        let tree = self.manager.tree();
        let mut out = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(id) = stack.pop() {
            let Some(node) = tree.get(&id) else { continue };
            out.push((id, node.parent().cloned()));
            let children: Vec<Identifier> = node.children().cloned().collect();
            stack.extend(children.into_iter().rev());
        }
        out
    }

    /// `setSelectedTab(holder)`: the tab packet to send when the selection changed.
    pub fn set_selected_tab(
        &mut self,
        tab: Option<&Identifier>,
    ) -> Option<ClientboundSelectAdvancementsTabPacket> {
        let old = self.last_selected_tab.clone();
        self.last_selected_tab = tab
            .filter(|id| {
                self.definition(id).is_some_and(|definition| {
                    definition.parent.is_none() && definition.display.is_some()
                })
            })
            .cloned();
        (old != self.last_selected_tab).then(|| ClientboundSelectAdvancementsTabPacket {
            tab: self.last_selected_tab.clone(),
        })
    }

    /// Every advancement with obtained criteria, and those criteria (sorted by name).
    pub fn obtained_criteria(&self) -> Vec<(Identifier, Vec<String>)> {
        self.progress
            .iter()
            .filter(|(_, progress)| progress.has_progress())
            .map(|(id, progress)| (id.clone(), progress.get_completed_criteria()))
            .collect()
    }

    /// The tab last selected through [`Self::set_selected_tab`].
    #[cfg(test)]
    pub fn selected_tab(&self) -> Option<&Identifier> {
        self.last_selected_tab.as_ref()
    }

    /// Whether the client currently knows about `id`.
    #[cfg(test)]
    pub fn is_visible(&self, id: &Identifier) -> bool {
        self.visible.contains(id)
    }

    /// Whether `id`'s criterion `name` is still waiting for `trigger`.
    #[cfg(test)]
    pub fn is_listening(&self, trigger: &Identifier, id: &Identifier, name: &str) -> bool {
        self.listeners
            .get(trigger)
            .is_some_and(|listeners| listeners.contains(&(id.clone(), name.to_string())))
    }
}

/// `Instant.now()` as epoch milliseconds (`CriterionProgress.grant`).
fn now_epoch_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
