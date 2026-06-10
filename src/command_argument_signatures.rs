use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::ops::Range;

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

impl SignableCommandModel {
    pub fn of(command: &SignableParsedCommandModel) -> Self {
        let mut arguments = Vec::new();
        command.visit_arguments(true, |argument| {
            if argument.is_signed {
                if let Some(range) = &argument.value_range {
                    if let Some(value) = java_string_range(&command.command, range.clone()) {
                        arguments.push(SignableArgumentModel {
                            name: argument.name.clone(),
                            value,
                        });
                    }
                }
            }
        });
        Self { arguments }
    }

    pub fn has_signable_arguments(command: &SignableParsedCommandModel) -> bool {
        !Self::of(command).arguments.is_empty()
    }

    pub fn get_argument(&self, name: &str) -> Option<&SignableArgumentModel> {
        self.arguments.iter().find(|argument| argument.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignableArgumentModel {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignableParsedCommandModel {
    pub command: String,
    pub root: SignableParseContextModel,
}

impl SignableParsedCommandModel {
    fn visit_arguments(
        &self,
        reject_root_redirects: bool,
        mut visitor: impl FnMut(VisitedArgument),
    ) {
        let root_id = self.root.root_id;
        let mut context = &self.root;
        loop {
            context.visit_node_arguments(&mut visitor);
            let Some(child) = &context.child else {
                break;
            };
            if reject_root_redirects && child.root_id != root_id {
                break;
            }
            context = child;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignableParseContextModel {
    pub root_id: usize,
    pub nodes: Vec<ParsedCommandNodeModel>,
    pub arguments: BTreeMap<String, ParsedArgumentRangeModel>,
    pub child: Option<Box<SignableParseContextModel>>,
}

impl SignableParseContextModel {
    fn visit_node_arguments(&self, visitor: &mut impl FnMut(VisitedArgument)) {
        for node in &self.nodes {
            if node.kind == ParsedCommandNodeKind::Argument {
                visitor(VisitedArgument {
                    name: node.name.clone(),
                    is_signed: node.is_signed,
                    value_range: self
                        .arguments
                        .get(&node.name)
                        .map(|value| value.range.clone()),
                });
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommandNodeModel {
    pub name: String,
    pub kind: ParsedCommandNodeKind,
    pub is_signed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedCommandNodeKind {
    Literal,
    Argument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArgumentRangeModel {
    pub range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VisitedArgument {
    name: String,
    is_signed: bool,
    value_range: Option<Range<usize>>,
}

fn java_string_range(command: &str, range: Range<usize>) -> Option<String> {
    let start = byte_index_for_utf16_index(command, range.start)?;
    let end = byte_index_for_utf16_index(command, range.end)?;
    command.get(start..end).map(ToString::to_string)
}

fn byte_index_for_utf16_index(input: &str, index: usize) -> Option<usize> {
    let mut utf16_position = 0;
    for (byte_index, ch) in input.char_indices() {
        if utf16_position == index {
            return Some(byte_index);
        }
        utf16_position += ch.len_utf16();
    }
    (utf16_position == index).then_some(input.len())
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

    fn argument_node(name: &str, is_signed: bool) -> ParsedCommandNodeModel {
        ParsedCommandNodeModel {
            name: name.to_string(),
            kind: ParsedCommandNodeKind::Argument,
            is_signed,
        }
    }

    fn literal_node(name: &str) -> ParsedCommandNodeModel {
        ParsedCommandNodeModel {
            name: name.to_string(),
            kind: ParsedCommandNodeKind::Literal,
            is_signed: false,
        }
    }

    fn context(
        root_id: usize,
        nodes: Vec<ParsedCommandNodeModel>,
        arguments: impl IntoIterator<Item = (&'static str, Range<usize>)>,
    ) -> SignableParseContextModel {
        SignableParseContextModel {
            root_id,
            nodes,
            arguments: arguments
                .into_iter()
                .map(|(name, range)| (name.to_string(), ParsedArgumentRangeModel { range }))
                .collect(),
            child: None,
        }
    }

    #[test]
    fn signable_command_of_visits_nodes_and_keeps_only_signed_non_null_arguments() {
        let parsed = SignableParsedCommandModel {
            command: "msg Steve hello there".to_string(),
            root: context(
                1,
                vec![
                    literal_node("msg"),
                    argument_node("target", false),
                    argument_node("message", true),
                    argument_node("missing", true),
                ],
                [("target", 4..9), ("message", 10..21)],
            ),
        };

        let command = SignableCommandModel::of(&parsed);

        assert_eq!(
            command.arguments,
            [SignableArgumentModel {
                name: "message".to_string(),
                value: "hello there".to_string(),
            }]
        );
        assert!(SignableCommandModel::has_signable_arguments(&parsed));
        assert_eq!(command.get_argument("message"), command.arguments.first());
        assert_eq!(command.get_argument("target"), None);
    }

    #[test]
    fn signable_command_follows_same_root_children_but_stops_at_root_redirects() {
        let mut root = context(1, vec![argument_node("first", true)], [("first", 4..7)]);
        let mut same_root_child =
            context(1, vec![argument_node("second", true)], [("second", 8..11)]);
        same_root_child.child = Some(Box::new(context(
            2,
            vec![argument_node("redirected", true)],
            [("redirected", 12..15)],
        )));
        root.child = Some(Box::new(same_root_child));
        let parsed = SignableParsedCommandModel {
            command: "run one two bad".to_string(),
            root,
        };

        let command = SignableCommandModel::of(&parsed);

        assert_eq!(
            command.arguments,
            [
                SignableArgumentModel {
                    name: "first".to_string(),
                    value: "one".to_string(),
                },
                SignableArgumentModel {
                    name: "second".to_string(),
                    value: "two".to_string(),
                },
            ]
        );
        assert_eq!(command.get_argument("redirected"), None);
    }

    #[test]
    fn signable_command_get_argument_returns_the_first_matching_name() {
        let command = SignableCommandModel {
            arguments: vec![
                SignableArgumentModel {
                    name: "message".to_string(),
                    value: "first".to_string(),
                },
                SignableArgumentModel {
                    name: "message".to_string(),
                    value: "second".to_string(),
                },
            ],
        };

        assert_eq!(
            command
                .get_argument("message")
                .map(|argument| argument.value.as_str()),
            Some("first")
        );
    }

    #[test]
    fn signable_command_ranges_use_java_utf16_positions() {
        let parsed = SignableParsedCommandModel {
            command: "say hi 🙂".to_string(),
            root: context(1, vec![argument_node("message", true)], [("message", 4..9)]),
        };

        let command = SignableCommandModel::of(&parsed);

        assert_eq!(command.arguments[0].value, "hi 🙂");
    }
}
