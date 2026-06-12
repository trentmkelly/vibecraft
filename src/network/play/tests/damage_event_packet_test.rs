use super::*;

const CLIENTBOUND_DAMAGE_EVENT_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundDamageEventPacket.java"
);
const DAMAGE_TYPE_JAVA: &str =
    include_str!("../../../../../decompiled-server-26.1.2/net/minecraft/world/damagesource/DamageType.java");

#[test]
fn clientbound_damage_event_packet_matches_java_codec_with_position() {
    assert_java_damage_event_sentinels();

    assert_eq!(CLIENTBOUND_DAMAGE_EVENT_PACKET_ID, 25);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_DAMAGE_EVENT_PACKET_ID),
        Some("damage_event")
    );

    let packet = ClientboundDamageEventPacket {
        entity_id: 300,
        source_type_id: 24,
        source_cause_id: -1,
        source_direct_id: 42,
        source_position: Some(Vec3 {
            x: 1.25,
            y: 64.0,
            z: -3.5,
        }),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        [
            vec![0xac, 0x02, 24, 0, 43, 1],
            1.25_f64.to_be_bytes().to_vec(),
            64.0_f64.to_be_bytes().to_vec(),
            (-3.5_f64).to_be_bytes().to_vec(),
        ]
        .concat()
    );
    assert_eq!(
        ClientboundDamageEventPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_damage_event_packet_matches_java_codec_without_position() {
    assert_java_damage_event_sentinels();

    let packet = ClientboundDamageEventPacket {
        entity_id: 7,
        source_type_id: 8,
        source_cause_id: 12,
        source_direct_id: -1,
        source_position: None,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![7, 8, 13, 0, 0]);
    assert_eq!(
        ClientboundDamageEventPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_damage_event_packet_rejects_malformed_payloads() {
    assert!(ClientboundDamageEventPacket::read(&mut cursor(vec![1, 2, 3, 4])).is_err());

    let mut payload = Vec::new();
    ClientboundDamageEventPacket {
        entity_id: 1,
        source_type_id: 2,
        source_cause_id: -1,
        source_direct_id: -1,
        source_position: None,
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);
    assert!(ClientboundDamageEventPacket::read(&mut cursor(payload)).is_err());
}

fn assert_java_damage_event_sentinels() {
    for sentinel in [
        "input.readVarInt()",
        "DamageType.STREAM_CODEC.decode(input)",
        "readOptionalEntityId(input)",
        "input.readOptional(i -> new Vec3(i.readDouble(), i.readDouble(), i.readDouble()))",
        "output.writeVarInt(this.entityId);",
        "DamageType.STREAM_CODEC.encode(output, this.sourceType);",
        "output.writeVarInt(id + 1);",
        "return input.readVarInt() - 1;",
        "output.writeOptional(this.sourcePosition",
        "return GamePacketTypes.CLIENTBOUND_DAMAGE_EVENT;",
        "listener.handleDamageEvent(this);",
    ] {
        assert!(
            CLIENTBOUND_DAMAGE_EVENT_JAVA.contains(sentinel),
            "missing ClientboundDamageEventPacket sentinel {sentinel}"
        );
    }
    assert!(
        DAMAGE_TYPE_JAVA
            .contains("STREAM_CODEC = ByteBufCodecs.holderRegistry(Registries.DAMAGE_TYPE)"),
        "missing DamageType holder registry stream codec sentinel"
    );
}
