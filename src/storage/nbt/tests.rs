use super::accounter::{
    NbtAccounter, NbtAccounterError, DEFAULT_NBT_QUOTA, UNCOMPRESSED_NBT_QUOTA,
};
use super::numeric::NbtNumericValue;
use super::snbt_string::escape_snbt_string_without_quotes;
use super::tag_metadata::{
    byte_array_size_in_bytes, int_array_size_in_bytes, list_size_in_bytes,
    long_array_size_in_bytes, root_parse_action, string_size_in_bytes, tag_type, NbtTagTypeLookup,
    ReportedNbtExceptionModel, RootParseAction, RootVisitResult, TAG_TYPES,
};
use super::{
    parse_snbt, read_gzip_named_tag, read_named_tag, read_named_tag_limited, write_gzip_named_tag,
    write_named_tag, NbtFieldSelector, Tag, DEFAULT_MAX_NBT_DEPTH,
};
use std::io::Cursor;

const NBT_ACCOUNTER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/NbtAccounter.java");
const NBT_EXCEPTION_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/NbtException.java");
const NBT_ACCOUNTER_EXCEPTION_JAVA: &str = vibecraft_java_source!("/net/minecraft/nbt/NbtAccounterException.java");
const NBT_FORMAT_EXCEPTION_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/NbtFormatException.java");
const NBT_PACKAGE_INFO_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/package-info.java");
const REPORTED_NBT_EXCEPTION_JAVA: &str = vibecraft_java_source!("/net/minecraft/nbt/ReportedNbtException.java");
const TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/Tag.java");
const TAG_VISITOR_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/TagVisitor.java");
const TAG_TYPE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/TagType.java");
const TAG_TYPES_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/TagTypes.java");
const END_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/EndTag.java");
const BYTE_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/ByteTag.java");
const SHORT_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/ShortTag.java");
const INT_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/IntTag.java");
const LONG_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/LongTag.java");
const STRING_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/StringTag.java");
const STRING_TAG_VISITOR_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/StringTagVisitor.java");
const COLLECTION_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/CollectionTag.java");
const BYTE_ARRAY_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/ByteArrayTag.java");
const INT_ARRAY_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/IntArrayTag.java");
const LONG_ARRAY_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/LongArrayTag.java");
const LIST_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/ListTag.java");
const NUMERIC_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/NumericTag.java");
const PRIMITIVE_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/PrimitiveTag.java");
const FLOAT_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/FloatTag.java");
const DOUBLE_TAG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/DoubleTag.java");
const SNBT_GRAMMAR_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/nbt/SnbtGrammar.java");

#[test]
fn nbt_accounter_matches_java_quota_depth_and_error_contracts() {
    for sentinel in [
        "public static final int DEFAULT_NBT_QUOTA = 2097152;",
        "public static final int UNCOMPRESSED_NBT_QUOTA = 104857600;",
        "private static final int MAX_STACK_DEPTH = 512;",
        "return new NbtAccounter(quota, 512);",
        "return new NbtAccounter(2097152L, 512);",
        "return new NbtAccounter(104857600L, 512);",
        "return new NbtAccounter(Long.MAX_VALUE, 512);",
        "Tried to account NBT tag with negative size:",
        "Tried to read NBT tag that was too big; tried to allocate:",
        "Tried to read NBT tag with too high complexity, depth > ",
        "NBT-Accounter tried to pop stack-depth at top-level",
    ] {
        assert!(
            NBT_ACCOUNTER_JAVA.contains(sentinel),
            "missing NbtAccounter sentinel {sentinel}"
        );
    }

    assert_eq!(DEFAULT_NBT_QUOTA, 2_097_152);
    assert_eq!(UNCOMPRESSED_NBT_QUOTA, 104_857_600);
    assert_eq!(DEFAULT_MAX_NBT_DEPTH, 512);

    let mut accounter = NbtAccounter::create(10);
    accounter.account_bytes(4).unwrap();
    accounter.account_entries(3, 2).unwrap();
    assert_eq!(accounter.usage(), 10);
    assert_eq!(
        accounter.account_bytes(1).unwrap_err(),
        NbtAccounterError::QuotaExceeded {
            usage: 10,
            size: 1,
            quota: 10
        }
    );
    assert_eq!(
        NbtAccounter::default_quota()
            .account_bytes(-1)
            .unwrap_err()
            .to_string(),
        "Tried to account NBT tag with negative size: -1"
    );

    let mut depth_limited = NbtAccounter::new(100, 1);
    depth_limited.push_depth().unwrap();
    assert_eq!(depth_limited.depth(), 1);
    assert_eq!(
        depth_limited.push_depth().unwrap_err(),
        NbtAccounterError::DepthExceeded { max_depth: 1 }
    );
    depth_limited.pop_depth().unwrap();
    assert_eq!(
        depth_limited.pop_depth().unwrap_err(),
        NbtAccounterError::PopAtTopLevel
    );

    NbtAccounter::uncompressed_quota()
        .account_bytes(UNCOMPRESSED_NBT_QUOTA)
        .unwrap();
    NbtAccounter::unlimited_heap()
        .account_bytes(i64::MAX)
        .unwrap();
}

#[test]
fn nbt_exception_and_package_metadata_match_java_hierarchy() {
    assert!(NBT_EXCEPTION_JAVA.contains("public class NbtException extends RuntimeException"));
    assert!(NBT_ACCOUNTER_EXCEPTION_JAVA
        .contains("public class NbtAccounterException extends NbtException"));
    assert!(
        NBT_FORMAT_EXCEPTION_JAVA.contains("public class NbtFormatException extends NbtException")
    );
    assert!(NBT_PACKAGE_INFO_JAVA.contains("@NullMarked"));
    assert!(NBT_PACKAGE_INFO_JAVA.contains("package net.minecraft.nbt;"));
    assert!(NBT_PACKAGE_INFO_JAVA.contains("import org.jspecify.annotations.NullMarked;"));
}

#[test]
fn nbt_tag_type_registry_metadata_matches_java() {
    for sentinel in [
        "byte TAG_END = 0;",
        "byte TAG_LONG_ARRAY = 12;",
        "int MAX_DEPTH = 512;",
        "TagType.createInvalid(typeId)",
        "return \"INVALID[\" + id + \"]\";",
        "return \"UNKNOWN_\" + id;",
        "case CONTINUE:",
        "case BREAK:",
        "this.skip(input, accounter);",
    ] {
        assert!(
            TAG_JAVA.contains(sentinel)
                || TAG_TYPE_JAVA.contains(sentinel)
                || TAG_TYPES_JAVA.contains(sentinel),
            "missing NBT tag metadata sentinel {sentinel}"
        );
    }

    let expected = [
        (0, "END", "TAG_End", Some(0), Some(8)),
        (1, "BYTE", "TAG_Byte", Some(1), Some(9)),
        (2, "SHORT", "TAG_Short", Some(2), Some(10)),
        (3, "INT", "TAG_Int", Some(4), Some(12)),
        (4, "LONG", "TAG_Long", Some(8), Some(16)),
        (5, "FLOAT", "TAG_Float", Some(4), Some(12)),
        (6, "DOUBLE", "TAG_Double", Some(8), Some(16)),
        (7, "BYTE[]", "TAG_Byte_Array", None, None),
        (8, "STRING", "TAG_String", None, None),
        (9, "LIST", "TAG_List", None, None),
        (10, "COMPOUND", "TAG_Compound", None, None),
        (11, "INT[]", "TAG_Int_Array", None, None),
        (12, "LONG[]", "TAG_Long_Array", None, None),
    ];
    assert_eq!(TAG_TYPES.len(), expected.len());
    for (id, name, pretty_name, static_payload_size, self_size_in_bytes) in expected {
        let NbtTagTypeLookup::Known(info) = tag_type(id) else {
            panic!("tag type {id} should be known");
        };
        assert_eq!(info.id, id as u8);
        assert_eq!(info.name, name);
        assert_eq!(info.pretty_name, pretty_name);
        assert_eq!(info.static_payload_size, static_payload_size);
        assert_eq!(info.self_size_in_bytes, self_size_in_bytes);
    }

    assert_eq!(
        tag_type(-1),
        NbtTagTypeLookup::Invalid {
            id: -1,
            name: "INVALID[-1]".to_string(),
            pretty_name: "UNKNOWN_-1".to_string(),
        }
    );
    assert_eq!(
        tag_type(99),
        NbtTagTypeLookup::Invalid {
            id: 99,
            name: "INVALID[99]".to_string(),
            pretty_name: "UNKNOWN_99".to_string(),
        }
    );
    assert_eq!(
        root_parse_action(RootVisitResult::Continue),
        RootParseAction::ParsePayload
    );
    assert_eq!(
        root_parse_action(RootVisitResult::Halt),
        RootParseAction::Stop
    );
    assert_eq!(
        root_parse_action(RootVisitResult::Break),
        RootParseAction::SkipPayload
    );
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn tag_interface_matches_java_ids_types_sizes_and_optional_accessors() {
    for sentinel in [
        "public sealed interface Tag permits CompoundTag, CollectionTag, PrimitiveTag, EndTag",
        "int OBJECT_HEADER = 8;",
        "int ARRAY_HEADER = 12;",
        "int OBJECT_REFERENCE = 4;",
        "int STRING_SIZE = 28;",
        "byte TAG_END = 0;",
        "byte TAG_LONG_ARRAY = 12;",
        "int MAX_DEPTH = 512;",
        "default Optional<String> asString()",
        "default Optional<Number> asNumber()",
        "default Optional<Boolean> asBoolean()",
        "default Optional<byte[]> asByteArray()",
        "default Optional<CompoundTag> asCompound()",
        "default Optional<ListTag> asList()",
        "return this.asByte().map(b -> b != 0);",
    ] {
        assert!(
            TAG_JAVA.contains(sentinel),
            "missing Tag sentinel {sentinel}"
        );
    }

    let expected_ids = [
        (Tag::End, 0, 8),
        (Tag::Byte(1), 1, 9),
        (Tag::Short(2), 2, 10),
        (Tag::Int(3), 3, 12),
        (Tag::Long(4), 4, 16),
        (Tag::Float(5.0), 5, 12),
        (Tag::Double(6.0), 6, 16),
        (Tag::ByteArray(vec![1, 2]), 7, 26),
        (Tag::String("a😀".to_string()), 8, 42),
        (
            Tag::List(vec![Tag::Int(1), Tag::String("x".to_string())]),
            9,
            94,
        ),
        (
            Tag::Compound(vec![
                ("a😀".to_string(), Tag::Byte(1)),
                ("nested".to_string(), Tag::List(vec![Tag::Int(2)])),
            ]),
            10,
            48 + (28 + 2 * 3 + 36 + 9) + (28 + 2 * 6 + 36 + 52),
        ),
        (Tag::IntArray(vec![1, 2]), 11, 32),
        (Tag::LongArray(vec![1, 2]), 12, 40),
    ];

    for (tag, id, size_in_bytes) in expected_ids {
        assert_eq!(tag.id(), id);
        let NbtTagTypeLookup::Known(info) = tag.tag_type() else {
            panic!("tag type {id} should be known");
        };
        assert_eq!(info.id, id);
        assert_eq!(tag.copy_tag(), tag);
        assert_eq!(tag.size_in_bytes(), size_in_bytes);
    }

    let numeric = Tag::Int(257);
    assert_eq!(numeric.as_number(), Some(NbtNumericValue::Int(257)));
    assert_eq!(numeric.as_byte(), Some(1));
    assert_eq!(numeric.as_short(), Some(257));
    assert_eq!(numeric.as_int(), Some(257));
    assert_eq!(numeric.as_long(), Some(257));
    assert_eq!(numeric.as_float(), Some(257.0));
    assert_eq!(numeric.as_double(), Some(257.0));
    assert_eq!(numeric.as_boolean(), Some(true));

    assert_eq!(Tag::Byte(0).as_boolean(), Some(false));
    assert_eq!(Tag::String("value".to_string()).as_string(), Some("value"));
    assert_eq!(Tag::String("value".to_string()).as_int(), None);
    assert_eq!(Tag::End.as_string(), None);
    assert_eq!(Tag::End.as_number(), None);
    assert_eq!(Tag::End.as_boolean(), None);

    let bytes = Tag::ByteArray(vec![1, -2]);
    assert_eq!(bytes.as_byte_array(), Some(&[1, -2][..]));
    assert_eq!(bytes.as_int_array(), None);
    let ints = Tag::IntArray(vec![3, 4]);
    assert_eq!(ints.as_int_array(), Some(&[3, 4][..]));
    let longs = Tag::LongArray(vec![5, 6]);
    assert_eq!(longs.as_long_array(), Some(&[5, 6][..]));

    let list = Tag::List(vec![Tag::Byte(1)]);
    assert_eq!(list.as_list(), Some(&[Tag::Byte(1)][..]));
    assert_eq!(list.as_compound(), None);
    let compound = Tag::Compound(vec![("key".to_string(), Tag::String("value".to_string()))]);
    assert_eq!(
        compound.as_compound(),
        Some(&[("key".to_string(), Tag::String("value".to_string()))][..])
    );
    assert_eq!(compound.as_list(), None);
}

#[test]
#[allow(clippy::too_many_lines)]
fn tag_visitor_dispatch_matches_java_visit_methods() {
    for sentinel in [
        "void visitString(StringTag tag);",
        "void visitByte(ByteTag tag);",
        "void visitShort(ShortTag tag);",
        "void visitInt(IntTag tag);",
        "void visitLong(LongTag tag);",
        "void visitFloat(FloatTag tag);",
        "void visitDouble(DoubleTag tag);",
        "void visitByteArray(ByteArrayTag tag);",
        "void visitIntArray(IntArrayTag tag);",
        "void visitLongArray(LongArrayTag tag);",
        "void visitList(ListTag tag);",
        "void visitCompound(CompoundTag tag);",
        "void visitEnd(EndTag tag);",
        "void accept(TagVisitor visitor);",
    ] {
        assert!(
            TAG_VISITOR_JAVA.contains(sentinel) || TAG_JAVA.contains(sentinel),
            "missing TagVisitor sentinel {sentinel}"
        );
    }

    #[derive(Default)]
    struct RecordingVisitor {
        visits: Vec<String>,
    }

    impl super::tag_access::NbtTagVisitor for RecordingVisitor {
        fn visit_string(&mut self, value: &str) {
            self.visits.push(format!("string:{value}"));
        }

        fn visit_byte(&mut self, value: i8) {
            self.visits.push(format!("byte:{value}"));
        }

        fn visit_short(&mut self, value: i16) {
            self.visits.push(format!("short:{value}"));
        }

        fn visit_int(&mut self, value: i32) {
            self.visits.push(format!("int:{value}"));
        }

        fn visit_long(&mut self, value: i64) {
            self.visits.push(format!("long:{value}"));
        }

        fn visit_float(&mut self, value: f32) {
            self.visits.push(format!("float:{value}"));
        }

        fn visit_double(&mut self, value: f64) {
            self.visits.push(format!("double:{value}"));
        }

        fn visit_byte_array(&mut self, value: &[i8]) {
            self.visits.push(format!("byte_array:{}", value.len()));
        }

        fn visit_int_array(&mut self, value: &[i32]) {
            self.visits.push(format!("int_array:{}", value.len()));
        }

        fn visit_long_array(&mut self, value: &[i64]) {
            self.visits.push(format!("long_array:{}", value.len()));
        }

        fn visit_list(&mut self, value: &[Tag]) {
            self.visits.push(format!("list:{}", value.len()));
        }

        fn visit_compound(&mut self, value: &[(String, Tag)]) {
            self.visits.push(format!("compound:{}", value.len()));
        }

        fn visit_end(&mut self) {
            self.visits.push("end".to_string());
        }
    }

    let tags = [
        Tag::String("text".to_string()),
        Tag::Byte(-1),
        Tag::Short(2),
        Tag::Int(3),
        Tag::Long(4),
        Tag::Float(5.5),
        Tag::Double(6.25),
        Tag::ByteArray(vec![1, 2]),
        Tag::IntArray(vec![3, 4, 5]),
        Tag::LongArray(vec![6]),
        Tag::List(vec![Tag::Byte(1), Tag::Byte(2)]),
        Tag::Compound(vec![("key".to_string(), Tag::Int(7))]),
        Tag::End,
    ];

    let mut visitor = RecordingVisitor::default();
    for tag in tags {
        tag.accept_tag_visitor(&mut visitor);
    }

    assert_eq!(
        visitor.visits,
        vec![
            "string:text",
            "byte:-1",
            "short:2",
            "int:3",
            "long:4",
            "float:5.5",
            "double:6.25",
            "byte_array:2",
            "int_array:3",
            "long_array:1",
            "list:2",
            "compound:1",
            "end",
        ]
    );
}

#[test]
fn reported_nbt_exception_is_reported_exception_wrapper_like_java() {
    assert!(REPORTED_NBT_EXCEPTION_JAVA
        .contains("public class ReportedNbtException extends ReportedException"));
    assert!(REPORTED_NBT_EXCEPTION_JAVA
        .contains("public ReportedNbtException(final CrashReport report)"));
    assert!(REPORTED_NBT_EXCEPTION_JAVA.contains("super(report);"));

    let reported = ReportedNbtExceptionModel::new("NBT crash report");
    assert_eq!(reported.crash_report, "NBT crash report");
}

#[test]
#[allow(clippy::too_many_lines)]
fn scalar_nbt_tag_classes_match_java_payloads_sizes_and_copy_contracts() {
    for (source, sentinels) in [
        (
            END_TAG_JAVA,
            [
                "private static final int SELF_SIZE_IN_BYTES = 8;",
                "public static final EndTag INSTANCE = new EndTag();",
                "return EndTag.INSTANCE;",
                "public EndTag copy()",
                "return this;",
            ],
        ),
        (
            BYTE_TAG_JAVA,
            [
                "private static final int SELF_SIZE_IN_BYTES = 9;",
                "public static final ByteTag ZERO = valueOf((byte)0);",
                "public static final ByteTag ONE = valueOf((byte)1);",
                "return ByteTag.Cache.cache[128 + data];",
                "output.writeByte(this.value);",
            ],
        ),
        (
            SHORT_TAG_JAVA,
            [
                "private static final int SELF_SIZE_IN_BYTES = 10;",
                "return i >= -128 && i <= 1024",
                "output.writeShort(this.value);",
                "return (byte)(this.value & 0xFF);",
                "public ShortTag copy()",
            ],
        ),
        (
            INT_TAG_JAVA,
            [
                "private static final int SELF_SIZE_IN_BYTES = 12;",
                "output.writeInt(this.value);",
                "return (short)(this.value & 65535);",
                "return (byte)(this.value & 0xFF);",
                "public IntTag copy()",
            ],
        ),
        (
            LONG_TAG_JAVA,
            [
                "private static final int SELF_SIZE_IN_BYTES = 16;",
                "output.writeLong(this.value);",
                "return (int)(this.value & -1L);",
                "return (byte)(this.value & 255L);",
                "public LongTag copy()",
            ],
        ),
        (
            FLOAT_TAG_JAVA,
            [
                "private static final int SELF_SIZE_IN_BYTES = 12;",
                "public static final FloatTag ZERO = new FloatTag(0.0F);",
                "output.writeFloat(this.value);",
                "return Mth.floor(this.value);",
                "public FloatTag copy()",
            ],
        ),
        (
            DOUBLE_TAG_JAVA,
            [
                "private static final int SELF_SIZE_IN_BYTES = 16;",
                "public static final DoubleTag ZERO = new DoubleTag(0.0);",
                "output.writeDouble(this.value);",
                "return (long)Math.floor(this.value);",
                "public DoubleTag copy()",
            ],
        ),
    ] {
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "missing scalar tag sentinel {sentinel}"
            );
        }
    }

    let scalar_cases = [
        (Tag::End, 0, Some(8), Vec::new()),
        (Tag::Byte(-2), 1, Some(9), vec![0xFE]),
        (Tag::Short(0x1234), 2, Some(10), vec![0x12, 0x34]),
        (
            Tag::Int(0x1234_5678),
            3,
            Some(12),
            vec![0x12, 0x34, 0x56, 0x78],
        ),
        (
            Tag::Long(0x0102_0304_0506_0708),
            4,
            Some(16),
            vec![1, 2, 3, 4, 5, 6, 7, 8],
        ),
        (
            Tag::Float(1.5),
            5,
            Some(12),
            1.5_f32.to_bits().to_be_bytes().to_vec(),
        ),
        (
            Tag::Double(-2.25),
            6,
            Some(16),
            (-2.25_f64).to_bits().to_be_bytes().to_vec(),
        ),
    ];

    for (tag, id, self_size_in_bytes, expected_payload) in scalar_cases {
        let NbtTagTypeLookup::Known(info) = tag_type(id) else {
            panic!("tag type {id} should be known");
        };
        assert_eq!(tag.id(), id as u8);
        assert_eq!(info.self_size_in_bytes, self_size_in_bytes);
        assert_eq!(info.static_payload_size, Some(expected_payload.len()));

        let mut payload = Vec::new();
        tag.write_payload(&mut payload).unwrap();
        assert_eq!(payload, expected_payload);
        let decoded = Tag::read_payload(id as u8, &mut payload.as_slice()).unwrap();
        assert_eq!(decoded, tag);
        assert_eq!(tag.clone(), tag);
    }
}

#[test]
fn string_tag_matches_java_modified_utf_size_and_quote_contracts() {
    for sentinel in [
        "private static final int SELF_SIZE_IN_BYTES = 36;",
        "private static final StringTag EMPTY = new StringTag(\"\");",
        "input.skipBytes(input.readUnsignedShort());",
        "return data.isEmpty() ? EMPTY : new StringTag(data);",
        "output.writeUTF(this.value);",
        "return 36 + 2 * this.value.length();",
        "return Optional.of(this.value);",
        "StringTag.quoteAndEscape(tag.value())",
        "StringTag.quoteAndEscape(input, this.builder);",
    ] {
        assert!(
            STRING_TAG_JAVA.contains(sentinel)
                || STRING_TAG_VISITOR_JAVA.contains(sentinel)
                || SNBT_GRAMMAR_JAVA.contains(sentinel),
            "missing StringTag sentinel {sentinel}"
        );
    }
    for sentinel in [
        "case '\\b' -> \"b\";",
        "case '\\t' -> \"t\";",
        "case '\\n' -> \"n\";",
        "case '\\f' -> \"f\";",
        "case '\\r' -> \"r\";",
        "c < ' ' ? \"x\" + HEX_ESCAPE.toHexDigits((byte)c) : null",
    ] {
        assert!(
            SNBT_GRAMMAR_JAVA.contains(sentinel),
            "missing SNBT escape sentinel {sentinel}"
        );
    }

    let value = "a\u{0000}b😀c";
    assert_eq!(
        string_size_in_bytes(value),
        36 + 2 * value.encode_utf16().count()
    );

    let mut payload = Vec::new();
    Tag::String(value.to_string())
        .write_payload(&mut payload)
        .unwrap();
    assert_eq!(&payload[..2], &[0x00, 0x0B]);
    assert_eq!(
        Tag::read_payload(8, &mut payload.as_slice()).unwrap(),
        Tag::String(value.to_string())
    );

    assert_eq!(Tag::String(String::new()).to_snbt(), "\"\"");
    assert_eq!(
        Tag::String("A \"quoted\" name".to_string()).to_snbt(),
        "'A \"quoted\" name'"
    );
    assert_eq!(
        Tag::String("it's fine".to_string()).to_snbt(),
        "\"it's fine\""
    );
    assert_eq!(
        Tag::String("line\n\u{0001}\\".to_string()).to_snbt(),
        "\"line\\n\\x01\\\\\""
    );
    assert_eq!(
        escape_snbt_string_without_quotes("\"'\n\\"),
        "\\\"\\'\\n\\\\"
    );
    assert_eq!(
        Tag::Compound(vec![
            ("true".to_string(), Tag::String("reserved".to_string())),
            ("valid_key".to_string(), Tag::String("ok".to_string())),
        ])
        .to_snbt(),
        "{\"true\":\"reserved\",valid_key:\"ok\"}"
    );
}

#[test]
fn primitive_array_tags_match_java_collection_payloads_and_sizes() {
    for sentinel in [
        "public sealed interface CollectionTag extends Tag, Iterable<Tag>",
        "return this.size() == 0;",
        "throw new NoSuchElementException();",
        "return CollectionTag.this.get(this.index++);",
    ] {
        assert!(
            COLLECTION_TAG_JAVA.contains(sentinel),
            "missing CollectionTag sentinel {sentinel}"
        );
    }

    for (source, sentinels) in [
        (
            BYTE_ARRAY_TAG_JAVA,
            [
                "private static final int SELF_SIZE_IN_BYTES = 24;",
                "accounter.accountBytes(1L, length);",
                "input.skipBytes(input.readInt() * 1);",
                "return 24 + 1 * this.data.length;",
                "return Optional.of(this.data);",
                "numeric.byteValue()",
            ],
        ),
        (
            INT_ARRAY_TAG_JAVA,
            [
                "private static final int SELF_SIZE_IN_BYTES = 24;",
                "accounter.accountBytes(4L, length);",
                "input.skipBytes(input.readInt() * 4);",
                "return 24 + 4 * this.data.length;",
                "return Optional.of(this.data);",
                "numeric.intValue()",
            ],
        ),
        (
            LONG_ARRAY_TAG_JAVA,
            [
                "private static final int SELF_SIZE_IN_BYTES = 24;",
                "accounter.accountBytes(8L, length);",
                "input.skipBytes(input.readInt() * 8);",
                "return 24 + 8 * this.data.length;",
                "return Optional.of(this.data);",
                "numeric.longValue()",
            ],
        ),
    ] {
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "missing primitive array tag sentinel {sentinel}"
            );
        }
    }

    let byte_array = Tag::ByteArray(vec![-1, 2]);
    let mut byte_payload = Vec::new();
    byte_array.write_payload(&mut byte_payload).unwrap();
    assert_eq!(byte_payload, vec![0, 0, 0, 2, 0xFF, 0x02]);
    assert_eq!(
        Tag::read_payload(7, &mut byte_payload.as_slice()).unwrap(),
        byte_array
    );
    assert_eq!(byte_array_size_in_bytes(2), 26);
    assert_eq!(byte_array.to_snbt(), "[B;-1B,2B]");

    let int_array = Tag::IntArray(vec![1, -2]);
    let mut int_payload = Vec::new();
    int_array.write_payload(&mut int_payload).unwrap();
    assert_eq!(
        int_payload,
        vec![0, 0, 0, 2, 0, 0, 0, 1, 0xFF, 0xFF, 0xFF, 0xFE]
    );
    assert_eq!(
        Tag::read_payload(11, &mut int_payload.as_slice()).unwrap(),
        int_array
    );
    assert_eq!(int_array_size_in_bytes(2), 32);
    assert_eq!(int_array.to_snbt(), "[I;1,-2]");

    let long_array = Tag::LongArray(vec![1, -2]);
    let mut long_payload = Vec::new();
    long_array.write_payload(&mut long_payload).unwrap();
    assert_eq!(
        long_payload,
        vec![0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE]
    );
    assert_eq!(
        Tag::read_payload(12, &mut long_payload.as_slice()).unwrap(),
        long_array
    );
    assert_eq!(long_array_size_in_bytes(2), 40);
    assert_eq!(long_array.to_snbt(), "[L;1L,-2L]");

    assert_eq!(Tag::ByteArray(vec![]).payload_size(), 4);
    assert_eq!(Tag::IntArray(vec![1, 2]).payload_size(), 12);
    assert_eq!(Tag::LongArray(vec![1, 2]).payload_size(), 20);
    assert_eq!(Tag::ByteArray(vec![1]).clone(), Tag::ByteArray(vec![1]));
}

#[test]
fn string_tag_visitor_matches_java_snbt_rendering_order_and_suffixes() {
    for sentinel in [
        "private static final Pattern UNQUOTED_KEY_MATCH = Pattern.compile(\"[A-Za-z._]+[A-Za-z0-9._+-]*\");",
        "this.builder.append(tag.value()).append('b');",
        "this.builder.append(tag.value()).append('s');",
        "this.builder.append(tag.value()).append('L');",
        "this.builder.append(tag.value()).append('f');",
        "this.builder.append(tag.value()).append('d');",
        "this.builder.append(\"[B;\");",
        "this.builder.append(data[i]).append('B');",
        "this.builder.append(\"[I;\");",
        "this.builder.append(\"[L;\");",
        "this.builder.append(data[i]).append('L');",
        "entries.sort(Entry.comparingByKey());",
        "!input.equalsIgnoreCase(\"true\") && !input.equalsIgnoreCase(\"false\")",
        "this.builder.append(\"END\");",
    ] {
        assert!(
            STRING_TAG_VISITOR_JAVA.contains(sentinel),
            "missing StringTagVisitor sentinel {sentinel}"
        );
    }

    assert_eq!(Tag::End.to_snbt(), "END");
    assert_eq!(Tag::Byte(-1).to_snbt(), "-1b");
    assert_eq!(Tag::Short(2).to_snbt(), "2s");
    assert_eq!(Tag::Int(3).to_snbt(), "3");
    assert_eq!(Tag::Long(4).to_snbt(), "4L");
    assert_eq!(Tag::Float(1.5).to_snbt(), "1.5f");
    assert_eq!(Tag::Double(2.25).to_snbt(), "2.25d");
    assert_eq!(Tag::ByteArray(vec![1, -2]).to_snbt(), "[B;1B,-2B]");
    assert_eq!(Tag::IntArray(vec![3, -4]).to_snbt(), "[I;3,-4]");
    assert_eq!(Tag::LongArray(vec![5, -6]).to_snbt(), "[L;5L,-6L]");
    assert_eq!(
        Tag::List(vec![
            Tag::String("x".to_string()),
            Tag::String("y".to_string())
        ])
        .to_snbt(),
        "[\"x\",\"y\"]"
    );
    assert_eq!(
        Tag::Compound(vec![
            ("z".to_string(), Tag::Int(3)),
            ("a".to_string(), Tag::Byte(1)),
            ("true".to_string(), Tag::String("reserved".to_string())),
            ("1bad".to_string(), Tag::String("number-start".to_string())),
        ])
        .to_snbt(),
        "{\"1bad\":\"number-start\",a:1b,\"true\":\"reserved\",z:3}"
    );
}

#[test]
fn list_tag_matches_java_homogeneous_and_wrapper_contracts() {
    for sentinel in [
        "private static final String WRAPPER_MARKER = \"\";",
        "private static final int SELF_SIZE_IN_BYTES = 36;",
        "if (typeId == 0 && count > 0)",
        "throw new NbtFormatException(\"Missing type on ListTag\");",
        "return count;",
        "if (homogenousType == 0)",
        "return 10;",
        "return tag instanceof CompoundTag compoundTag && !isWrapper(compoundTag) ? compoundTag : wrapElement(tag);",
        "return new CompoundTag(Map.of(\"\", tag));",
        "this.add(tryUnwrap(compound));",
        "size += 4 * this.list.size();",
        "copy.add(tag.copy());",
        "return Optional.of(this);",
    ] {
        assert!(
            LIST_TAG_JAVA.contains(sentinel),
            "missing ListTag sentinel {sentinel}"
        );
    }

    let homogenous = Tag::List(vec![Tag::Int(1), Tag::Int(-2)]);
    let mut homogenous_payload = Vec::new();
    homogenous.write_payload(&mut homogenous_payload).unwrap();
    assert_eq!(
        homogenous_payload,
        vec![3, 0, 0, 0, 2, 0, 0, 0, 1, 0xFF, 0xFF, 0xFF, 0xFE]
    );
    assert_eq!(
        Tag::read_payload(9, &mut homogenous_payload.as_slice()).unwrap(),
        homogenous
    );

    let mixed = Tag::List(vec![Tag::Int(1), Tag::String("x".to_string())]);
    let mut mixed_payload = Vec::new();
    mixed.write_payload(&mut mixed_payload).unwrap();
    assert_eq!(&mixed_payload[..5], &[10, 0, 0, 0, 2]);
    assert_eq!(
        Tag::read_payload(9, &mut mixed_payload.as_slice()).unwrap(),
        mixed
    );
    assert_eq!(mixed.to_snbt(), "[1,\"x\"]");

    let wrapper_compound = Tag::List(vec![Tag::Compound(vec![("".to_string(), Tag::Int(7))])]);
    let mut wrapper_payload = Vec::new();
    wrapper_compound
        .write_payload(&mut wrapper_payload)
        .unwrap();
    assert_eq!(
        Tag::read_payload(9, &mut wrapper_payload.as_slice()).unwrap(),
        wrapper_compound
    );

    let missing_type = vec![0, 0, 0, 0, 1];
    let err = Tag::read_payload(9, &mut missing_type.as_slice()).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("Missing type on ListTag"));

    assert_eq!(list_size_in_bytes(&[12, 12]), 68);
    assert_eq!(Tag::List(vec![]).payload_size(), 5);
    assert_eq!(Tag::List(vec![Tag::Int(1), Tag::Int(2)]).payload_size(), 13);
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn numeric_and_primitive_tag_interface_conversions_match_java() {
    for sentinel in [
        "public sealed interface NumericTag extends PrimitiveTag",
        "default Optional<Number> asNumber()",
        "default Optional<Boolean> asBoolean()",
        "return Optional.of(this.byteValue() != 0);",
        "public sealed interface PrimitiveTag extends Tag permits NumericTag, StringTag",
        "default Tag copy()",
        "return this;",
        "return Mth.floor(this.value);",
        "return (byte)(Mth.floor(this.value) & 0xFF);",
    ] {
        assert!(
            NUMERIC_TAG_JAVA.contains(sentinel)
                || PRIMITIVE_TAG_JAVA.contains(sentinel)
                || FLOAT_TAG_JAVA.contains(sentinel)
                || DOUBLE_TAG_JAVA.contains(sentinel),
            "missing numeric/primitive sentinel {sentinel}"
        );
    }

    assert_eq!(
        Tag::Byte(-1).numeric_value(),
        Some(NbtNumericValue::Byte(-1))
    );
    assert_eq!(Tag::String("primitive".to_string()).numeric_value(), None);
    assert!(Tag::Double(-1.2).is_primitive());
    assert!(Tag::String(String::new()).is_primitive());
    assert!(!Tag::List(vec![]).is_primitive());
    assert!(!Tag::Compound(vec![]).is_primitive());

    let int_value = Tag::Int(257).numeric_value().unwrap();
    assert_eq!(int_value.byte_value(), 1);
    assert_eq!(int_value.short_value(), 257);
    assert_eq!(int_value.int_value(), 257);
    assert_eq!(int_value.long_value(), 257);
    assert_eq!(int_value.float_value(), 257.0);
    assert_eq!(int_value.double_value(), 257.0);
    assert!(int_value.boolean_value());

    let long_value = Tag::Long(0x1_0000_0001).numeric_value().unwrap();
    assert_eq!(long_value.int_value(), 1);
    assert_eq!(long_value.byte_value(), 1);

    let float_value = Tag::Float(-1.2).numeric_value().unwrap();
    assert_eq!(float_value.int_value(), -2);
    assert_eq!(float_value.short_value(), -2);
    assert_eq!(float_value.byte_value(), -2);
    assert_eq!(float_value.long_value(), -1);

    let double_value = Tag::Double(258.75).numeric_value().unwrap();
    assert_eq!(double_value.int_value(), 258);
    assert_eq!(double_value.short_value(), 258);
    assert_eq!(double_value.byte_value(), 2);
    assert_eq!(double_value.float_value(), 258.75_f32);
}

#[test]
fn nbt_strings_use_java_modified_utf8_not_standard_utf8() {
    // Null char: Java modified UTF-8 encodes U+0000 as two bytes C0 80 (NOT the
    // single 0x00 standard UTF-8 uses), with an unsigned u16 length prefix.
    let mut buf = Vec::new();
    Tag::String("\u{0000}".to_string())
        .write_payload(&mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x00, 0x02, 0xC0, 0x80]);

    // Supplementary char (emoji U+1F600): modified UTF-8 emits the UTF-16
    // surrogate pair D83D/DE00 as two 3-byte sequences = 6 bytes total (vs 4 in
    // standard UTF-8).
    let mut emoji = Vec::new();
    Tag::String("\u{1F600}".to_string())
        .write_payload(&mut emoji)
        .unwrap();
    assert_eq!(emoji, vec![0x00, 0x06, 0xED, 0xA0, 0xBD, 0xED, 0xB8, 0x80]);

    // BMP 3-byte char (snowman U+2603) matches standard UTF-8 (E2 98 83).
    let mut snowman = Vec::new();
    Tag::String("\u{2603}".to_string())
        .write_payload(&mut snowman)
        .unwrap();
    assert_eq!(snowman, vec![0x00, 0x03, 0xE2, 0x98, 0x83]);

    // Round-trip every case (including a mixed string) through read_payload.
    for s in [
        "",
        "abc",
        "\u{0000}",
        "\u{1F600}",
        "\u{2603}",
        "a\u{0000}b😀c",
    ] {
        let mut bytes = Vec::new();
        Tag::String(s.to_string())
            .write_payload(&mut bytes)
            .unwrap();
        let decoded = Tag::read_payload(8, &mut bytes.as_slice()).unwrap();
        assert_eq!(decoded, Tag::String(s.to_string()));
    }
}

#[test]
fn exposes_all_vanilla_tag_ids() {
    assert_eq!(Tag::End.id(), 0);
    assert_eq!(Tag::Byte(0).id(), 1);
    assert_eq!(Tag::Short(0).id(), 2);
    assert_eq!(Tag::Int(0).id(), 3);
    assert_eq!(Tag::Long(0).id(), 4);
    assert_eq!(Tag::Float(0.0).id(), 5);
    assert_eq!(Tag::Double(0.0).id(), 6);
    assert_eq!(Tag::ByteArray(vec![]).id(), 7);
    assert_eq!(Tag::String(String::new()).id(), 8);
    assert_eq!(Tag::List(vec![]).id(), 9);
    assert_eq!(Tag::Compound(vec![]).id(), 10);
    assert_eq!(Tag::IntArray(vec![]).id(), 11);
    assert_eq!(Tag::LongArray(vec![]).id(), 12);
}

#[test]
fn round_trips_named_compound_with_nested_values() {
    let tag = Tag::Compound(vec![
        ("name".to_string(), Tag::String("VibeCraft".to_string())),
        ("health".to_string(), Tag::Float(20.0)),
        (
            "pos".to_string(),
            Tag::List(vec![Tag::Double(1.0), Tag::Double(2.0)]),
        ),
        ("ints".to_string(), Tag::IntArray(vec![1, 2, 3])),
        ("longs".to_string(), Tag::LongArray(vec![4, 5, 6])),
    ]);

    let mut bytes = Vec::new();
    write_named_tag(&mut bytes, "root", &tag).unwrap();
    let (name, decoded) = read_named_tag(&mut Cursor::new(bytes)).unwrap();

    assert_eq!(name, "root");
    assert_eq!(decoded, tag);
}

#[test]
fn mixed_type_lists_use_java_wrapper_encoding() {
    let mut bytes = Vec::new();
    let mixed = Tag::List(vec![Tag::Int(1), Tag::String("bad".to_string())]);
    mixed.write_payload(&mut bytes).unwrap();
    assert_eq!(&bytes[..5], &[10, 0, 0, 0, 2]);
    assert_eq!(Tag::read_payload(9, &mut bytes.as_slice()).unwrap(), mixed);
}

#[test]
fn compressed_nbt_round_trips_named_tags() {
    let tag = Tag::Compound(vec![("DataVersion".to_string(), Tag::Int(4790))]);
    let mut bytes = Vec::new();
    write_gzip_named_tag(&mut bytes, "level", &tag).unwrap();

    assert_eq!(&bytes[..2], &[0x1f, 0x8b]);
    let (name, decoded) = read_gzip_named_tag(Cursor::new(bytes)).unwrap();
    assert_eq!(name, "level");
    assert_eq!(decoded, tag);
}

#[test]
fn snbt_printer_size_accounting_and_traversal_cover_nested_tags() {
    let tag = Tag::Compound(vec![
        (
            "name".to_string(),
            Tag::String("A \"quoted\" name".to_string()),
        ),
        ("bytes".to_string(), Tag::ByteArray(vec![1, 2])),
        (
            "nested".to_string(),
            Tag::List(vec![Tag::Int(1), Tag::Int(2)]),
        ),
    ]);

    // Byte-array elements use uppercase 'B', matching Java StringTagVisitor.
    assert_eq!(
        tag.to_snbt(),
        "{bytes:[B;1B,2B],name:'A \"quoted\" name',nested:[1,2]}"
    );
    assert_eq!(tag.payload_size(), 61);

    let mut ids = Vec::new();
    tag.visit_depth_first(&mut |visited| ids.push(visited.id()));
    assert_eq!(ids, vec![10, 8, 7, 9, 3, 3]);
}

#[test]
fn bounded_nbt_reads_reject_excessive_recursion() {
    let tag = Tag::Compound(vec![(
        "outer".to_string(),
        Tag::Compound(vec![("inner".to_string(), Tag::Int(1))]),
    )]);
    let mut bytes = Vec::new();
    write_named_tag(&mut bytes, "root", &tag).unwrap();

    let err = read_named_tag_limited(&mut Cursor::new(bytes), 1).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("depth limit"));
}

#[test]
fn nbt_size_tracker_and_field_selectors_cover_streaming_use_cases() {
    let tag = Tag::Compound(vec![
        (
            "Data".to_string(),
            Tag::Compound(vec![
                ("DataVersion".to_string(), Tag::Int(4790)),
                ("LevelName".to_string(), Tag::String("world".to_string())),
            ]),
        ),
        ("Other".to_string(), Tag::Byte(1)),
    ]);

    let tracked = tag.tracked_size();
    assert_eq!(tracked.nodes, 5);
    assert_eq!(tracked.max_depth, 2);
    assert!(tracked.payload_bytes >= tag.payload_size());

    let selectors = [
        NbtFieldSelector::dotted("Data.DataVersion"),
        NbtFieldSelector::dotted("Data.Missing"),
        NbtFieldSelector::dotted("Other"),
    ];
    let mut selected = Vec::new();
    tag.visit_selected_fields(&selectors, |selector, value| {
        selected.push((selector.path.join("."), value.clone()));
    });
    assert_eq!(
        selected,
        vec![
            ("Data.DataVersion".to_string(), Tag::Int(4790)),
            ("Other".to_string(), Tag::Byte(1)),
        ]
    );
}

#[test]
fn snbt_parser_round_trips_printer_shapes_and_reports_errors() {
    let tag = Tag::Compound(vec![
        (
            "name".to_string(),
            Tag::String("A \"quoted\" name".to_string()),
        ),
        ("bytes".to_string(), Tag::ByteArray(vec![1, 2])),
        ("ints".to_string(), Tag::IntArray(vec![3, 4])),
        ("longs".to_string(), Tag::LongArray(vec![5, 6])),
        (
            "nested".to_string(),
            Tag::List(vec![Tag::Int(1), Tag::Int(2)]),
        ),
        ("enabled".to_string(), Tag::Byte(1)),
    ]);

    let printed = tag.to_snbt();
    assert_eq!(
        printed,
        "{bytes:[B;1B,2B],enabled:1b,ints:[I;3,4],longs:[L;5L,6L],name:'A \"quoted\" name',nested:[1,2]}"
    );
    assert_eq!(parse_snbt(&printed).unwrap().to_snbt(), printed);
    assert_eq!(parse_snbt("{flag:true}").unwrap().to_snbt(), "{flag:1b}");
    assert_eq!(parse_snbt("12s").unwrap(), Tag::Short(12));
    assert_eq!(parse_snbt("3.5f").unwrap(), Tag::Float(3.5));
    assert!(parse_snbt("{broken")
        .unwrap_err()
        .to_string()
        .contains("expected"));
}

#[test]
fn text_component_visitor_matches_plain_java_layout_rules() {
    let tag = Tag::Compound(vec![
        ("z".to_string(), Tag::Int(3)),
        ("a key".to_string(), Tag::String("quoted".to_string())),
        (
            "list".to_string(),
            Tag::List(vec![
                Tag::Compound(vec![("inner".to_string(), Tag::Byte(1))]),
                Tag::Compound(vec![("inner".to_string(), Tag::Byte(2))]),
            ]),
        ),
    ]);

    assert_eq!(
        tag.to_text_component_plain("  ", true),
        "{\n  \"a key\": \"quoted\",\n  list: [\n    {\n      inner: 1b\n    },\n    {\n      inner: 2b\n    }\n  ],\n  z: 3\n}"
    );
    let component = tag.to_text_component("", true);
    assert!(component.to_json().starts_with("{\"text\":\"\",\"extra\""));
    assert!(component.to_json().contains("\"text\":\"a key\""));
}

#[test]
fn text_component_visitor_folds_deep_tags_and_long_arrays() {
    let long_array = Tag::ByteArray((0..130).map(|value| value as i8).collect());
    let rendered = long_array.to_text_component_plain("", true);
    assert!(rendered.starts_with("[B; 0b, 1b"));
    assert!(rendered.ends_with(",<...>]"));

    let mut deep = Tag::Int(1);
    for index in 0..65 {
        deep = Tag::Compound(vec![(format!("level{index}"), deep)]);
    }
    assert!(deep.to_text_component_plain("", true).contains("{<...>}"));
}
