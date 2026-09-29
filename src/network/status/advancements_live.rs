//! Live wiring of `PlayerAdvancements` into the play session.
//!
//! Java gives every `ServerPlayer` a `PlayerAdvancements` (created by
//! `PlayerList.getPlayerAdvancements`, saved by `PlayerList.save`, reloaded by
//! `PlayerList.reloadResources`) whose `flushDirty` runs at the end of
//! `ServerPlayer.tick`. This module does the same for a connection thread:
//!
//! * [`begin_session`] loads the player's `players/advancements/<uuid>.json` and
//!   registers it so `/advancement` can reach it; the returned guard saves and
//!   unregisters on any way out of the session,
//! * [`tick_player_advancements`] raises the triggers whose events exist
//!   (`inventory_changed` through the `broadcastChanges` slot diff, `recipe_unlocked`,
//!   `tick`), runs the queued reward/announcement side effects, detects a datapack
//!   `/reload` and sends the `ClientboundUpdateAdvancementsPacket`,
//! * [`try_handle_seen_advancements`] is `ServerGamePacketListenerImpl.handleSeenAdvancements`.
//!
//! Triggers whose events the server cannot raise yet:
//! TODO(advancement-trigger-events): `location`/`tick`-with-predicate (needs the location
//! predicate), `player_killed_entity`, `entity_hurt_player`, `player_hurt_entity`,
//! `killed_by_arrow` (mob combat has no kill event), `consume_item`/`using_item`/
//! `filled_bucket`/`brewed_potion`/`enchanted_item` (no item-use completion events),
//! `recipe_crafted` (crafting results carry no trigger), `placed_block`/`item_used_on_block`
//! (need the loot-condition `location` predicate), `changed_dimension`/`nether_travel`
//! (no dimension change), `bred_animals`/`tame_animal`/`summoned_entity`/`villager_trade`/
//! `effects_changed` and the remaining entity-driven triggers.

mod command;
mod rewards;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::io::{self, Cursor};
use std::sync::Arc;

use super::*;
use crate::game_rules::SharedGameRules;
use crate::network::play::{
    ServerboundSeenAdvancementsAction, ServerboundSeenAdvancementsPacket,
    CLIENTBOUND_SELECT_ADVANCEMENTS_TAB_PACKET_ID, CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID,
};
use crate::player_advancements::registry::{
    lock_advancements, AdvancementRegistry, SharedPlayerAdvancements,
};
use crate::player_advancements::triggers::{
    inventory_changed_matches, player_trigger_matches, recipe_unlocked_matches, BuiltinItemTags,
    TriggerEnvironment,
};
use crate::player_advancements::{PlayerAdvancements, PlayerAdvancementsConfig};
use crate::registry_pipeline::server_resources::{active_resources, LoadedResources};
pub(super) use command::{
    apply_command_advancements, apply_command_advancements_for, seed_command_advancements,
};
use rewards::RewardContext;

/// The player inventory slots in the order `InventoryMenu` lists them (armor head to
/// feet, main inventory, hotbar, offhand), which is the order `broadcastChanges` walks.
const MENU_ORDER: [usize; 4 + 27 + 9 + 1] = menu_order();

const fn menu_order() -> [usize; 41] {
    let mut order = [0; 41];
    // Armor slots 5..=8: head, chest, legs, feet = inventory slots 39..=36.
    let mut i = 0;
    while i < 4 {
        order[i] = 39 - i;
        i += 1;
    }
    // Main inventory slots 9..=35.
    while i < 31 {
        order[i] = 9 + (i - 4);
        i += 1;
    }
    // Hotbar 36..=44 = inventory slots 0..=8.
    while i < 40 {
        order[i] = i - 31;
        i += 1;
    }
    // Offhand 45 = inventory slot 40.
    order[40] = 40;
    order
}

/// How many reward/trigger passes one tick may chain (a reward can unlock a recipe whose
/// trigger completes another advancement, and so on).
const MAX_SETTLE_PASSES: usize = 32;

/// Per-connection state the tick keeps between calls.
struct Session {
    shared: SharedPlayerAdvancements,
    resources: Arc<LoadedResources>,
    /// `AbstractContainerMenu.lastSlots`, by player inventory slot.
    last_slots: Vec<ItemStack>,
}

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
}

/// Saves and unregisters the player's advancements when the session ends
/// (`PlayerList.remove`: `save`, `stopListening`, `advancements.remove`).
pub(super) struct AdvancementSessionGuard {
    uuid: String,
    shared: SharedPlayerAdvancements,
}

impl Drop for AdvancementSessionGuard {
    fn drop(&mut self) {
        {
            let mut advancements = lock_advancements(&self.shared);
            advancements.save();
            advancements.stop_listening();
        }
        AdvancementRegistry::global().remove_if_same(&self.uuid, &self.shared);
        SESSION.with(|session| *session.borrow_mut() = None);
    }
}

/// `PlayerList.getPlayerAdvancements(player)`: loads the progress and registers it.
pub(super) fn begin_session(
    profile: &NameAndId,
    layout: &WorldLayout,
    game_rules: &SharedGameRules,
) -> io::Result<AdvancementSessionGuard> {
    let resources = active_resources().map_err(io::Error::other)?;
    let advancements = PlayerAdvancements::new(PlayerAdvancementsConfig {
        manager: Arc::clone(&resources.content.advancements),
        layout: layout.clone(),
        uuid: profile.uuid.clone(),
        game_rules: Arc::clone(game_rules),
    });
    let shared = AdvancementRegistry::global().insert(&profile.uuid, advancements);
    SESSION.with(|session| {
        *session.borrow_mut() = Some(Session {
            shared: Arc::clone(&shared),
            resources,
            last_slots: vec![ItemStack::empty(); MENU_ORDER.len() + 8],
        });
    });
    Ok(AdvancementSessionGuard {
        uuid: profile.uuid.clone(),
        shared,
    })
}

/// What one tick needs from the connection.
pub(super) struct AdvancementTickContext<'a> {
    pub profile: &'a NameAndId,
    pub bus: &'a WorldPacketBus,
    pub world_items: &'a Arc<Mutex<WorldItemEntities>>,
}

/// The advancement part of `ServerPlayer.tick`: inventory `broadcastChanges` triggers,
/// `CriteriaTriggers.TICK`, then `advancements.flushDirty(this, true)`.
pub(super) fn tick_player_advancements(
    stream: &mut ClientStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: &AdvancementTickContext<'_>,
) -> io::Result<()> {
    let Some((shared, resources)) = SESSION.with(|session| {
        session
            .borrow()
            .as_ref()
            .map(|session| (Arc::clone(&session.shared), Arc::clone(&session.resources)))
    }) else {
        return Ok(());
    };
    let resources = reload_if_needed(&shared, resources);
    settle(stream, compression, play_state, context, &shared, &resources)?;
    lock_advancements(&shared).trigger(&trigger_id("tick"), player_trigger_matches);
    settle(stream, compression, play_state, context, &shared, &resources)?;
    let packet = lock_advancements(&shared).flush_dirty(true);
    if let Some(packet) = packet {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID,
            |payload| packet.write(payload),
        )?;
    }
    Ok(())
}

/// `PlayerList.reloadResources`: when the datapack advancements were replaced, re-validate
/// this player's progress against them (the next flush resends everything).
fn reload_if_needed(
    shared: &SharedPlayerAdvancements,
    resources: Arc<LoadedResources>,
) -> Arc<LoadedResources> {
    let Ok(current) = active_resources() else {
        return resources;
    };
    let mut advancements = lock_advancements(shared);
    if !Arc::ptr_eq(advancements.manager(), &current.content.advancements) {
        advancements.reload(Arc::clone(&current.content.advancements));
        SESSION.with(|session| {
            if let Some(session) = session.borrow_mut().as_mut() {
                session.resources = Arc::clone(&current);
            }
        });
    }
    current
}

/// Raises every pending trigger and runs the resulting side effects until nothing more
/// changes (a reward can change the inventory or unlock recipes, which raise triggers).
fn settle(
    stream: &mut ClientStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: &AdvancementTickContext<'_>,
    shared: &SharedPlayerAdvancements,
    resources: &Arc<LoadedResources>,
) -> io::Result<()> {
    let reward_context = RewardContext {
        profile: context.profile,
        bus: context.bus,
        world_items: context.world_items,
        resources,
    };
    for _ in 0..MAX_SETTLE_PASSES {
        let tags = RegistryItemTags { resources };
        let mut progressed = raise_inventory_changed(shared, play_state, &tags);
        progressed |= raise_recipe_unlocked(shared, play_state);
        let events = lock_advancements(shared).take_events();
        progressed |= !events.is_empty();
        for event in &events {
            rewards::handle_event(stream, compression, play_state, &reward_context, event)?;
        }
        if !progressed {
            break;
        }
    }
    Ok(())
}

/// `AbstractContainerMenu.broadcastChanges` -> `ServerPlayer.containerListener.slotChanged`:
/// every player inventory slot whose stack differs from the last one seen raises
/// `InventoryChangeTrigger.trigger(player, inventory, changedItem)`.
fn raise_inventory_changed(
    shared: &SharedPlayerAdvancements,
    play_state: &PlaySessionState,
    tags: &dyn TriggerEnvironment,
) -> bool {
    let inventory = play_state.inventory_menu.player_inventory();
    let trigger = trigger_id("inventory_changed");
    let mut raised = false;
    SESSION.with(|session| {
        let mut session = session.borrow_mut();
        let Some(session) = session.as_mut() else {
            return;
        };
        for slot in MENU_ORDER {
            let current = inventory.get(slot);
            if session.last_slots[slot] == *current {
                continue;
            }
            session.last_slots[slot] = current.clone();
            raised = true;
            lock_advancements(shared).trigger(&trigger, |spec| {
                inventory_changed_matches(spec, inventory, current, tags)
            });
        }
    });
    raised
}

/// `ServerRecipeBook.addRecipes` -> `RecipeUnlockedTrigger.trigger(player, recipe)`.
fn raise_recipe_unlocked(shared: &SharedPlayerAdvancements, play_state: &mut PlaySessionState) -> bool {
    let unlocked = play_state.inventory_menu.drain_recipe_trigger_events();
    let trigger = trigger_id("recipe_unlocked");
    for recipe in &unlocked {
        let Ok(recipe) = Identifier::parse(recipe) else {
            continue;
        };
        lock_advancements(shared)
            .trigger(&trigger, |spec| recipe_unlocked_matches(spec, &recipe));
    }
    !unlocked.is_empty()
}

/// `minecraft:<path>` as a trigger id.
fn trigger_id(path: &str) -> Identifier {
    Identifier::with_default_namespace(path)
        .unwrap_or_else(|err| panic!("invalid trigger id {path}: {err}"))
}

/// Item tags from the loaded item registry, falling back to the built-in tag tables.
struct RegistryItemTags<'a> {
    resources: &'a LoadedResources,
}

impl TriggerEnvironment for RegistryItemTags<'_> {
    fn item_in_tag(&self, item: &str, tag: &str) -> bool {
        let registries = &self.resources.registries;
        let registry = Identifier::parse("minecraft:item")
            .ok()
            .and_then(|key| registries.lookup(&key));
        let members = registry.zip(Identifier::parse(tag).ok()).and_then(|(registry, tag)| {
            registry.tags().get(&tag).map(|ids| (registry, ids))
        });
        match members {
            Some((registry, ids)) => ids
                .iter()
                .any(|id| registry.elements()[*id].key.to_string() == item),
            None => BuiltinItemTags.item_in_tag(item, tag),
        }
    }
}

/// `ServerGamePacketListenerImpl.handleSeenAdvancements`: opening a tab selects it. Returns
/// whether `packet_id` was the seen-advancements packet.
pub(super) fn try_handle_seen_advancements(
    stream: &mut ClientStream,
    compression: CompressionState,
    packet_id: i32,
    input: &mut Cursor<Vec<u8>>,
    uuid: &str,
) -> io::Result<bool> {
    if packet_id != SERVERBOUND_SEEN_ADVANCEMENTS_PACKET_ID {
        return Ok(false);
    }
    let packet = ServerboundSeenAdvancementsPacket::read(input)?;
    if packet.action != ServerboundSeenAdvancementsAction::OpenedTab {
        return Ok(true);
    }
    let Some(tab) = packet.tab else {
        return Ok(true);
    };
    let Some(shared) = AdvancementRegistry::global().get(uuid) else {
        return Ok(true);
    };
    let known = lock_advancements(&shared).manager().get(&tab).is_some();
    if !known {
        return Ok(true);
    }
    let selected = lock_advancements(&shared).set_selected_tab(Some(&tab));
    if let Some(selected) = selected {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_SELECT_ADVANCEMENTS_TAB_PACKET_ID,
            |payload| selected.write(payload),
        )?;
    }
    Ok(true)
}
