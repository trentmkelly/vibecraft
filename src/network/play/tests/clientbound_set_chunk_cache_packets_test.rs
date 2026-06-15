use super::*;

const CLIENTBOUND_SET_CHUNK_CACHE_CENTER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetChunkCacheCenterPacket.java");
const CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetChunkCacheRadiusPacket.java");

#[test]
fn clientbound_set_chunk_cache_packets_match_java_codecs() {
    for sentinel in [
        "this.x = input.readVarInt();",
        "this.z = input.readVarInt();",
        "output.writeVarInt(this.x);",
        "output.writeVarInt(this.z);",
        "return GamePacketTypes.CLIENTBOUND_SET_CHUNK_CACHE_CENTER;",
        "listener.handleSetChunkCacheCenter(this);",
        "public int getX()",
        "public int getZ()",
    ] {
        assert!(
            CLIENTBOUND_SET_CHUNK_CACHE_CENTER_JAVA.contains(sentinel),
            "missing ClientboundSetChunkCacheCenterPacket sentinel {sentinel}"
        );
    }
    for sentinel in [
        "this.radius = input.readVarInt();",
        "output.writeVarInt(this.radius);",
        "return GamePacketTypes.CLIENTBOUND_SET_CHUNK_CACHE_RADIUS;",
        "listener.handleSetChunkCacheRadius(this);",
        "public int getRadius()",
    ] {
        assert!(
            CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_JAVA.contains(sentinel),
            "missing ClientboundSetChunkCacheRadiusPacket sentinel {sentinel}"
        );
    }

    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID, 94);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID),
        Some("set_chunk_cache_center")
    );
    assert_eq!(CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID, 95);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID),
        Some("set_chunk_cache_radius")
    );

    let center = ClientboundSetChunkCacheCenterPacket { x: 12, z: -34 };
    let mut center_payload = Vec::new();
    center.write(&mut center_payload).unwrap();
    assert_eq!(center_payload, vec![12, 0xde, 0xff, 0xff, 0xff, 0x0f]);
    assert_eq!(
        ClientboundSetChunkCacheCenterPacket::read(&mut cursor(center_payload)).unwrap(),
        center
    );

    let radius = ClientboundSetChunkCacheRadiusPacket { radius: 33 };
    let mut radius_payload = Vec::new();
    radius.write(&mut radius_payload).unwrap();
    assert_eq!(radius_payload, vec![33]);
    assert_eq!(
        ClientboundSetChunkCacheRadiusPacket::read(&mut cursor(radius_payload)).unwrap(),
        radius
    );
}

#[test]
fn clientbound_set_chunk_cache_packets_reject_malformed_payloads() {
    assert!(ClientboundSetChunkCacheCenterPacket::read(&mut cursor(vec![1])).is_err());
    assert!(ClientboundSetChunkCacheCenterPacket::read(&mut cursor(vec![1, 2, 0])).is_err());
    assert!(ClientboundSetChunkCacheRadiusPacket::read(&mut cursor(Vec::new())).is_err());
    assert!(ClientboundSetChunkCacheRadiusPacket::read(&mut cursor(vec![1, 0])).is_err());
}
