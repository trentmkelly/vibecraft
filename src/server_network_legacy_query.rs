#![allow(dead_code)]

use crate::server_info::ServerInfo;

pub const CUSTOM_PAYLOAD_PACKET_ID: u8 = 250;
pub const CUSTOM_PAYLOAD_PACKET_PING_CHANNEL: &str = "MC|PingHost";
pub const GET_INFO_PACKET_ID: u8 = 254;
pub const GET_INFO_PACKET_VERSION_1: u8 = 1;
pub const DISCONNECT_PACKET_ID: u8 = 255;
pub const FAKE_PROTOCOL_VERSION: i32 = 127;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyQueryReadAction {
    RespondAndClose(Vec<u8>),
    ConnectNormally,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyPingKind {
    Version0,
    Version1,
    Version16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyPingHostPayload {
    pub protocol_version: u8,
    pub host: String,
    pub port: i32,
}

pub fn write_legacy_string(to_send: &mut Vec<u8>, value: &str) {
    let utf16 = value.encode_utf16().collect::<Vec<_>>();
    let len = (utf16.len() as u16).to_be_bytes();
    to_send.extend_from_slice(&len);
    for code_unit in utf16 {
        to_send.extend_from_slice(&code_unit.to_be_bytes());
    }
}

pub fn read_legacy_string(input: &mut &[u8]) -> Option<String> {
    if input.len() < 2 {
        return None;
    }
    let char_count = u16::from_be_bytes([input[0], input[1]]) as usize;
    *input = &input[2..];
    let byte_count = char_count.checked_mul(2)?;
    if input.len() < byte_count {
        return None;
    }

    let mut units = Vec::with_capacity(char_count);
    for chunk in input[..byte_count].chunks_exact(2) {
        units.push(u16::from_be_bytes([chunk[0], chunk[1]]));
    }
    *input = &input[byte_count..];
    String::from_utf16(&units).ok()
}

pub fn create_version0_response(server: &impl ServerInfo) -> String {
    format!(
        "{}§{}§{}",
        server.get_motd(),
        server.get_player_count(),
        server.get_max_players()
    )
}

pub fn create_version1_response(server: &impl ServerInfo) -> String {
    format!(
        "§1\0{}\0{}\0{}\0{}\0{}",
        FAKE_PROTOCOL_VERSION,
        server.get_server_version(),
        server.get_motd(),
        server.get_player_count(),
        server.get_max_players()
    )
}

pub fn create_legacy_disconnect_packet(reason: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(DISCONNECT_PACKET_ID);
    write_legacy_string(&mut out, reason);
    out
}

pub fn read_custom_payload_packet(input: &mut &[u8]) -> Option<LegacyPingHostPayload> {
    let packet_id = read_u8(input)?;
    if packet_id != CUSTOM_PAYLOAD_PACKET_ID {
        return None;
    }

    let channel_id = read_legacy_string(input)?;
    if channel_id != CUSTOM_PAYLOAD_PACKET_PING_CHANNEL {
        return None;
    }

    let payload_size = read_u16(input)? as usize;
    if input.len() != payload_size {
        return None;
    }

    let protocol_version = read_u8(input)?;
    if protocol_version < 73 {
        return None;
    }

    let host = read_legacy_string(input)?;
    let port = read_i32(input)?;
    (port <= 65_535).then_some(LegacyPingHostPayload {
        protocol_version,
        host,
        port,
    })
}

pub fn classify_legacy_query(request: &[u8]) -> Option<LegacyPingKind> {
    let mut input = request;
    let packet_id = read_u8(&mut input)?;
    if packet_id != GET_INFO_PACKET_ID {
        return None;
    }

    if input.is_empty() {
        return Some(LegacyPingKind::Version0);
    }

    let version = read_u8(&mut input)?;
    if version != GET_INFO_PACKET_VERSION_1 {
        return None;
    }

    if input.is_empty() {
        Some(LegacyPingKind::Version1)
    } else {
        read_custom_payload_packet(&mut input).map(|_| LegacyPingKind::Version16)
    }
}

pub fn handle_legacy_query_read(request: &[u8], server: &impl ServerInfo) -> LegacyQueryReadAction {
    match classify_legacy_query(request) {
        Some(LegacyPingKind::Version0) => {
            LegacyQueryReadAction::RespondAndClose(create_legacy_disconnect_packet(
                &create_version0_response(server),
            ))
        }
        Some(LegacyPingKind::Version1 | LegacyPingKind::Version16) => {
            LegacyQueryReadAction::RespondAndClose(create_legacy_disconnect_packet(
                &create_version1_response(server),
            ))
        }
        None => LegacyQueryReadAction::ConnectNormally,
    }
}

fn read_u8(input: &mut &[u8]) -> Option<u8> {
    let value = *input.first()?;
    *input = &input[1..];
    Some(value)
}

fn read_u16(input: &mut &[u8]) -> Option<u16> {
    if input.len() < 2 {
        return None;
    }
    let value = u16::from_be_bytes([input[0], input[1]]);
    *input = &input[2..];
    Some(value)
}

fn read_i32(input: &mut &[u8]) -> Option<i32> {
    if input.len() < 4 {
        return None;
    }
    let value = i32::from_be_bytes([input[0], input[1], input[2], input[3]]);
    *input = &input[4..];
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_info::StaticServerInfo;

    fn server() -> StaticServerInfo {
        StaticServerInfo::new("A Minecraft Server", "26.1.2", 7, 20)
    }

    fn ping_host_request(protocol_version: u8, channel: &str, host: &str, port: i32) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.push(protocol_version);
        write_legacy_string(&mut payload, host);
        payload.extend_from_slice(&port.to_be_bytes());

        let mut request = vec![GET_INFO_PACKET_ID, GET_INFO_PACKET_VERSION_1, CUSTOM_PAYLOAD_PACKET_ID];
        write_legacy_string(&mut request, channel);
        request.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        request.extend_from_slice(&payload);
        request
    }

    #[test]
    fn legacy_protocol_constants_match_java() {
        assert_eq!(CUSTOM_PAYLOAD_PACKET_ID, 250);
        assert_eq!(CUSTOM_PAYLOAD_PACKET_PING_CHANNEL, "MC|PingHost");
        assert_eq!(GET_INFO_PACKET_ID, 254);
        assert_eq!(GET_INFO_PACKET_VERSION_1, 1);
        assert_eq!(DISCONNECT_PACKET_ID, 255);
        assert_eq!(FAKE_PROTOCOL_VERSION, 127);
    }

    #[test]
    fn legacy_strings_are_utf16be_with_java_char_count_prefix() {
        let mut encoded = Vec::new();
        write_legacy_string(&mut encoded, "A§😀");

        assert_eq!(u16::from_be_bytes([encoded[0], encoded[1]]), 4);
        assert_eq!(
            encoded,
            vec![0, 4, 0, b'A', 0, 0xA7, 0xD8, 0x3D, 0xDE, 0]
        );

        let mut input = encoded.as_slice();
        assert_eq!(read_legacy_string(&mut input), Some("A§😀".to_string()));
        assert!(input.is_empty());
    }

    #[test]
    fn legacy_response_bodies_and_disconnect_packet_match_java() {
        let server = server();

        assert_eq!(
            create_version0_response(&server),
            "A Minecraft Server§7§20"
        );
        assert_eq!(
            create_version1_response(&server),
            "§1\0".to_string()
                + "127\0"
                + "26.1.2\0"
                + "A Minecraft Server\0"
                + "7\0"
                + "20"
        );

        let packet = create_legacy_disconnect_packet("bye");
        assert_eq!(packet[0], DISCONNECT_PACKET_ID);
        assert_eq!(u16::from_be_bytes([packet[1], packet[2]]), 3);
        let mut body = &packet[1..];
        assert_eq!(read_legacy_string(&mut body), Some("bye".to_string()));
    }

    #[test]
    fn legacy_query_classification_matches_java_channel_read_branches() {
        assert_eq!(
            classify_legacy_query(&[GET_INFO_PACKET_ID]),
            Some(LegacyPingKind::Version0)
        );
        assert_eq!(
            classify_legacy_query(&[GET_INFO_PACKET_ID, GET_INFO_PACKET_VERSION_1]),
            Some(LegacyPingKind::Version1)
        );

        let request = ping_host_request(127, CUSTOM_PAYLOAD_PACKET_PING_CHANNEL, "localhost", 25_565);
        assert_eq!(classify_legacy_query(&request), Some(LegacyPingKind::Version16));

        assert_eq!(classify_legacy_query(&[0]), None);
        assert_eq!(classify_legacy_query(&[GET_INFO_PACKET_ID, 2]), None);
        assert_eq!(
            classify_legacy_query(&ping_host_request(72, CUSTOM_PAYLOAD_PACKET_PING_CHANNEL, "localhost", 25_565)),
            None
        );
        assert_eq!(
            classify_legacy_query(&ping_host_request(127, "minecraft:bad", "localhost", 25_565)),
            None
        );
        assert_eq!(
            classify_legacy_query(&ping_host_request(127, CUSTOM_PAYLOAD_PACKET_PING_CHANNEL, "localhost", 65_536)),
            None
        );
    }

    #[test]
    fn legacy_query_read_responds_or_connects_normally_like_java() {
        let server = server();

        match handle_legacy_query_read(&[GET_INFO_PACKET_ID], &server) {
            LegacyQueryReadAction::RespondAndClose(packet) => {
                assert_eq!(packet, create_legacy_disconnect_packet(&create_version0_response(&server)));
            }
            LegacyQueryReadAction::ConnectNormally => panic!("version 0 ping should respond"),
        }

        match handle_legacy_query_read(&[GET_INFO_PACKET_ID, GET_INFO_PACKET_VERSION_1], &server) {
            LegacyQueryReadAction::RespondAndClose(packet) => {
                assert_eq!(packet, create_legacy_disconnect_packet(&create_version1_response(&server)));
            }
            LegacyQueryReadAction::ConnectNormally => panic!("version 1 ping should respond"),
        }

        assert_eq!(
            handle_legacy_query_read(&[0], &server),
            LegacyQueryReadAction::ConnectNormally
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn legacy_query_sources_match_java_26_1_2() {
        const LEGACY_PROTOCOL_UTILS: &str =
            vibecraft_java_source!("/net/minecraft/server/network/LegacyProtocolUtils.java");
        const LEGACY_QUERY_HANDLER: &str =
            vibecraft_java_source!("/net/minecraft/server/network/LegacyQueryHandler.java");

        for sentinel in [
            "public class LegacyProtocolUtils",
            "public static final int CUSTOM_PAYLOAD_PACKET_ID = 250;",
            "public static final String CUSTOM_PAYLOAD_PACKET_PING_CHANNEL = \"MC|PingHost\";",
            "public static final int GET_INFO_PACKET_ID = 254;",
            "public static final int GET_INFO_PACKET_VERSION_1 = 1;",
            "public static final int DISCONNECT_PACKET_ID = 255;",
            "public static final int FAKE_PROTOCOL_VERSION = 127;",
            "toSend.writeShort(str.length());",
            "toSend.writeCharSequence(str, StandardCharsets.UTF_16BE);",
            "int charCount = msg.readShort();",
            "int byteCount = charCount * 2;",
            "msg.toString(msg.readerIndex(), byteCount, StandardCharsets.UTF_16BE);",
            "msg.skipBytes(byteCount);",
        ] {
            assert!(
                LEGACY_PROTOCOL_UTILS.contains(sentinel),
                "LegacyProtocolUtils.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public class LegacyQueryHandler extends ChannelInboundHandlerAdapter",
            "private final ServerInfo server;",
            "if (in.readUnsignedByte() != 254)",
            "if (length == 0)",
            "String body = createVersion0Response(this.server);",
            "if (in.readUnsignedByte() != 1)",
            "if (!readCustomPayloadPacket(in))",
            "String body = createVersion1Response(this.server);",
            "in.release();",
            "connectNormally = false;",
            "in.resetReaderIndex();",
            "ctx.channel().pipeline().remove(this);",
            "ctx.fireChannelRead(msg);",
            "short packetId = in.readUnsignedByte();",
            "if (packetId != 250)",
            "String channelId = LegacyProtocolUtils.readLegacyString(in);",
            "if (!\"MC|PingHost\".equals(channelId))",
            "int payloadSize = in.readUnsignedShort();",
            "if (in.readableBytes() != payloadSize)",
            "short protocolVersion = in.readUnsignedByte();",
            "if (protocolVersion < 73)",
            "int port = in.readInt();",
            "return port <= 65535;",
            "\"%s§%d§%d\"",
            "\"§1\\u0000%d\\u0000%s\\u0000%s\\u0000%d\\u0000%d\"",
            "127,",
            "out.writeByte(255);",
            "LegacyProtocolUtils.writeLegacyString(out, reason);",
        ] {
            assert!(
                LEGACY_QUERY_HANDLER.contains(sentinel),
                "LegacyQueryHandler.java is missing sentinel: {sentinel}"
            );
        }
    }
}
