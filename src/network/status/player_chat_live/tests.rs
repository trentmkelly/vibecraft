use super::*;
use crate::network::varint::read_var_i32;
use std::io::Cursor;

fn profile() -> NameAndId {
    NameAndId {
        uuid: "00000000-0000-0000-0000-000000000001".to_string(),
        name: "Alice".to_string(),
    }
}

#[test]
fn unsigned_chat_body_has_java_unsigned_shape() {
    let body = unsigned_chat_body(&profile(), "hello", 1234).unwrap();
    let mut input = Cursor::new(body);
    // sender UUID, then link index 0 and an absent signature.
    let mut uuid = [0u8; 16];
    std::io::Read::read_exact(&mut input, &mut uuid).unwrap();
    assert_eq!(uuid[15], 1);
    assert_eq!(read_var_i32(&mut input).unwrap(), 0);
    let mut absent = [0u8; 1];
    std::io::Read::read_exact(&mut input, &mut absent).unwrap();
    assert_eq!(absent[0], 0);
}

#[test]
fn bus_stamps_each_recipients_own_chat_index() {
    let bus = crate::network::world_broadcast::WorldPacketBus::default();
    let sub = bus.subscribe(7);
    assert!(bus.publish_player_chat(7, CLIENTBOUND_PLAYER_CHAT_PACKET_ID, &[0xAA]));
    assert!(bus.publish_player_chat(7, CLIENTBOUND_PLAYER_CHAT_PACKET_ID, &[0xAA]));
    assert!(!bus.publish_player_chat(8, CLIENTBOUND_PLAYER_CHAT_PACKET_ID, &[0xAA]));
    let mut out = Vec::new();
    sub.drain_into(&mut out, CompressionState::disabled()).unwrap();
    // Two frames of `len, id, index, 0xAA`; indices 0 then 1.
    assert_eq!(out[2], 0);
    assert_eq!(out[6], 1);
}

#[test]
fn secure_profile_is_not_enforced_without_key_validation() {
    let dir = std::env::temp_dir().join("vibecraft_chat_live_props");
    let mut properties =
        ServerProperties::load_or_default(&dir.join("server.properties")).unwrap();
    properties.enforce_secure_profile = true;
    properties.online_mode = true;
    assert!(!enforce_secure_profile(&properties));
}
