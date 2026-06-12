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
    use crate::network::varint::{read_var_i32, write_var_i32};
    use std::io::Cursor;

    const CLIENT_INTENT_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/handshake/ClientIntent.java"
    );
    const CLIENT_INTENTION_PACKET_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/handshake/ClientIntentionPacket.java"
    );
    const HANDSHAKE_PACKET_TYPES_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/handshake/HandshakePacketTypes.java"
    );
    const HANDSHAKE_PROTOCOLS_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/handshake/HandshakeProtocols.java"
    );
    const SERVER_HANDSHAKE_PACKET_LISTENER_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/handshake/ServerHandshakePacketListener.java"
    );

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
    fn handshake_sources_match_java_packet_contracts() {
        assert_java_contains(
            CLIENT_INTENT_JAVA,
            &[
                "STATUS_ID = 1",
                "LOGIN_ID = 2",
                "TRANSFER_ID = 3",
                "case 1 -> STATUS;",
                "case 2 -> LOGIN;",
                "case 3 -> TRANSFER;",
                "throw new IllegalArgumentException(\"Unknown connection intent: \" + id)",
            ],
        );
        assert_java_contains(
            CLIENT_INTENTION_PACKET_JAVA,
            &[
                "public static final StreamCodec<FriendlyByteBuf, ClientIntentionPacket> STREAM_CODEC",
                "private static final int MAX_HOST_LENGTH = 255;",
                "input.readVarInt()",
                "input.readUtf(255)",
                "input.readUnsignedShort()",
                "ClientIntent.byId(input.readVarInt())",
                "output.writeVarInt(this.protocolVersion);",
                "output.writeUtf(this.hostName);",
                "output.writeShort(this.port);",
                "output.writeVarInt(this.intention.id());",
                "return HandshakePacketTypes.CLIENT_INTENTION;",
                "listener.handleIntention(this);",
                "return true;",
            ],
        );
        assert_java_contains(
            HANDSHAKE_PACKET_TYPES_JAVA,
            &[
                "CLIENT_INTENTION = createServerbound(\"intention\")",
                "new PacketType<>(PacketFlow.SERVERBOUND, Identifier.withDefaultNamespace(id))",
            ],
        );
        assert_java_contains(
            HANDSHAKE_PROTOCOLS_JAVA,
            &[
                "ConnectionProtocol.HANDSHAKING",
                "builder.addPacket(HandshakePacketTypes.CLIENT_INTENTION, ClientIntentionPacket.STREAM_CODEC)",
                "SERVERBOUND_TEMPLATE.bind(FriendlyByteBuf::new)",
            ],
        );
        assert_java_contains(
            SERVER_HANDSHAKE_PACKET_LISTENER_JAVA,
            &[
                "extends ServerPacketListener",
                "return ConnectionProtocol.HANDSHAKING;",
                "void handleIntention(ClientIntentionPacket packet);",
            ],
        );

        let mut payload = Vec::new();
        let packet = ClientIntentionPacket {
            protocol_version: 775,
            host_name: "example.org".to_string(),
            port: 25565,
            intention: ClientIntent::Transfer,
        };
        packet.write(&mut payload).unwrap();
        let mut input = Cursor::new(payload);
        assert_eq!(read_var_i32(&mut input).unwrap(), 775);
        assert_eq!(super::read_string(&mut input, 255).unwrap(), "example.org");
        assert_eq!(
            [input.get_ref()[input.position() as usize], input.get_ref()[input.position() as usize + 1]],
            25565u16.to_be_bytes()
        );
        input.set_position(input.position() + 2);
        assert_eq!(read_var_i32(&mut input).unwrap(), ClientIntent::Transfer.id());
    }

    #[test]
    fn maps_vanilla_intent_ids() {
        assert_eq!(ClientIntent::by_id(1).unwrap(), ClientIntent::Status);
        assert_eq!(ClientIntent::by_id(2).unwrap(), ClientIntent::Login);
        assert_eq!(ClientIntent::by_id(3).unwrap(), ClientIntent::Transfer);
        assert!(ClientIntent::by_id(4).is_err());
    }

    #[test]
    fn client_intention_packet_matches_java_bounds_and_error_paths() {
        let long_host = "a".repeat(256);
        let packet = ClientIntentionPacket {
            protocol_version: 775,
            host_name: long_host,
            port: 25565,
            intention: ClientIntent::Status,
        };
        assert!(packet.write(&mut Vec::new()).is_err());

        let mut invalid_intent = Vec::new();
        write_var_i32(&mut invalid_intent, 775).unwrap();
        super::write_string(&mut invalid_intent, "localhost", 255).unwrap();
        invalid_intent.extend_from_slice(&25565u16.to_be_bytes());
        write_var_i32(&mut invalid_intent, 4).unwrap();
        let err = ClientIntentionPacket::read(&mut Cursor::new(invalid_intent)).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("Unknown connection intent: 4"));
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

    fn assert_java_contains(source: &str, sentinels: &[&str]) {
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "missing Java source sentinel {sentinel}"
            );
        }
    }
}
