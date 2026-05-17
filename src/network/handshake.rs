#![allow(dead_code)]

use std::io::{self, Read, Write};

use crate::network::codec::{read_string, write_string};
use crate::network::varint::{read_var_i32, write_var_i32};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientIntent {
    Status,
    Login,
    Transfer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientIntentionPacket {
    pub protocol_version: i32,
    pub host_name: String,
    pub port: u16,
    pub intention: ClientIntent,
}

impl ClientIntent {
    pub fn by_id(id: i32) -> Result<Self, String> {
        match id {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            3 => Ok(Self::Transfer),
            _ => Err(format!("Unknown connection intent: {id}")),
        }
    }

    pub fn id(self) -> i32 {
        match self {
            Self::Status => 1,
            Self::Login => 2,
            Self::Transfer => 3,
        }
    }
}

impl ClientIntentionPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let protocol_version = read_var_i32(reader)?;
        let host_name = read_string(reader, 255)?;
        let mut port = [0u8; 2];
        reader.read_exact(&mut port)?;
        let intention = ClientIntent::by_id(read_var_i32(reader)?)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        Ok(Self {
            protocol_version,
            host_name,
            port: u16::from_be_bytes(port),
            intention,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.protocol_version)?;
        write_string(writer, &self.host_name, 255)?;
        writer.write_all(&self.port.to_be_bytes())?;
        write_var_i32(writer, self.intention.id())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolState {
    Handshake,
    Status,
    Login,
    Transfer,
}

pub fn transition_from_handshake(intent: ClientIntent) -> ProtocolState {
    match intent {
        ClientIntent::Status => ProtocolState::Status,
        ClientIntent::Login => ProtocolState::Login,
        ClientIntent::Transfer => ProtocolState::Transfer,
    }
}

#[cfg(test)]
mod tests {
    use super::{transition_from_handshake, ClientIntent, ClientIntentionPacket, ProtocolState};
    use std::io::Cursor;

    #[test]
    fn round_trips_client_intention_packet() {
        let packet = ClientIntentionPacket {
            protocol_version: 775,
            host_name: "localhost".to_string(),
            port: 25565,
            intention: ClientIntent::Login,
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        assert_eq!(
            ClientIntentionPacket::read(&mut Cursor::new(bytes)).unwrap(),
            packet
        );
    }

    #[test]
    fn maps_vanilla_intent_ids() {
        assert_eq!(ClientIntent::by_id(1).unwrap(), ClientIntent::Status);
        assert_eq!(ClientIntent::by_id(2).unwrap(), ClientIntent::Login);
        assert_eq!(ClientIntent::by_id(3).unwrap(), ClientIntent::Transfer);
        assert!(ClientIntent::by_id(4).is_err());
    }

    #[test]
    fn transitions_from_handshake_to_status_login_and_transfer() {
        assert_eq!(
            transition_from_handshake(ClientIntent::Status),
            ProtocolState::Status
        );
        assert_eq!(
            transition_from_handshake(ClientIntent::Login),
            ProtocolState::Login
        );
        assert_eq!(
            transition_from_handshake(ClientIntent::Transfer),
            ProtocolState::Transfer
        );
    }
}
