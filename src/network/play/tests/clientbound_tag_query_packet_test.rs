use super::*;

const CLIENTBOUND_TAG_QUERY_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTagQueryPacket.java"
);
const FRIENDLY_BYTE_BUF_JAVA: &str =
    include_str!("../../../../../decompiled-server-26.1.2/net/minecraft/network/FriendlyByteBuf.java");

#[test]
fn clientbound_tag_query_packet_matches_java_codec() {
    for sentinel in [
        "this.transactionId = input.readVarInt();",
        "this.tag = input.readNbt();",
        "output.writeVarInt(this.transactionId);",
        "output.writeNbt(this.tag);",
        "return GamePacketTypes.CLIENTBOUND_TAG_QUERY;",
        "listener.handleTagQueryPacket(this);",
        "public boolean isSkippable()",
        "return true;",
    ] {
        assert!(
            CLIENTBOUND_TAG_QUERY_JAVA.contains(sentinel),
            "missing ClientboundTagQueryPacket sentinel {sentinel}"
        );
    }
    for sentinel in [
        "if (tag == null) {\n         tag = EndTag.INSTANCE;",
        "NbtIo.writeAnyTag(tag, new ByteBufOutputStream(output));",
        "Tag result = readNbt(input, NbtAccounter.defaultQuota());",
        "if (result != null && !(result instanceof CompoundTag))",
        "return tag.getId() == 0 ? null : tag;",
    ] {
        assert!(
            FRIENDLY_BYTE_BUF_JAVA.contains(sentinel),
            "missing FriendlyByteBuf NBT sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_TAG_QUERY_PACKET_ID, 123);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_TAG_QUERY_PACKET_ID),
        Some("tag_query")
    );

    let packet = ClientboundTagQueryPacket {
        transaction_id: 300,
        tag: Some(Tag::Compound(vec![("answer".to_string(), Tag::Int(42))])),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![
            0xac, 0x02, // transaction id
            10, // compound tag id, no root name in FriendlyByteBuf.writeNbt
            3, 0, 6, b'a', b'n', b's', b'w', b'e', b'r', 0, 0, 0, 42, 0
        ]
    );
    assert_eq!(
        ClientboundTagQueryPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_tag_query_packet_writes_null_as_end_tag() {
    let packet = ClientboundTagQueryPacket {
        transaction_id: 1,
        tag: None,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![1, 0]);
    assert_eq!(
        ClientboundTagQueryPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_tag_query_packet_rejects_non_compound_and_trailing_payloads() {
    let mut non_compound = vec![1, 3];
    non_compound.extend_from_slice(&7_i32.to_be_bytes());
    assert!(ClientboundTagQueryPacket::read(&mut cursor(non_compound)).is_err());

    let trailing = vec![1, 0, 0];
    assert!(ClientboundTagQueryPacket::read(&mut cursor(trailing)).is_err());
}
