use std::io::{self, Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPingRequestPacket {
    pub time: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundPongResponsePacket {
    pub time: i64,
}

impl ServerboundPingRequestPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            time: read_i64_be(reader)?,
        })
    }

    #[allow(dead_code, reason = "Java exposes a symmetric packet codec; the server only decodes this packet")]
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.time.to_be_bytes())
    }
}

impl ClientboundPongResponsePacket {
    #[allow(dead_code, reason = "Java exposes a symmetric packet codec; the server only encodes this packet")]
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            time: read_i64_be(reader)?,
        })
    }

    pub fn from_request(request: ServerboundPingRequestPacket) -> Self {
        Self { time: request.time }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.time.to_be_bytes())
    }
}

fn read_i64_be<R: Read>(reader: &mut R) -> io::Result<i64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(i64::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::{ClientboundPongResponsePacket, ServerboundPingRequestPacket};
    use std::io::Cursor;

    #[test]
    fn reads_serverbound_ping_request_as_big_endian_long() {
        let mut input = Cursor::new(123456789_i64.to_be_bytes());
        let packet = ServerboundPingRequestPacket::read(&mut input).unwrap();
        assert_eq!(packet.time, 123456789);

        let mut output = Vec::new();
        packet.write(&mut output).unwrap();
        assert_eq!(output, 123456789_i64.to_be_bytes());
    }

    #[test]
    fn writes_clientbound_pong_response_as_big_endian_long() {
        let response = ClientboundPongResponsePacket { time: -42 };
        let mut output = Vec::new();
        response.write(&mut output).unwrap();
        assert_eq!(output, (-42_i64).to_be_bytes());
        assert_eq!(
            ClientboundPongResponsePacket::read(&mut Cursor::new(output)).unwrap(),
            response
        );
    }

    #[test]
    fn pong_echoes_ping_time() {
        let request = ServerboundPingRequestPacket { time: 123 };
        assert_eq!(
            ClientboundPongResponsePacket::from_request(request).time,
            123
        );
    }

    #[test]
    fn ping_packet_type_sources_match_java_names_and_listener_methods() {
        const PING_PACKET_TYPES_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/ping/PingPacketTypes.java");
        const CLIENT_LISTENER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/ping/ClientPongPacketListener.java");
        const SERVER_LISTENER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/ping/ServerPingPacketListener.java");

        assert!(PING_PACKET_TYPES_JAVA.contains(
            "CLIENTBOUND_PONG_RESPONSE = createClientbound(\"pong_response\")"
        ));
        assert!(PING_PACKET_TYPES_JAVA.contains(
            "SERVERBOUND_PING_REQUEST = createServerbound(\"ping_request\")"
        ));
        assert!(CLIENT_LISTENER_JAVA.contains(
            "void handlePongResponse(ClientboundPongResponsePacket packet);"
        ));
        assert!(SERVER_LISTENER_JAVA
            .contains("void handlePingRequest(ServerboundPingRequestPacket packet);"));
    }
}
