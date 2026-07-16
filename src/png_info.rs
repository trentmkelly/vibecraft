//! PNG signature and IHDR metadata parsing matching Minecraft's `PngInfo`.

#![allow(dead_code)]

use std::io::{self, Read};

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const IHDR_TYPE: [u8; 4] = *b"IHDR";
const IHDR_SIZE: i32 = 13;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PngInfo {
    pub width: i32,
    pub height: i32,
}

impl PngInfo {
    pub fn from_stream(mut input: impl Read) -> io::Result<Self> {
        let mut signature = [0; 8];
        input.read_exact(&mut signature)?;
        if signature != PNG_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Bad PNG Signature: 0x{:016X}", u64::from_be_bytes(signature)),
            ));
        }

        let mut length = [0; 4];
        input.read_exact(&mut length)?;
        let header_size = i32::from_be_bytes(length);
        if header_size != IHDR_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Bad length for IHDR chunk: {header_size}"),
            ));
        }

        let mut chunk_type = [0; 4];
        input.read_exact(&mut chunk_type)?;
        if chunk_type != IHDR_TYPE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Bad type for IHDR chunk: 0x{:08X}", u32::from_be_bytes(chunk_type)),
            ));
        }

        let mut dimensions = [0; 8];
        input.read_exact(&mut dimensions)?;
        Ok(Self {
            width: i32::from_be_bytes([dimensions[0], dimensions[1], dimensions[2], dimensions[3]]),
            height: i32::from_be_bytes([
                dimensions[4],
                dimensions[5],
                dimensions[6],
                dimensions[7],
            ]),
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Self::from_stream(bytes)
    }

    pub fn validate_header(buffer: &[u8]) -> io::Result<()> {
        if buffer.len() < 16 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "PNG header missing"));
        }
        if buffer[0..8] != PNG_SIGNATURE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Bad PNG Signature"));
        }
        if i32::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]) != IHDR_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Bad length for IHDR chunk!",
            ));
        }
        if buffer[12..16] != IHDR_TYPE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Bad type for IHDR chunk!",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PngInfo;

    fn png_header(width: i32, height: i32) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        bytes.extend_from_slice(&13_i32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn png_info_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/PngInfo.java");
        assert_eq!(JAVA.lines().count(), 64);
        for fragment in [
            "public record PngInfo(int width, int height)",
            "private static final long PNG_HEADER",
            "private static final int IHDR_TYPE",
            "private static final int IHDR_SIZE = 13",
            "Bad PNG Signature",
            "Bad length for IHDR chunk",
            "Bad type for IHDR chunk",
            "public static PngInfo fromBytes",
            "public static void validateHeader",
            "ByteOrder.BIG_ENDIAN",
        ] {
            assert!(JAVA.contains(fragment), "missing PngInfo source fragment: {fragment}");
        }
    }

    #[test]
    fn png_info_parses_and_validates_big_endian_ihdr() {
        let bytes = png_header(320, 240);
        assert_eq!(PngInfo::from_bytes(&bytes).ok(), Some(PngInfo { width: 320, height: 240 }));
        assert!(PngInfo::validate_header(&bytes).is_ok());

        let mut bad_signature = bytes.clone();
        bad_signature[0] = 0;
        assert!(PngInfo::from_bytes(&bad_signature).is_err());
        assert_eq!(PngInfo::validate_header(&bad_signature).unwrap_err().to_string(), "Bad PNG Signature");

        let mut bad_length = bytes.clone();
        bad_length[11] = 12;
        let bad_length_error = match PngInfo::from_bytes(&bad_length) {
            Ok(_) => panic!("invalid length unexpectedly parsed"),
            Err(error) => error,
        };
        assert_eq!(bad_length_error.to_string(), "Bad length for IHDR chunk: 12");
        assert!(PngInfo::validate_header(&bad_length).is_err());

        let mut bad_type = bytes;
        bad_type[12] = b'X';
        assert!(PngInfo::from_bytes(&bad_type).is_err());
        assert!(PngInfo::validate_header(&bad_type).is_err());
        assert!(PngInfo::validate_header(&[0; 15]).is_err());
    }
}
