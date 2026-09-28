//! Per-connection secure-chat state for the live play loop.
//!
//! Mirrors the `lastSeenMessages` field of Java
//! `ServerGamePacketListenerImpl` (`new LastSeenMessagesValidator(20)`) together
//! with its `handleChatAck` and `unpackAndApplyLastSeen` entry points. A failed
//! validation disconnects the player with
//! `multiplayer.disconnect.chat_validation_failed` (`CHAT_VALIDATION_FAILED`).

use crate::chat_trust::last_seen_tracking::LastSeenMessagesValidatorModel;
use crate::chat_trust::{LastSeenMessages, LastSeenMessagesUpdateModel};
use crate::network::play::LastSeenMessagesUpdate;

/// Java `ServerGamePacketListenerImpl.CHAT_VALIDATION_FAILED` translation key.
pub(super) const CHAT_VALIDATION_FAILED: &str = "multiplayer.disconnect.chat_validation_failed";

/// Window size passed to Java's `new LastSeenMessagesValidator(20)`.
const LAST_SEEN_WINDOW: usize = 20;

/// Server-side chat acknowledgement state of one connection.
#[derive(Debug, Clone)]
pub(crate) struct LiveChatState {
    last_seen: LastSeenMessagesValidatorModel,
}

impl LiveChatState {
    pub(crate) fn new() -> Self {
        Self {
            last_seen: LastSeenMessagesValidatorModel::new(LAST_SEEN_WINDOW),
        }
    }

    /// Java `handleChatAck`: advances the acknowledged window by `offset`.
    /// `Err` carries the validation message; the caller must disconnect with
    /// [`CHAT_VALIDATION_FAILED`].
    pub(crate) fn apply_ack_offset(&mut self, offset: i32) -> Result<(), String> {
        self.last_seen.apply_offset(offset)
    }

    /// Java `unpackAndApplyLastSeen`: validates a chat/command `LastSeenMessages.Update`.
    pub(crate) fn unpack_and_apply_last_seen(
        &mut self,
        update: &LastSeenMessagesUpdate,
    ) -> Result<LastSeenMessages, String> {
        self.last_seen.apply_update(&update_model(update))
    }
}

/// Expands the wire update (Java `FriendlyByteBuf.readFixedBitSet(20)` is
/// `BitSet.valueOf` over 3 little-endian bytes, so bits 20..24 are preserved and
/// later rejected by the validator's `length() > lastSeenCount` check).
fn update_model(update: &LastSeenMessagesUpdate) -> LastSeenMessagesUpdateModel {
    let acknowledged = update
        .acknowledged
        .iter()
        .flat_map(|byte| (0..8).map(move |bit| byte >> bit & 1 == 1))
        .collect();
    LastSeenMessagesUpdateModel {
        offset: update.offset,
        acknowledged,
        checksum: update.checksum,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(offset: i32, acknowledged: [u8; 3], checksum: u8) -> LastSeenMessagesUpdate {
        LastSeenMessagesUpdate {
            offset,
            acknowledged: acknowledged.to_vec(),
            checksum,
        }
    }

    #[test]
    fn empty_update_is_accepted() {
        let mut state = LiveChatState::new();
        let last_seen = state
            .unpack_and_apply_last_seen(&update(0, [0; 3], 0))
            .expect("empty update is valid");
        assert!(last_seen.entries.is_empty());
    }

    #[test]
    fn ack_offset_beyond_tracked_messages_is_rejected() {
        let mut state = LiveChatState::new();
        let error = state.apply_ack_offset(1).unwrap_err();
        assert_eq!(
            error,
            "Advanced last seen window by 1 messages, but expected at most 0"
        );
        assert!(state.apply_ack_offset(-1).is_err());
        assert!(state.apply_ack_offset(0).is_ok());
    }

    #[test]
    fn acknowledging_unknown_message_is_rejected() {
        let mut state = LiveChatState::new();
        let error = state
            .unpack_and_apply_last_seen(&update(0, [1, 0, 0], 0))
            .unwrap_err();
        assert!(error.contains("acknowledged unknown or previously ignored message at index 0"));
    }

    #[test]
    fn bits_beyond_the_window_are_rejected() {
        let mut state = LiveChatState::new();
        let error = state
            .unpack_and_apply_last_seen(&update(0, [0, 0, 0x80], 0))
            .unwrap_err();
        assert_eq!(
            error,
            "Last seen update contained 24 messages, but maximum window size is 20"
        );
    }

    #[test]
    fn disconnect_key_matches_java_translation() {
        assert_eq!(
            CHAT_VALIDATION_FAILED,
            "multiplayer.disconnect.chat_validation_failed"
        );
    }
}
