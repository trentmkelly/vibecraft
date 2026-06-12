use super::super::{
    read_packet, read_string, write_status_response_packet, MAX_STATUS_RESPONSE_JSON_CHARS,
};
use crate::network::varint::read_var_i32;
use std::io::{self, Cursor};

#[test]
pub fn status_protocol_package_matches_java_packet_contracts() {
    const CLIENT_LISTENER_JAVA: &str = include_str!(
        "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/status/ClientStatusPacketListener.java"
    );
    const STATUS_RESPONSE_JAVA: &str = include_str!(
        "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/status/ClientboundStatusResponsePacket.java"
    );
    const SERVER_STATUS_JAVA: &str = include_str!(
        "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/status/ServerStatus.java"
    );
    const SERVER_LISTENER_JAVA: &str = include_str!(
        "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/status/ServerStatusPacketListener.java"
    );
    const STATUS_REQUEST_JAVA: &str = include_str!(
        "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/status/ServerboundStatusRequestPacket.java"
    );
    const PACKET_TYPES_JAVA: &str = include_str!(
        "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/status/StatusPacketTypes.java"
    );
    const STATUS_PROTOCOLS_JAVA: &str = include_str!(
        "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/status/StatusProtocols.java"
    );

    assert!(CLIENT_LISTENER_JAVA.contains("return ConnectionProtocol.STATUS;"));
    assert!(CLIENT_LISTENER_JAVA
        .contains("void handleStatusResponse(ClientboundStatusResponsePacket packet);"));
    assert!(SERVER_LISTENER_JAVA.contains("return ConnectionProtocol.STATUS;"));
    assert!(SERVER_LISTENER_JAVA
        .contains("void handleStatusRequest(ServerboundStatusRequestPacket packet);"));
    assert!(STATUS_RESPONSE_JAVA.contains("lenientJson(32767)"));
    assert!(STATUS_RESPONSE_JAVA.contains("StatusPacketTypes.CLIENTBOUND_STATUS_RESPONSE"));
    assert!(STATUS_REQUEST_JAVA.contains("StreamCodec.unit(INSTANCE)"));
    assert!(STATUS_REQUEST_JAVA.contains("StatusPacketTypes.SERVERBOUND_STATUS_REQUEST"));
    assert!(SERVER_STATUS_JAVA
        .contains("lenientOptionalFieldOf(\"description\", CommonComponents.EMPTY)"));
    assert!(SERVER_STATUS_JAVA.contains("lenientOptionalFieldOf(\"players\")"));
    assert!(SERVER_STATUS_JAVA.contains("lenientOptionalFieldOf(\"version\")"));
    assert!(SERVER_STATUS_JAVA.contains("lenientOptionalFieldOf(\"favicon\")"));
    assert!(SERVER_STATUS_JAVA.contains("lenientOptionalFieldOf(\"enforcesSecureChat\", false)"));
    assert!(PACKET_TYPES_JAVA
        .contains("CLIENTBOUND_STATUS_RESPONSE = createClientbound(\"status_response\")"));
    assert!(PACKET_TYPES_JAVA
        .contains("SERVERBOUND_STATUS_REQUEST = createServerbound(\"status_request\")"));
    assert!(STATUS_PROTOCOLS_JAVA.contains(
        ".addPacket(StatusPacketTypes.SERVERBOUND_STATUS_REQUEST, ServerboundStatusRequestPacket.STREAM_CODEC)"
    ));
    assert!(STATUS_PROTOCOLS_JAVA.contains(
        ".addPacket(PingPacketTypes.SERVERBOUND_PING_REQUEST, ServerboundPingRequestPacket.STREAM_CODEC)"
    ));
    assert!(STATUS_PROTOCOLS_JAVA.contains(
        ".addPacket(StatusPacketTypes.CLIENTBOUND_STATUS_RESPONSE, ClientboundStatusResponsePacket.STREAM_CODEC)"
    ));
    assert!(STATUS_PROTOCOLS_JAVA.contains(
        ".addPacket(PingPacketTypes.CLIENTBOUND_PONG_RESPONSE, ClientboundPongResponsePacket.STREAM_CODEC)"
    ));
}

#[test]
pub fn status_response_packet_writes_java_id_and_json_bound() {
    let json = "{\"description\":{\"text\":\"ok\"}}";
    let mut framed = Vec::new();
    write_status_response_packet(&mut framed, json).unwrap();

    let payload = read_packet(&mut Cursor::new(framed)).unwrap();
    let mut payload = Cursor::new(payload);
    assert_eq!(read_var_i32(&mut payload).unwrap(), 0);
    assert_eq!(
        read_string(&mut payload, MAX_STATUS_RESPONSE_JSON_CHARS).unwrap(),
        json
    );
    assert_eq!(payload.position(), payload.get_ref().len() as u64);

    let oversized = "x".repeat(MAX_STATUS_RESPONSE_JSON_CHARS + 1);
    assert_eq!(
        write_status_response_packet(&mut Vec::new(), &oversized)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
}
