#![allow(dead_code)]

use std::io::{self, Read, Write};

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::network::varint::{read_frame_length, read_var_i32, write_var_i32};

pub const DEFAULT_COMPRESSION_THRESHOLD: i32 = 256;
pub const MAXIMUM_COMPRESSED_LENGTH: usize = 2 * 1024 * 1024;
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

#[derive(Debug)]
pub struct CompressionDecoder {
    threshold: i32,
    validate_decompressed: bool,
}

impl CompressionDecoder {
    pub fn new(threshold: i32, validate_decompressed: bool) -> Self {
        Self {
            threshold,
            validate_decompressed,
        }
    }

    pub fn decode(&self, frame: &[u8], output: &mut Vec<Vec<u8>>) -> io::Result<()> {
        output.push(decode_compression_frame_with_validation(
            self.threshold,
            self.validate_decompressed,
            frame,
        )?);
        Ok(())
    }

    pub fn set_threshold(&mut self, threshold: i32, validate_decompressed: bool) {
        self.threshold = threshold;
        self.validate_decompressed = validate_decompressed;
    }

    pub fn threshold(&self) -> i32 {
        self.threshold
    }

    pub fn validate_decompressed(&self) -> bool {
        self.validate_decompressed
    }
}

#[derive(Debug)]
pub struct CompressionEncoder {
    encode_buf: [u8; 8192],
    threshold: i32,
}

impl CompressionEncoder {
    pub fn new(threshold: i32) -> Self {
        Self {
            encode_buf: [0; 8192],
            threshold,
        }
    }

    pub fn encode(&mut self, uncompressed: &[u8], output: &mut Vec<u8>) -> io::Result<()> {
        if uncompressed.len() > MAX_UNCOMPRESSED_PACKET_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Packet too big (is {}, should be less than {})",
                    uncompressed.len(),
                    MAX_UNCOMPRESSED_PACKET_SIZE
                ),
            ));
        }

        if self.threshold >= 0 && uncompressed.len() < self.threshold as usize {
            write_var_i32(output, 0)?;
            output.extend_from_slice(uncompressed);
            return Ok(());
        }

        write_var_i32(output, uncompressed.len() as i32)?;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(uncompressed)?;
        let compressed = encoder.finish()?;
        for chunk in compressed.chunks(self.encode_buf.len()) {
            output.extend_from_slice(chunk);
        }
        Ok(())
    }

    pub fn threshold(&self) -> i32 {
        self.threshold
    }

    pub fn set_threshold(&mut self, threshold: i32) {
        self.threshold = threshold;
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
    decode_compression_frame_with_validation(threshold, true, frame)
}

fn decode_compression_frame_with_validation(
    threshold: i32,
    validate_decompressed: bool,
    frame: &[u8],
) -> io::Result<Vec<u8>> {
    let mut input = frame;
    let data_length = read_var_i32(&mut input)?;
    if data_length < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid uncompressed packet length",
        ));
    }
    if data_length == 0 {
        return Ok(input.to_vec());
    }
    if validate_decompressed {
        if data_length < threshold {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "compressed packet below compression threshold",
            ));
        }
        if data_length as usize > MAX_UNCOMPRESSED_PACKET_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid uncompressed packet length",
            ));
        }
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

#[cfg(all(test, vibecraft_has_decompiled_sources))]
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
                .unwrap(),
            vec![1, 2, 3, 4]
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

    #[test]
    fn compression_decoder_and_encoder_match_java_handler_rules() {
        const COMPRESSION_DECODER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/CompressionDecoder.java");
        const COMPRESSION_ENCODER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/CompressionEncoder.java");

        for sentinel in [
            "public static final int MAXIMUM_COMPRESSED_LENGTH = 2097152;",
            "public static final int MAXIMUM_UNCOMPRESSED_LENGTH = 8388608;",
            "if (uncompressedLength == 0) {",
            "out.add(in.readBytes(in.readableBytes()));",
            "if (uncompressedLength < this.threshold)",
            "if (actualUncompressedLength != uncompressedLength)",
            "public void setThreshold(final int threshold, final boolean validateDecompressed)",
        ] {
            assert!(
                COMPRESSION_DECODER_JAVA.contains(sentinel),
                "missing CompressionDecoder sentinel {sentinel}"
            );
        }
        for sentinel in [
            "private final byte[] encodeBuf = new byte[8192];",
            "if (uncompressedLength > 8388608)",
            "if (uncompressedLength < this.threshold)",
            "VarInt.write(out, 0);",
            "this.deflater.finish();",
            "public int getThreshold()",
            "public void setThreshold(final int threshold)",
        ] {
            assert!(
                COMPRESSION_ENCODER_JAVA.contains(sentinel),
                "missing CompressionEncoder sentinel {sentinel}"
            );
        }

        assert_eq!(MAXIMUM_COMPRESSED_LENGTH, 2_097_152);
        assert_eq!(MAX_UNCOMPRESSED_PACKET_SIZE, 8_388_608);

        let mut encoder = CompressionEncoder::new(8);
        let mut encoded_small = Vec::new();
        encoder.encode(b"small", &mut encoded_small).unwrap();
        assert_eq!(encoded_small, vec![0, b's', b'm', b'a', b'l', b'l']);

        let mut decoder = CompressionDecoder::new(8, true);
        let mut decoded = Vec::new();
        decoder.decode(&encoded_small, &mut decoded).unwrap();
        assert_eq!(decoded, vec![b"small".to_vec()]);

        let mut encoded_large = Vec::new();
        encoder
            .encode(b"large enough for compression", &mut encoded_large)
            .unwrap();
        decoded.clear();
        decoder.decode(&encoded_large, &mut decoded).unwrap();
        assert_eq!(decoded, vec![b"large enough for compression".to_vec()]);

        decoder.set_threshold(64, false);
        assert_eq!(decoder.threshold(), 64);
        assert!(!decoder.validate_decompressed());
        decoded.clear();
        decoder.decode(&encoded_large, &mut decoded).unwrap();
        assert_eq!(decoded, vec![b"large enough for compression".to_vec()]);

        encoder.set_threshold(64);
        assert_eq!(encoder.threshold(), 64);

        encoder.set_threshold(-1);
        let mut encoded_negative_threshold = Vec::new();
        encoder
            .encode(b"x", &mut encoded_negative_threshold)
            .unwrap();
        assert_ne!(encoded_negative_threshold[0], 0);
    }
}
