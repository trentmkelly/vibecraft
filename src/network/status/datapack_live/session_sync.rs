//! The per-player half of `PlayerList.reloadResources`.
//!
//! Java resends each player the recipe synchronisation and their recipe book:
//!
//! ```text
//! for (ServerPlayer player : this.players) {
//!     player.connection.send(recipes);                  // ClientboundUpdateRecipesPacket
//!     player.getRecipeBook().sendInitialRecipeBook(player);
//! }
//! ```
//!
//! The recipe book lives in each connection's play state, which only that
//! connection's thread may touch, so the reload does not push these packets itself.
//! Every session remembers the recipe manager it last synchronised (the one its join
//! packets were built from) and, once per tick, compares it with the manager the
//! server now runs; a difference means a reload happened and the packets go out.

use std::cell::RefCell;
use std::io;
use std::net::TcpStream;
use std::sync::Arc;

use crate::network::compression::CompressionState;
use crate::network::play::{
    write_clientbound_update_recipes_packet, CLIENTBOUND_UPDATE_RECIPES_PACKET_ID,
};
use crate::network::status::chunk_c::write_initial_recipe_book;
use crate::network::status::chunk_e_2::write_framed_packet_with_compression;
use crate::network::status::PlaySessionState;
use crate::recipe_system::RecipeManagerModel;
use crate::registry_pipeline::server_resources::installed_recipe_manager;

thread_local! {
    /// The recipe manager this connection thread's player last synchronised.
    static SYNCED_RECIPES: RefCell<Option<Arc<RecipeManagerModel>>> = const { RefCell::new(None) };
}

/// Records the recipe manager the session's join packets are built from. Called on
/// the connection thread before the join, so a reload that lands between the accept
/// and the first tick is still noticed.
pub(in crate::network::status) fn begin_session_sync(joined_with: &Arc<RecipeManagerModel>) {
    SYNCED_RECIPES.with(|synced| *synced.borrow_mut() = Some(Arc::clone(joined_with)));
}

/// Sends the recipe synchronisation and recipe book when the server's recipes were
/// reloaded since the last call. A no-op without an installed server state or a
/// session that never called [`begin_session_sync`].
pub(in crate::network::status) fn sync_reloaded_recipes(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
) -> io::Result<()> {
    sync_to(stream, compression, play_state, installed_recipe_manager())
}

/// [`sync_reloaded_recipes`] against an explicit `current` manager.
fn sync_to(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    current: Option<Arc<RecipeManagerModel>>,
) -> io::Result<()> {
    let Some(current) = current else {
        return Ok(());
    };
    let changed = SYNCED_RECIPES.with(|synced| {
        let mut synced = synced.borrow_mut();
        match synced.as_ref() {
            Some(previous) if !Arc::ptr_eq(previous, &current) => {
                *synced = Some(Arc::clone(&current));
                true
            }
            _ => false,
        }
    });
    if !changed {
        return Ok(());
    }
    play_state
        .inventory_menu
        .replace_recipes(current.recipe_map().clone());
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_UPDATE_RECIPES_PACKET_ID,
        |payload| write_clientbound_update_recipes_packet(payload, &current),
    )?;
    write_initial_recipe_book(stream, compression, play_state, current.recipe_map())
}

#[cfg(test)]
mod tests;
