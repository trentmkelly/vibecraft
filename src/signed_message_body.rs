use std::io::{self, Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::chat_trust::{
    LastSeenMessages, MessageSignature, MessageSignatureCache, PackedLastSeenMessages,
    PackedMessageSignatureModel,
};
use crate::network::codec::{read_string, write_string};
use crate::network::varint::{read_var_i32, write_var_i32};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageBody {
    pub content: String,
    pub epoch_seconds: i64,
    pub epoch_millis: i64,
    pub salt: i64,
    pub last_seen: LastSeenMessages,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedSignedMessageBody {
    pub content: String,
    pub epoch_millis: i64,
    pub salt: i64,
    pub last_seen: PackedLastSeenMessages,
}

impl SignedMessageBody {
    pub const MAX_CONTENT_LENGTH: usize = 256;

    pub fn unsigned(content: impl Into<String>) -> Self {
        let epoch_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        Self::unsigned_at(content, epoch_millis)
    }

    pub fn unsigned_at(content: impl Into<String>, epoch_millis: i64) -> Self {
        Self {
            content: content.into(),
            epoch_seconds: epoch_millis.div_euclid(1000),
            epoch_millis,
            salt: 0,
            last_seen: LastSeenMessages::empty(),
        }
    }

    pub fn update_signature(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.salt.to_be_bytes());
        output.extend_from_slice(&self.epoch_seconds.to_be_bytes());
        let content_bytes = self.content.as_bytes();
        output.extend_from_slice(&(content_bytes.len() as i32).to_be_bytes());
        output.extend_from_slice(content_bytes);
        output.extend_from_slice(&self.last_seen.update_signature_bytes());
    }

    pub fn pack(&self, cache: &MessageSignatureCache) -> PackedSignedMessageBody {
        PackedSignedMessageBody {
            content: self.content.clone(),
            epoch_millis: self.epoch_millis,
            salt: self.salt,
            last_seen: self.last_seen.pack(cache),
        }
    }
}

impl PackedSignedMessageBody {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            content: read_string(reader, SignedMessageBody::MAX_CONTENT_LENGTH)?,
            epoch_millis: read_i64(reader)?,
            salt: read_i64(reader)?,
            last_seen: PackedLastSeenMessages::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.content, SignedMessageBody::MAX_CONTENT_LENGTH)?;
        writer.write_all(&self.epoch_millis.to_be_bytes())?;
        writer.write_all(&self.salt.to_be_bytes())?;
        self.last_seen.write(writer)
    }

    pub fn unpack(&self, cache: &MessageSignatureCache) -> Option<SignedMessageBody> {
        Some(SignedMessageBody {
            content: self.content.clone(),
            epoch_seconds: self.epoch_millis.div_euclid(1000),
            epoch_millis: self.epoch_millis,
            salt: self.salt,
            last_seen: self.last_seen.unpack(cache)?,
        })
    }
}

impl PackedLastSeenMessages {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let count = read_var_i32(reader)?;
        if count < 0 || count as usize > LastSeenMessages::LAST_SEEN_MESSAGES_MAX_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "last seen message count out of range",
            ));
        }
        let mut entries = Vec::with_capacity(count as usize);
        for _ in 0..count {
            entries.push(PackedMessageSignatureModel::read(reader)?);
        }
        Ok(Self { entries })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.entries.len() > LastSeenMessages::LAST_SEEN_MESSAGES_MAX_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many last seen messages",
            ));
        }
        write_var_i32(writer, self.entries.len() as i32)?;
        for entry in &self.entries {
            entry.write(writer)?;
        }
        Ok(())
    }
}

impl PackedMessageSignatureModel {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let id = read_var_i32(reader)? - 1;
        if id == Self::FULL_SIGNATURE {
            Ok(Self::Full(MessageSignature::read(reader)?))
        } else {
            Ok(Self::Id(id))
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Full(signature) => {
                write_var_i32(writer, Self::FULL_SIGNATURE + 1)?;
                signature.write(writer)
            }
            Self::Id(id) => write_var_i32(writer, id + 1),
        }
    }
}

fn read_i64<R: Read>(reader: &mut R) -> io::Result<i64> {
    let mut bytes = [0; 8];
    reader.read_exact(&mut bytes)?;
    Ok(i64::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(byte: u8) -> MessageSignature {
        MessageSignature(vec![byte; MessageSignature::BYTES])
    }

    #[test]
    fn signed_message_body_updates_signature_with_java_field_order() {
        let first = signature(1);
        let second = signature(2);
        let body = SignedMessageBody {
            content: "hi \u{1f642}".to_string(),
            epoch_seconds: 42,
            epoch_millis: 42_999,
            salt: -7,
            last_seen: LastSeenMessages {
                entries: vec![first.clone(), second.clone()],
            },
        };

        let mut bytes = Vec::new();
        body.update_signature(&mut bytes);

        let mut expected = Vec::new();
        expected.extend_from_slice(&(-7_i64).to_be_bytes());
        expected.extend_from_slice(&42_i64.to_be_bytes());
        expected.extend_from_slice(&(body.content.len() as i32).to_be_bytes());
        expected.extend_from_slice(body.content.as_bytes());
        expected.extend_from_slice(&2_i32.to_be_bytes());
        expected.extend_from_slice(first.bytes());
        expected.extend_from_slice(second.bytes());
        assert_eq!(bytes, expected);
    }

    #[test]
    fn signed_message_body_unsigned_uses_zero_salt_empty_last_seen_and_epoch_seconds() {
        let body = SignedMessageBody::unsigned_at("hello", 12_345);

        assert_eq!(body.content, "hello");
        assert_eq!(body.epoch_millis, 12_345);
        assert_eq!(body.epoch_seconds, 12);
        assert_eq!(body.salt, 0);
        assert_eq!(body.last_seen, LastSeenMessages::empty());
    }

    #[test]
    fn packed_signed_message_body_uses_java_wire_order_and_cache_unpacking() {
        let cached = signature(8);
        let full = signature(9);
        let mut cache = MessageSignatureCache::create_default();
        cache.push(cached.clone());
        let body = SignedMessageBody {
            content: "hello".to_string(),
            epoch_seconds: 123,
            epoch_millis: 123_456,
            salt: 77,
            last_seen: LastSeenMessages {
                entries: vec![cached.clone(), full.clone()],
            },
        };
        let packed = body.pack(&cache);

        assert_eq!(
            packed.last_seen.entries,
            vec![
                PackedMessageSignatureModel::Id(0),
                PackedMessageSignatureModel::Full(full.clone()),
            ]
        );
        assert_eq!(packed.unpack(&cache), Some(body));

        let mut bytes = Vec::new();
        packed
            .write(&mut bytes)
            .unwrap_or_else(|err| panic!("{err}"));
        let decoded = PackedSignedMessageBody::read(&mut bytes.as_slice())
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(decoded, packed);
    }

    #[test]
    fn packed_signed_message_body_rejects_java_collection_and_content_limits() {
        let overlong = "x".repeat(SignedMessageBody::MAX_CONTENT_LENGTH + 1);
        let packed = PackedSignedMessageBody {
            content: overlong,
            epoch_millis: 0,
            salt: 0,
            last_seen: PackedLastSeenMessages::empty(),
        };
        assert!(packed.write(&mut Vec::new()).is_err());

        let too_many = PackedLastSeenMessages {
            entries: vec![
                PackedMessageSignatureModel::Id(0);
                LastSeenMessages::LAST_SEEN_MESSAGES_MAX_LENGTH + 1
            ],
        };
        assert!(too_many.write(&mut Vec::new()).is_err());

        let mut bytes = Vec::new();
        write_var_i32(
            &mut bytes,
            (LastSeenMessages::LAST_SEEN_MESSAGES_MAX_LENGTH + 1) as i32,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert!(PackedLastSeenMessages::read(&mut bytes.as_slice()).is_err());
    }
}
