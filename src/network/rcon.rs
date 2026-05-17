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
const MAX_RESPONSE_CHARS: usize = 4096;

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
    if length < 10 || length > 1460 {
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
    Ok(thread::Builder::new()
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
        .map_err(|err| format!("Failed to start RCON listener thread: {err}"))?)
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
    loop {
        let packet = match read_packet(&mut stream) {
            Ok(Some(packet)) => packet,
            Ok(None) | Err(_) => break,
        };
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
        read_packet, write_packet, RconPacket, RconSession, SERVERDATA_AUTH,
        SERVERDATA_AUTH_FAILURE, SERVERDATA_AUTH_RESPONSE, SERVERDATA_EXECCOMMAND,
        SERVERDATA_RESPONSE_VALUE,
    };
    use std::io::Cursor;

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
}
