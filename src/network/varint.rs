#![allow(dead_code)]

use std::io::{self, Read, Write};

use crate::network::bandwidth::{BandwidthDebugMonitor, BandwidthSampleLogger};

pub const MAX_VAR_I32_SIZE: usize = 5;
pub const MAX_VAR_I64_SIZE: usize = 10;
pub const MAX_VARINT21_BYTES: usize = 3;

pub fn has_varint_continuation_bit(byte: u8) -> bool {
    byte & 0x80 == 0x80
}

pub fn read_var_i32<R: Read + ?Sized>(reader: &mut R) -> io::Result<i32> {
    let mut value = 0i32;
    let mut bytes = 0;

    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        let current = byte[0];
        if bytes == MAX_VAR_I32_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarInt is too big",
            ));
        }
        value |= ((current & 0x7F) as i32) << (bytes * 7);
        bytes += 1;

        if !has_varint_continuation_bit(current) {
            return Ok(value);
        }
    }
}

/// Read a packet frame-length prefix, 1:1 with Java `Varint21FrameDecoder`: the
/// length VarInt is at most 3 bytes — a value needing a 4th byte throws
/// `CorruptedFrameException("length wider than 21-bit")` — and a zero length is
/// rejected (`"Frame length cannot be zero"`). The result is therefore in
/// `1..=2^21-1` (2,097,151), matching the protocol's frame cap.
pub fn read_frame_length<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut value = 0i32;
    for shift in 0..MAX_VARINT21_BYTES {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        let current = byte[0];
        value |= ((current & 0x7F) as i32) << (shift * 7);
        if !has_varint_continuation_bit(current) {
            if value == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Frame length cannot be zero",
                ));
            }
            return Ok(value);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "length wider than 21-bit",
    ))
}

pub fn decode_varint21_frame<L: BandwidthSampleLogger>(
    input: &[u8],
    monitor: Option<&BandwidthDebugMonitor<L>>,
) -> io::Result<Option<(Vec<u8>, usize)>> {
    let mut value = 0i32;
    for index in 0..MAX_VARINT21_BYTES {
        let Some(&byte) = input.get(index) else {
            return Ok(None);
        };
        value |= ((byte & 0x7F) as i32) << (index * 7);
        if !has_varint_continuation_bit(byte) {
            if value == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Frame length cannot be zero",
                ));
            }
            let header_len = index + 1;
            let body_len = value as usize;
            let total_len = header_len + body_len;
            if input.len() < total_len {
                return Ok(None);
            }
            if let Some(monitor) = monitor {
                monitor.on_receive(total_len as i32);
            }
            return Ok(Some((input[header_len..total_len].to_vec(), total_len)));
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "length wider than 21-bit",
    ))
}

pub fn prepend_varint21_frame(payload: &[u8]) -> io::Result<Vec<u8>> {
    let header_len = var_i32_len(payload.len() as i32);
    if header_len > MAX_VARINT21_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Packet too large: size {} is over 8", payload.len()),
        ));
    }
    let mut output = Vec::with_capacity(header_len + payload.len());
    write_var_i32(&mut output, payload.len() as i32)?;
    output.extend_from_slice(payload);
    Ok(output)
}

pub fn write_var_i32<W: Write + ?Sized>(writer: &mut W, mut value: i32) -> io::Result<()> {
    loop {
        if (value & !0x7F) == 0 {
            writer.write_all(&[value as u8])?;
            return Ok(());
        }

        writer.write_all(&[((value & 0x7F) | 0x80) as u8])?;
        value = ((value as u32) >> 7) as i32;
    }
}

pub fn read_var_i64<R: Read + ?Sized>(reader: &mut R) -> io::Result<i64> {
    let mut value = 0i64;
    let mut bytes = 0;

    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        let current = byte[0];
        if bytes == MAX_VAR_I64_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarLong is too big",
            ));
        }
        value |= ((current & 0x7F) as i64) << (bytes * 7);
        bytes += 1;

        if !has_varint_continuation_bit(current) {
            return Ok(value);
        }
    }
}

pub fn write_var_i64<W: Write + ?Sized>(writer: &mut W, mut value: i64) -> io::Result<()> {
    loop {
        if (value & !0x7F) == 0 {
            writer.write_all(&[value as u8])?;
            return Ok(());
        }

        writer.write_all(&[((value & 0x7F) | 0x80) as u8])?;
        value = ((value as u64) >> 7) as i64;
    }
}

pub fn encode_var_i32(value: i32) -> Vec<u8> {
    let mut out = Vec::new();
    let mut value = value;
    loop {
        if (value & !0x7F) == 0 {
            out.push(value as u8);
            return out;
        }
        out.push(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

pub fn encode_var_i64(value: i64) -> Vec<u8> {
    let mut out = Vec::new();
    let mut value = value;
    loop {
        if (value & !0x7F) == 0 {
            out.push(value as u8);
            return out;
        }
        out.push(((value & 0x7F) | 0x80) as u8);
        value = ((value as u64) >> 7) as i64;
    }
}

pub fn var_i32_len(value: i32) -> usize {
    for i in 1..MAX_VAR_I32_SIZE {
        if (value & (-1i32 << (i * 7))) == 0 {
            return i;
        }
    }
    MAX_VAR_I32_SIZE
}

pub fn var_i64_len(value: i64) -> usize {
    for i in 1..MAX_VAR_I64_SIZE {
        if (value & (-1i64 << (i * 7))) == 0 {
            return i;
        }
    }
    MAX_VAR_I64_SIZE
}

#[cfg(test)]
mod tests {
    use super::{
        decode_varint21_frame, encode_var_i32, encode_var_i64, has_varint_continuation_bit,
        prepend_varint21_frame, read_frame_length, read_var_i32, read_var_i64, var_i32_len,
        var_i64_len, write_var_i32,
    };
    use crate::network::bandwidth::{BandwidthDebugMonitor, BandwidthSampleLogger};
    use std::io::Cursor;
    use std::sync::Mutex;

    #[derive(Default)]
    struct TestLogger {
        samples: Mutex<Vec<i64>>,
    }

    impl BandwidthSampleLogger for TestLogger {
        fn log_sample(&self, sample: i64) {
            self.samples.lock().unwrap().push(sample);
        }
    }

    #[test]
    fn varint_and_varlong_match_java_constants_and_continuation_bits() {
        const VAR_INT_JAVA: &str =
            include_str!("../../../decompiled-server-26.1.2/net/minecraft/network/VarInt.java");
        const VAR_LONG_JAVA: &str =
            include_str!("../../../decompiled-server-26.1.2/net/minecraft/network/VarLong.java");

        for sentinel in [
            "public static final int MAX_VARINT_SIZE = 5;",
            "private static final int DATA_BITS_MASK = 127;",
            "private static final int CONTINUATION_BIT_MASK = 128;",
            "return (in & 128) == 128;",
            "throw new RuntimeException(\"VarInt too big\");",
            "value >>>= 7;",
        ] {
            assert!(
                VAR_INT_JAVA.contains(sentinel),
                "missing VarInt sentinel {sentinel}"
            );
        }
        for sentinel in [
            "private static final int MAX_VARLONG_SIZE = 10;",
            "private static final int DATA_BITS_MASK = 127;",
            "private static final int CONTINUATION_BIT_MASK = 128;",
            "return (in & 128) == 128;",
            "throw new RuntimeException(\"VarLong too big\");",
            "value >>>= 7;",
        ] {
            assert!(
                VAR_LONG_JAVA.contains(sentinel),
                "missing VarLong sentinel {sentinel}"
            );
        }

        assert_eq!(super::MAX_VAR_I32_SIZE, 5);
        assert_eq!(super::MAX_VAR_I64_SIZE, 10);
        assert!(has_varint_continuation_bit(0x80));
        assert!(has_varint_continuation_bit(0xFF));
        assert!(!has_varint_continuation_bit(0x7F));
    }

    #[test]
    fn encodes_vanilla_varint_examples() {
        assert_eq!(encode_var_i32(0), [0x00]);
        assert_eq!(encode_var_i32(1), [0x01]);
        assert_eq!(encode_var_i32(127), [0x7F]);
        assert_eq!(encode_var_i32(128), [0x80, 0x01]);
        assert_eq!(encode_var_i32(255), [0xFF, 0x01]);
        assert_eq!(encode_var_i32(2147483647), [0xFF, 0xFF, 0xFF, 0xFF, 0x07]);
        assert_eq!(encode_var_i32(-1), [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        assert_eq!(encode_var_i32(i32::MIN), [0x80, 0x80, 0x80, 0x80, 0x08]);
    }

    #[test]
    fn round_trips_representative_values() {
        for value in [0, 1, 2, 127, 128, 255, 25565, 775, i32::MAX, i32::MIN, -1] {
            let bytes = encode_var_i32(value);
            let decoded = read_var_i32(&mut Cursor::new(bytes)).unwrap();
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn varint_byte_size_matches_java_boundaries() {
        let cases = [
            (0, 1),
            (1, 1),
            (127, 1),
            (128, 2),
            (16_383, 2),
            (16_384, 3),
            (2_097_151, 3),
            (2_097_152, 4),
            (268_435_455, 4),
            (268_435_456, 5),
            (i32::MAX, 5),
            (i32::MIN, 5),
            (-1, 5),
        ];
        for (value, expected) in cases {
            assert_eq!(var_i32_len(value), expected);
            assert_eq!(encode_var_i32(value).len(), expected);
        }
    }

    #[test]
    fn rejects_oversized_varints_like_java() {
        let mut too_big = Cursor::new([0x80, 0x80, 0x80, 0x80, 0x80, 0x00]);
        let error = read_var_i32(&mut too_big).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(error.to_string(), "VarInt is too big");

        let mut truncated = Cursor::new([0x80, 0x80, 0x80, 0x80, 0x80]);
        assert_eq!(
            read_var_i32(&mut truncated).unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn frame_length_matches_varint21_frame_decoder() {
        const VARINT21_FRAME_DECODER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/Varint21FrameDecoder.java"
        );

        for sentinel in [
            "private static final int MAX_VARINT21_BYTES = 3;",
            "throw new CorruptedFrameException(\"length wider than 21-bit\");",
            "throw new CorruptedFrameException(\"Frame length cannot be zero\");",
            "this.monitor.onReceive(length + VarInt.getByteSize(length));",
            "out.add(in.readBytes(length));",
        ] {
            assert!(
                VARINT21_FRAME_DECODER_JAVA.contains(sentinel),
                "missing Varint21FrameDecoder sentinel {sentinel}"
            );
        }

        // Valid 1-byte and the 3-byte 21-bit maximum (2^21 - 1).
        assert_eq!(read_frame_length(&mut Cursor::new([0x05])).unwrap(), 5);
        let mut max = Vec::new();
        write_var_i32(&mut max, (1 << 21) - 1).unwrap();
        assert_eq!(max.len(), 3);
        assert_eq!(
            read_frame_length(&mut Cursor::new(max)).unwrap(),
            (1 << 21) - 1
        );

        // Zero length is rejected (Java "Frame length cannot be zero").
        let zero_err = read_frame_length(&mut Cursor::new([0x00])).unwrap_err();
        assert_eq!(zero_err.kind(), std::io::ErrorKind::InvalidData);
        assert!(zero_err.to_string().contains("zero"));

        // A length needing a 4th byte (2^21) is rejected ("length wider than 21-bit").
        let mut wide = Vec::new();
        write_var_i32(&mut wide, 1 << 21).unwrap();
        assert_eq!(wide.len(), 4);
        let wide_err = read_frame_length(&mut Cursor::new(wide)).unwrap_err();
        assert_eq!(wide_err.kind(), std::io::ErrorKind::InvalidData);
        assert!(wide_err.to_string().contains("21-bit"));

        assert_eq!(
            decode_varint21_frame::<TestLogger>(&[0x05, 1, 2], None).unwrap(),
            None
        );

        let monitor = BandwidthDebugMonitor::new(TestLogger::default());
        let decoded = decode_varint21_frame(&[0x03, 9, 8, 7, 6], Some(&monitor)).unwrap();
        assert_eq!(decoded, Some((vec![9, 8, 7], 4)));
        assert_eq!(monitor.pending_bytes_received(), 4);
    }

    #[test]
    fn varint21_length_field_prepender_matches_java() {
        const VARINT21_PREPENDER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/Varint21LengthFieldPrepender.java"
        );

        for sentinel in [
            "public static final int MAX_VARINT21_BYTES = 3;",
            "int bodyLength = msg.readableBytes();",
            "int headerLength = VarInt.getByteSize(bodyLength);",
            "throw new EncoderException(\"Packet too large: size \" + bodyLength + \" is over 8\");",
            "VarInt.write(out, bodyLength);",
            "out.writeBytes(msg, msg.readerIndex(), bodyLength);",
        ] {
            assert!(
                VARINT21_PREPENDER_JAVA.contains(sentinel),
                "missing Varint21LengthFieldPrepender sentinel {sentinel}"
            );
        }

        assert_eq!(
            prepend_varint21_frame(&[1, 2, 3]).unwrap(),
            vec![3, 1, 2, 3]
        );
        let max = vec![0; (1 << 21) - 1];
        let encoded = prepend_varint21_frame(&max).unwrap();
        assert_eq!(&encoded[..3], &[0xFF, 0xFF, 0x7F]);
        assert_eq!(encoded.len(), max.len() + 3);
        let too_large = vec![0; 1 << 21];
        let error = prepend_varint21_frame(&too_large).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("Packet too large"));
    }

    #[test]
    fn encodes_vanilla_varlong_examples() {
        assert_eq!(encode_var_i64(0), [0x00]);
        assert_eq!(encode_var_i64(1), [0x01]);
        assert_eq!(encode_var_i64(127), [0x7F]);
        assert_eq!(encode_var_i64(128), [0x80, 0x01]);
        assert_eq!(
            encode_var_i64(9223372036854775807),
            [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        assert_eq!(
            encode_var_i64(-1),
            [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
        );
        assert_eq!(
            encode_var_i64(i64::MIN),
            [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]
        );
    }

    #[test]
    fn round_trips_representative_varlong_values() {
        for value in [0, 1, 127, 128, 255, 25565, 775, i64::MAX, i64::MIN, -1] {
            let bytes = encode_var_i64(value);
            let decoded = read_var_i64(&mut Cursor::new(bytes)).unwrap();
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn varlong_byte_size_matches_java_boundaries() {
        let cases = [
            (0, 1),
            (127, 1),
            (128, 2),
            (16_383, 2),
            (16_384, 3),
            (2_097_151, 3),
            (2_097_152, 4),
            (268_435_455, 4),
            (268_435_456, 5),
            (34_359_738_367, 5),
            (34_359_738_368, 6),
            (4_398_046_511_103, 6),
            (4_398_046_511_104, 7),
            (562_949_953_421_311, 7),
            (562_949_953_421_312, 8),
            (72_057_594_037_927_935, 8),
            (72_057_594_037_927_936, 9),
            (9_223_372_036_854_775_807, 9),
            (i64::MIN, 10),
            (-1, 10),
        ];
        for (value, expected) in cases {
            assert_eq!(var_i64_len(value), expected);
            assert_eq!(encode_var_i64(value).len(), expected);
        }
    }

    #[test]
    fn rejects_oversized_varlongs_like_java() {
        let mut too_big = Cursor::new([0x80; 11]);
        let error = read_var_i64(&mut too_big).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(error.to_string(), "VarLong is too big");

        let mut truncated = Cursor::new([0x80; 10]);
        assert_eq!(
            read_var_i64(&mut truncated).unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }
}
