//! The per-player reload sync: recipes + recipe book go out exactly when the recipe
//! manager the session last synchronised is replaced.

use std::io::Read;
use std::net::TcpListener;
use std::path::Path;
use std::time::Duration;

use super::*;
use crate::network::play::{CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID, CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID};
use crate::network::varint::read_var_i32;
use crate::recipe_system::load_recipe_directory;

/// A connected loopback pair: (server side written to, client side read from).
fn socket_pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|e| panic!("{e}"));
    let client = TcpStream::connect(listener.local_addr().unwrap_or_else(|e| panic!("{e}")))
        .unwrap_or_else(|e| panic!("{e}"));
    let (server, _) = listener.accept().unwrap_or_else(|e| panic!("{e}"));
    client
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap_or_else(|e| panic!("{e}"));
    (server, client)
}

/// Every packet id the client received so far.
fn received_ids(client: &mut TcpStream) -> Vec<i32> {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    while let Ok(read) = client.read(&mut chunk) {
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    let mut reader = &bytes[..];
    let mut ids = Vec::new();
    while !reader.is_empty() {
        let payload = CompressionState::disabled()
            .decode_packet(&mut reader)
            .unwrap_or_else(|e| panic!("frame: {e}"));
        ids.push(read_var_i32(&mut &payload[..]).unwrap_or_else(|e| panic!("id: {e}")));
    }
    ids
}

fn vanilla_recipes() -> Arc<RecipeManagerModel> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/recipe");
    Arc::new(load_recipe_directory(&dir).unwrap_or_else(|e| panic!("{e}")))
}

#[test]
fn a_replaced_recipe_manager_resends_recipes_and_the_recipe_book_once() {
    let (mut server, mut client) = socket_pair();
    let compression = CompressionState::disabled();
    let mut play_state = PlaySessionState::default();
    let joined_with = Arc::new(RecipeManagerModel::default());
    begin_session_sync(&joined_with);

    // Same manager as at join: nothing to send.
    sync_to(&mut server, compression, &mut play_state, Some(Arc::clone(&joined_with)))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(received_ids(&mut client).is_empty());

    // `/reload` installed a new manager: recipes, then the recipe book.
    let reloaded = vanilla_recipes();
    sync_to(&mut server, compression, &mut play_state, Some(Arc::clone(&reloaded)))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        received_ids(&mut client),
        vec![
            CLIENTBOUND_UPDATE_RECIPES_PACKET_ID,
            CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID,
            CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID,
        ]
    );

    // The player's own menu now resolves recipes against the reloaded map.
    play_state
        .inventory_menu
        .grant_initial_recipes(["minecraft:stick"]);
    assert!(play_state
        .inventory_menu
        .recipe_book_known_recipes()
        .contains(&"minecraft:stick"));

    // The session now follows the reloaded manager: no repeat, and a further
    // reload sends again.
    sync_to(&mut server, compression, &mut play_state, Some(Arc::clone(&reloaded)))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(received_ids(&mut client).is_empty());
    sync_to(&mut server, compression, &mut play_state, Some(vanilla_recipes()))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(received_ids(&mut client).len(), 3);
}

#[test]
fn without_a_server_state_or_a_started_session_nothing_is_sent() {
    let (mut server, mut client) = socket_pair();
    let compression = CompressionState::disabled();
    let mut play_state = PlaySessionState::default();

    sync_to(&mut server, compression, &mut play_state, None).unwrap_or_else(|e| panic!("{e}"));
    // No `begin_session_sync` on this thread yet.
    sync_to(&mut server, compression, &mut play_state, Some(vanilla_recipes()))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(received_ids(&mut client).is_empty());
}
