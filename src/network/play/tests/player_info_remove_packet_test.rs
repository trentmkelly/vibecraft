use super::*;

const CLIENTBOUND_PLAYER_INFO_REMOVE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundPlayerInfoRemovePacket.java");

#[test]
fn clientbound_player_info_remove_packet_matches_java_codec() {
    for sentinel in [
        "this(input.readList(UUIDUtil.STREAM_CODEC));",
        "output.writeCollection(this.profileIds, UUIDUtil.STREAM_CODEC);",
        "return GamePacketTypes.CLIENTBOUND_PLAYER_INFO_REMOVE;",
        "listener.handlePlayerInfoRemove(this);",
    ] {
        assert!(
            CLIENTBOUND_PLAYER_INFO_REMOVE_JAVA.contains(sentinel),
            "missing ClientboundPlayerInfoRemovePacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_PLAYER_INFO_REMOVE_PACKET_ID, 69);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_INFO_REMOVE_PACKET_ID),
        Some("player_info_remove")
    );

    let first = Uuid([
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff,
    ]);
    let second = Uuid([
        0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
        0x00,
    ]);

    let packet = ClientboundPlayerInfoRemovePacket {
        profile_ids: vec![first, second],
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(payload[0], 2);
    assert_eq!(&payload[1..17], &first.0);
    assert_eq!(&payload[17..33], &second.0);
    assert_eq!(payload.len(), 33);
    assert_eq!(
        ClientboundPlayerInfoRemovePacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_player_info_remove_packet_rejects_malformed_payloads() {
    assert!(ClientboundPlayerInfoRemovePacket::read(&mut cursor(vec![1; 16])).is_err());

    let mut payload = Vec::new();
    ClientboundPlayerInfoRemovePacket {
        profile_ids: vec![Uuid([1; 16])],
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);
    assert!(ClientboundPlayerInfoRemovePacket::read(&mut cursor(payload)).is_err());
}
