use super::*;

const CLIENTBOUND_SERVER_DATA_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundServerDataPacket.java"
);
const COMPONENT_SERIALIZATION_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/chat/ComponentSerialization.java"
);
const BYTE_BUF_CODECS_JAVA: &str =
    include_str!("../../../../../decompiled-server-26.1.2/net/minecraft/network/codec/ByteBufCodecs.java");
const FRIENDLY_BYTE_BUF_JAVA: &str =
    include_str!("../../../../../decompiled-server-26.1.2/net/minecraft/network/FriendlyByteBuf.java");

#[test]
fn clientbound_server_data_packet_matches_java_codec() {
    for sentinel in [
        "public record ClientboundServerDataPacket(Component motd, Optional<byte[]> iconBytes)",
        "ComponentSerialization.TRUSTED_CONTEXT_FREE_STREAM_CODEC",
        "ClientboundServerDataPacket::motd",
        "ByteBufCodecs.BYTE_ARRAY.apply(ByteBufCodecs::optional)",
        "ClientboundServerDataPacket::iconBytes",
        "return GamePacketTypes.CLIENTBOUND_SERVER_DATA;",
        "listener.handleServerData(this);",
    ] {
        assert!(
            CLIENTBOUND_SERVER_DATA_JAVA.contains(sentinel),
            "missing ClientboundServerDataPacket sentinel {sentinel}"
        );
    }
    assert!(
        COMPONENT_SERIALIZATION_JAVA
            .contains("TRUSTED_CONTEXT_FREE_STREAM_CODEC = ByteBufCodecs.fromCodecTrusted(CODEC)"),
        "missing trusted context-free component sentinel"
    );
    assert!(
        BYTE_BUF_CODECS_JAVA.contains("FriendlyByteBuf.writeByteArray(output, value);"),
        "missing byte-array encode sentinel"
    );
    assert!(
        FRIENDLY_BYTE_BUF_JAVA.contains("VarInt.write(output, bytes.length);"),
        "missing byte-array length sentinel"
    );

    assert_eq!(CLIENTBOUND_SERVER_DATA_PACKET_ID, 86);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SERVER_DATA_PACKET_ID),
        Some("server_data")
    );

    let packet = ClientboundServerDataPacket {
        motd: ComponentJson("{\"text\":\"VibeCraft\"}".to_string()),
        icon_bytes: Some(vec![0xde, 0xad, 0xbe, 0xef]),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![
            8, // trusted component encoded through NBT string tag
            0, 9, b'V', b'i', b'b', b'e', b'C', b'r', b'a', b'f', b't',
            1, // icon present
            4, 0xde, 0xad, 0xbe, 0xef,
        ]
    );
    assert_eq!(
        ClientboundServerDataPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );

    let no_icon = ClientboundServerDataPacket {
        motd: ComponentJson("{\"text\":\"No icon\"}".to_string()),
        icon_bytes: None,
    };
    let mut payload = Vec::new();
    no_icon.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![8, 0, 7, b'N', b'o', b' ', b'i', b'c', b'o', b'n', 0]
    );
    assert_eq!(
        ClientboundServerDataPacket::read(&mut cursor(payload)).unwrap(),
        no_icon
    );
}

#[test]
fn clientbound_server_data_packet_rejects_malformed_payloads() {
    assert!(ClientboundServerDataPacket::read(&mut cursor(Vec::new())).is_err());

    let mut truncated_icon = Vec::new();
    ClientboundServerDataPacket {
        motd: ComponentJson("{\"text\":\"x\"}".to_string()),
        icon_bytes: Some(vec![1, 2, 3]),
    }
    .write(&mut truncated_icon)
    .unwrap();
    truncated_icon.pop();
    assert!(ClientboundServerDataPacket::read(&mut cursor(truncated_icon)).is_err());

    let mut trailing = Vec::new();
    ClientboundServerDataPacket {
        motd: ComponentJson("{\"text\":\"x\"}".to_string()),
        icon_bytes: None,
    }
    .write(&mut trailing)
    .unwrap();
    trailing.push(0);
    assert!(ClientboundServerDataPacket::read(&mut cursor(trailing)).is_err());
}
