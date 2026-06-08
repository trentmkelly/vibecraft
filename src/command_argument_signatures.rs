use std::io::{self, Read, Write};

use crate::network::codec::{read_string, write_string};
use crate::network::varint::{read_var_i32, write_var_i32};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentSignaturesModel {
    pub entries: Vec<ArgumentSignatureEntryModel>,
}

impl ArgumentSignaturesModel {
    pub const MAX_ARGUMENT_COUNT: usize = 8;

    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let count = read_var_i32(reader)?;
        if count < 0 || count as usize > Self::MAX_ARGUMENT_COUNT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "argument signature count out of range",
            ));
        }
        let mut entries = Vec::with_capacity(count as usize);
        for _ in 0..count {
            entries.push(ArgumentSignatureEntryModel::read(reader)?);
        }
        Ok(Self { entries })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.entries.len() > Self::MAX_ARGUMENT_COUNT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many argument signatures",
            ));
        }
        write_var_i32(writer, self.entries.len() as i32)?;
        for entry in &self.entries {
            entry.write(writer)?;
        }
        Ok(())
    }

    pub fn sign_command(
        command: &SignableCommandModel,
        signer: impl Fn(&str) -> Option<MessageSignatureModel>,
    ) -> Self {
        let entries = command
            .arguments
            .iter()
            .filter_map(|argument| {
                signer(&argument.value).map(|signature| ArgumentSignatureEntryModel {
                    name: argument.name.clone(),
                    signature,
                })
            })
            .collect();
        Self { entries }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentSignatureEntryModel {
    pub name: String,
    pub signature: MessageSignatureModel,
}

impl ArgumentSignatureEntryModel {
    pub const MAX_ARGUMENT_NAME_LENGTH: usize = 16;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            name: read_string(reader, Self::MAX_ARGUMENT_NAME_LENGTH)?,
            signature: MessageSignatureModel::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.name, Self::MAX_ARGUMENT_NAME_LENGTH)?;
        self.signature.write(writer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSignatureModel(pub [u8; MessageSignatureModel::BYTES]);

impl MessageSignatureModel {
    pub const BYTES: usize = 256;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut bytes = [0; Self::BYTES];
        reader.read_exact(&mut bytes)?;
        Ok(Self(bytes))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignableCommandModel {
    pub arguments: Vec<SignableArgumentModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignableArgumentModel {
    pub name: String,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(byte: u8) -> MessageSignatureModel {
        MessageSignatureModel([byte; MessageSignatureModel::BYTES])
    }

    fn entry(name: &str, byte: u8) -> ArgumentSignatureEntryModel {
        ArgumentSignatureEntryModel {
            name: name.to_string(),
            signature: signature(byte),
        }
    }

    #[test]
    fn empty_constant_matches_java_empty_list() {
        assert_eq!(ArgumentSignaturesModel::empty().entries, []);
    }

    #[test]
    fn write_and_read_use_varint_count_name_utf_and_256_byte_signature() {
        let signatures = ArgumentSignaturesModel {
            entries: vec![entry("message", 8)],
        };
        let mut bytes = Vec::new();

        signatures.write(&mut bytes).unwrap();

        let mut expected = Vec::new();
        write_var_i32(&mut expected, 1).unwrap();
        write_string(
            &mut expected,
            "message",
            ArgumentSignatureEntryModel::MAX_ARGUMENT_NAME_LENGTH,
        )
        .unwrap();
        expected.extend_from_slice(&[8; MessageSignatureModel::BYTES]);
        assert_eq!(bytes, expected);
        assert_eq!(
            ArgumentSignaturesModel::read(&mut bytes.as_slice()).unwrap(),
            signatures
        );
    }

    #[test]
    fn read_rejects_more_than_eight_entries_before_reading_entries() {
        let mut bytes = Vec::new();
        write_var_i32(&mut bytes, 9).unwrap();

        assert!(ArgumentSignaturesModel::read(&mut bytes.as_slice()).is_err());
    }

    #[test]
    fn write_rejects_more_than_eight_entries() {
        let signatures = ArgumentSignaturesModel {
            entries: vec![entry("a", 1); 9],
        };

        assert!(signatures.write(&mut Vec::new()).is_err());
    }

    #[test]
    fn entry_read_and_write_enforce_sixteen_character_name_limit() {
        let exact_limit = entry("1234567890123456", 2);
        assert!(exact_limit.write(&mut Vec::new()).is_ok());

        let overlong = entry("12345678901234567", 2);
        assert!(overlong.write(&mut Vec::new()).is_err());

        let mut bytes = Vec::new();
        write_string(&mut bytes, "12345678901234567", 32767).unwrap();
        bytes.extend_from_slice(&[2; MessageSignatureModel::BYTES]);
        assert!(ArgumentSignatureEntryModel::read(&mut bytes.as_slice()).is_err());
    }

    #[test]
    fn sign_command_signs_argument_values_and_filters_null_signatures() {
        let command = SignableCommandModel {
            arguments: vec![
                SignableArgumentModel {
                    name: "target".to_string(),
                    value: "Steve".to_string(),
                },
                SignableArgumentModel {
                    name: "message".to_string(),
                    value: "hello".to_string(),
                },
            ],
        };

        let signed = ArgumentSignaturesModel::sign_command(&command, |value| {
            if value == "hello" {
                Some(signature(7))
            } else {
                None
            }
        });

        assert_eq!(signed.entries, [entry("message", 7)]);
    }

    #[test]
    fn sign_command_preserves_command_argument_order() {
        let command = SignableCommandModel {
            arguments: vec![
                SignableArgumentModel {
                    name: "first".to_string(),
                    value: "one".to_string(),
                },
                SignableArgumentModel {
                    name: "second".to_string(),
                    value: "two".to_string(),
                },
            ],
        };

        let signed = ArgumentSignaturesModel::sign_command(&command, |value| {
            Some(signature(value.as_bytes()[0]))
        });

        assert_eq!(
            signed.entries,
            [entry("first", b'o'), entry("second", b't')]
        );
    }
}
