#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const SERVERDATA_RESPONSE_VALUE: i32 = 0;
pub const SERVERDATA_EXECCOMMAND: i32 = 2;
pub const SERVERDATA_AUTH_RESPONSE: i32 = 2;
pub const SERVERDATA_AUTH: i32 = 3;
pub const SERVERDATA_AUTH_FAILURE: i32 = -1;
pub const RCON_MAX_PACKET_SIZE: usize = 1460;
pub const RCON_HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
const MAX_RESPONSE_CHARS: usize = 4096;

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkDataOutputStreamModel {
    bytes: Vec<u8>,
}

impl NetworkDataOutputStreamModel {
    pub fn new(size: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(size),
        }
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.bytes.extend_from_slice(data);
    }

    pub fn write_string(&mut self, data: &str) {
        self.bytes.extend_from_slice(data.as_bytes());
        self.bytes.push(0);
    }

    pub fn write(&mut self, data: i32) {
        self.bytes.push(data as u8);
    }

    pub fn write_short(&mut self, data: i16) {
        self.bytes.extend_from_slice(&data.to_le_bytes());
    }

    pub fn write_int(&mut self, data: i32) {
        self.bytes.extend_from_slice(&data.to_le_bytes());
    }

    pub fn write_float(&mut self, data: f32) {
        self.bytes.extend_from_slice(&data.to_bits().to_le_bytes());
    }

    pub fn to_byte_array(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    pub fn reset(&mut self) {
        self.bytes.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RconPacket {
    pub request_id: i32,
    pub packet_type: i32,
    pub payload: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RconSession {
    password: String,
    authenticated: bool,
    broadcast_to_ops: bool,
    broadcasts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RconConsoleSourceModel {
    buffer: String,
    should_rcon_broadcast: bool,
    respawn_pos: (i32, i32, i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RconCommandSourceStackModel {
    pub source_name: String,
    pub display_name: String,
    pub position: (f64, f64, f64),
    pub rotation: (f32, f32),
    pub permission: &'static str,
    pub server_attached: bool,
    pub entity_attached: bool,
}

impl RconConsoleSourceModel {
    pub fn new(should_rcon_broadcast: bool, respawn_pos: (i32, i32, i32)) -> Self {
        Self {
            buffer: String::new(),
            should_rcon_broadcast,
            respawn_pos,
        }
    }

    pub fn prepare_for_command(&mut self) {
        self.buffer.clear();
    }

    pub fn get_command_response(&self) -> &str {
        &self.buffer
    }

    pub fn create_command_source_stack(&self) -> RconCommandSourceStackModel {
        RconCommandSourceStackModel {
            source_name: "Rcon".to_string(),
            display_name: "Rcon".to_string(),
            position: (
                f64::from(self.respawn_pos.0),
                f64::from(self.respawn_pos.1),
                f64::from(self.respawn_pos.2),
            ),
            rotation: (0.0, 0.0),
            permission: "LevelBasedPermissionSet.OWNER",
            server_attached: true,
            entity_attached: false,
        }
    }

    pub fn send_system_message(&mut self, message: &str) {
        self.buffer.push_str(message);
    }

    pub fn accepts_success(&self) -> bool {
        true
    }

    pub fn accepts_failure(&self) -> bool {
        true
    }

    pub fn should_inform_admins(&self) -> bool {
        self.should_rcon_broadcast
    }
}

impl RconSession {
    pub fn new(password: impl Into<String>, broadcast_to_ops: bool) -> Self {
        Self {
            password: password.into(),
            authenticated: false,
            broadcast_to_ops,
            broadcasts: Vec::new(),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    pub fn broadcasts(&self) -> &[String] {
        &self.broadcasts
    }

    pub fn handle_packet<F>(&mut self, packet: RconPacket, run_command: F) -> Vec<RconPacket>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        match packet.packet_type {
            SERVERDATA_AUTH => {
                if !packet.payload.is_empty() && packet.payload == self.password {
                    self.authenticated = true;
                    vec![RconPacket {
                        request_id: packet.request_id,
                        packet_type: SERVERDATA_AUTH_RESPONSE,
                        payload: String::new(),
                    }]
                } else {
                    self.authenticated = false;
                    vec![auth_failure_packet()]
                }
            }
            SERVERDATA_EXECCOMMAND => {
                if !self.authenticated {
                    return vec![auth_failure_packet()];
                }
                if self.broadcast_to_ops {
                    self.broadcasts.push(packet.payload.clone());
                }
                let response = run_command(&packet.payload)
                    .unwrap_or_else(|err| format!("Error executing: {} ({err})", packet.payload));
                fragment_response(packet.request_id, &response)
            }
            other => fragment_response(packet.request_id, &format!("Unknown request {:x}", other)),
        }
    }
}

pub fn read_packet<R: Read>(reader: &mut R) -> std::io::Result<Option<RconPacket>> {
    let mut length_bytes = [0u8; 4];
    match reader.read_exact(&mut length_bytes) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(err) => return Err(err),
    }
    let length = i32::from_le_bytes(length_bytes);
    if !(10..=1460).contains(&length) {
        return Ok(None);
    }

    let mut body = vec![0u8; length as usize];
    reader.read_exact(&mut body)?;
    let request_id = i32::from_le_bytes([body[0], body[1], body[2], body[3]]);
    let packet_type = i32::from_le_bytes([body[4], body[5], body[6], body[7]]);
    let payload_end = body[8..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|index| index + 8)
        .unwrap_or(body.len());
    let payload = String::from_utf8_lossy(&body[8..payload_end]).into_owned();
    Ok(Some(RconPacket {
        request_id,
        packet_type,
        payload,
    }))
}

pub fn rcon_string_from_byte_array(bytes: &[u8], offset: usize, length: usize) -> String {
    if bytes.is_empty() || length == 0 {
        return String::new();
    }

    let max = length.saturating_sub(1).min(bytes.len().saturating_sub(1));
    let start = offset.min(max);
    let mut end = start;
    while end < max && bytes[end] != 0 {
        end += 1;
    }

    String::from_utf8_lossy(&bytes[start..end]).into_owned()
}

pub fn rcon_int_from_byte_array(bytes: &[u8], offset: usize) -> i32 {
    rcon_int_from_byte_array_with_length(bytes, offset, bytes.len())
}

pub fn rcon_int_from_byte_array_with_length(bytes: &[u8], offset: usize, length: usize) -> i32 {
    if length.saturating_sub(offset) < 4 || bytes.len().saturating_sub(offset) < 4 {
        return 0;
    }

    i32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

pub fn rcon_int_from_network_byte_array(bytes: &[u8], offset: usize, length: usize) -> i32 {
    if length.saturating_sub(offset) < 4 || bytes.len().saturating_sub(offset) < 4 {
        return 0;
    }

    i32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

pub fn rcon_byte_to_hex_string(byte: u8) -> String {
    format!(
        "{}{}",
        RCON_HEX_CHARS[((byte & 0xF0) >> 4) as usize],
        RCON_HEX_CHARS[(byte & 0x0F) as usize]
    )
}

pub fn write_packet<W: Write>(writer: &mut W, packet: &RconPacket) -> std::io::Result<()> {
    let payload = packet.payload.as_bytes();
    let length = payload.len() + 10;
    writer.write_all(&(length as i32).to_le_bytes())?;
    writer.write_all(&packet.request_id.to_le_bytes())?;
    writer.write_all(&packet.packet_type.to_le_bytes())?;
    writer.write_all(payload)?;
    writer.write_all(&[0, 0])
}

pub fn spawn_rcon_server<F>(
    bind_ip: &str,
    port: u16,
    password: String,
    broadcast_to_ops: bool,
    run_command: F,
) -> Result<JoinHandle<()>, String>
where
    F: Fn(&str) -> String + Send + Sync + 'static,
{
    if password.is_empty() {
        return Err("No rcon password set in server.properties, rcon disabled".to_string());
    }
    let address = format!("{bind_ip}:{port}");
    let listener = TcpListener::bind(&address)
        .map_err(|err| format!("Failed to bind RCON listener on {address}: {err}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("Failed to configure RCON listener: {err}"))?;
    let run_command = Arc::new(run_command);
    thread::Builder::new()
        .name("RCON Listener".to_string())
        .spawn(move || loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    let command_runner = Arc::clone(&run_command);
                    let password = password.clone();
                    let _ = thread::Builder::new()
                        .name("RCON Client".to_string())
                        .spawn(move || {
                            handle_client(stream, password, broadcast_to_ops, command_runner)
                        });
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        })
        .map_err(|err| format!("Failed to start RCON listener thread: {err}"))
}

fn handle_client<F>(
    mut stream: TcpStream,
    password: String,
    broadcast_to_ops: bool,
    run_command: Arc<F>,
) where
    F: Fn(&str) -> String + Send + Sync + 'static,
{
    let mut session = RconSession::new(password, broadcast_to_ops);
    while let Ok(Some(packet)) = read_packet(&mut stream) {
        for response in session.handle_packet(packet, |command| Ok(run_command(command))) {
            if write_packet(&mut stream, &response).is_err() {
                return;
            }
        }
    }
}

fn auth_failure_packet() -> RconPacket {
    RconPacket {
        request_id: SERVERDATA_AUTH_FAILURE,
        packet_type: SERVERDATA_AUTH_RESPONSE,
        payload: String::new(),
    }
}

fn fragment_response(request_id: i32, response: &str) -> Vec<RconPacket> {
    if response.is_empty() {
        return vec![RconPacket {
            request_id,
            packet_type: SERVERDATA_RESPONSE_VALUE,
            payload: String::new(),
        }];
    }
    let mut packets = Vec::new();
    let mut remaining = response;
    while !remaining.is_empty() {
        let end = remaining
            .char_indices()
            .nth(MAX_RESPONSE_CHARS)
            .map(|(index, _)| index)
            .unwrap_or(remaining.len());
        packets.push(RconPacket {
            request_id,
            packet_type: SERVERDATA_RESPONSE_VALUE,
            payload: remaining[..end].to_string(),
        });
        remaining = &remaining[end..];
    }
    packets
}

#[cfg(test)]
mod tests {
    use super::{
        rcon_byte_to_hex_string, rcon_int_from_byte_array, rcon_int_from_byte_array_with_length,
        rcon_int_from_network_byte_array, rcon_string_from_byte_array, read_packet, write_packet,
        NetworkDataOutputStreamModel, RconConsoleSourceModel, RconPacket, RconSession,
        RCON_MAX_PACKET_SIZE, SERVERDATA_AUTH, SERVERDATA_AUTH_FAILURE, SERVERDATA_AUTH_RESPONSE,
        SERVERDATA_EXECCOMMAND, SERVERDATA_RESPONSE_VALUE,
    };
    use std::io::Cursor;

    const NETWORK_DATA_OUTPUT_STREAM_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/rcon/NetworkDataOutputStream.java");
    const PKT_UTILS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/rcon/PktUtils.java");
    const RCON_CONSOLE_SOURCE_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/rcon/RconConsoleSource.java");

    #[test]
    fn network_data_output_stream_writes_little_endian_values_like_java() {
        assert_contains_all(
            NETWORK_DATA_OUTPUT_STREAM_JAVA,
            &[
                "new ByteArrayOutputStream(size)",
                "this.dataOutputStream.write(data, 0, data.length);",
                "data.getBytes(StandardCharsets.UTF_8)",
                "this.dataOutputStream.write(0);",
                "this.dataOutputStream.writeShort(Short.reverseBytes(data));",
                "this.dataOutputStream.writeInt(Integer.reverseBytes(data));",
                "Integer.reverseBytes(Float.floatToIntBits(data))",
                "this.outputStream.reset();",
            ],
        );

        let mut output = NetworkDataOutputStreamModel::new(32);
        output.write_bytes(&[0xAA, 0xBB]);
        output.write_string("hé");
        output.write(0x123);
        output.write_short(0x1234);
        output.write_int(0x1234_5678);
        output.write_float(1.0);

        assert_eq!(
            output.to_byte_array(),
            vec![
                0xAA, 0xBB, 0x68, 0xC3, 0xA9, 0, 0x23, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12,
                0, 0, 0x80, 0x3F,
            ]
        );

        output.reset();
        assert!(output.to_byte_array().is_empty());
    }

    #[test]
    fn pkt_utils_decode_strings_ints_and_hex_like_java() {
        assert_contains_all(
            PKT_UTILS_JAVA,
            &[
                "public static final int MAX_PACKET_SIZE = 1460;",
                "HEX_CHAR = new char[]{'0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'}",
                "new String(b, offset, i - offset, StandardCharsets.UTF_8)",
                "b[offset + 3] << 24 | (b[offset + 2] & 0xFF) << 16 | (b[offset + 1] & 0xFF) << 8 | b[offset] & 0xFF",
                "b[offset] << 24 | (b[offset + 1] & 0xFF) << 16 | (b[offset + 2] & 0xFF) << 8 | b[offset + 3] & 0xFF",
                "HEX_CHAR[(b & 240) >>> 4]",
            ],
        );

        assert_eq!(RCON_MAX_PACKET_SIZE, 1460);
        assert_eq!(
            rcon_string_from_byte_array(b"hello\0ignored", 0, 13),
            "hello"
        );
        assert_eq!(rcon_string_from_byte_array("hé\0x".as_bytes(), 0, 5), "hé");
        assert_eq!(rcon_string_from_byte_array(b"abc", 10, 3), "");
        assert_eq!(rcon_int_from_byte_array(&[0x78, 0x56, 0x34, 0x12], 0), 0x1234_5678);
        assert_eq!(
            rcon_int_from_byte_array_with_length(&[1, 2, 3, 4, 5], 1, 5),
            0x0504_0302
        );
        assert_eq!(rcon_int_from_byte_array_with_length(&[1, 2, 3], 0, 3), 0);
        assert_eq!(
            rcon_int_from_network_byte_array(&[0x12, 0x34, 0x56, 0x78], 0, 4),
            0x1234_5678
        );
        assert_eq!(rcon_int_from_network_byte_array(&[1, 2, 3], 0, 3), 0);
        assert_eq!(rcon_byte_to_hex_string(0xAF), "af");
        assert_eq!(rcon_byte_to_hex_string(0x05), "05");
    }

    #[test]
    fn rcon_console_source_buffers_messages_and_resets_like_java() {
        let mut source = RconConsoleSourceModel::new(true, (10, 64, -3));

        source.send_system_message("first");
        source.send_system_message(" second");
        assert_eq!(source.get_command_response(), "first second");

        source.prepare_for_command();
        assert_eq!(source.get_command_response(), "");

        assert!(source.accepts_success());
        assert!(source.accepts_failure());
        assert!(source.should_inform_admins());
        assert!(!RconConsoleSourceModel::new(false, (0, 64, 0)).should_inform_admins());
    }

    #[test]
    fn rcon_console_source_stack_matches_java_owner_rcon_source() {
        assert_contains_all(
            RCON_CONSOLE_SOURCE_JAVA,
            &[
                "private static final String RCON = \"Rcon\";",
                "private static final Component RCON_COMPONENT = Component.literal(\"Rcon\");",
                "this.buffer.setLength(0);",
                "return this.buffer.toString();",
                "Vec3.atLowerCornerOf(level.getRespawnData().pos())",
                "Vec2.ZERO",
                "LevelBasedPermissionSet.OWNER",
                "\"Rcon\", RCON_COMPONENT",
                "this.buffer.append(message.getString());",
                "return this.server.shouldRconBroadcast();",
            ],
        );

        let stack = RconConsoleSourceModel::new(false, (10, 64, -3)).create_command_source_stack();
        assert_eq!(stack.source_name, "Rcon");
        assert_eq!(stack.display_name, "Rcon");
        assert_eq!(stack.position, (10.0, 64.0, -3.0));
        assert_eq!(stack.rotation, (0.0, 0.0));
        assert_eq!(stack.permission, "LevelBasedPermissionSet.OWNER");
        assert!(stack.server_attached);
        assert!(!stack.entity_attached);
    }

    #[test]
    fn rcon_packets_use_little_endian_length_id_type_and_two_nulls() {
        let packet = RconPacket {
            request_id: 12,
            packet_type: SERVERDATA_AUTH,
            payload: "secret".to_string(),
        };
        let mut bytes = Vec::new();
        write_packet(&mut bytes, &packet).unwrap();

        assert_eq!(i32::from_le_bytes(bytes[0..4].try_into().unwrap()), 16);
        assert_eq!(i32::from_le_bytes(bytes[4..8].try_into().unwrap()), 12);
        assert_eq!(i32::from_le_bytes(bytes[8..12].try_into().unwrap()), 3);
        assert_eq!(&bytes[12..], b"secret\0\0");
        assert_eq!(read_packet(&mut Cursor::new(bytes)).unwrap(), Some(packet));
    }

    #[test]
    fn rcon_auth_success_and_failure_match_vanilla_ids() {
        let mut session = RconSession::new("secret", false);

        let failure = session.handle_packet(
            RconPacket {
                request_id: 1,
                packet_type: SERVERDATA_AUTH,
                payload: "wrong".to_string(),
            },
            |_| Ok(String::new()),
        );
        assert_eq!(failure[0].request_id, SERVERDATA_AUTH_FAILURE);
        assert_eq!(failure[0].packet_type, SERVERDATA_AUTH_RESPONSE);
        assert!(!session.is_authenticated());

        let success = session.handle_packet(
            RconPacket {
                request_id: 2,
                packet_type: SERVERDATA_AUTH,
                payload: "secret".to_string(),
            },
            |_| Ok(String::new()),
        );
        assert_eq!(success[0].request_id, 2);
        assert_eq!(success[0].packet_type, SERVERDATA_AUTH_RESPONSE);
        assert!(session.is_authenticated());
    }

    #[test]
    fn rcon_requires_auth_then_executes_and_broadcasts_commands() {
        let mut session = RconSession::new("secret", true);
        let denied = session.handle_packet(
            RconPacket {
                request_id: 1,
                packet_type: SERVERDATA_EXECCOMMAND,
                payload: "list".to_string(),
            },
            |_| Ok("players".to_string()),
        );
        assert_eq!(denied[0].request_id, SERVERDATA_AUTH_FAILURE);

        session.handle_packet(
            RconPacket {
                request_id: 2,
                packet_type: SERVERDATA_AUTH,
                payload: "secret".to_string(),
            },
            |_| Ok(String::new()),
        );
        let response = session.handle_packet(
            RconPacket {
                request_id: 3,
                packet_type: SERVERDATA_EXECCOMMAND,
                payload: "list".to_string(),
            },
            |command| Ok(format!("ran {command}")),
        );

        assert_eq!(response[0].request_id, 3);
        assert_eq!(response[0].packet_type, SERVERDATA_RESPONSE_VALUE);
        assert_eq!(response[0].payload, "ran list");
        assert_eq!(session.broadcasts(), &["list".to_string()]);
    }

    #[test]
    fn rcon_fragments_long_responses_and_reports_unknown_requests() {
        let mut session = RconSession::new("secret", false);
        session.handle_packet(
            RconPacket {
                request_id: 1,
                packet_type: SERVERDATA_AUTH,
                payload: "secret".to_string(),
            },
            |_| Ok(String::new()),
        );

        let long = "x".repeat(4097);
        let response = session.handle_packet(
            RconPacket {
                request_id: 9,
                packet_type: SERVERDATA_EXECCOMMAND,
                payload: "long".to_string(),
            },
            |_| Ok(long),
        );
        assert_eq!(response.len(), 2);
        assert_eq!(response[0].payload.len(), 4096);
        assert_eq!(response[1].payload.len(), 1);

        let unknown = session.handle_packet(
            RconPacket {
                request_id: 10,
                packet_type: 0xFEED,
                payload: String::new(),
            },
            |_| Ok(String::new()),
        );
        assert_eq!(unknown[0].payload, "Unknown request feed");
    }

    fn assert_contains_all(source: &str, sentinels: &[&str]) {
        for sentinel in sentinels {
            assert!(source.contains(sentinel), "missing rcon sentinel {sentinel}");
        }
    }
}
