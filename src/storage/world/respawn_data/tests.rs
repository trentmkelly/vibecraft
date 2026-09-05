use super::*;
use crate::network::play::ClientboundSetDefaultSpawnPositionPacket;

#[test]
fn factory_and_default_match_java_rotations() {
    let default = LevelRespawnData::default();
    assert_eq!(default.dimension.to_string(), "minecraft:overworld");
    assert_eq!(
        (default.x, default.y, default.z, default.yaw, default.pitch),
        (0, 0, 0, 0.0, 0.0)
    );
    for (input, expected) in [
        (180.0, -180.0),
        (-180.0, -180.0),
        (540.0, -180.0),
        (-541.0, 179.0),
        (725.0, 5.0),
    ] {
        let value = LevelRespawnData::of(default.dimension.clone(), (-3, 75, 9), input, 120.0);
        assert_eq!(value.yaw, expected);
        assert_eq!(value.pitch, 90.0);
        assert_eq!((value.x, value.y, value.z), (-3, 75, 9));
    }
    let value = LevelRespawnData::of(default.dimension.clone(), (0, 0, 0), -0.0, -120.0);
    assert_eq!(value.yaw.to_bits(), (-0.0f32).to_bits());
    assert_eq!(value.pitch, -90.0);
    let value = LevelRespawnData::of(default.dimension, (0, 0, 0), f32::INFINITY, f32::NAN);
    assert!(value.yaw.is_nan());
    assert!(value.pitch.is_nan());
}

#[test]
fn persisted_record_preserves_dimension_position_and_both_rotations() {
    let value = LevelRespawnData {
        dimension: Identifier::parse("custom:sky").unwrap(),
        x: -13,
        y: 320,
        z: 17,
        yaw: 180.0,
        pitch: -90.0,
    };
    let tag = value.to_nbt().unwrap();
    assert_eq!(
        tag,
        Tag::Compound(vec![
            ("dimension".to_owned(), Tag::String("custom:sky".to_owned())),
            ("pos".to_owned(), Tag::IntArray(vec![-13, 320, 17])),
            ("yaw".to_owned(), Tag::Float(180.0)),
            ("pitch".to_owned(), Tag::Float(-90.0)),
        ])
    );
    // The codec accepts +180; it does not call the normalizing factory.
    assert_eq!(LevelRespawnData::from_nbt(&tag).unwrap(), value);
    let packet = ClientboundSetDefaultSpawnPositionPacket {
        respawn_data: value.clone(),
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    assert_eq!(
        ClientboundSetDefaultSpawnPositionPacket::read(&mut bytes.as_slice())
            .unwrap()
            .respawn_data,
        value
    );
}

#[test]
fn persisted_codec_requires_fields_and_validates_without_clamping() {
    let Tag::Compound(fields) = LevelRespawnData::default().to_nbt().unwrap() else {
        panic!("compound")
    };
    for index in 0..fields.len() {
        let mut missing = fields.clone();
        missing.remove(index);
        assert!(LevelRespawnData::from_nbt(&Tag::Compound(missing)).is_err());
    }
    for (name, invalid) in [
        ("dimension", Tag::String("Bad:dimension".to_owned())),
        ("pos", Tag::IntArray(vec![1, 2])),
        ("pos", Tag::IntArray(vec![1, 2, 3, 4])),
        ("yaw", Tag::Float(180.1)),
        ("pitch", Tag::Float(-90.1)),
        ("yaw", Tag::Float(f32::NAN)),
        ("pitch", Tag::Float(f32::INFINITY)),
        ("pitch", Tag::String("0".to_owned())),
    ] {
        let mut invalid_fields = fields.clone();
        invalid_fields
            .iter_mut()
            .find(|(key, _)| key == name)
            .unwrap()
            .1 = invalid;
        assert!(
            LevelRespawnData::from_nbt(&Tag::Compound(invalid_fields)).is_err(),
            "{name}"
        );
    }
    let invalid = LevelRespawnData {
        pitch: 91.0,
        ..Default::default()
    };
    assert!(invalid.to_nbt().is_err());
    // STREAM_CODEC transports raw floats; only the persisted codec enforces ranges.
    let packet = ClientboundSetDefaultSpawnPositionPacket {
        respawn_data: invalid,
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    assert_eq!(
        ClientboundSetDefaultSpawnPositionPacket::read(&mut bytes.as_slice()).unwrap(),
        packet
    );
}

#[test]
fn nbt_number_conversion_and_extra_fields_follow_map_codec() {
    let value = Tag::Compound(vec![
        ("dimension".to_owned(), Tag::String("overworld".to_owned())),
        (
            "pos".to_owned(),
            Tag::List(vec![
                Tag::Long(4_294_967_297),
                Tag::Double(-2.9),
                Tag::Byte(3),
            ]),
        ),
        ("yaw".to_owned(), Tag::Int(45)),
        ("pitch".to_owned(), Tag::Double(-12.5)),
        ("unused".to_owned(), Tag::Byte(1)),
    ]);
    let decoded = LevelRespawnData::from_nbt(&value).unwrap();
    assert_eq!((decoded.x, decoded.y, decoded.z), (1, -2, 3));
    assert_eq!((decoded.yaw, decoded.pitch), (45.0, -12.5));
    assert_eq!(decoded.dimension.to_string(), "minecraft:overworld");
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn source_contract_covers_factory_defaults_and_codec_boundaries() {
    const JAVA: &str = vibecraft_java_source!("/net/minecraft/world/level/storage/LevelData.java");
    for fragment in [
        "GlobalPos.of(Level.OVERWORLD, BlockPos.ZERO), 0.0F, 0.0F",
        "Codec.floatRange(-180.0F, 180.0F)",
        "Codec.floatRange(-90.0F, 90.0F)",
        "pos.immutable()",
        "Mth.wrapDegrees(yaw)",
        "Mth.clamp(pitch, -90.0F, 90.0F)",
        "GlobalPos.STREAM_CODEC",
    ] {
        assert!(JAVA.contains(fragment), "missing Java contract: {fragment}");
    }
}

#[test]
fn primary_level_data_loads_and_saves_the_spawn_compound() {
    use crate::storage::world::PrimaryLevelData;
    let mut fields = vec![
        (
            "DataVersion".to_owned(),
            Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        ),
        (
            "Version".to_owned(),
            Tag::Compound(vec![
                (
                    "Id".to_owned(),
                    Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
                ),
                ("Name".to_owned(), Tag::String("26.1.2".to_owned())),
            ]),
        ),
        ("LevelName".to_owned(), Tag::String("spawn test".to_owned())),
        ("GameType".to_owned(), Tag::Int(0)),
        ("Difficulty".to_owned(), Tag::Byte(2)),
        // Legacy fields are ignored by PrimaryLevelData.parse at this version.
        ("SpawnX".to_owned(), Tag::Int(123)),
    ];
    let wrap = |fields| Tag::Compound(vec![("Data".to_owned(), Tag::Compound(fields))]);
    let missing = PrimaryLevelData::from_level_dat(&wrap(fields.clone())).unwrap();
    assert_eq!(missing.spawn, LevelRespawnData::default());
    fields.push(("spawn".to_owned(), Tag::String("invalid".to_owned())));
    let invalid = PrimaryLevelData::from_level_dat(&wrap(fields.clone())).unwrap();
    assert_eq!(invalid.spawn, LevelRespawnData::default());

    let expected = LevelRespawnData::of(
        Identifier::parse("custom:sky").unwrap(),
        (20, 90, -45),
        181.0,
        25.0,
    );
    fields.last_mut().unwrap().1 = expected.to_nbt().unwrap();
    let mut loaded = PrimaryLevelData::from_level_dat(&wrap(fields)).unwrap();
    assert_eq!(loaded.spawn, expected);
    let saved = loaded.to_level_dat().unwrap();
    assert_eq!(
        PrimaryLevelData::from_level_dat(&saved).unwrap().spawn,
        expected
    );
    let Tag::Compound(root) = saved else {
        panic!("root")
    };
    let Tag::Compound(data) = &root[0].1 else {
        panic!("data")
    };
    assert!(data
        .iter()
        .any(|(key, tag)| key == "spawn" && tag == &expected.to_nbt().unwrap()));
    assert!(!data
        .iter()
        .any(|(key, _)| ["SpawnX", "SpawnY", "SpawnZ", "SpawnAngle"].contains(&key.as_str())));
    loaded.spawn.pitch = 91.0;
    assert!(loaded.to_level_dat().is_err());
}
