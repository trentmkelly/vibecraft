use super::*;

const CLIENTBOUND_PLAYER_ABILITIES_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundPlayerAbilitiesPacket.java");

#[test]
fn clientbound_player_abilities_packet_matches_java_codec() {
    for sentinel in [
        "private static final int FLAG_INVULNERABLE = 1;",
        "private static final int FLAG_FLYING = 2;",
        "private static final int FLAG_CAN_FLY = 4;",
        "private static final int FLAG_INSTABUILD = 8;",
        "this.invulnerable = abilities.invulnerable;",
        "this.isFlying = abilities.flying;",
        "this.canFly = abilities.mayfly;",
        "this.instabuild = abilities.instabuild;",
        "byte bitfield = input.readByte();",
        "this.flyingSpeed = input.readFloat();",
        "this.walkingSpeed = input.readFloat();",
        "output.writeByte(bitfield);",
        "output.writeFloat(this.flyingSpeed);",
        "output.writeFloat(this.walkingSpeed);",
        "return GamePacketTypes.CLIENTBOUND_PLAYER_ABILITIES;",
        "listener.handlePlayerAbilities(this);",
        "public boolean isInvulnerable()",
        "public boolean isFlying()",
        "public boolean canFly()",
        "public boolean canInstabuild()",
    ] {
        assert!(
            CLIENTBOUND_PLAYER_ABILITIES_JAVA.contains(sentinel),
            "missing ClientboundPlayerAbilitiesPacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID, 64);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID),
        Some("player_abilities")
    );

    let packet = ClientboundPlayerAbilitiesPacket {
        invulnerable: true,
        flying: false,
        can_fly: true,
        instant_build: true,
        flying_speed: 0.05,
        walking_speed: 0.1,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        [
            vec![0b1101],
            0.05_f32.to_be_bytes().to_vec(),
            0.1_f32.to_be_bytes().to_vec(),
        ]
        .concat()
    );
    assert_eq!(
        ClientboundPlayerAbilitiesPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_player_abilities_packet_rejects_malformed_payloads() {
    assert!(ClientboundPlayerAbilitiesPacket::read(&mut cursor(vec![0; 8])).is_err());

    let mut payload = Vec::new();
    ClientboundPlayerAbilitiesPacket {
        invulnerable: false,
        flying: true,
        can_fly: false,
        instant_build: false,
        flying_speed: 1.0,
        walking_speed: 2.0,
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);
    assert!(ClientboundPlayerAbilitiesPacket::read(&mut cursor(payload)).is_err());
}
