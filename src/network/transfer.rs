use std::io::{self, Read, Write};

use crate::network::codec::{read_string, write_string};
use crate::network::varint::{read_var_i32, write_var_i32};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTransferPacket {
    pub host: String,
    pub port: i32,
}

impl ClientboundTransferPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            host: read_string(reader, 32767)?,
            port: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.host, 32767)?;
        write_var_i32(writer, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundTransferPacket;
    use std::io::Cursor;

    #[test]
    fn round_trips_clientbound_transfer_packet() {
        let packet = ClientboundTransferPacket {
            host: "example.org".to_string(),
            port: 25565,
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.push("example.org".len() as u8);
        expected.extend_from_slice(b"example.org");
        expected.extend_from_slice(&[0xdd, 0xc7, 0x01]);
        assert_eq!(bytes, expected);
        assert_eq!(
            ClientboundTransferPacket::read(&mut Cursor::new(bytes)).unwrap(),
            packet
        );
    }
}
