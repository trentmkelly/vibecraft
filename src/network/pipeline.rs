#![allow(dead_code)]

use std::io::{self, Cursor};

use crate::network::compression::CompressionState;
use crate::network::encryption::MinecraftCipher;
use crate::network::varint::{read_var_i32, write_var_i32};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPacket {
    pub id: i32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct NetworkPipeline {
    compression: CompressionState,
    inbound_cipher: Option<MinecraftCipher>,
    outbound_cipher: Option<MinecraftCipher>,
}

impl Default for NetworkPipeline {
    fn default() -> Self {
        Self {
            compression: CompressionState::disabled(),
            inbound_cipher: None,
            outbound_cipher: None,
        }
    }
}

impl NetworkPipeline {
    pub fn enable_compression(&mut self, threshold: i32) {
        self.compression = CompressionState::enabled(threshold);
    }

    pub fn enable_encryption(&mut self, shared_secret: [u8; 16]) {
        self.inbound_cipher = Some(MinecraftCipher::new(shared_secret));
        self.outbound_cipher = Some(MinecraftCipher::new(shared_secret));
    }

    pub fn encode_packet(&mut self, id: i32, payload: &[u8]) -> io::Result<Vec<u8>> {
        let mut packet = Vec::new();
        write_var_i32(&mut packet, id)?;
        packet.extend_from_slice(payload);

        let mut frame = self.compression.encode_packet(&packet)?;
        if let Some(cipher) = &mut self.outbound_cipher {
            cipher.apply_encrypt(&mut frame);
        }
        Ok(frame)
    }

    pub fn decode_packet(&mut self, frame: &[u8]) -> io::Result<DecodedPacket> {
        let mut bytes = frame.to_vec();
        if let Some(cipher) = &mut self.inbound_cipher {
            cipher.apply_decrypt(&mut bytes);
        }
        let packet = self.compression.decode_packet(&mut Cursor::new(bytes))?;
        let mut packet_input = Cursor::new(packet);
        let id = read_var_i32(&mut packet_input)?;
        let position = packet_input.position() as usize;
        let payload = packet_input.into_inner()[position..].to_vec();
        Ok(DecodedPacket { id, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_round_trips_uncompressed_packet_id_and_payload() {
        let mut encoder = NetworkPipeline::default();
        let mut decoder = NetworkPipeline::default();

        let frame = encoder.encode_packet(0x2a, b"payload").unwrap();
        let packet = decoder.decode_packet(&frame).unwrap();

        assert_eq!(
            packet,
            DecodedPacket {
                id: 0x2a,
                payload: b"payload".to_vec()
            }
        );
    }

    #[test]
    fn pipeline_applies_compression_before_encryption_and_reverses_on_decode() {
        let secret = *b"0123456789abcdef";
        let mut encoder = NetworkPipeline::default();
        let mut decoder = NetworkPipeline::default();
        encoder.enable_compression(8);
        decoder.enable_compression(8);
        encoder.enable_encryption(secret);
        decoder.enable_encryption(secret);

        let payload = b"large enough packet payload for zlib".repeat(3);
        let frame = encoder.encode_packet(0x10, &payload).unwrap();
        assert!(!frame.windows(payload.len()).any(|window| window == payload));

        let decoded = decoder.decode_packet(&frame).unwrap();
        assert_eq!(decoded.id, 0x10);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn pipeline_keeps_encryption_state_across_multiple_frames() {
        let secret = *b"fedcba9876543210";
        let mut encoder = NetworkPipeline::default();
        let mut decoder = NetworkPipeline::default();
        encoder.enable_encryption(secret);
        decoder.enable_encryption(secret);

        let first = encoder.encode_packet(1, b"one").unwrap();
        let second = encoder.encode_packet(2, b"two").unwrap();

        assert_eq!(decoder.decode_packet(&first).unwrap().payload, b"one");
        assert_eq!(decoder.decode_packet(&second).unwrap().payload, b"two");
    }
}
