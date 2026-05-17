#![allow(dead_code)]

use std::io::{self, Read, Write};

use crate::network::codec::{
    read_collection, read_component, read_identifier, read_optional, read_string, read_uuid,
    write_component, write_identifier, write_optional, write_string, write_uuid, ComponentJson,
    Uuid,
};
use crate::network::cookie::{CookieResponseStatus, CookieState, ServerboundCookieResponsePacket};
use crate::network::varint::{read_var_i32, write_var_i32};
use crate::player_access::NameAndId;
use crate::registry::Identifier;

pub const MAX_LOGIN_CUSTOM_QUERY_PAYLOAD_SIZE: usize = 1_048_576;
pub const SERVERBOUND_HELLO_PACKET_ID: i32 = 0;
pub const SERVERBOUND_KEY_PACKET_ID: i32 = 1;
pub const SERVERBOUND_CUSTOM_QUERY_ANSWER_PACKET_ID: i32 = 2;
pub const SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID: i32 = 3;
pub const SERVERBOUND_COOKIE_RESPONSE_PACKET_ID: i32 = 4;
pub const CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID: i32 = 0;
pub const CLIENTBOUND_HELLO_PACKET_ID: i32 = 1;
pub const CLIENTBOUND_LOGIN_FINISHED_PACKET_ID: i32 = 2;
pub const CLIENTBOUND_LOGIN_COMPRESSION_PACKET_ID: i32 = 3;
pub const CLIENTBOUND_CUSTOM_QUERY_PACKET_ID: i32 = 4;
pub const CLIENTBOUND_COOKIE_REQUEST_PACKET_ID: i32 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundHelloPacket {
    pub name: String,
    pub profile_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundHelloPacket {
    pub server_id: String,
    pub public_key: Vec<u8>,
    pub challenge: Vec<u8>,
    pub should_authenticate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundKeyPacket {
    pub key_bytes: Vec<u8>,
    pub encrypted_challenge: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomQueryPacket {
    pub transaction_id: i32,
    pub channel: Identifier,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCustomQueryAnswerPacket {
    pub transaction_id: i32,
    pub payload: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLoginCompressionPacket {
    pub compression_threshold: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLoginDisconnectPacket {
    pub reason: ComponentJson,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLoginFinishedPacket {
    pub profile: NameAndId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundLoginAcknowledgedPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginState {
    Hello,
    Authenticating,
    Accepted,
    Complete,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginSession {
    pub state: LoginState,
    pub profile: Option<NameAndId>,
    pub compression_threshold: Option<i32>,
    pub disconnect_reason: Option<String>,
    pub pending_custom_queries: Vec<i32>,
    pub custom_query_answers: Vec<ServerboundCustomQueryAnswerPacket>,
    pub cookies: CookieState,
}

impl Default for LoginSession {
    fn default() -> Self {
        Self {
            state: LoginState::Hello,
            profile: None,
            compression_threshold: None,
            disconnect_reason: None,
            pending_custom_queries: Vec::new(),
            custom_query_answers: Vec::new(),
            cookies: CookieState::default(),
        }
    }
}

impl ServerboundHelloPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            name: read_string(reader, 16)?,
            profile_id: read_uuid(reader)?,
        };
        if !is_valid_player_name(&packet.name) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid characters in username",
            ));
        }
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.name, 16)?;
        write_uuid(writer, self.profile_id)
    }
}

fn is_valid_player_name(name: &str) -> bool {
    name.chars().count() <= 16 && name.chars().all(|c| c > ' ' && c < '\u{7f}')
}

impl ClientboundHelloPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            server_id: read_string(reader, 20)?,
            public_key: read_byte_array(reader)?,
            challenge: read_byte_array(reader)?,
            should_authenticate: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.server_id, 20)?;
        write_byte_array(writer, &self.public_key)?;
        write_byte_array(writer, &self.challenge)?;
        write_bool(writer, self.should_authenticate)
    }
}

impl ServerboundKeyPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            key_bytes: read_byte_array(reader)?,
            encrypted_challenge: read_byte_array(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_byte_array(writer, &self.key_bytes)?;
        write_byte_array(writer, &self.encrypted_challenge)
    }
}

impl ClientboundCustomQueryPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            transaction_id: read_var_i32(reader)?,
            channel: read_identifier(reader)?,
            payload: read_remaining_limited(reader, MAX_LOGIN_CUSTOM_QUERY_PAYLOAD_SIZE)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.transaction_id)?;
        write_identifier(writer, &self.channel)?;
        write_limited_payload(writer, &self.payload, MAX_LOGIN_CUSTOM_QUERY_PAYLOAD_SIZE)
    }
}

impl ServerboundCustomQueryAnswerPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            transaction_id: read_var_i32(reader)?,
            payload: read_optional(reader, |reader| {
                read_remaining_limited(reader, MAX_LOGIN_CUSTOM_QUERY_PAYLOAD_SIZE)
            })?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.transaction_id)?;
        write_optional(writer, self.payload.as_ref(), |writer, payload| {
            write_limited_payload(writer, payload, MAX_LOGIN_CUSTOM_QUERY_PAYLOAD_SIZE)
        })
    }
}

impl ClientboundLoginCompressionPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            compression_threshold: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.compression_threshold)
    }
}

impl ClientboundLoginDisconnectPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            reason: read_component(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_component(writer, &self.reason)
    }
}

impl ClientboundLoginFinishedPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let uuid = read_uuid(reader)?;
        let name = read_string(reader, 16)?;
        let _properties = read_collection(reader, |reader| {
            let _name = read_string(reader, 64)?;
            let _value = read_string(reader, 32767)?;
            let _signature = read_optional(reader, |reader| read_string(reader, 1024))?;
            Ok(())
        })?;
        Ok(Self {
            profile: NameAndId {
                name,
                uuid: hyphenate_uuid(uuid),
            },
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_uuid(writer, uuid_from_hyphenated(&self.profile.uuid)?)?;
        write_string(writer, &self.profile.name, 16)?;
        crate::network::varint::write_var_i32(writer, 0)
    }
}

impl ServerboundLoginAcknowledgedPacket {
    pub fn read<R: Read>(_reader: &mut R) -> io::Result<Self> {
        Ok(Self)
    }

    pub fn write<W: Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl LoginSession {
    pub fn start_encryption(
        &mut self,
        server_id: impl Into<String>,
        public_key: Vec<u8>,
        challenge: Vec<u8>,
        should_authenticate: bool,
    ) -> ClientboundHelloPacket {
        self.state = LoginState::Authenticating;
        ClientboundHelloPacket {
            server_id: server_id.into(),
            public_key,
            challenge,
            should_authenticate,
        }
    }

    pub fn send_custom_query(
        &mut self,
        transaction_id: i32,
        channel: Identifier,
        payload: Vec<u8>,
    ) -> io::Result<ClientboundCustomQueryPacket> {
        if payload.len() > MAX_LOGIN_CUSTOM_QUERY_PAYLOAD_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "custom query payload too large",
            ));
        }
        self.pending_custom_queries.push(transaction_id);
        Ok(ClientboundCustomQueryPacket {
            transaction_id,
            channel,
            payload,
        })
    }

    pub fn handle_custom_query_answer(
        &mut self,
        answer: ServerboundCustomQueryAnswerPacket,
    ) -> io::Result<()> {
        let Some(index) = self
            .pending_custom_queries
            .iter()
            .position(|id| *id == answer.transaction_id)
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected custom query answer",
            ));
        };
        self.pending_custom_queries.remove(index);
        self.custom_query_answers.push(answer);
        Ok(())
    }

    pub fn handle_cookie_response(
        &mut self,
        response: ServerboundCookieResponsePacket,
    ) -> io::Result<CookieResponseStatus> {
        self.cookies.handle_response(response)
    }

    pub fn accept_offline_hello(
        &mut self,
        hello: ServerboundHelloPacket,
    ) -> ClientboundLoginFinishedPacket {
        let profile = NameAndId::create_offline(&hello.name);
        self.profile = Some(profile.clone());
        self.state = LoginState::Accepted;
        ClientboundLoginFinishedPacket { profile }
    }

    pub fn acknowledge(&mut self, _packet: ServerboundLoginAcknowledgedPacket) {
        self.state = LoginState::Complete;
    }

    pub fn disconnect(&mut self, reason: impl Into<String>) {
        self.state = LoginState::Disconnected;
        self.disconnect_reason = Some(reason.into());
    }

    pub fn set_compression(&mut self, threshold: i32) {
        self.compression_threshold = Some(threshold);
    }

    pub fn can_finish(&self) -> bool {
        self.profile.is_some() && self.pending_custom_queries.is_empty()
    }
}

fn uuid_from_hyphenated(value: &str) -> io::Result<Uuid> {
    let hex: String = value.chars().filter(|ch| *ch != '-').collect();
    if hex.len() != 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid UUID length",
        ));
    }
    let mut bytes = [0u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&hex[start..start + 2], 16)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    }
    Ok(Uuid(bytes))
}

fn hyphenate_uuid(uuid: Uuid) -> String {
    let bytes = uuid.0;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

fn read_bool<R: Read>(reader: &mut R) -> io::Result<bool> {
    let mut byte = [0u8; 1];
    reader.read_exact(&mut byte)?;
    match byte[0] {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid boolean value",
        )),
    }
}

fn write_bool<W: Write>(writer: &mut W, value: bool) -> io::Result<()> {
    writer.write_all(&[u8::from(value)])
}

fn read_byte_array<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let length = read_var_i32(reader)?;
    if length < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative byte array length",
        ));
    }
    let mut bytes = vec![0; length as usize];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn write_byte_array<W: Write>(writer: &mut W, bytes: &[u8]) -> io::Result<()> {
    write_var_i32(writer, bytes.len() as i32)?;
    writer.write_all(bytes)
}

fn read_remaining_limited<R: Read>(reader: &mut R, max_size: usize) -> io::Result<Vec<u8>> {
    let mut payload = Vec::new();
    reader.take(max_size as u64 + 1).read_to_end(&mut payload)?;
    if payload.len() > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "payload too large",
        ));
    }
    Ok(payload)
}

fn write_limited_payload<W: Write>(
    writer: &mut W,
    payload: &[u8],
    max_size: usize,
) -> io::Result<()> {
    if payload.len() > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "payload too large",
        ));
    }
    writer.write_all(payload)
}

#[cfg(test)]
mod tests {
    use super::{
        ClientboundCustomQueryPacket, ClientboundHelloPacket, ClientboundLoginCompressionPacket,
        ClientboundLoginDisconnectPacket, LoginSession, LoginState,
        ServerboundCustomQueryAnswerPacket, ServerboundHelloPacket, ServerboundKeyPacket,
        ServerboundLoginAcknowledgedPacket, CLIENTBOUND_COOKIE_REQUEST_PACKET_ID,
        CLIENTBOUND_CUSTOM_QUERY_PACKET_ID, CLIENTBOUND_HELLO_PACKET_ID,
        CLIENTBOUND_LOGIN_COMPRESSION_PACKET_ID, CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID,
        CLIENTBOUND_LOGIN_FINISHED_PACKET_ID, MAX_LOGIN_CUSTOM_QUERY_PAYLOAD_SIZE,
        SERVERBOUND_COOKIE_RESPONSE_PACKET_ID, SERVERBOUND_CUSTOM_QUERY_ANSWER_PACKET_ID,
        SERVERBOUND_HELLO_PACKET_ID, SERVERBOUND_KEY_PACKET_ID,
        SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID,
    };
    use crate::network::codec::{ComponentJson, Uuid};
    use crate::network::cookie::ServerboundCookieResponsePacket;
    use crate::registry::Identifier;
    use std::io::{self, Cursor};

    #[test]
    fn reads_and_writes_serverbound_hello() {
        let packet = ServerboundHelloPacket {
            name: "Steve".to_string(),
            profile_id: Uuid([1; 16]),
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        assert_eq!(
            ServerboundHelloPacket::read(&mut Cursor::new(bytes)).unwrap(),
            packet
        );
    }

    #[test]
    fn validates_serverbound_hello_names_like_vanilla() {
        for name in [
            "Steve",
            "abcdefghijklmnop",
            "CaseName",
            "casename",
            "dash-name",
            "period.name",
        ] {
            let packet = ServerboundHelloPacket {
                name: name.to_string(),
                profile_id: Uuid([1; 16]),
            };
            let mut bytes = Vec::new();
            packet.write(&mut bytes).unwrap();
            assert_eq!(
                ServerboundHelloPacket::read(&mut Cursor::new(bytes))
                    .unwrap()
                    .name,
                name
            );
        }

        for name in [
            "has space",
            "newline\nname",
            "seventeen_chars__",
            "nonasciié",
            "delete\u{7f}name",
        ] {
            let mut bytes = Vec::new();
            crate::network::codec::write_string(&mut bytes, name, 32).unwrap();
            crate::network::codec::write_uuid(&mut bytes, Uuid([1; 16])).unwrap();
            let err = ServerboundHelloPacket::read(&mut Cursor::new(bytes)).unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        }
    }

    #[test]
    fn offline_login_accepts_hello_and_acknowledges() {
        let mut session = LoginSession::default();
        let finished = session.accept_offline_hello(ServerboundHelloPacket {
            name: "Steve".to_string(),
            profile_id: Uuid([0; 16]),
        });
        assert_eq!(session.state, LoginState::Accepted);
        assert_eq!(
            finished.profile.uuid,
            "5627dd98-e6be-3c21-b8a8-e92344183641"
        );

        session.acknowledge(ServerboundLoginAcknowledgedPacket);
        assert_eq!(session.state, LoginState::Complete);
    }

    #[test]
    fn writes_login_finished_profile_with_empty_properties() {
        let mut session = LoginSession::default();
        let packet = session.accept_offline_hello(ServerboundHelloPacket {
            name: "Steve".to_string(),
            profile_id: Uuid([0; 16]),
        });
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        assert!(bytes.ends_with(&[0]));
        assert_eq!(
            super::ClientboundLoginFinishedPacket::read(&mut Cursor::new(bytes))
                .unwrap()
                .profile,
            packet.profile
        );
    }

    #[test]
    fn login_packet_ids_match_vanilla_protocol_order() {
        assert_eq!(SERVERBOUND_HELLO_PACKET_ID, 0);
        assert_eq!(SERVERBOUND_KEY_PACKET_ID, 1);
        assert_eq!(SERVERBOUND_CUSTOM_QUERY_ANSWER_PACKET_ID, 2);
        assert_eq!(SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID, 3);
        assert_eq!(SERVERBOUND_COOKIE_RESPONSE_PACKET_ID, 4);
        assert_eq!(CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID, 0);
        assert_eq!(CLIENTBOUND_HELLO_PACKET_ID, 1);
        assert_eq!(CLIENTBOUND_LOGIN_FINISHED_PACKET_ID, 2);
        assert_eq!(CLIENTBOUND_LOGIN_COMPRESSION_PACKET_ID, 3);
        assert_eq!(CLIENTBOUND_CUSTOM_QUERY_PACKET_ID, 4);
        assert_eq!(CLIENTBOUND_COOKIE_REQUEST_PACKET_ID, 5);
    }

    #[test]
    fn round_trips_login_encryption_packets() {
        let hello = ClientboundHelloPacket {
            server_id: "".to_string(),
            public_key: vec![1, 2, 3],
            challenge: vec![4, 5],
            should_authenticate: true,
        };
        let mut bytes = Vec::new();
        hello.write(&mut bytes).unwrap();
        assert_eq!(
            ClientboundHelloPacket::read(&mut Cursor::new(bytes)).unwrap(),
            hello
        );

        let key = ServerboundKeyPacket {
            key_bytes: vec![8, 9],
            encrypted_challenge: vec![10, 11, 12],
        };
        let mut bytes = Vec::new();
        key.write(&mut bytes).unwrap();
        assert_eq!(
            ServerboundKeyPacket::read(&mut Cursor::new(bytes)).unwrap(),
            key
        );
    }

    #[test]
    fn round_trips_login_custom_query_packets() {
        let query = ClientboundCustomQueryPacket {
            transaction_id: 42,
            channel: Identifier::parse("rustcraft:query").unwrap(),
            payload: vec![1, 2, 3, 4],
        };
        let mut bytes = Vec::new();
        query.write(&mut bytes).unwrap();
        assert_eq!(
            ClientboundCustomQueryPacket::read(&mut Cursor::new(bytes)).unwrap(),
            query
        );

        let answer = ServerboundCustomQueryAnswerPacket {
            transaction_id: 42,
            payload: Some(vec![5, 6, 7]),
        };
        let mut bytes = Vec::new();
        answer.write(&mut bytes).unwrap();
        assert_eq!(
            ServerboundCustomQueryAnswerPacket::read(&mut Cursor::new(bytes)).unwrap(),
            answer
        );
    }

    #[test]
    fn round_trips_login_compression_and_disconnect_packets() {
        let compression = ClientboundLoginCompressionPacket {
            compression_threshold: 256,
        };
        let mut bytes = Vec::new();
        compression.write(&mut bytes).unwrap();
        assert_eq!(
            ClientboundLoginCompressionPacket::read(&mut Cursor::new(bytes)).unwrap(),
            compression
        );

        let disconnect = ClientboundLoginDisconnectPacket {
            reason: ComponentJson("{\"text\":\"bye\"}".to_string()),
        };
        let mut bytes = Vec::new();
        disconnect.write(&mut bytes).unwrap();
        assert_eq!(
            ClientboundLoginDisconnectPacket::read(&mut Cursor::new(bytes)).unwrap(),
            disconnect
        );
    }

    #[test]
    fn rejects_custom_query_payloads_larger_than_vanilla_limit() {
        let query = ClientboundCustomQueryPacket {
            transaction_id: 1,
            channel: Identifier::parse("rustcraft:query").unwrap(),
            payload: vec![0; MAX_LOGIN_CUSTOM_QUERY_PAYLOAD_SIZE + 1],
        };
        assert!(query.write(&mut Vec::new()).is_err());
    }

    #[test]
    fn login_session_tracks_encryption_custom_queries_and_finish_readiness() {
        let mut session = LoginSession::default();
        let encryption = session.start_encryption("", vec![1, 2], vec![3, 4], true);
        assert_eq!(session.state, LoginState::Authenticating);
        assert!(encryption.should_authenticate);

        let query = session
            .send_custom_query(
                7,
                Identifier::parse("rustcraft:login").unwrap(),
                vec![1, 2, 3],
            )
            .unwrap();
        assert_eq!(query.transaction_id, 7);
        assert_eq!(session.pending_custom_queries, vec![7]);
        assert!(!session.can_finish());

        session
            .handle_custom_query_answer(ServerboundCustomQueryAnswerPacket {
                transaction_id: 7,
                payload: Some(vec![4, 5]),
            })
            .unwrap();
        assert!(session.pending_custom_queries.is_empty());
        assert_eq!(session.custom_query_answers.len(), 1);

        session.accept_offline_hello(ServerboundHelloPacket {
            name: "Steve".to_string(),
            profile_id: Uuid([0; 16]),
        });
        assert!(session.can_finish());
    }

    #[test]
    fn login_session_rejects_unexpected_custom_query_answers_and_tracks_cookies() {
        let mut session = LoginSession::default();
        let err = session
            .handle_custom_query_answer(ServerboundCustomQueryAnswerPacket {
                transaction_id: 99,
                payload: None,
            })
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);

        let key = Identifier::parse("rustcraft:login").unwrap();
        session.cookies.request_cookie(key.clone());
        let status = session
            .handle_cookie_response(ServerboundCookieResponsePacket {
                key: key.clone(),
                payload: Some(vec![1, 2, 3]),
            })
            .unwrap();
        assert_eq!(
            status,
            crate::network::cookie::CookieResponseStatus::Stored(vec![1, 2, 3])
        );
        assert_eq!(session.cookies.get(&key), Some([1, 2, 3].as_slice()));
    }
}
