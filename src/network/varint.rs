use std::io::{self, Read, Write};

pub const MAX_VAR_I32_SIZE: usize = 5;
pub const MAX_VAR_I64_SIZE: usize = 10;

pub fn read_var_i32<R: Read>(reader: &mut R) -> io::Result<i32> {
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

        if current & 0x80 == 0 {
            return Ok(value);
        }
    }
}

pub fn write_var_i32<W: Write>(writer: &mut W, mut value: i32) -> io::Result<()> {
    loop {
        if (value & !0x7F) == 0 {
            writer.write_all(&[value as u8])?;
            return Ok(());
        }

        writer.write_all(&[((value & 0x7F) | 0x80) as u8])?;
        value = ((value as u32) >> 7) as i32;
    }
}

pub fn read_var_i64<R: Read>(reader: &mut R) -> io::Result<i64> {
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

        if current & 0x80 == 0 {
            return Ok(value);
        }
    }
}

pub fn write_var_i64<W: Write>(writer: &mut W, mut value: i64) -> io::Result<()> {
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
    write_var_i32(&mut out, value).expect("Vec write cannot fail");
    out
}

pub fn encode_var_i64(value: i64) -> Vec<u8> {
    let mut out = Vec::new();
    write_var_i64(&mut out, value).expect("Vec write cannot fail");
    out
}

/// Matches Java `VarInt.getByteSize`: negative values always occupy five bytes.
pub fn var_i32_len(value: i32) -> usize {
    for i in 1..MAX_VAR_I32_SIZE {
        if (value & (-1i32 << (i * 7))) == 0 {
            return i;
        }
    }
    MAX_VAR_I32_SIZE
}

/// Matches Java `VarLong.getByteSize`: negative values always occupy ten bytes.
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
        encode_var_i32, encode_var_i64, read_var_i32, read_var_i64, var_i32_len, var_i64_len,
    };
    use std::io::Cursor;

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
