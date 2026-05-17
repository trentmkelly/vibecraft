use std::io::{self, Read, Write};

pub fn read_var_i32<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut value = 0i32;
    let mut position = 0;

    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        let current = byte[0];
        value |= ((current & 0x7F) as i32) << position;

        if current & 0x80 == 0 {
            return Ok(value);
        }

        position += 7;
        if position >= 35 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarInt is too big",
            ));
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
    let mut position = 0;

    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        let current = byte[0];
        value |= ((current & 0x7F) as i64) << position;

        if current & 0x80 == 0 {
            return Ok(value);
        }

        position += 7;
        if position >= 70 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarLong is too big",
            ));
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

#[cfg(test)]
mod tests {
    use super::{encode_var_i32, encode_var_i64, read_var_i32, read_var_i64};
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
    }

    #[test]
    fn round_trips_representative_values() {
        for value in [0, 1, 2, 127, 128, 255, 25565, 775, i32::MAX, -1] {
            let bytes = encode_var_i32(value);
            let decoded = read_var_i32(&mut Cursor::new(bytes)).unwrap();
            assert_eq!(decoded, value);
        }
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
    }

    #[test]
    fn round_trips_representative_varlong_values() {
        for value in [0, 1, 127, 128, 255, 25565, 775, i64::MAX, -1] {
            let bytes = encode_var_i64(value);
            let decoded = read_var_i64(&mut Cursor::new(bytes)).unwrap();
            assert_eq!(decoded, value);
        }
    }
}
