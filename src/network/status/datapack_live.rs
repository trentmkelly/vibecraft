//! Live `/reload` and `/datapack` support: seeds the command model from the server's
//! pack repository and applies its reload requests.
//!
//! Java: `ReloadCommand.reloadPacks` -> `MinecraftServer.reloadResources` swaps the
//! resources (tags of every registry, recipes, loot tables, advancements and functions
//! are re-read from the newly selected packs) and then `PlayerList.reloadResources`
//! sends every player a fresh `ClientboundUpdateTagsPacket` and
//! `ClientboundUpdateRecipesPacket` plus their recipe book.

use std::io;

use crate::command::{CommandFunctionTag, ServerCommandState};
use crate::network::varint::write_var_i32;
use crate::network::world_broadcast::WorldPacketBus;
use crate::registry_pipeline::server_resources::{LoadedResources, ServerResources};
use crate::registry_pipeline::sync::{serialize_tags_to_network, write_update_tags_packet};

/// `GamePacketTypes.CLIENTBOUND_UPDATE_TAGS` (`ClientboundUpdateTagsPacket`).
const CLIENTBOUND_UPDATE_TAGS_PACKET_ID: i32 = 134;

/// What applying the recorded pack requests did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DataPackOutcome {
    /// The command did not touch the packs.
    Untouched,
    /// The requested reload completed and the new tags were broadcast.
    Reloaded,
    /// `MinecraftServer.reloadResources` failed; the previous packs stay active.
    Failed,
}

/// Whether `line` is a `/datapack` or `/reload` command, the only ones that read the
/// pack repository (scanning the `datapacks` folder is not free).
fn touches_data_packs(line: &str) -> bool {
    let root = line.trim_start().trim_start_matches('/');
    matches!(root.split_whitespace().next(), Some("datapack" | "reload"))
}

/// Whether `line` can run a function (`/function`, `/schedule function`,
/// `execute ... function`), the only commands that read the function library.
fn mentions_functions(line: &str) -> bool {
    line.contains("function")
}

/// Seeds the pack fields of a command state from the live repository
/// (`server.getPackRepository()`, `worldData.getDataConfiguration()`), so the
/// `/datapack` and `/reload` commands see the real available/selected packs, and the
/// function library (`server.getFunctions()`) for commands that run functions.
pub(super) fn seed_data_pack_state(state: &mut ServerCommandState, line: &str) {
    let Some(resources) = ServerResources::installed() else {
        return;
    };
    if touches_data_packs(line) {
        seed_from(resources, state);
    }
    if mentions_functions(line) {
        seed_functions(resources, state);
    }
}

/// `ServerFunctionManager.getFunction/getTag` as the command model's function tables.
fn seed_functions(resources: &ServerResources, state: &mut ServerCommandState) {
    let current = resources.current();
    let library = &current.content.functions;
    state.available_functions = library.get_functions().values().cloned().collect();
    state.function_tags = library
        .get_available_tags()
        .map(|tag| CommandFunctionTag {
            id: tag.to_string(),
            functions: library
                .get_tag(tag)
                .iter()
                .map(|function| function.id.clone())
                .collect(),
        })
        .collect();
}

fn seed_from(resources: &ServerResources, state: &mut ServerCommandState) {
    let listing = resources.list_packs();
    state.available_data_packs = listing.available;
    state.selected_data_packs = listing.selected;
    state.disabled_data_packs = listing.disabled;
    state.feature_data_packs = listing.feature_packs;
    state.unavailable_feature_data_packs = listing.unavailable_feature_packs;
    state.datapack_directory = Some(resources.datapack_directory());
}

/// Runs the reload the executed command asked for (`ReloadCommand.reloadPacks`) and
/// broadcasts the result to every player on `bus`.
pub(super) fn apply_data_pack_requests(
    state: &ServerCommandState,
    bus: &WorldPacketBus,
) -> DataPackOutcome {
    match ServerResources::installed() {
        Some(resources) => apply_requests(resources, state, bus),
        None => DataPackOutcome::Untouched,
    }
}

fn apply_requests(
    resources: &ServerResources,
    state: &ServerCommandState,
    bus: &WorldPacketBus,
) -> DataPackOutcome {
    let Some(request) = state.reload_requests.last() else {
        return DataPackOutcome::Untouched;
    };
    match resources.reload(&request.selected_packs) {
        Ok(loaded) => {
            if let Err(error) = broadcast_reload(bus, &loaded) {
                crate::log::log_warn(&format!("Failed to send reloaded resources: {error}"));
            }
            DataPackOutcome::Reloaded
        }
        Err(error) => {
            // `ReloadCommand.reloadPacks`: `LOGGER.warn("Failed to execute reload", ...)`.
            crate::log::log_warn(&format!("Failed to execute reload: {error}"));
            DataPackOutcome::Failed
        }
    }
}

/// `PlayerList.reloadResources`: the tags of every registry go to all players at once.
/// The recipe synchronisation and recipe book are per player and are sent by each
/// session itself (see [`session_sync`]).
fn broadcast_reload(bus: &WorldPacketBus, loaded: &LoadedResources) -> io::Result<()> {
    let mut tags = Vec::new();
    write_var_i32(&mut tags, CLIENTBOUND_UPDATE_TAGS_PACKET_ID)?;
    write_update_tags_packet(&mut tags, &serialize_tags_to_network(&loaded.registries))?;
    bus.publish(&tags);
    Ok(())
}

mod session_sync;
pub(super) use session_sync::{begin_session_sync, sync_reloaded_recipes};

#[cfg(test)]
mod tests;
