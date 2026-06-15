use super::*;

const CLIENTBOUND_PROJECTILE_POWER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundProjectilePowerPacket.java");

#[test]
fn clientbound_projectile_power_packet_matches_java_codec() {
    for sentinel in [
        "this.id = input.readVarInt();",
        "this.accelerationPower = input.readDouble();",
        "output.writeVarInt(this.id);",
        "output.writeDouble(this.accelerationPower);",
        "return GamePacketTypes.CLIENTBOUND_PROJECTILE_POWER;",
        "listener.handleProjectilePowerPacket(this);",
        "public int getId()",
        "public double getAccelerationPower()",
    ] {
        assert!(
            CLIENTBOUND_PROJECTILE_POWER_JAVA.contains(sentinel),
            "missing ClientboundProjectilePowerPacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_PROJECTILE_POWER_PACKET_ID, 135);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PROJECTILE_POWER_PACKET_ID),
        Some("projectile_power")
    );

    let packet = ClientboundProjectilePowerPacket {
        id: 300,
        acceleration_power: 1.25,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        [vec![0xac, 0x02], 1.25_f64.to_be_bytes().to_vec()].concat()
    );
    assert_eq!(
        ClientboundProjectilePowerPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_projectile_power_packet_rejects_malformed_payloads() {
    assert!(ClientboundProjectilePowerPacket::read(&mut cursor(vec![1; 8])).is_err());

    let mut payload = Vec::new();
    ClientboundProjectilePowerPacket {
        id: 1,
        acceleration_power: 2.0,
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);
    assert!(ClientboundProjectilePowerPacket::read(&mut cursor(payload)).is_err());
}
