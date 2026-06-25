use super::*;

const CLIENTBOUND_LEVEL_PARTICLES_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundLevelParticlesPacket.java");

fn particle_prefix(particle_id: i32, particle_data: Vec<u8>) -> Vec<u8> {
    let mut payload = Vec::new();
    ClientboundLevelParticlesPacket {
        particle_id,
        override_limiter: true,
        always_show: false,
        position: Vec3 {
            x: 1.25,
            y: -2.5,
            z: 3.75,
        },
        offset: Vec3 {
            x: 0.125,
            y: 0.25,
            z: 0.5,
        },
        max_speed: 1.5,
        count: -7,
        particle_data,
    }
    .write(&mut payload)
    .unwrap();
    payload
}

fn fixed_fields_prefix() -> Vec<u8> {
    let mut expected = Vec::new();
    expected.extend_from_slice(&[1, 0]);
    expected.extend_from_slice(&1.25_f64.to_be_bytes());
    expected.extend_from_slice(&(-2.5_f64).to_be_bytes());
    expected.extend_from_slice(&3.75_f64.to_be_bytes());
    expected.extend_from_slice(&0.125_f32.to_be_bytes());
    expected.extend_from_slice(&0.25_f32.to_be_bytes());
    expected.extend_from_slice(&0.5_f32.to_be_bytes());
    expected.extend_from_slice(&1.5_f32.to_be_bytes());
    expected.extend_from_slice(&(-7_i32).to_be_bytes());
    expected
}

#[test]
fn clientbound_level_particles_packet_matches_java_fixed_fields_and_simple_particle_codec() {
    assert_java_contains(
        CLIENTBOUND_LEVEL_PARTICLES_JAVA,
        &[
            "this.overrideLimiter = input.readBoolean();",
            "this.alwaysShow = input.readBoolean();",
            "this.x = input.readDouble();",
            "this.y = input.readDouble();",
            "this.z = input.readDouble();",
            "this.xDist = input.readFloat();",
            "this.yDist = input.readFloat();",
            "this.zDist = input.readFloat();",
            "this.maxSpeed = input.readFloat();",
            "this.count = input.readInt();",
            "this.particle = ParticleTypes.STREAM_CODEC.decode(input);",
            "ParticleTypes.STREAM_CODEC.encode(output, this.particle);",
            "return GamePacketTypes.CLIENTBOUND_LEVEL_PARTICLES;",
            "listener.handleParticleEvent(this);",
        ],
    );
    assert_eq!(CLIENTBOUND_LEVEL_PARTICLES_PACKET_ID, 47);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_LEVEL_PARTICLES_PACKET_ID),
        Some("level_particles")
    );

    let mut expected = fixed_fields_prefix();
    // Java ParticleTypes registration order: angry_villager is the first simple particle.
    write_var_i32(&mut expected, 0).unwrap();

    assert_eq!(particle_prefix(0, Vec::new()), expected);
}

#[test]
fn clientbound_level_particles_packet_encodes_java_particle_specific_payloads() {
    let mut block_data = Vec::new();
    // Java BlockParticleOption uses ByteBufCodecs.idMapper(Block.BLOCK_STATE_REGISTRY).
    write_var_i32(&mut block_data, 300).unwrap();
    let mut expected_block = fixed_fields_prefix();
    // Java ParticleTypes registration order: block is ID 1.
    write_var_i32(&mut expected_block, 1).unwrap();
    expected_block.extend_from_slice(&block_data);
    assert_eq!(particle_prefix(1, block_data), expected_block);

    let mut dust_data = Vec::new();
    // Java DustParticleOptions writes RGB int then scale float.
    dust_data.extend_from_slice(&0x00ff_8800_i32.to_be_bytes());
    dust_data.extend_from_slice(&0.75_f32.to_be_bytes());
    let mut expected_dust = fixed_fields_prefix();
    // Java ParticleTypes registration order: dust is ID 14.
    write_var_i32(&mut expected_dust, 14).unwrap();
    expected_dust.extend_from_slice(&dust_data);
    assert_eq!(particle_prefix(14, dust_data), expected_dust);
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing ClientboundLevelParticlesPacket sentinel {sentinel}"
        );
    }
}
