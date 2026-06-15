use super::*;

const CLIENTBOUND_EXPLODE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundExplodePacket.java");
const EXPLOSION_PARTICLE_INFO_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/core/particles/ExplosionParticleInfo.java");
const WEIGHTED_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/util/random/Weighted.java");
const WEIGHTED_LIST_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/util/random/WeightedList.java");
const SOUND_EVENT_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/sounds/SoundEvent.java");
const BYTE_BUF_CODECS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/codec/ByteBufCodecs.java");

#[test]
fn clientbound_explode_packet_matches_java_codec_with_knockback_and_weighted_particles() {
    assert_explode_java_sentinels();
    assert_eq!(CLIENTBOUND_EXPLODE_PACKET_ID, 36);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_EXPLODE_PACKET_ID),
        Some("explode")
    );

    let packet = ClientboundExplodePacket {
        center: Vec3 {
            x: 1.0,
            y: -2.0,
            z: 3.5,
        },
        radius: 4.25,
        block_count: 6,
        player_knockback: Some(Vec3 {
            x: 0.25,
            y: 0.5,
            z: -0.75,
        }),
        explosion_particle: RawParticleOptions {
            particle_id: 300,
            data: vec![0xaa, 0xbb],
        },
        explosion_sound: SoundEventHolder::Registered { id: 5 },
        block_particles: vec![
            WeightedExplosionParticle {
                value: ExplosionParticleInfo {
                    particle: RawParticleOptions {
                        particle_id: 1,
                        data: Vec::new(),
                    },
                    scaling: 2.0,
                    speed: 3.0,
                },
                weight: 7,
            },
            WeightedExplosionParticle {
                value: ExplosionParticleInfo {
                    particle: RawParticleOptions {
                        particle_id: 2,
                        data: vec![0xcc],
                    },
                    scaling: 0.5,
                    speed: 1.5,
                },
                weight: 127,
            },
        ],
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    let expected = [
        1.0_f64.to_be_bytes().to_vec(),
        (-2.0_f64).to_be_bytes().to_vec(),
        3.5_f64.to_be_bytes().to_vec(),
        4.25_f32.to_be_bytes().to_vec(),
        6_i32.to_be_bytes().to_vec(),
        vec![1],
        0.25_f64.to_be_bytes().to_vec(),
        0.5_f64.to_be_bytes().to_vec(),
        (-0.75_f64).to_be_bytes().to_vec(),
        vec![
            0xac, 0x02, 0xaa, 0xbb, // explosion particle: VarInt id 300 + raw options
            6,    // registered sound holder id 5 encoded as id + 1
            2,    // WeightedList size
            1,    // first block particle id
        ],
        2.0_f32.to_be_bytes().to_vec(),
        3.0_f32.to_be_bytes().to_vec(),
        vec![
            7, // first weight
            2, 0xcc, // second block particle id + raw options
        ],
        0.5_f32.to_be_bytes().to_vec(),
        1.5_f32.to_be_bytes().to_vec(),
        vec![127],
    ]
    .concat();

    assert_eq!(payload, expected);
}

fn assert_explode_java_sentinels() {
    for sentinel in [
        "Vec3.STREAM_CODEC",
        "ByteBufCodecs.FLOAT",
        "ByteBufCodecs.INT",
        "Vec3.STREAM_CODEC.apply(ByteBufCodecs::optional)",
        "ParticleTypes.STREAM_CODEC",
        "SoundEvent.STREAM_CODEC",
        "WeightedList.streamCodec(ExplosionParticleInfo.STREAM_CODEC)",
        "return GamePacketTypes.CLIENTBOUND_EXPLODE;",
        "listener.handleExplosion(this);",
    ] {
        assert!(
            CLIENTBOUND_EXPLODE_JAVA.contains(sentinel),
            "missing ClientboundExplodePacket sentinel {sentinel}"
        );
    }
    for sentinel in [
        "ParticleTypes.STREAM_CODEC",
        "ExplosionParticleInfo::particle",
        "ByteBufCodecs.FLOAT",
        "ExplosionParticleInfo::scaling",
        "ExplosionParticleInfo::speed",
    ] {
        assert!(
            EXPLOSION_PARTICLE_INFO_JAVA.contains(sentinel),
            "missing ExplosionParticleInfo sentinel {sentinel}"
        );
    }
    assert!(
        WEIGHTED_LIST_JAVA.contains(
            "Weighted.streamCodec(elementCodec).apply(ByteBufCodecs.list()).map(WeightedList::of, WeightedList::unwrap)"
        ),
        "missing WeightedList stream codec sentinel"
    );
    assert!(
        WEIGHTED_JAVA.contains(
            "StreamCodec.composite(valueCodec, Weighted::value, ByteBufCodecs.VAR_INT, Weighted::weight, Weighted::new)"
        ),
        "missing Weighted stream codec sentinel"
    );
    assert!(
        SOUND_EVENT_JAVA.contains(
            "public static final StreamCodec<RegistryFriendlyByteBuf, Holder<SoundEvent>> STREAM_CODEC = ByteBufCodecs.holder(Registries.SOUND_EVENT, DIRECT_STREAM_CODEC);"
        ),
        "missing SoundEvent holder codec sentinel"
    );
    for sentinel in [
        "private static final int DIRECT_HOLDER_ID = 0;",
        "id == 0 ? Holder.direct(directCodec.decode(input))",
        "VarInt.write(output, id + 1);",
        "VarInt.write(output, 0);",
    ] {
        assert!(
            BYTE_BUF_CODECS_JAVA.contains(sentinel),
            "missing ByteBufCodecs holder sentinel {sentinel}"
        );
    }
}

#[test]
fn clientbound_explode_packet_matches_java_codec_without_knockback_and_with_direct_sound() {
    let packet = ClientboundExplodePacket {
        center: Vec3 {
            x: 0.0,
            y: 64.0,
            z: 0.0,
        },
        radius: 0.0,
        block_count: -1,
        player_knockback: None,
        explosion_particle: RawParticleOptions {
            particle_id: 1,
            data: Vec::new(),
        },
        explosion_sound: SoundEventHolder::Direct {
            location: Identifier::new("minecraft", "entity.generic.explode").unwrap(),
            fixed_range: Some(16.0),
        },
        block_particles: Vec::new(),
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    let sound_name = "minecraft:entity.generic.explode".as_bytes();
    let expected = [
        0.0_f64.to_be_bytes().to_vec(),
        64.0_f64.to_be_bytes().to_vec(),
        0.0_f64.to_be_bytes().to_vec(),
        0.0_f32.to_be_bytes().to_vec(),
        (-1_i32).to_be_bytes().to_vec(),
        vec![
            0, // absent optional knockback
            1, // simple explosion particle id
            0, // direct sound holder sentinel
            sound_name.len() as u8,
        ],
        sound_name.to_vec(),
        vec![1], // fixed sound range present
        16.0_f32.to_be_bytes().to_vec(),
        vec![0], // empty WeightedList
    ]
    .concat();

    assert_eq!(payload, expected);
}
