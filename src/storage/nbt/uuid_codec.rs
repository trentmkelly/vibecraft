//! UUIDUtil.CODEC: four signed words in most-significant-first order.
use super::{
    nbt_ops::{NbtOpsModel, NbtOpsResult},
    Tag,
};
use crate::network::codec::Uuid;

pub fn uuid_from_nbt(tag: &Tag) -> Result<Uuid, String> {
    let words = match NbtOpsModel::INSTANCE.get_int_stream(tag) {
        NbtOpsResult::Success(words) => words,
        NbtOpsResult::Error { message, .. } => return Err(message),
    };
    if words.len() != 4 {
        return Err("Input is not a list of 4 ints".to_owned());
    }
    let mut bytes = [0; 16];
    for (index, word) in words.iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    Ok(Uuid(bytes))
}

pub fn uuid_to_nbt(uuid: Uuid) -> Tag {
    Tag::IntArray(
        uuid.0
            .chunks_exact(4)
            .map(|bytes| i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_codec_preserves_signed_words_and_byte_order() {
        let words = vec![0x12345678, -1, i32::MIN, 0x76543210];
        let tag = Tag::IntArray(words);
        let uuid = uuid_from_nbt(&tag).unwrap();
        assert_eq!(
            uuid.0,
            [
                0x12, 0x34, 0x56, 0x78, 0xff, 0xff, 0xff, 0xff, 0x80, 0, 0, 0, 0x76, 0x54, 0x32,
                0x10
            ]
        );
        assert_eq!(uuid_to_nbt(uuid), tag);
        let numeric = Tag::List(vec![
            Tag::Double(-1.9),
            Tag::Long(4_294_967_297),
            Tag::Short(2),
            Tag::Byte(3),
        ]);
        assert_eq!(
            uuid_to_nbt(uuid_from_nbt(&numeric).unwrap()),
            Tag::IntArray(vec![-1, 1, 2, 3])
        );
    }

    #[test]
    fn uuid_codec_rejects_wrong_lengths_and_string_alternatives() {
        for size in [0, 3, 5] {
            assert_eq!(
                uuid_from_nbt(&Tag::IntArray(vec![0; size])).unwrap_err(),
                "Input is not a list of 4 ints"
            );
        }
        assert!(uuid_from_nbt(&Tag::String(
            "00000000-0000-0000-0000-000000000000".to_owned()
        ))
        .is_err());
    }
}
