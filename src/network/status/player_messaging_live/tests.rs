use super::*;
use crate::command::{PlayerDisconnect, ServerCommandState};
use crate::network::varint::read_var_i32;

fn loopback_stream() -> ClientStream {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let client = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let _server = listener.accept().unwrap();
    ClientStream::new(client)
}

struct Player {
    profile: NameAndId,
    guard: ActiveLoginGuard,
    inbox: Subscription,
}

fn join(registry: &ActiveLoginRegistry, name: &str) -> Player {
    let profile = NameAndId::create_offline(name);
    let (guard, _) = registry
        .register_replacing(&profile.uuid, name, &loopback_stream())
        .unwrap();
    guard.mark_in_play();
    let inbox = registry.world_bus.subscribe(guard.token);
    Player { profile, guard, inbox }
}

/// Drains an inbox into `(packet id, whole plain payload)` pairs.
fn drain(player: &Player) -> Vec<(i32, Vec<u8>)> {
    let mut framed = Vec::new();
    player
        .inbox
        .drain_into(&mut framed, CompressionState::disabled())
        .unwrap();
    let mut rest = framed.as_slice();
    let mut packets = Vec::new();
    while !rest.is_empty() {
        let length = read_var_i32(&mut rest).unwrap() as usize;
        let (payload, tail) = rest.split_at(length);
        packets.push((read_var_i32(&mut &payload[..]).unwrap(), payload.to_vec()));
        rest = tail;
    }
    packets
}

fn contains(haystack: &[u8], needle: &str) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle.as_bytes())
}

fn effects<'a>(
    guard: &'a ActiveLoginGuard,
    state: &'a ServerCommandState,
    success_key: Option<&'static str>,
) -> CommandEffects<'a> {
    CommandEffects { guard, state, success_key, send_feedback: true }
}

fn chat_event(kind: ChatCommandKind, sender: &NameAndId, targets: &[&str], message: &str) -> ChatCommandEvent {
    ChatCommandEvent {
        kind,
        sender: Some(sender.clone()),
        targets: targets.iter().map(|name| NameAndId::create_offline(name)).collect(),
        message: message.to_string(),
    }
}

#[test]
fn chat_types_use_alphabetical_registry_ids() {
    assert_eq!(chat_type_id("chat"), 0);
    assert_eq!(chat_type_id("emote_command"), 1);
    assert_eq!(chat_type_id("msg_command_incoming"), 2);
    assert_eq!(chat_type_id("msg_command_outgoing"), 3);
    assert_eq!(chat_type_id("say_command"), 4);
}

#[test]
fn say_reaches_every_online_player_as_disguised_chat() {
    let registry = ActiveLoginRegistry::default();
    let alex = join(&registry, "Alex");
    let steve = join(&registry, "Steve");
    let mut state = ServerCommandState::default();
    state.chat_events.push(chat_event(ChatCommandKind::Say, &alex.profile, &[], "hello all"));
    effects(&alex.guard, &state, None).apply().unwrap();
    for player in [&alex, &steve] {
        let packets = drain(player);
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].0, CLIENTBOUND_DISGUISED_CHAT_PACKET_ID);
        assert!(contains(&packets[0].1, "hello all"));
    }
}

#[test]
fn msg_sends_outgoing_copy_to_source_and_incoming_to_target_only() {
    let registry = ActiveLoginRegistry::default();
    let alex = join(&registry, "Alex");
    let steve = join(&registry, "Steve");
    let bystander = join(&registry, "Bystander");
    let mut state = ServerCommandState::default();
    state.chat_events.push(chat_event(ChatCommandKind::Private, &alex.profile, &["steve"], "psst"));
    effects(&alex.guard, &state, None).apply().unwrap();

    let outgoing = drain(&alex);
    assert_eq!(outgoing.len(), 1);
    assert!(contains(&outgoing[0].1, "psst"));
    let incoming = drain(&steve);
    assert_eq!(incoming.len(), 1);
    assert_ne!(incoming[0].1, outgoing[0].1, "incoming and outgoing chat types differ");
    assert!(drain(&bystander).is_empty());
}

#[test]
fn msg_to_an_offline_player_fails_without_delivering() {
    let registry = ActiveLoginRegistry::default();
    let alex = join(&registry, "Alex");
    let steve = join(&registry, "Steve");
    let mut state = ServerCommandState::default();
    state.chat_events.push(chat_event(ChatCommandKind::Private, &alex.profile, &["Steve", "Ghost"], "hi"));
    effects(&alex.guard, &state, None).apply().unwrap();
    let failure = drain(&alex);
    assert_eq!(failure.len(), 1);
    assert!(contains(&failure[0].1, "argument.entity.notfound.player"));
    assert!(drain(&steve).is_empty());
}

#[test]
fn tellraw_delivers_system_chat_to_targets_only() {
    let registry = ActiveLoginRegistry::default();
    let alex = join(&registry, "Alex");
    let steve = join(&registry, "Steve");
    let mut state = ServerCommandState::default();
    state.chat_events.push(chat_event(
        ChatCommandKind::TellRaw,
        &alex.profile,
        &["Steve"],
        r#"{"text":"raw hi","color":"gold"}"#,
    ));
    effects(&alex.guard, &state, None).apply().unwrap();
    let delivered = drain(&steve);
    assert_eq!(delivered[0].0, CLIENTBOUND_SYSTEM_CHAT_PACKET_ID);
    assert!(contains(&delivered[0].1, "raw hi"));
    assert!(drain(&alex).is_empty());
}

#[test]
fn tellraw_rejects_invalid_json() {
    let registry = ActiveLoginRegistry::default();
    let alex = join(&registry, "Alex");
    let mut state = ServerCommandState::default();
    state.chat_events.push(chat_event(ChatCommandKind::TellRaw, &alex.profile, &["Alex"], "{oops"));
    effects(&alex.guard, &state, None).apply().unwrap();
    assert!(contains(&drain(&alex)[0].1, "argument.component.invalid"));
}

#[test]
fn json_components_convert_to_network_nbt() {
    assert_eq!(json_component_to_tag("\"plain\""), Some(text_tag("plain")));
    let Some(Tag::Compound(fields)) =
        json_component_to_tag(r#"{"text":"a","bold":true,"extra":["b",{"text":"c"}]}"#)
    else {
        panic!("expected compound");
    };
    assert!(fields.contains(&("bold".to_string(), Tag::Byte(1))));
    let extra = fields.iter().find(|(k, _)| k == "extra").unwrap();
    assert_eq!(
        extra.1,
        Tag::List(vec![text_tag("b"), Tag::Compound(vec![("text".to_string(), Tag::String("c".into()))])])
    );
    assert_eq!(json_component_to_tag("[\"x\"]"), Some(text_tag("x")));
    assert_eq!(json_component_to_tag("nope"), None);
}

#[test]
fn kick_disconnects_only_the_target_and_reports_success() {
    let registry = ActiveLoginRegistry::default();
    let admin = join(&registry, "Admin");
    let victim = join(&registry, "Victim");
    let other = join(&registry, "Other");
    let mut state = ServerCommandState::default();
    state.disconnected_players.push(PlayerDisconnect {
        player: NameAndId::create_offline("victim"),
        reason: "multiplayer.disconnect.kicked".to_string(),
    });
    effects(&admin.guard, &state, Some(KICK_SUCCESS_KEY)).apply().unwrap();

    let notice = drain(&victim);
    assert_eq!(notice.len(), 1);
    assert_eq!(notice[0].0, CLIENTBOUND_DISCONNECT_PACKET_ID);
    assert!(contains(&notice[0].1, "multiplayer.disconnect.kicked"));
    assert!(victim.inbox.is_closing());
    assert!(!other.inbox.is_closing());
    assert!(drain(&other).is_empty());
    let feedback = drain(&admin);
    assert_eq!(feedback[0].0, CLIENTBOUND_SYSTEM_CHAT_PACKET_ID);
    assert!(contains(&feedback[0].1, KICK_SUCCESS_KEY));
}

#[test]
fn kick_uses_literal_reason_text_and_honours_send_command_feedback() {
    let registry = ActiveLoginRegistry::default();
    let admin = join(&registry, "Admin");
    let victim = join(&registry, "Victim");
    let mut state = ServerCommandState::default();
    state.disconnected_players.push(PlayerDisconnect {
        player: NameAndId::create_offline("Victim"),
        reason: "griefing".to_string(),
    });
    let mut silent = effects(&admin.guard, &state, Some(KICK_SUCCESS_KEY));
    silent.send_feedback = false;
    silent.apply().unwrap();
    assert!(contains(&drain(&victim)[0].1, "griefing"));
    assert!(drain(&admin).is_empty());
}

#[test]
fn kick_of_an_offline_player_reports_not_found() {
    let registry = ActiveLoginRegistry::default();
    let admin = join(&registry, "Admin");
    let mut state = ServerCommandState::default();
    state.disconnected_players.push(PlayerDisconnect {
        player: NameAndId::create_offline("Ghost"),
        reason: "x".to_string(),
    });
    effects(&admin.guard, &state, Some(KICK_SUCCESS_KEY)).apply().unwrap();
    assert!(contains(&drain(&admin)[0].1, "argument.entity.notfound.player"));
}

#[test]
fn join_is_broadcast_to_everyone_and_quit_skips_the_leaver() {
    let registry = ActiveLoginRegistry::default();
    let alex = join(&registry, "Alex");
    let steve = join(&registry, "Steve");
    alex.guard.broadcast_player_joined(&alex.profile).unwrap();
    for player in [&alex, &steve] {
        let joined = drain(player);
        assert!(contains(&joined[0].1, "multiplayer.player.joined"));
        assert!(contains(&joined[0].1, "yellow"));
    }
    alex.guard.broadcast_player_left(&alex.profile).unwrap();
    assert!(drain(&alex).is_empty());
    assert!(contains(&drain(&steve)[0].1, "multiplayer.player.left"));
}

#[test]
fn shutdown_disconnects_every_in_play_player_with_the_shutdown_reason() {
    let registry = ActiveLoginRegistry::default();
    let alex = join(&registry, "Alex");
    let steve = join(&registry, "Steve");
    registry.disconnect_all_for_shutdown_within(Duration::ZERO);
    for player in [&alex, &steve] {
        let notice = drain(player);
        assert_eq!(notice[0].0, CLIENTBOUND_DISCONNECT_PACKET_ID);
        assert!(contains(&notice[0].1, "multiplayer.disconnect.server_shutdown"));
        assert!(player.inbox.is_closing());
    }
}
