//! Cross-session player messaging on top of the [`WorldPacketBus`].
//!
//! Java reaches other players through `PlayerList` (`getPlayers`,
//! `broadcastSystemMessage`, `broadcastChatMessage`) and
//! `ServerGamePacketListenerImpl.disconnect`. VibeCraft runs one thread per
//! connection, so this module resolves players through the
//! [`ActiveLoginRegistry`] session map and delivers by publishing plain packet
//! payloads to the target's bus inbox (drained by that session's own tick).
//!
//! Wired here:
//! * `/kick` and the other command-driven disconnects (`KickCommand.kickPlayers`),
//! * server shutdown (`PlayerList.removeAll` -> `multiplayer.disconnect.server_shutdown`),
//! * `/say`, `/me`, `/msg`, `/tell`, `/w` (`PlayerList.broadcastChatMessage`,
//!   `OutgoingChatMessage.Disguised`) and `/tellraw` (`Player.sendSystemMessage`),
//! * the join/quit broadcasts (`PlayerList.placeNewPlayer` / `removePlayerFromWorld`).

use super::*;
use crate::chat_component::Component;
use crate::command::{ChatCommandEvent, ChatCommandKind};
use crate::player_list::SERVER_SHUTDOWN_DISCONNECT;
use crate::network::play::{
    ChatTypeBound, ClientboundDisguisedChatPacket, CLIENTBOUND_DISGUISED_CHAT_PACKET_ID,
};
use crate::network::varint::write_var_i32;

/// Feedback key `KickCommand` sends per kicked player.
pub(super) const KICK_SUCCESS_KEY: &str = "commands.kick.success";
/// How long shutdown waits for kicked sessions to flush their disconnect packet.
const SHUTDOWN_FLUSH_TIMEOUT: Duration = Duration::from_secs(2);

/// Encodes a packet as a bus payload: `VarInt id` + body, unframed.
fn plain_payload(
    packet_id: i32,
    write_body: impl FnOnce(&mut Vec<u8>) -> io::Result<()>,
) -> io::Result<Vec<u8>> {
    let mut payload = Vec::new();
    write_var_i32(&mut payload, packet_id)?;
    write_body(&mut payload)?;
    Ok(payload)
}

/// `ClientboundSystemChatPacket` (`ServerPlayer.sendSystemMessage`) as a payload.
fn system_chat_payload(content: Tag) -> io::Result<Vec<u8>> {
    plain_payload(CLIENTBOUND_SYSTEM_CHAT_PACKET_ID, |body| {
        ClientboundSystemChatPacket { content, overlay: false }.write(body)
    })
}

/// Play-state `ClientboundDisconnectPacket` carrying `reason_json`.
pub(super) fn disconnect_payload(reason_json: String) -> io::Result<Vec<u8>> {
    plain_payload(CLIENTBOUND_DISCONNECT_PACKET_ID, |body| {
        ClientboundDisconnectPacket { reason: ComponentJson(reason_json) }.write(body)
    })
}

/// Registry id of a chat type (`Registry.getId`). The synced `chat_type` registry
/// is loaded from the data pack in sorted resource order, and the wire id is the
/// element's position in it.
fn chat_type_id(name: &str) -> i32 {
    crate::registry_pipeline::registry_element_id("minecraft:chat_type", name)
        .unwrap_or_else(|| panic!("unknown chat type {name}")) as i32
}

fn text_tag(text: &str) -> Tag {
    literal_component_tag(text)
}

/// Java `Player.getDisplayName` for a team-less player: the name with a
/// `suggest_command` click event, a `show_entity` hover event and the name as
/// insertion (`Player.decorateDisplayNameComponent`).
pub(super) fn player_display_name_tag(profile: &NameAndId) -> Tag {
    let uuid = uuid_from_hyphenated(&profile.uuid).map(|uuid| uuid.0).unwrap_or([0; 16]);
    let uuid_ints = uuid
        .chunks(4)
        .map(|word| i32::from_be_bytes([word[0], word[1], word[2], word[3]]))
        .collect();
    let string = |value: &str| Tag::String(value.to_string());
    Tag::Compound(vec![
        ("text".to_string(), string(&profile.name)),
        (
            "click_event".to_string(),
            Tag::Compound(vec![
                ("action".to_string(), string("suggest_command")),
                ("command".to_string(), string(&format!("/tell {} ", profile.name))),
            ]),
        ),
        (
            "hover_event".to_string(),
            Tag::Compound(vec![
                ("action".to_string(), string("show_entity")),
                ("id".to_string(), string("minecraft:player")),
                ("uuid".to_string(), Tag::IntArray(uuid_ints)),
                ("name".to_string(), text_tag(&profile.name)),
            ]),
        ),
        ("insertion".to_string(), string(&profile.name)),
    ])
}

/// `Component.translatable(key, args).withStyle(YELLOW)`, used by the join/quit messages.
fn yellow_translatable(key: &str, args: Vec<Tag>) -> Tag {
    Tag::Compound(vec![
        ("translate".to_string(), Tag::String(key.to_string())),
        ("with".to_string(), Tag::List(args)),
        ("color".to_string(), Tag::String("yellow".to_string())),
    ])
}

/// Serializes a `ClientboundDisguisedChatPacket` (`OutgoingChatMessage.Disguised`).
fn disguised_chat_payload(
    content: Tag,
    chat_type: &str,
    sender_name: Tag,
    target_name: Option<Tag>,
) -> io::Result<Vec<u8>> {
    plain_payload(CLIENTBOUND_DISGUISED_CHAT_PACKET_ID, |body| {
        ClientboundDisguisedChatPacket {
            message: content,
            chat_type: ChatTypeBound {
                chat_type_id: chat_type_id(chat_type),
                name: sender_name,
                target_name,
            },
        }
        .write(body)
    })
}

/// Converts a `/tellraw` JSON text component into its network NBT form
/// (`ComponentSerialization` -> `NbtOps`). Returns `None` for malformed JSON
/// (Java: `argument.component.invalid`).
pub(super) fn json_component_to_tag(json: &str) -> Option<Tag> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    component_value_to_tag(&value)
}

fn component_value_to_tag(value: &serde_json::Value) -> Option<Tag> {
    use serde_json::Value;
    Some(match value {
        Value::String(text) => text_tag(text),
        // A JSON array is its first element with the rest as siblings.
        Value::Array(items) => {
            let mut parts = items.iter().map(component_value_to_tag);
            let first = parts.next()??;
            let rest: Vec<Tag> = parts.collect::<Option<_>>()?;
            if rest.is_empty() {
                first
            } else {
                Tag::Compound(vec![
                    ("text".to_string(), Tag::String(String::new())),
                    ("extra".to_string(), Tag::List(std::iter::once(first).chain(rest).collect())),
                ])
            }
        }
        Value::Object(_) => object_value_to_tag(value)?,
        Value::Bool(flag) => text_tag(&flag.to_string()),
        Value::Number(number) => text_tag(&number.to_string()),
        Value::Null => return None,
    })
}

/// Generic JSON object -> compound, wrapping bare strings in component lists
/// (`extra`, `with`) so every list element is a compound (NBT lists are homogeneous).
fn object_value_to_tag(value: &serde_json::Value) -> Option<Tag> {
    use serde_json::Value;
    let Value::Object(map) = value else { return None };
    let mut entries = Vec::with_capacity(map.len());
    for (key, field) in map {
        let tag = match (key.as_str(), field) {
            ("extra" | "with", Value::Array(items)) => Tag::List(
                items.iter().map(component_value_to_tag).collect::<Option<_>>()?,
            ),
            (_, Value::Object(_)) => object_value_to_tag(field)?,
            (_, Value::String(text)) => Tag::String(text.clone()),
            (_, Value::Bool(flag)) => Tag::Byte(i8::from(*flag)),
            (_, Value::Number(number)) => match number.as_i64().and_then(|n| i32::try_from(n).ok()) {
                Some(int) => Tag::Int(int),
                None => Tag::Double(number.as_f64()?),
            },
            (_, Value::Array(items)) => {
                Tag::List(items.iter().map(object_value_to_tag).collect::<Option<_>>()?)
            }
            (_, Value::Null) => return None,
        };
        entries.push((key.clone(), tag));
    }
    Some(Tag::Compound(entries))
}

impl ActiveLoginGuard {
    /// Token and profile of the in-play player named `name` (case-insensitive,
    /// Java `PlayerList.getPlayerByName`).
    pub(super) fn find_in_play_by_name(&self, name: &str) -> Option<(NameAndId, u64)> {
        let sessions = self.sessions.lock().ok()?;
        sessions
            .iter()
            .find(|(_, session)| session.in_play && session.name.eq_ignore_ascii_case(name))
            .map(|(uuid, session)| {
                (NameAndId { uuid: uuid.clone(), name: session.name.clone() }, session.token)
            })
    }

    /// Token of the in-play player with `uuid`.
    fn in_play_token(&self, uuid: &str) -> Option<u64> {
        let sessions = self.sessions.lock().ok()?;
        sessions.get(uuid).filter(|session| session.in_play).map(|session| session.token)
    }

    /// Queues `payload` for this session itself.
    pub(super) fn send_to_self(&self, payload: &[u8]) -> bool {
        self.world_bus.publish_to(self.token, payload)
    }

    /// Ends the session of in-play player `uuid` with the given disconnect
    /// component (Java `ServerGamePacketListenerImpl.disconnect(Component)`).
    fn disconnect_player(&self, uuid: &str, reason_json: String) -> io::Result<bool> {
        let Some(token) = self.in_play_token(uuid) else { return Ok(false) };
        Ok(self.world_bus.disconnect(token, &disconnect_payload(reason_json)?))
    }

    /// Java `PlayerList.broadcastSystemMessage(component, false)` including the
    /// console log leg (`MinecraftServer.sendSystemMessage`).
    fn broadcast_system_message(&self, content: Tag, log_text: &str) -> io::Result<()> {
        self.world_bus.publish(&system_chat_payload(content)?);
        log_info(log_text);
        Ok(())
    }

    /// `multiplayer.player.joined` (Java `PlayerList.placeNewPlayer`).
    // TODO(join-renamed): Java picks `multiplayer.player.joined.renamed` when the
    // user cache already held a different name for this UUID; the cache is
    // refreshed at login (`cache_login_profile`) before the old name is visible here.
    pub(super) fn broadcast_player_joined(&self, profile: &NameAndId) -> io::Result<()> {
        self.broadcast_system_message(
            yellow_translatable("multiplayer.player.joined", vec![player_display_name_tag(profile)]),
            &format!("{} joined the game", profile.name),
        )
    }

    /// `multiplayer.player.left` (Java `ServerGamePacketListenerImpl.removePlayerFromWorld`).
    pub(super) fn broadcast_player_left(&self, profile: &NameAndId) -> io::Result<()> {
        // The leaving session is still subscribed but is about to close, so it is
        // skipped like the departed connection in Java.
        let payload = system_chat_payload(yellow_translatable(
            "multiplayer.player.left",
            vec![player_display_name_tag(profile)],
        ))?;
        self.world_bus.publish_except(self.token, &payload);
        log_info(&format!("{} left the game", profile.name));
        Ok(())
    }
}

/// JSON of a disconnect reason: a `multiplayer.*` translation key (as produced by
/// the command model for the ban family) or literal `/kick <reason>` text.
fn disconnect_reason_json(reason: &str) -> String {
    if reason.starts_with("multiplayer.") {
        Component::translatable(reason, Vec::new()).to_json()
    } else {
        Component::literal(reason).to_json()
    }
}

/// Reason tag for `commands.kick.success`'s second argument.
fn disconnect_reason_tag(reason: &str) -> Tag {
    if reason.starts_with("multiplayer.") {
        Tag::Compound(vec![("translate".to_string(), Tag::String(reason.to_string()))])
    } else {
        text_tag(reason)
    }
}

/// Everything a finished in-game command asks of other sessions.
pub(super) struct CommandEffects<'a> {
    pub guard: &'a ActiveLoginGuard,
    pub state: &'a ServerCommandState,
    /// Feedback key of the successful result, if any (selects `/kick` feedback).
    pub success_key: Option<&'static str>,
    /// `GameRules.SEND_COMMAND_FEEDBACK` for the executor.
    pub send_feedback: bool,
}

/// Delivers a finished in-game command's cross-session effects (see [`CommandEffects`]).
pub(super) fn apply_command_effects(
    guard: &ActiveLoginGuard,
    state: &ServerCommandState,
    result: &Result<crate::command::CommandResult, crate::command::CommandError>,
    game_rules: &SharedGameRules,
) -> io::Result<()> {
    CommandEffects {
        guard,
        state,
        success_key: result.as_ref().ok().map(|ok| ok.feedback_key),
        send_feedback: lock_status_mutex(game_rules).bool("send_command_feedback"),
    }
    .apply()
}

/// A dedicated server is always "published" (`MinecraftServer.isPublished`), which
/// `/kick` requires and `/publish` rejects.
pub(super) fn dedicated_publish_request(properties: &ServerProperties) -> crate::command::PublishRequest {
    crate::command::PublishRequest {
        port: properties.server_port,
        allow_commands: true,
        gamemode: None,
    }
}

impl CommandEffects<'_> {
    /// Applies the disconnects and chat deliveries recorded in the command state.
    pub(super) fn apply(&self) -> io::Result<()> {
        self.apply_disconnects()?;
        for event in &self.state.chat_events {
            self.apply_chat_event(event)?;
        }
        Ok(())
    }

    /// Java `sendFailure`: a red translatable message to the executor.
    fn send_failure(&self, key: &str) -> io::Result<()> {
        let tag = Tag::Compound(vec![
            ("translate".to_string(), Tag::String(key.to_string())),
            ("color".to_string(), Tag::String("red".to_string())),
        ]);
        self.guard.send_to_self(&system_chat_payload(tag)?);
        Ok(())
    }

    /// `PlayerDisconnect` requests from `/kick` (`KickCommand.kickPlayers`) and the
    /// ban family. `/kick` also reports `commands.kick.success` per kicked player.
    fn apply_disconnects(&self) -> io::Result<()> {
        let is_kick = self.success_key == Some(KICK_SUCCESS_KEY);
        for request in &self.state.disconnected_players {
            let Some((target, _)) = self.guard.find_in_play_by_name(&request.player.name) else {
                if is_kick {
                    self.send_failure("argument.entity.notfound.player")?;
                }
                continue;
            };
            self.guard.disconnect_player(&target.uuid, disconnect_reason_json(&request.reason))?;
            if is_kick && self.send_feedback {
                let feedback = Tag::Compound(vec![
                    ("translate".to_string(), Tag::String(KICK_SUCCESS_KEY.to_string())),
                    (
                        "with".to_string(),
                        Tag::List(vec![
                            player_display_name_tag(&target),
                            disconnect_reason_tag(&request.reason),
                        ]),
                    ),
                ]);
                self.guard.send_to_self(&system_chat_payload(feedback)?);
            }
        }
        Ok(())
    }

    fn apply_chat_event(&self, event: &ChatCommandEvent) -> io::Result<()> {
        match event.kind {
            ChatCommandKind::Say => self.broadcast_chat(event, "say_command"),
            ChatCommandKind::Emote => self.broadcast_chat(event, "emote_command"),
            ChatCommandKind::Private => self.send_private_message(event),
            ChatCommandKind::TellRaw => self.send_tellraw(event),
            // TODO(teammsg-live): `/teammsg` needs live team membership delivery.
            ChatCommandKind::Team => Ok(()),
        }
    }

    /// Display name of the source (`CommandSourceStack.getDisplayName`); the server
    /// console is named `Server`.
    fn source_name_tag(event: &ChatCommandEvent) -> Tag {
        event.sender.as_ref().map_or_else(|| text_tag("Server"), player_display_name_tag)
    }

    /// `/say` and `/me`: `PlayerList.broadcastChatMessage` to every online player. The
    /// argument is unsigned (offline chat), so `PlayerChatMessage.isSystem()` selects
    /// `OutgoingChatMessage.Disguised`.
    fn broadcast_chat(&self, event: &ChatCommandEvent, chat_type: &str) -> io::Result<()> {
        let payload = disguised_chat_payload(
            text_tag(&event.message),
            chat_type,
            Self::source_name_tag(event),
            None,
        )?;
        self.guard.world_bus.publish(&payload);
        let sender = event.sender.as_ref().map_or("Server", |sender| sender.name.as_str());
        log_info(&format!("[{sender}] {}", event.message));
        Ok(())
    }

    /// Resolves every target by name; `argument.entity.notfound.player` aborts the
    /// whole command when one is offline (`EntityArgument.getPlayers`).
    fn resolve_targets(&self, event: &ChatCommandEvent) -> io::Result<Option<Vec<(NameAndId, u64)>>> {
        let resolved: Option<Vec<_>> = event
            .targets
            .iter()
            .map(|target| self.guard.find_in_play_by_name(&target.name))
            .collect();
        if resolved.is_none() {
            self.send_failure("argument.entity.notfound.player")?;
        }
        Ok(resolved)
    }

    /// `/msg`, `/tell`, `/w` (`MsgCommand.sendMessage`): outgoing copy to the source,
    /// incoming copy to each target.
    fn send_private_message(&self, event: &ChatCommandEvent) -> io::Result<()> {
        let Some(targets) = self.resolve_targets(event)? else { return Ok(()) };
        let source_name = Self::source_name_tag(event);
        for (target, token) in targets {
            let outgoing = disguised_chat_payload(
                text_tag(&event.message),
                "msg_command_outgoing",
                source_name.clone(),
                Some(player_display_name_tag(&target)),
            )?;
            self.guard.send_to_self(&outgoing);
            let incoming = disguised_chat_payload(
                text_tag(&event.message),
                "msg_command_incoming",
                source_name.clone(),
                None,
            )?;
            self.guard.world_bus.publish_to(token, &incoming);
        }
        Ok(())
    }

    /// `/tellraw` (`TellRawCommand`): the parsed component as a system message.
    // TODO(tellraw-resolve): `ComponentArgument.getResolvedComponent` also resolves
    // selector/score/nbt contents against each target; they are delivered unresolved.
    fn send_tellraw(&self, event: &ChatCommandEvent) -> io::Result<()> {
        let Some(component) = json_component_to_tag(&event.message) else {
            return self.send_failure("argument.component.invalid");
        };
        let Some(targets) = self.resolve_targets(event)? else { return Ok(()) };
        let payload = system_chat_payload(component)?;
        for (_, token) in targets {
            self.guard.world_bus.publish_to(token, &payload);
        }
        Ok(())
    }
}

impl ActiveLoginRegistry {
    /// Java `PlayerList.removeAll`: disconnects every in-play player with
    /// `multiplayer.disconnect.server_shutdown`, then waits (bounded) for the
    /// sessions to flush the notice and save their state.
    pub(super) fn disconnect_all_for_shutdown(&self) {
        self.disconnect_all_for_shutdown_within(SHUTDOWN_FLUSH_TIMEOUT);
    }

    fn disconnect_all_for_shutdown_within(&self, flush_timeout: Duration) {
        let reason = Component::translatable(SERVER_SHUTDOWN_DISCONNECT, Vec::new()).to_json();
        let Ok(payload) = disconnect_payload(reason) else { return };
        let tokens: Vec<u64> = match self.sessions.lock() {
            Ok(sessions) => sessions
                .values()
                .filter(|session| session.in_play)
                .map(|session| session.token)
                .collect(),
            Err(_) => return,
        };
        for token in tokens {
            self.world_bus.disconnect(token, &payload);
        }
        let deadline = Instant::now() + flush_timeout;
        while Instant::now() < deadline && self.has_in_play_sessions() {
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn has_in_play_sessions(&self) -> bool {
        self.sessions.lock().is_ok_and(|sessions| sessions.values().any(|s| s.in_play))
    }
}

#[cfg(test)]
mod tests;
