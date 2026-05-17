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
}

impl ClientboundPongResponsePacket {
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
    }

    #[test]
    fn writes_clientbound_pong_response_as_big_endian_long() {
        let response = ClientboundPongResponsePacket { time: -42 };
        let mut output = Vec::new();
        response.write(&mut output).unwrap();
        assert_eq!(output, (-42_i64).to_be_bytes());
    }

    #[test]
    fn pong_echoes_ping_time() {
        let request = ServerboundPingRequestPacket { time: 123 };
        assert_eq!(
            ClientboundPongResponsePacket::from_request(request).time,
            123
        );
    }
}
