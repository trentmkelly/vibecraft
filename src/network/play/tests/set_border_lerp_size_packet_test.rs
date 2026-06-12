use super::*;

const CLIENTBOUND_SET_BORDER_LERP_SIZE_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetBorderLerpSizePacket.java"
);

#[test]
fn clientbound_set_border_lerp_size_packet_matches_java_codec() {
    for sentinel in [
        "this.oldSize = input.readDouble();",
        "this.newSize = input.readDouble();",
        "this.lerpTime = input.readVarLong();",
        "output.writeDouble(this.oldSize);",
        "output.writeDouble(this.newSize);",
        "output.writeVarLong(this.lerpTime);",
        "return GamePacketTypes.CLIENTBOUND_SET_BORDER_LERP_SIZE;",
        "listener.handleSetBorderLerpSize(this);",
        "public long getLerpTime()",
    ] {
        assert!(
            CLIENTBOUND_SET_BORDER_LERP_SIZE_JAVA.contains(sentinel),
            "missing ClientboundSetBorderLerpSizePacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_SET_BORDER_LERP_SIZE_PACKET_ID, 89);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_BORDER_LERP_SIZE_PACKET_ID),
        Some("set_border_lerp_size")
    );

    let packet = ClientboundSetBorderLerpSizePacket {
        old_size: 100.0,
        new_size: 200.0,
        lerp_time: 300,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    let mut expected = Vec::new();
    expected.extend_from_slice(&100.0_f64.to_be_bytes());
    expected.extend_from_slice(&200.0_f64.to_be_bytes());
    expected.extend_from_slice(&[0xac, 0x02]);
    assert_eq!(payload, expected);
    assert_eq!(
        ClientboundSetBorderLerpSizePacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_set_border_lerp_size_packet_rejects_malformed_payloads() {
    assert!(ClientboundSetBorderLerpSizePacket::read(&mut cursor(vec![0; 16])).is_err());
    let mut payload = Vec::new();
    ClientboundSetBorderLerpSizePacket {
        old_size: 1.0,
        new_size: 2.0,
        lerp_time: 3,
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);
    assert!(ClientboundSetBorderLerpSizePacket::read(&mut cursor(payload)).is_err());
}
