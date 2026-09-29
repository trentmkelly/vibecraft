//! `/advancement grant|revoke` against the live players' advancements.
//!
//! The command model works on a self-contained copy of the advancement tree and of each
//! target's obtained criteria (`ServerCommandState::advancements` /
//! `player_advancements`). [`seed_command_advancements`] fills that copy from the live
//! manager and registry before the command runs, and [`apply_command_advancements`]
//! replays the difference on the real `PlayerAdvancements`
//! (`AdvancementCommands.Action.perform` -> `award` / `revoke`), so rewards, the
//! completion announcement and the client packets all follow the normal path.
//!
//! Java has no `/advancement test`; the command tree is `grant` / `revoke` only.

use std::collections::{BTreeSet, HashMap};
use std::io;

use super::*;
use crate::command::{
    AdvancementDefinition as CommandAdvancement, PlayerAdvancementProgress, ServerCommandState,
};
use crate::player_advancements::registry::AdvancementRegistry;

/// The obtained criteria the command started from.
pub(in crate::network::status) struct CommandSeed {
    before: Vec<PlayerAdvancementProgress>,
}

/// Whether `line` can touch advancements.
fn mentions_advancements(line: &str) -> bool {
    line.contains("advancement")
}

/// Copies the live advancement tree and the online players' progress into `state`.
/// Returns `None` (leaving `state` alone) for commands that cannot use them.
pub(in crate::network::status) fn seed_command_advancements(
    state: &mut ServerCommandState,
    line: &str,
) -> Option<CommandSeed> {
    if !mentions_advancements(line) {
        return None;
    }
    let resources = active_resources().ok()?;
    state.advancements = resources
        .content
        .advancements
        .get_all_advancements()
        .map(|holder| {
            let definition = holder.value();
            CommandAdvancement {
                id: definition.id.to_string(),
                parent: definition.parent.as_ref().map(ToString::to_string),
                criteria: definition.criteria.iter().cloned().collect(),
            }
        })
        .collect();
    for player in state.online_players.clone() {
        let Some(shared) = AdvancementRegistry::global().get(&player.uuid) else {
            continue;
        };
        for (id, completed_criteria) in lock_advancements(&shared).obtained_criteria() {
            state.player_advancements.push(PlayerAdvancementProgress {
                player: player.clone(),
                advancement: id.to_string(),
                completed_criteria,
            });
        }
    }
    Some(CommandSeed {
        before: state.player_advancements.clone(),
    })
}

/// Applies what the command changed to the live players.
pub(in crate::network::status) fn apply_command_advancements(
    seed: Option<CommandSeed>,
    state: &ServerCommandState,
    sessions: &Arc<Mutex<HashMap<String, ActiveLoginSession>>>,
    bus: &WorldPacketBus,
) -> io::Result<()> {
    let Some(seed) = seed else {
        return Ok(());
    };
    let mut players: Vec<&NameAndId> = Vec::new();
    for entry in &state.player_advancements {
        if !players.iter().any(|player| player.uuid == entry.player.uuid) {
            players.push(&entry.player);
        }
    }
    for player in players {
        let hides_toasts = state
            .advancement_flush_events
            .iter()
            .any(|event| event.player.uuid == player.uuid && event.hide_advancement_toasts);
        let Some(shared) = AdvancementRegistry::global().get(&player.uuid) else {
            continue;
        };
        let token = lock_status_mutex(sessions)
            .get(&player.uuid)
            .filter(|session| session.in_play)
            .map(|session| session.token);
        // `Action.perform`: with `showAdvancements == false` the pending changes are
        // flushed with toasts first, and the command's own with toasts suppressed.
        if hides_toasts {
            send_flush(&shared, true, token, bus)?;
        }
        replay_changes(&shared, &player.uuid, &seed.before, &state.player_advancements);
        if hides_toasts {
            send_flush(&shared, false, token, bus)?;
        }
    }
    Ok(())
}

/// [`apply_command_advancements`] for a command a player ran.
pub(in crate::network::status) fn apply_command_advancements_for(
    seed: Option<CommandSeed>,
    state: &ServerCommandState,
    guard: &ActiveLoginGuard,
) -> io::Result<()> {
    apply_command_advancements(seed, state, &guard.sessions, &guard.world_bus)
}

/// `flushDirty(player, showAdvancements)` for a player whose session owns the connection.
fn send_flush(
    shared: &SharedPlayerAdvancements,
    show_advancements: bool,
    token: Option<u64>,
    bus: &WorldPacketBus,
) -> io::Result<()> {
    let packet = lock_advancements(shared).flush_dirty(show_advancements);
    let (Some(packet), Some(token)) = (packet, token) else {
        return Ok(());
    };
    let mut payload = Vec::new();
    write_var_i32(&mut payload, CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID)?;
    packet.write(&mut payload)?;
    bus.publish_to(token, &payload);
    Ok(())
}

/// `award`s and `revoke`s the criteria that differ between `before` and `after`.
fn replay_changes(
    shared: &SharedPlayerAdvancements,
    uuid: &str,
    before: &[PlayerAdvancementProgress],
    after: &[PlayerAdvancementProgress],
) {
    let obtained = |entries: &[PlayerAdvancementProgress], advancement: &str| -> BTreeSet<String> {
        entries
            .iter()
            .filter(|entry| entry.player.uuid == uuid && entry.advancement == advancement)
            .flat_map(|entry| entry.completed_criteria.iter().cloned())
            .collect()
    };
    let advancements: BTreeSet<&str> = after
        .iter()
        .filter(|entry| entry.player.uuid == uuid)
        .map(|entry| entry.advancement.as_str())
        .collect();
    let mut live = lock_advancements(shared);
    for advancement in advancements {
        let Ok(id) = Identifier::parse(advancement) else {
            continue;
        };
        let (was, now) = (obtained(before, advancement), obtained(after, advancement));
        for criterion in now.difference(&was) {
            live.award(&id, criterion);
        }
        for criterion in was.difference(&now) {
            live.revoke(&id, criterion);
        }
    }
}
