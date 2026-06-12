use super::*;
use super::super::chunk_waypoint::*;

const CLIENTBOUND_TRACKED_WAYPOINT_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTrackedWaypointPacket.java"
);
const TRACKED_WAYPOINT_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/world/waypoints/TrackedWaypoint.java"
);
const WAYPOINT_JAVA: &str =
    include_str!("../../../../../decompiled-server-26.1.2/net/minecraft/world/waypoints/Waypoint.java");
const WAYPOINT_STYLE_ASSETS_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/world/waypoints/WaypointStyleAssets.java"
);

#[test]
fn clientbound_tracked_waypoint_packet_matches_java_codec() {
    assert_java_contains(
        CLIENTBOUND_TRACKED_WAYPOINT_JAVA,
        &[
            "Operation.STREAM_CODEC",
            "TrackedWaypoint.STREAM_CODEC",
            "removeWaypoint(final UUID identifier)",
            "addWaypointPosition(final UUID identifier, final Waypoint.Icon icon, final Vec3i position)",
            "addWaypointChunk(final UUID identifier, final Waypoint.Icon icon, final ChunkPos chunk)",
            "addWaypointAzimuth(final UUID identifier, final Waypoint.Icon icon, final float angle)",
            "return GamePacketTypes.CLIENTBOUND_WAYPOINT;",
            "listener.handleWaypoint(this);",
            "ByIdMap.OutOfBoundsStrategy.WRAP",
        ],
    );
    assert_java_contains(
        TRACKED_WAYPOINT_JAVA,
        &[
            "byteBuf.writeEither(this.identifier, UUIDUtil.STREAM_CODEC, FriendlyByteBuf::writeUtf);",
            "Waypoint.Icon.STREAM_CODEC.encode(byteBuf, this.icon);",
            "byteBuf.writeEnum(this.type);",
            "VarInt.write(buf, this.vector.getX());",
            "VarInt.write(buf, this.chunkPos.x());",
            "buf.writeFloat(this.angle);",
            "EMPTY(TrackedWaypoint.EmptyWaypoint::new)",
            "VEC3I(TrackedWaypoint.Vec3iWaypoint::new)",
            "CHUNK(TrackedWaypoint.ChunkWaypoint::new)",
            "AZIMUTH(TrackedWaypoint.AzimuthWaypoint::new)",
        ],
    );
    assert_java_contains(
        WAYPOINT_JAVA,
        &[
            "ResourceKey.streamCodec(WaypointStyleAssets.ROOT_ID)",
            "ByteBufCodecs.optional(ByteBufCodecs.RGB_COLOR)",
            "public static final Waypoint.Icon NULL = new Waypoint.Icon();",
            "public ResourceKey<WaypointStyleAsset> style = WaypointStyleAssets.DEFAULT;",
            "public Optional<Integer> color = Optional.empty();",
        ],
    );
    assert_java_contains(
        WAYPOINT_STYLE_ASSETS_JAVA,
        &[
            "ResourceKey.createRegistryKey(Identifier.withDefaultNamespace(\"waypoint_style_asset\"))",
            "ResourceKey<WaypointStyleAsset> DEFAULT = createId(\"default\");",
            "ResourceKey<WaypointStyleAsset> BOWTIE = createId(\"bowtie\");",
        ],
    );

    assert_eq!(CLIENTBOUND_WAYPOINT_PACKET_ID, 138);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_WAYPOINT_PACKET_ID),
        Some("waypoint")
    );

    let id = Uuid([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let bowtie = WaypointIcon::new(id_("minecraft:bowtie"), Some(-0x00_112234));
    assert_waypoint_round_trip(
        ClientboundTrackedWaypointPacket::add_waypoint_position(id, bowtie.clone(), 300, -64, 42),
        vec![
            0, 1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 16, b'm', b'i',
            b'n', b'e', b'c', b'r', b'a', b'f', b't', b':', b'b', b'o', b'w', b't', b'i',
            b'e', 1, 0xee, 0xdd, 0xcc, 1, 0xac, 0x02, 0xc0, 0xff, 0xff, 0xff, 0x0f, 42,
        ],
    );
    assert_waypoint_round_trip(
        ClientboundTrackedWaypointPacket::update_waypoint_chunk(id, WaypointIcon::null(), -2, 300),
        vec![
            2, 1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, b'm', b'i',
            b'n', b'e', b'c', b'r', b'a', b'f', b't', b':', b'd', b'e', b'f', b'a', b'u',
            b'l', b't', 0, 2, 0xfe, 0xff, 0xff, 0xff, 0x0f, 0xac, 0x02,
        ],
    );
    assert_waypoint_round_trip(
        ClientboundTrackedWaypointPacket::add_waypoint_azimuth(id, bowtie, 1.5),
        vec![
            0, 1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 16, b'm', b'i',
            b'n', b'e', b'c', b'r', b'a', b'f', b't', b':', b'b', b'o', b'w', b't', b'i',
            b'e', 1, 0xee, 0xdd, 0xcc, 3, 0x3f, 0xc0, 0, 0,
        ],
    );
    assert_waypoint_round_trip(
        ClientboundTrackedWaypointPacket::remove_waypoint(id),
        vec![
            1, 1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, b'm', b'i',
            b'n', b'e', b'c', b'r', b'a', b'f', b't', b':', b'd', b'e', b'f', b'a', b'u',
            b'l', b't', 0, 0,
        ],
    );
}

#[test]
fn clientbound_tracked_waypoint_packet_reads_string_ids_and_wraps_operation_ids() {
    let mut payload = Vec::new();
    write_var_i32(&mut payload, 5).unwrap(); // Java Operation.BY_ID wraps 5 to UPDATE.
    write_bool(&mut payload, false).unwrap();
    write_string(&mut payload, "debug-waypoint", 32767).unwrap();
    write_identifier(&mut payload, &id_("minecraft:default")).unwrap();
    write_bool(&mut payload, false).unwrap();
    write_var_i32(&mut payload, 1).unwrap();
    write_var_i32(&mut payload, 1).unwrap();
    write_var_i32(&mut payload, 2).unwrap();
    write_var_i32(&mut payload, 3).unwrap();

    assert_eq!(
        ClientboundTrackedWaypointPacket::read(&mut cursor(payload)).unwrap(),
        ClientboundTrackedWaypointPacket {
            operation: TrackedWaypointOperation::Update,
            waypoint: TrackedWaypoint {
                identifier: TrackedWaypointIdentifier::String("debug-waypoint".to_string()),
                icon: WaypointIcon::null(),
                kind: TrackedWaypointKind::Vec3i { x: 1, y: 2, z: 3 },
            },
        }
    );
}

#[test]
fn clientbound_tracked_waypoint_packet_rejects_malformed_payloads() {
    assert!(ClientboundTrackedWaypointPacket::read(&mut cursor(vec![])).is_err());

    let id = Uuid([1; 16]);
    let mut trailing = Vec::new();
    ClientboundTrackedWaypointPacket::remove_waypoint(id)
        .write(&mut trailing)
        .unwrap();
    trailing.push(0);
    assert!(ClientboundTrackedWaypointPacket::read(&mut cursor(trailing)).is_err());

    let mut invalid_type = Vec::new();
    write_var_i32(&mut invalid_type, 0).unwrap();
    write_bool(&mut invalid_type, true).unwrap();
    write_uuid(&mut invalid_type, id).unwrap();
    write_identifier(&mut invalid_type, &id_("minecraft:default")).unwrap();
    write_bool(&mut invalid_type, false).unwrap();
    write_var_i32(&mut invalid_type, 4).unwrap();
    assert!(ClientboundTrackedWaypointPacket::read(&mut cursor(invalid_type)).is_err());
}

fn assert_waypoint_round_trip(packet: ClientboundTrackedWaypointPacket, expected: Vec<u8>) {
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, expected);
    assert_eq!(
        ClientboundTrackedWaypointPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn id_(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn write_bool<W: std::io::Write>(writer: &mut W, value: bool) -> std::io::Result<()> {
    writer.write_all(&[u8::from(value)])
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(source.contains(sentinel), "missing Java sentinel {sentinel}");
    }
}
