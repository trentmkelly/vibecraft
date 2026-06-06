#![allow(dead_code)]

use std::io::{self, Read, Write};

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::network::varint::{read_frame_length, read_var_i32, write_var_i32};

pub const DEFAULT_COMPRESSION_THRESHOLD: i32 = 256;
pub const MAX_UNCOMPRESSED_PACKET_SIZE: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionState {
    threshold: Option<i32>,
}

impl CompressionState {
    pub fn disabled() -> Self {
        Self { threshold: None }
    }

    pub fn enabled(threshold: i32) -> Self {
        if threshold < 0 {
            Self::disabled()
        } else {
            Self {
                threshold: Some(threshold),
            }
        }
    }

    pub fn threshold(self) -> Option<i32> {
        self.threshold
    }

    pub fn encode_packet(&self, payload: &[u8]) -> io::Result<Vec<u8>> {
        match self.threshold {
            None => encode_uncompressed_frame(payload),
            Some(threshold) if payload.len() < threshold as usize => {
                encode_compression_frame(0, payload)
            }
            Some(_threshold) => {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(payload)?;
                let compressed = encoder.finish()?;
                encode_compression_frame(payload.len() as i32, &compressed)
            }
        }
    }

    pub fn decode_packet<R: Read>(&self, reader: &mut R) -> io::Result<Vec<u8>> {
        let frame = read_packet_frame(reader)?;
        match self.threshold {
            None => Ok(frame),
            Some(threshold) => decode_compression_frame(threshold, &frame),
        }
    }
}

fn encode_uncompressed_frame(payload: &[u8]) -> io::Result<Vec<u8>> {
    let mut frame = Vec::new();
    write_var_i32(&mut frame, payload.len() as i32)?;
    frame.extend_from_slice(payload);
    Ok(frame)
}

fn encode_compression_frame(data_length: i32, payload: &[u8]) -> io::Result<Vec<u8>> {
    let mut body = Vec::new();
    write_var_i32(&mut body, data_length)?;
    body.extend_from_slice(payload);

    let mut frame = Vec::new();
    write_var_i32(&mut frame, body.len() as i32)?;
    frame.extend_from_slice(&body);
    Ok(frame)
}

fn read_packet_frame<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    // The outer (post-compression) frame length is the same 21-bit-max,
    // non-zero VarInt as Java Varint21FrameDecoder.
    let length = read_frame_length(reader)?;
    let mut payload = vec![0u8; length as usize];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

fn decode_compression_frame(threshold: i32, frame: &[u8]) -> io::Result<Vec<u8>> {
    let mut input = frame;
    let data_length = read_var_i32(&mut input)?;
    if data_length < 0 || data_length as usize > MAX_UNCOMPRESSED_PACKET_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid uncompressed packet length",
        ));
    }
    if data_length == 0 {
        if input.len() >= threshold as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "uncompressed packet exceeds compression threshold",
            ));
        }
        return Ok(input.to_vec());
    }
    if data_length < threshold {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "compressed packet below compression threshold",
        ));
    }

    let mut decoder = ZlibDecoder::new(input);
    let mut payload = Vec::with_capacity(data_length as usize);
    decoder.read_to_end(&mut payload)?;
    if payload.len() != data_length as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "decompressed packet length mismatch",
        ));
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::codec::cursor;

    #[test]
    fn disabled_compression_uses_plain_packet_length_frame() {
        let state = CompressionState::disabled();
        let payload = vec![0x01, 0x02, 0x03];
        let encoded = state.encode_packet(&payload).unwrap();

        assert_eq!(encoded, vec![0x03, 0x01, 0x02, 0x03]);
        assert_eq!(state.decode_packet(&mut cursor(encoded)).unwrap(), payload);
    }

    #[test]
    fn enabled_compression_leaves_small_packets_uncompressed_with_zero_data_length() {
        let state = CompressionState::enabled(10);
        let payload = vec![0x2a; 4];
        let encoded = state.encode_packet(&payload).unwrap();

        assert_eq!(encoded[0], 5);
        assert_eq!(encoded[1], 0);
        assert_eq!(state.decode_packet(&mut cursor(encoded)).unwrap(), payload);
    }

    #[test]
    fn enabled_compression_round_trips_large_packets_with_zlib_payload() {
        let state = CompressionState::enabled(8);
        let payload = b"this packet is large enough to compress".repeat(4);
        let encoded = state.encode_packet(&payload).unwrap();

        assert_ne!(encoded[1], 0);
        assert_eq!(state.decode_packet(&mut cursor(encoded)).unwrap(), payload);
    }

    #[test]
    fn enabled_compression_rejects_threshold_violations() {
        let state = CompressionState::enabled(4);

        let oversized_uncompressed = encode_compression_frame(0, &[1, 2, 3, 4]).unwrap();
        assert_eq!(
            state
                .decode_packet(&mut cursor(oversized_uncompressed))
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );

        let compressed_too_small = encode_compression_frame(2, &[0x78, 0x9c, 0x03, 0x00]).unwrap();
        assert_eq!(
            state
                .decode_packet(&mut cursor(compressed_too_small))
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }
}
