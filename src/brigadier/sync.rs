//! Per-session bookkeeping for `ClientboundCommandsPacket`.
//!
//! Java sends the command tree on join (`PlayerList.placeNewPlayer`) and again
//! whenever the player's permissions change (`PlayerList.sendPlayerPermissionLevel`
//! -> `Commands.sendCommands`). The session loop polls the player's permission
//! level once per iteration and resends when it differs from the last tree sent.

use std::io;

use super::CommandGraph;
use crate::network::play::ClientboundCommandsPacket;

/// Tracks which permission level the client's command tree was built for.
#[derive(Debug, Default)]
pub struct CommandTreeSync {
    /// Level of the tree last sent to the client; `None` before the join tree.
    sent_level: Option<u8>,
    /// The player's current permission level.
    current_level: u8,
}

impl CommandTreeSync {
    /// Records the player's current permission level (called every loop iteration).
    pub fn observe_permission_level(&mut self, level: u8) {
        self.current_level = level;
    }

    /// `true` until the join-time tree has been sent.
    pub fn needs_initial_send(&self) -> bool {
        self.sent_level.is_none()
    }

    /// `true` when a tree was sent but the player's level has changed since.
    pub fn needs_resend(&self) -> bool {
        self.sent_level
            .is_some_and(|sent| sent != self.current_level)
    }

    /// Builds the packet for the current level and marks it as sent
    /// (`Commands.sendCommands`).
    pub fn take_packet(&mut self) -> io::Result<ClientboundCommandsPacket> {
        let packet =
            CommandGraph::served().commands_packet_for_permission_level(self.current_level)?;
        self.sent_level = Some(self.current_level);
        Ok(packet)
    }
}
