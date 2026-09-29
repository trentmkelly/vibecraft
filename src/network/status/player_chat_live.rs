//! Live plain-chat path: Java `ServerGamePacketListenerImpl.handleChat` ->
//! `tryHandleChat` -> `broadcastChatMessage` and `PlayerList.broadcastChatMessage`
//! for the unsigned (offline / no chat session) decoder.
//!
//! Every recipient gets a `ClientboundPlayerChatPacket` of chat type
//! `minecraft:chat` (`OutgoingChatMessage.Player`), stamped with that
//! connection's own `nextChatIndex` by [`WorldPacketBus::publish_player_chat`].
//!
//! TODO(secure-chat-chain): signed chains need a validated `RemoteChatSession`
//! (`ServerboundChatSessionUpdate`), which requires the online-mode services key
//! that VibeCraft does not have; until then `enforceSecureProfile()` is treated as
//! false (`canValidateProfileKeys` is false) and every message uses
//! `SignedMessageChain.Decoder.unsigned`.
//! TODO(text-filtering): Java's `filterTextPacket` is a pass-through without a
//! filter service, so every message carries `FilterMask.PASS_THROUGH` and
//! `shouldFilterMessageTo` never yields a masked message.

use super::player_messaging_live::{chat_type_id, player_display_name_tag};
use super::*;
use crate::log::log_warn;
use crate::network::common::ChatVisibility;
use crate::network::play::{
    BoundChatTypeData, ClientboundPlayerChatPacket, FilterMaskData, SignedMessageBodyPacked,
    CLIENTBOUND_PLAYER_CHAT_PACKET_ID,
};

/// Translation key of `SignedMessageChain.DecodeException.MISSING_PROFILE_KEY`.
const MISSING_PROFILE_KEY: &str = "chat.disabled.missingProfileKey";
/// Translation key sent when the sender hides chat (`ChatVisiblity.HIDDEN`).
const CHAT_DISABLED_OPTIONS: &str = "chat.disabled.options";

/// `MinecraftServer.services.canValidateProfileKeys()`; false until online-mode
/// session services exist (see the module TODO).
const CAN_VALIDATE_PROFILE_KEYS: bool = false;

/// Java `MinecraftServer.enforceSecureProfile()`:
/// `enforceSecureProfile && onlineMode && services.canValidateProfileKeys()`.
fn enforce_secure_profile(properties: &ServerProperties) -> bool {
    properties.enforce_secure_profile && properties.online_mode && CAN_VALIDATE_PROFILE_KEYS
}

/// `Component.translatable(key).withStyle(RED)` as a system-chat payload.
fn red_translatable(key: &str) -> Tag {
    Tag::Compound(vec![
        ("translate".to_string(), Tag::String(key.to_string())),
        ("color".to_string(), Tag::String("red".to_string())),
    ])
}

/// Serializes a `Tag` as a network NBT component (chat type name).
fn network_tag_bytes(tag: &Tag) -> io::Result<Vec<u8>> {
    let mut bytes = vec![tag.id()];
    tag.write_payload(&mut bytes)?;
    Ok(bytes)
}

/// Body of an unsigned `ClientboundPlayerChatPacket` minus its leading
/// `globalIndex`. Java `PlayerChatMessage.unsigned(sender, content)`: link index 0,
/// no signature, `SignedMessageBody.unsigned` (now, salt 0, empty last-seen) and an
/// unsigned content only when the decorated component differs from the literal
/// (`ChatDecorator.PLAIN` never does).
fn unsigned_chat_body(sender: &NameAndId, content: &str, now_millis: i64) -> io::Result<Vec<u8>> {
    let sender_uuid = uuid_from_hyphenated(&sender.uuid)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid sender uuid"))?;
    let packet = ClientboundPlayerChatPacket {
        global_index: 0,
        sender: sender_uuid,
        index: 0,
        signature: None,
        body: SignedMessageBodyPacked {
            content: content.to_string(),
            timestamp_epoch_millis: now_millis,
            salt: 0,
            last_seen: Vec::new(),
        },
        unsigned_content_payload: None,
        filter_mask: FilterMaskData::PassThrough,
        chat_type: BoundChatTypeData {
            chat_type_id: chat_type_id("chat"),
            name_payload: network_tag_bytes(&player_display_name_tag(sender))?,
            target_name_payload: None,
        },
    };
    let mut full = Vec::new();
    packet.write(&mut full)?;
    // Strip the placeholder `globalIndex` VarInt (0 encodes as one byte).
    Ok(full.split_off(1))
}

fn now_epoch_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as i64)
}

impl ActiveLoginGuard {
    /// Java `PlayerList.broadcastChatMessage(message, sender, chatType)` for an
    /// unsigned message: logs it to the console (`Not Secure`, since it carries no
    /// signature) and sends it to every in-play player whose chat visibility is
    /// `FULL` (`ServerPlayer.sendChatMessage` / `acceptsChatMessages`).
    pub(super) fn broadcast_unsigned_player_chat(
        &self,
        sender: &NameAndId,
        content: &str,
    ) -> io::Result<()> {
        let body = unsigned_chat_body(sender, content, now_epoch_millis())?;
        log_info(&format!("[Not Secure] <{}> {content}", sender.name));
        for token in self.chat_recipient_tokens() {
            self.world_bus
                .publish_player_chat(token, CLIENTBOUND_PLAYER_CHAT_PACKET_ID, &body);
        }
        Ok(())
    }

    /// Tokens of the in-play sessions that accept chat (`ChatVisiblity.FULL`).
    fn chat_recipient_tokens(&self) -> Vec<u64> {
        let Ok(sessions) = self.sessions.lock() else { return Vec::new() };
        sessions
            .values()
            .filter(|s| s.in_play && s.chat_visibility == ChatVisibility::Full)
            .map(|s| s.token)
            .collect()
    }

    /// This session's own `ServerPlayer.getChatVisibility()`.
    fn own_chat_visibility(&self) -> ChatVisibility {
        self.sessions
            .lock()
            .ok()
            .and_then(|sessions| {
                sessions
                    .values()
                    .find(|s| s.token == self.token)
                    .map(|s| s.chat_visibility)
            })
            .unwrap_or(ChatVisibility::Full)
    }
}

/// Java `tryHandleChat(message, false, ...)` followed by the unsigned
/// `getSignedMessage` decode and `broadcastChatMessage`.
pub(super) fn handle_plain_chat(
    stream: &mut ClientStream,
    compression: CompressionState,
    profile: &NameAndId,
    message: &str,
    active_login: &ActiveLoginGuard,
    properties: &ServerProperties,
) -> io::Result<()> {
    if chat_message_is_illegal(message) {
        return write_disconnect_component(
            stream,
            compression,
            "multiplayer.disconnect.illegal_characters",
        );
    }
    if active_login.own_chat_visibility() == ChatVisibility::Hidden {
        return write_system_chat_tag(stream, compression, red_translatable(CHAT_DISABLED_OPTIONS));
    }
    if enforce_secure_profile(properties) {
        // handleMessageDecodeFailure: warn + red system message to the sender.
        log_warn(&format!(
            "Failed to update secure chat state for {}: '{MISSING_PROFILE_KEY}'",
            profile.name
        ));
        return write_system_chat_tag(stream, compression, red_translatable(MISSING_PROFILE_KEY));
    }
    active_login.broadcast_unsigned_player_chat(profile, message)?;
    Ok(())
}

fn write_system_chat_tag(
    stream: &mut ClientStream,
    compression: CompressionState,
    content: Tag,
) -> io::Result<()> {
    write_framed_packet_with_compression(stream, compression, CLIENTBOUND_SYSTEM_CHAT_PACKET_ID, |payload| {
        ClientboundSystemChatPacket { content, overlay: false }.write(payload)
    })
}

#[cfg(test)]
mod tests;
