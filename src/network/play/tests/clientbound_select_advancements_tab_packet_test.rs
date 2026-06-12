use super::*;

const CLIENTBOUND_SELECT_ADVANCEMENTS_TAB_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSelectAdvancementsTabPacket.java"
);

#[test]
fn clientbound_select_advancements_tab_packet_matches_java_codec() {
    for sentinel in [
        "this.tab = input.readNullable(FriendlyByteBuf::readIdentifier);",
        "output.writeNullable(this.tab, FriendlyByteBuf::writeIdentifier);",
        "return GamePacketTypes.CLIENTBOUND_SELECT_ADVANCEMENTS_TAB;",
        "listener.handleSelectAdvancementsTab(this);",
        "public @Nullable Identifier getTab()",
    ] {
        assert!(
            CLIENTBOUND_SELECT_ADVANCEMENTS_TAB_JAVA.contains(sentinel),
            "missing ClientboundSelectAdvancementsTabPacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_SELECT_ADVANCEMENTS_TAB_PACKET_ID, 85);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SELECT_ADVANCEMENTS_TAB_PACKET_ID),
        Some("select_advancements_tab")
    );

    let packet = ClientboundSelectAdvancementsTabPacket {
        tab: Some(Identifier::parse("minecraft:story/root").unwrap()),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        [
            vec![1, 20],
            b"minecraft:story/root".to_vec(),
        ]
        .concat()
    );
    assert_eq!(
        ClientboundSelectAdvancementsTabPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );

    let clear = ClientboundSelectAdvancementsTabPacket { tab: None };
    let mut payload = Vec::new();
    clear.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0]);
    assert_eq!(
        ClientboundSelectAdvancementsTabPacket::read(&mut cursor(payload)).unwrap(),
        clear
    );
}

#[test]
fn clientbound_select_advancements_tab_packet_rejects_malformed_payloads() {
    assert!(ClientboundSelectAdvancementsTabPacket::read(&mut cursor(Vec::new())).is_err());
    assert!(ClientboundSelectAdvancementsTabPacket::read(&mut cursor(vec![1])).is_err());
    assert!(ClientboundSelectAdvancementsTabPacket::read(&mut cursor(vec![0, 0])).is_err());
}
