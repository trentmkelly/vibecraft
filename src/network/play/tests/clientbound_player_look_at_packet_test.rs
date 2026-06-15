use super::*;

const CLIENTBOUND_PLAYER_LOOK_AT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundPlayerLookAtPacket.java");

#[test]
fn clientbound_player_look_at_packet_matches_java_codec_with_entity_target() {
    assert_java_player_look_at_sentinels();

    assert_eq!(CLIENTBOUND_PLAYER_LOOK_AT_PACKET_ID, 71);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_LOOK_AT_PACKET_ID),
        Some("player_look_at")
    );

    let packet = ClientboundPlayerLookAtPacket {
        from_anchor: EntityAnchor::Eyes,
        x: 10.0,
        y: 64.5,
        z: -7.25,
        target_entity: Some((33, EntityAnchor::Feet)),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        [
            vec![1],
            10.0_f64.to_be_bytes().to_vec(),
            64.5_f64.to_be_bytes().to_vec(),
            (-7.25_f64).to_be_bytes().to_vec(),
            vec![1, 33, 0],
        ]
        .concat()
    );
    assert_eq!(
        ClientboundPlayerLookAtPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_player_look_at_packet_matches_java_codec_without_entity_target() {
    assert_java_player_look_at_sentinels();

    let packet = ClientboundPlayerLookAtPacket {
        from_anchor: EntityAnchor::Feet,
        x: 1.25,
        y: -2.5,
        z: 3.75,
        target_entity: None,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        [
            vec![0],
            1.25_f64.to_be_bytes().to_vec(),
            (-2.5_f64).to_be_bytes().to_vec(),
            3.75_f64.to_be_bytes().to_vec(),
            vec![0],
        ]
        .concat()
    );
    assert_eq!(
        ClientboundPlayerLookAtPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_player_look_at_packet_rejects_malformed_payloads() {
    assert!(ClientboundPlayerLookAtPacket::read(&mut cursor(vec![0])).is_err());
    assert!(ClientboundPlayerLookAtPacket::read(&mut cursor(vec![2])).is_err());

    let mut payload = Vec::new();
    ClientboundPlayerLookAtPacket {
        from_anchor: EntityAnchor::Feet,
        x: 0.0,
        y: 1.0,
        z: 2.0,
        target_entity: Some((4, EntityAnchor::Eyes)),
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);
    assert!(ClientboundPlayerLookAtPacket::read(&mut cursor(payload)).is_err());
}

fn assert_java_player_look_at_sentinels() {
    for sentinel in [
        "this.fromAnchor = input.readEnum(EntityAnchorArgument.Anchor.class);",
        "this.x = input.readDouble();",
        "this.y = input.readDouble();",
        "this.z = input.readDouble();",
        "this.atEntity = input.readBoolean();",
        "this.entity = input.readVarInt();",
        "this.toAnchor = input.readEnum(EntityAnchorArgument.Anchor.class);",
        "output.writeEnum(this.fromAnchor);",
        "output.writeDouble(this.x);",
        "output.writeDouble(this.y);",
        "output.writeDouble(this.z);",
        "output.writeBoolean(this.atEntity);",
        "output.writeVarInt(this.entity);",
        "output.writeEnum(this.toAnchor);",
        "return GamePacketTypes.CLIENTBOUND_PLAYER_LOOK_AT;",
        "listener.handleLookAt(this);",
    ] {
        assert!(
            CLIENTBOUND_PLAYER_LOOK_AT_JAVA.contains(sentinel),
            "missing ClientboundPlayerLookAtPacket sentinel {sentinel}"
        );
    }
}
