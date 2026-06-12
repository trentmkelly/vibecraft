use super::*;

const CLIENTBOUND_START_CONFIGURATION_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundStartConfigurationPacket.java"
);

#[test]
fn clientbound_start_configuration_packet_matches_java_unit_codec() {
    for sentinel in [
        "public static final ClientboundStartConfigurationPacket INSTANCE",
        "StreamCodec.unit(INSTANCE)",
        "return GamePacketTypes.CLIENTBOUND_START_CONFIGURATION;",
        "listener.handleConfigurationStart(this);",
        "public boolean isTerminal()",
        "return true;",
    ] {
        assert!(
            CLIENTBOUND_START_CONFIGURATION_JAVA.contains(sentinel),
            "Java source missing sentinel: {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_START_CONFIGURATION_PACKET_ID, 118);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_START_CONFIGURATION_PACKET_ID),
        Some("start_configuration")
    );

    let mut payload = Vec::new();
    ClientboundStartConfigurationPacket
        .write(&mut payload)
        .unwrap();
    assert!(payload.is_empty());
}
