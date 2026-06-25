use super::*;
use super::super::chunk_debug::{debug_entity_name, debug_entity_name_for_uuid};

const DEBUG_ENTITY_NAME_GENERATOR_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/DebugEntityNameGenerator.java");
const RANDOM_SOURCE_JAVA: &str = vibecraft_java_source!("/net/minecraft/util/RandomSource.java");
const SINGLE_THREADED_RANDOM_SOURCE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/world/level/levelgen/SingleThreadedRandomSource.java");
const BIT_RANDOM_SOURCE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/world/level/levelgen/BitRandomSource.java");
const UTIL_JAVA: &str = vibecraft_java_source!("/net/minecraft/util/Util.java");

#[test]
fn debug_entity_name_generator_matches_java_uuid_fallback() {
    assert_java_contains(
        DEBUG_ENTITY_NAME_GENERATOR_JAVA,
        &[
            "if (entity instanceof Player)",
            "return entity.getPlainTextName();",
            "Component customName = entity.getCustomName();",
            "customName != null ? customName.getString() : getEntityName(entity.getUUID())",
            "RandomSource.createThreadLocalInstance(uuid.hashCode() >> 2)",
            "getRandomString(random, NAMES_FIRST_PART)",
            "getRandomString(random, NAMES_SECOND_PART)",
            "\"Slim\"",
            "\"Scar\"",
            "\"Fox\"",
            "\"Fist\"",
        ],
        "DebugEntityNameGenerator",
    );
    assert_java_contains(
        RANDOM_SOURCE_JAVA,
        &[
            "static RandomSource createThreadLocalInstance(final long seed)",
            "return new SingleThreadedRandomSource(seed);",
        ],
        "RandomSource",
    );
    assert_java_contains(
        SINGLE_THREADED_RANDOM_SOURCE_JAVA,
        &[
            "this.seed = (seed ^ 25214903917L) & 281474976710655L;",
            "long newSeed = this.seed * 25214903917L + 11L & 281474976710655L;",
        ],
        "SingleThreadedRandomSource",
    );
    assert_java_contains(
        BIT_RANDOM_SOURCE_JAVA,
        &[
            "default int nextInt(final int bound)",
            "sample = this.next(31);",
            "modulo = sample % bound;",
        ],
        "BitRandomSource",
    );
    assert_java_contains(
        UTIL_JAVA,
        &[
            "public static <T> T getRandom(final T[] array, final RandomSource random)",
            "return array[random.nextInt(array.length)];",
        ],
        "Util.getRandom",
    );

    assert_eq!(debug_entity_name_for_uuid(Uuid([0; 16])), "SlimShirt");
    assert_eq!(
        debug_entity_name_for_uuid(Uuid([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x23,
        ])),
        "RoughMouth"
    );
    assert_eq!(
        debug_entity_name_for_uuid(Uuid([
            0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44,
            0x66, 0x55, 0x44, 0x00, 0x00,
        ])),
        "ThinHair"
    );
    assert_eq!(
        debug_entity_name_for_uuid(Uuid([
            0xde, 0x30, 0x5d, 0x54, 0x75, 0xb4, 0x43, 0x1b, 0xad, 0xb2, 0xeb,
            0x6b, 0x9e, 0x54, 0x60, 0x14,
        ])),
        "HardHair"
    );
    assert_eq!(
        debug_entity_name_for_uuid(Uuid([
            0x7d, 0x44, 0x48, 0x40, 0x9d, 0xc0, 0x11, 0xd1, 0xb2, 0x45, 0x5f,
            0xfd, 0xce, 0x74, 0xfa, 0xd2,
        ])),
        "SillyFist"
    );
}

#[test]
fn debug_entity_name_generator_uses_java_entity_precedence() {
    let fallback = Uuid([
        0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
        0x00, 0x00,
    ]);
    assert_eq!(
        debug_entity_name(true, "PlainPlayer", Some("CustomPlayer"), fallback),
        "PlainPlayer"
    );
    assert_eq!(
        debug_entity_name(false, "IgnoredPlain", Some("CustomMob"), fallback),
        "CustomMob"
    );
    assert_eq!(
        debug_entity_name(false, "IgnoredPlain", None, fallback),
        "ThinHair"
    );
}

fn assert_java_contains(source: &str, sentinels: &[&str], class_name: &str) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing {class_name} sentinel {sentinel}"
        );
    }
}
