use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::network::codec::{write_identifier, write_optional};
use crate::network::login::{
    LoginSession, ServerboundHelloPacket, ServerboundLoginAcknowledgedPacket,
    CLIENTBOUND_LOGIN_FINISHED_PACKET_ID, SERVERBOUND_HELLO_PACKET_ID,
    SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID,
};
use crate::network::ping::{ClientboundPongResponsePacket, ServerboundPingRequestPacket};
use crate::network::play::{
    ClientboundLoginPacket, CommonPlayerSpawnInfo, GameMode, CLIENTBOUND_LOGIN_PACKET_ID,
    CLIENTBOUND_PLAYER_POSITION_PACKET_ID, CLIENTBOUND_SET_HELD_SLOT_PACKET_ID,
};
use crate::network::varint::{read_var_i32, write_var_i32};
use crate::registry::Identifier;
use crate::server_properties::ServerProperties;

const VERSION_NAME: &str = "26.1.2";
const PROTOCOL_VERSION: i32 = 775;
const MAX_PACKET_SIZE: usize = 2 * 1024 * 1024;
const CLIENTBOUND_CONFIGURATION_FINISH_PACKET_ID: i32 = 3;
const CLIENTBOUND_CONFIGURATION_UPDATE_ENABLED_FEATURES_PACKET_ID: i32 = 12;
const SERVERBOUND_CONFIGURATION_FINISH_PACKET_ID: i32 = 3;

pub fn run_status_server(
    bind_ip: &str,
    port: u16,
    properties: &ServerProperties,
) -> Result<(), String> {
    let address = format!("{bind_ip}:{port}");
    let listener = TcpListener::bind(&address)
        .map_err(|err| format!("Failed to bind status listener on {address}: {err}"))?;
    let favicon = load_favicon(Path::new("server-icon.png"))
        .map_err(|err| format!("Failed to load server-icon.png: {err}"))?;
    println!("Status listener bound to {address}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let properties = properties.clone();
                let favicon = favicon.clone();
                thread::spawn(move || {
                    if let Err(err) =
                        handle_status_connection(stream, &properties, favicon.as_deref())
                    {
                        eprintln!("status connection error: {err}");
                    }
                });
            }
            Err(err) => eprintln!("status accept error: {err}"),
        }
    }

    Ok(())
}

fn handle_status_connection(
    mut stream: TcpStream,
    properties: &ServerProperties,
    favicon: Option<&str>,
) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let mut first = [0u8; 1];
    if stream.peek(&mut first)? == 1 && first[0] == 0xFE {
        return handle_legacy_status_tcp_connection(&mut stream, properties);
    }

    let handshake = read_packet(&mut stream)?;
    let mut input = Cursor::new(handshake);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected handshake",
        ));
    }

    let _protocol = read_var_i32(&mut input)?;
    let _server_address = read_string(&mut input, 255)?;
    let mut port_bytes = [0u8; 2];
    input.read_exact(&mut port_bytes)?;
    let _server_port = u16::from_be_bytes(port_bytes);
    let next_state = read_var_i32(&mut input)?;
    if next_state == 2 {
        return handle_login_connection(&mut stream, properties);
    }
    if next_state != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported handshake target state",
        ));
    }

    loop {
        let packet = read_packet(&mut stream)?;
        let mut input = Cursor::new(packet);
        match read_var_i32(&mut input)? {
            0 => {
                let json = status_json(properties, favicon);
                write_status_response_packet(&mut stream, &json)?;
            }
            1 => {
                let request = ServerboundPingRequestPacket::read(&mut input)?;
                write_status_pong_packet(&mut stream, request)?;
                return Ok(());
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unknown status packet",
                ))
            }
        }
    }
}

fn handle_login_connection(
    stream: &mut TcpStream,
    properties: &ServerProperties,
) -> io::Result<()> {
    let packet = read_packet(stream)?;
    let mut input = Cursor::new(packet);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != SERVERBOUND_HELLO_PACKET_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected login hello",
        ));
    }

    let mut login = LoginSession::default();
    let finished = login.accept_offline_hello(ServerboundHelloPacket::read(&mut input)?);
    write_framed_packet(stream, CLIENTBOUND_LOGIN_FINISHED_PACKET_ID, |payload| {
        finished.write(payload)
    })?;

    let packet = read_packet(stream)?;
    let mut input = Cursor::new(packet);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected login acknowledgement",
        ));
    }
    login.acknowledge(ServerboundLoginAcknowledgedPacket::read(&mut input)?);

    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_UPDATE_ENABLED_FEATURES_PACKET_ID,
        |payload| {
            write_var_i32(payload, 1)?;
            write_identifier(payload, &Identifier::parse("minecraft:vanilla").unwrap())
        },
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_FINISH_PACKET_ID,
        |_payload| Ok(()),
    )?;

    let packet = read_packet(stream)?;
    let mut input = Cursor::new(packet);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != SERVERBOUND_CONFIGURATION_FINISH_PACKET_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected configuration finish",
        ));
    }

    write_minimal_play_join(stream, properties)?;
    loop {
        match read_packet(stream) {
            Ok(_packet) => {}
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::UnexpectedEof
                        | io::ErrorKind::ConnectionReset
                        | io::ErrorKind::TimedOut
                ) =>
            {
                return Ok(())
            }
            Err(err) => return Err(err),
        }
    }
}

fn write_minimal_play_join(
    stream: &mut TcpStream,
    properties: &ServerProperties,
) -> io::Result<()> {
    let login = ClientboundLoginPacket {
        player_id: 1,
        hardcore: properties.hardcore,
        levels: vec![Identifier::parse("minecraft:overworld").unwrap()],
        max_players: properties.max_players as i32,
        chunk_radius: properties.view_distance as i32,
        simulation_distance: properties.simulation_distance as i32,
        reduced_debug_info: false,
        show_death_screen: true,
        do_limited_crafting: false,
        spawn_info: CommonPlayerSpawnInfo::default(),
        enforces_secure_chat: false,
    };
    write_framed_packet(stream, CLIENTBOUND_LOGIN_PACKET_ID, |payload| {
        write_clientbound_login_packet(payload, &login)
    })?;
    write_framed_packet(stream, CLIENTBOUND_SET_HELD_SLOT_PACKET_ID, |payload| {
        write_var_i32(payload, 0)
    })?;
    write_framed_packet(stream, CLIENTBOUND_PLAYER_POSITION_PACKET_ID, |payload| {
        write_var_i32(payload, 0)?;
        write_vec3(payload, 0.5, 80.0, 0.5)?;
        write_vec3(payload, 0.0, 0.0, 0.0)?;
        payload.write_all(&0.0f32.to_be_bytes())?;
        payload.write_all(&0.0f32.to_be_bytes())?;
        write_var_i32(payload, 0)
    })
}

fn write_clientbound_login_packet<W: Write>(
    writer: &mut W,
    packet: &ClientboundLoginPacket,
) -> io::Result<()> {
    writer.write_all(&packet.player_id.to_be_bytes())?;
    write_bool(writer, packet.hardcore)?;
    write_var_i32(writer, packet.levels.len() as i32)?;
    for level in &packet.levels {
        write_identifier(writer, level)?;
    }
    write_var_i32(writer, packet.max_players)?;
    write_var_i32(writer, packet.chunk_radius)?;
    write_var_i32(writer, packet.simulation_distance)?;
    write_bool(writer, packet.reduced_debug_info)?;
    write_bool(writer, packet.show_death_screen)?;
    write_bool(writer, packet.do_limited_crafting)?;
    write_common_spawn_info(writer, &packet.spawn_info)?;
    write_bool(writer, packet.enforces_secure_chat)
}

fn write_common_spawn_info<W: Write>(
    writer: &mut W,
    spawn_info: &CommonPlayerSpawnInfo,
) -> io::Result<()> {
    write_identifier(writer, &spawn_info.dimension_type)?;
    write_identifier(writer, &spawn_info.dimension)?;
    writer.write_all(&spawn_info.seed.to_be_bytes())?;
    writer.write_all(&[spawn_info.game_mode as u8])?;
    writer.write_all(&[match spawn_info.previous_game_mode {
        Some(GameMode::Survival) => 0,
        Some(GameMode::Creative) => 1,
        Some(GameMode::Adventure) => 2,
        Some(GameMode::Spectator) => 3,
        None => 255,
    }])?;
    write_bool(writer, spawn_info.is_debug)?;
    write_bool(writer, spawn_info.is_flat)?;
    write_optional(
        writer,
        spawn_info.last_death_location.as_ref(),
        |writer, (dimension, pos)| {
            write_identifier(writer, dimension)?;
            for coordinate in pos {
                writer.write_all(&coordinate.to_be_bytes())?;
            }
            Ok(())
        },
    )?;
    write_var_i32(writer, spawn_info.portal_cooldown)?;
    write_var_i32(writer, spawn_info.sea_level)
}

fn write_vec3<W: Write>(writer: &mut W, x: f64, y: f64, z: f64) -> io::Result<()> {
    writer.write_all(&x.to_be_bytes())?;
    writer.write_all(&y.to_be_bytes())?;
    writer.write_all(&z.to_be_bytes())
}

fn write_bool<W: Write>(writer: &mut W, value: bool) -> io::Result<()> {
    writer.write_all(&[u8::from(value)])
}

fn write_framed_packet<W, F>(writer: &mut W, packet_id: i32, write_body: F) -> io::Result<()>
where
    W: Write,
    F: FnOnce(&mut Vec<u8>) -> io::Result<()>,
{
    let mut payload = Vec::new();
    write_var_i32(&mut payload, packet_id)?;
    write_body(&mut payload)?;
    write_packet(writer, &payload)
}

fn write_status_response_packet<W: Write>(writer: &mut W, json: &str) -> io::Result<()> {
    let mut payload = Vec::new();
    write_var_i32(&mut payload, 0)?;
    write_string(&mut payload, json)?;
    write_packet(writer, &payload)
}

fn write_status_pong_packet<W: Write>(
    writer: &mut W,
    request: ServerboundPingRequestPacket,
) -> io::Result<()> {
    let response = ClientboundPongResponsePacket::from_request(request);
    let mut payload = Vec::new();
    write_var_i32(&mut payload, 1)?;
    response.write(&mut payload)?;
    write_packet(writer, &payload)
}

fn handle_legacy_status_tcp_connection(
    stream: &mut TcpStream,
    properties: &ServerProperties,
) -> io::Result<()> {
    stream.set_nonblocking(true)?;
    let mut request = Vec::new();
    let mut buffer = [0u8; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => request.extend_from_slice(&buffer[..count]),
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => {
                let _ = stream.set_nonblocking(false);
                return Err(err);
            }
        }
    }
    stream.set_nonblocking(false)?;
    let response = legacy_status_response(&request, properties)?;
    stream.write_all(&response)
}

fn handle_legacy_status_connection<W: Read + Write>(
    stream: &mut W,
    properties: &ServerProperties,
) -> io::Result<()> {
    let mut request = Vec::new();
    stream.read_to_end(&mut request)?;
    let response = legacy_status_response(&request, properties)?;
    stream.write_all(&response)
}

fn legacy_status_response(request: &[u8], properties: &ServerProperties) -> io::Result<Vec<u8>> {
    if request.first() != Some(&0xFE) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected legacy query packet",
        ));
    }

    let body = match &request[1..] {
        [] => legacy_version0_response(properties),
        [0x01] => legacy_version1_response(properties),
        [0x01, tail @ ..] if read_legacy_ping_host_payload(tail).is_some() => {
            legacy_version1_response(properties)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "malformed legacy query packet",
            ))
        }
    };

    Ok(legacy_disconnect_packet(&body))
}

fn read_legacy_ping_host_payload(input: &[u8]) -> Option<()> {
    let mut input = Cursor::new(input);
    let mut packet_id = [0u8; 1];
    input.read_exact(&mut packet_id).ok()?;
    if packet_id[0] != 250 {
        return None;
    }

    let channel = read_legacy_string(&mut input).ok()?;
    if channel != "MC|PingHost" {
        return None;
    }

    let mut size = [0u8; 2];
    input.read_exact(&mut size).ok()?;
    let payload_size = u16::from_be_bytes(size) as u64;
    if input.get_ref().len() as u64 - input.position() != payload_size {
        return None;
    }

    let mut protocol = [0u8; 1];
    input.read_exact(&mut protocol).ok()?;
    if protocol[0] < 73 {
        return None;
    }

    let _host = read_legacy_string(&mut input).ok()?;
    let mut port = [0u8; 4];
    input.read_exact(&mut port).ok()?;
    (u32::from_be_bytes(port) <= u16::MAX as u32).then_some(())
}

fn legacy_version0_response(properties: &ServerProperties) -> String {
    format!("{}§{}§{}", properties.motd, 0, properties.max_players)
}

fn legacy_version1_response(properties: &ServerProperties) -> String {
    format!(
        "§1\0{}\0{}\0{}\0{}\0{}",
        127, VERSION_NAME, properties.motd, 0, properties.max_players
    )
}

fn legacy_disconnect_packet(reason: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(3 + reason.len() * 2);
    out.push(255);
    write_legacy_string(&mut out, reason).expect("legacy string write to vec cannot fail");
    out
}

fn read_legacy_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let mut length = [0u8; 2];
    reader.read_exact(&mut length)?;
    let char_count = u16::from_be_bytes(length) as usize;
    let mut bytes = vec![0u8; char_count * 2];
    reader.read_exact(&mut bytes)?;
    String::from_utf16(
        &bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>(),
    )
    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn write_legacy_string<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    let utf16 = value.encode_utf16().collect::<Vec<_>>();
    let len = u16::try_from(utf16.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "legacy string too long"))?;
    writer.write_all(&len.to_be_bytes())?;
    for code_unit in utf16 {
        writer.write_all(&code_unit.to_be_bytes())?;
    }
    Ok(())
}

fn read_packet<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > MAX_PACKET_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid packet length",
        ));
    }

    let mut payload = vec![0u8; length as usize];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

fn write_packet<W: Write>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    write_var_i32(writer, payload.len() as i32)?;
    writer.write_all(payload)
}

fn read_string<R: Read>(reader: &mut R, max_chars: usize) -> io::Result<String> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > max_chars * 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid string length",
        ));
    }

    let mut bytes = vec![0u8; length as usize];
    reader.read_exact(&mut bytes)?;
    let string =
        String::from_utf8(bytes).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    if string.chars().count() > max_chars {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "string too long",
        ));
    }
    Ok(string)
}

fn write_string<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    write_var_i32(writer, value.len() as i32)?;
    writer.write_all(value.as_bytes())
}

fn load_favicon(path: &Path) -> io::Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }

    let bytes = fs::read(path)?;
    Ok(Some(format!(
        "data:image/png;base64,{}",
        encode_base64(&bytes)
    )))
}

pub fn status_json(properties: &ServerProperties, favicon: Option<&str>) -> String {
    let players = if properties.hide_online_players {
        "\"players\":{\"max\":0,\"online\":0}".to_string()
    } else {
        format!(
            "\"players\":{{\"max\":{},\"online\":0,\"sample\":[]}}",
            properties.max_players
        )
    };

    let favicon = favicon
        .map(|value| format!(",\"favicon\":\"{}\"", escape_json_string(value)))
        .unwrap_or_default();

    format!(
        "{{\"version\":{{\"name\":\"{}\",\"protocol\":{}}},{},\"description\":{{\"text\":\"{}\"}}{},\"enforcesSecureChat\":true}}",
        VERSION_NAME,
        PROTOCOL_VERSION,
        players,
        escape_json_string(&properties.motd),
        favicon
    )
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);

        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            out.push('=');
        }
    }

    out
}

fn escape_json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        encode_base64, escape_json_string, handle_legacy_status_connection,
        legacy_disconnect_packet, legacy_version0_response, legacy_version1_response, read_packet,
        status_json, write_legacy_string, write_status_pong_packet,
    };
    use crate::network::ping::ServerboundPingRequestPacket;
    use crate::network::varint::read_var_i32;
    use crate::server_properties::ServerProperties;
    use std::io::{Cursor, Read, Write};
    use std::path::Path;

    fn test_properties() -> ServerProperties {
        ServerProperties::load_or_default(Path::new("definitely-missing-test-server.properties"))
            .unwrap()
    }

    #[test]
    fn escapes_status_description() {
        assert_eq!(
            escape_json_string("A \"quoted\" server"),
            "A \\\"quoted\\\" server"
        );
    }

    #[test]
    fn includes_26_1_2_protocol_in_status_json() {
        let properties = test_properties();
        let json = status_json(&properties, None);
        assert!(json.contains("\"name\":\"26.1.2\""));
        assert!(json.contains("\"protocol\":775"));
        assert!(json.contains("\"max\":20"));
    }

    #[test]
    fn includes_favicon_when_present() {
        let properties = test_properties();
        let json = status_json(&properties, Some("data:image/png;base64,iVBORw0KGgo="));
        assert!(json.contains("\"favicon\":\"data:image/png;base64,iVBORw0KGgo=\""));
    }

    #[test]
    fn encodes_base64_padding_cases() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
    }

    #[test]
    fn formats_legacy_status_responses_like_vanilla() {
        let properties = test_properties();

        assert_eq!(
            legacy_version0_response(&properties),
            "A Minecraft Server§0§20"
        );
        assert_eq!(
            legacy_version1_response(&properties),
            "§1\0127\026.1.2\0A Minecraft Server\00\020"
        );

        let packet = legacy_disconnect_packet("hello");
        assert_eq!(packet[0], 255);
        assert_eq!(u16::from_be_bytes([packet[1], packet[2]]), 5);
    }

    #[test]
    fn handles_legacy_1_6_ping_host_payload() {
        let properties = test_properties();
        let mut request = vec![0xFE, 0x01, 0xFA];
        write_legacy_string(&mut request, "MC|PingHost").unwrap();
        let mut payload = vec![127];
        write_legacy_string(&mut payload, "localhost").unwrap();
        payload.extend_from_slice(&25565u32.to_be_bytes());
        request.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        request.extend_from_slice(&payload);

        let mut stream = CursorStream::new(request);
        handle_legacy_status_connection(&mut stream, &properties).unwrap();

        assert_eq!(stream.written[0], 255);
        assert_eq!(
            u16::from_be_bytes([stream.written[1], stream.written[2]]),
            37
        );
        assert_eq!(&stream.written[3..7], &[0, 0xA7, 0, b'1']);
    }

    #[test]
    fn status_ping_packet_echoes_payload_for_client_latency_measurement() {
        let request = ServerboundPingRequestPacket {
            time: 0x0102_0304_0506_0708,
        };
        let mut output = Vec::new();
        write_status_pong_packet(&mut output, request).unwrap();

        let packet = read_packet(&mut Cursor::new(output)).unwrap();
        let mut input = Cursor::new(packet);
        assert_eq!(read_var_i32(&mut input).unwrap(), 1);
        let mut echoed_time = [0u8; 8];
        input.read_exact(&mut echoed_time).unwrap();
        assert_eq!(i64::from_be_bytes(echoed_time), request.time);
    }

    #[derive(Debug)]
    struct CursorStream {
        read: Cursor<Vec<u8>>,
        written: Vec<u8>,
    }

    impl CursorStream {
        fn new(read: Vec<u8>) -> Self {
            Self {
                read: Cursor::new(read),
                written: Vec::new(),
            }
        }
    }

    impl Read for CursorStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.read.read(buf)
        }
    }

    impl Write for CursorStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.written.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
