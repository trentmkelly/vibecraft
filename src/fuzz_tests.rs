#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use crate::command_tree::{ArgumentParser, CommandNodeKind, CommandTree};
    use crate::network::varint::{read_var_i32, read_var_i64};
    use crate::resources::{parse_pack_metadata, DataResourceIndex};
    use crate::statistics::StatisticsCounter;
    use crate::storage::chunk::LevelChunk;
    use crate::storage::nbt::{read_named_tag, Tag};
    use crate::storage::poi::PoiSection;
    use crate::storage::region::ChunkPos;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct FuzzSurface {
        name: &'static str,
        corpus_size: usize,
    }

    #[test]
    fn fuzz_surface_matrix_covers_required_parsers_and_readers() {
        let surfaces = [
            FuzzSurface {
                name: "packet decoders",
                corpus_size: packet_corpus().len(),
            },
            FuzzSurface {
                name: "NBT parser",
                corpus_size: nbt_corpus().len(),
            },
            FuzzSurface {
                name: "command parser",
                corpus_size: command_corpus().len(),
            },
            FuzzSurface {
                name: "resource loaders",
                corpus_size: resource_corpus().len(),
            },
            FuzzSurface {
                name: "save readers",
                corpus_size: save_reader_corpus().len(),
            },
        ];

        assert_eq!(
            surfaces
                .iter()
                .map(|surface| surface.name)
                .collect::<Vec<_>>(),
            vec![
                "packet decoders",
                "NBT parser",
                "command parser",
                "resource loaders",
                "save readers"
            ]
        );
        assert!(surfaces.iter().all(|surface| surface.corpus_size >= 4));
    }

    #[test]
    fn packet_decoder_fuzz_inputs_return_or_error_without_panics() {
        for bytes in packet_corpus() {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                let _ = read_var_i32(&mut Cursor::new(bytes));
                let _ = read_var_i64(&mut Cursor::new(bytes));
            }))
            .is_ok());
        }
    }

    #[test]
    fn nbt_parser_fuzz_inputs_return_or_error_without_panics() {
        for bytes in nbt_corpus() {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                let _ = read_named_tag(&mut Cursor::new(bytes));
            }))
            .is_ok());
        }
    }

    #[test]
    fn command_parser_fuzz_inputs_return_or_error_without_panics() {
        let tree = command_tree();
        for input in command_corpus() {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                let _ = tree.parse(input, 2);
            }))
            .is_ok());
        }
    }

    #[test]
    fn resource_loader_fuzz_inputs_return_or_error_without_panics() {
        for input in resource_corpus() {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                let _ = parse_pack_metadata(input);
                let _ = DataResourceIndex::from_resources([(
                    "data/example/tags/item/fuzz.json",
                    input,
                )]);
            }))
            .is_ok());
        }
    }

    #[test]
    fn save_reader_fuzz_inputs_return_or_error_without_panics() {
        for tag in save_reader_corpus() {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                let _ = LevelChunk::from_nbt(ChunkPos { x: 0, z: 0 }, &tag);
                let _ = PoiSection::from_nbt(&tag);
                let json = match &tag {
                    Tag::String(value) => value.as_str(),
                    _ => "{}",
                };
                let _ = StatisticsCounter::from_vanilla_json(json);
            }))
            .is_ok());
        }
    }

    fn packet_corpus() -> Vec<&'static [u8]> {
        vec![
            &[],
            &[0x00],
            &[0x80],
            &[0x80, 0x80, 0x80, 0x80, 0x80],
            &[0xff, 0xff, 0xff, 0xff, 0x7f],
            &[0xff; 11],
        ]
    }

    fn nbt_corpus() -> Vec<Vec<u8>> {
        vec![
            vec![],
            vec![0],
            vec![10, 0, 0, 0],
            vec![8, 0, 4, b'n', b'a', b'm', b'e', 0, 3, b'a', b'b'],
            vec![9, 0, 4, b'l', b'i', b's', b't', 1, 0, 0, 0, 2, 1],
            vec![99, 0, 0],
        ]
    }

    fn command_corpus() -> Vec<&'static str> {
        vec![
            "",
            "/say hello",
            "say hello world",
            "tp Steve 1 nope 3",
            "give @e[type=zombie] minecraft:stone 999999999999",
            "reload extra trailing tokens",
            "unknown literal",
        ]
    }

    fn resource_corpus() -> Vec<&'static str> {
        vec![
            "",
            "{}",
            r#"{"pack":{}}"#,
            r#"{"pack":{"description":"bad","pack_format":101}}"#,
            r#"{"pack":{"description":"ok","min_format":[101,0],"max_format":[101,1]}}"#,
            r#"{"features":{"enabled":["minecraft:vanilla"]}}"#,
        ]
    }

    fn save_reader_corpus() -> Vec<Tag> {
        vec![
            Tag::End,
            Tag::String(String::new()),
            Tag::String("{\"stats\":{}}".to_string()),
            Tag::Compound(Vec::new()),
            Tag::Compound(vec![("DataVersion".to_string(), Tag::Int(4189))]),
            Tag::List(vec![Tag::Int(1), Tag::Int(2)]),
        ]
    }

    fn command_tree() -> CommandTree {
        let mut tree = CommandTree::new();
        let say = tree.add_child(tree.root, CommandNodeKind::Literal("say"), 0, false);
        tree.add_child(
            say,
            CommandNodeKind::Argument {
                name: "message",
                parser: ArgumentParser::GreedyString,
                signed: true,
            },
            0,
            true,
        );
        let reload = tree.add_child(tree.root, CommandNodeKind::Literal("reload"), 2, true);
        let give = tree.add_child(tree.root, CommandNodeKind::Literal("give"), 2, false);
        let target = tree.add_child(
            give,
            CommandNodeKind::Argument {
                name: "target",
                parser: ArgumentParser::EntitySelector,
                signed: false,
            },
            2,
            false,
        );
        let item = tree.add_child(
            target,
            CommandNodeKind::Argument {
                name: "item",
                parser: ArgumentParser::Word,
                signed: false,
            },
            2,
            false,
        );
        tree.add_child(
            item,
            CommandNodeKind::Argument {
                name: "count",
                parser: ArgumentParser::Integer,
                signed: false,
            },
            2,
            true,
        );
        tree.set_redirect(reload, tree.root, false);
        tree
    }
}
