use super::*;

#[test]
fn clientbound_set_entity_data_packet_matches_java_packed_items_codec() {
    assert_eq!(CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID, 99);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID),
        Some("set_entity_data")
    );

    let mut payload = Vec::new();
    ClientboundSetEntityDataPacket {
        id: 300,
        packed_items: vec![
            EntityDataValue::typed(0, EntityMetadataValue::Byte(-1)).unwrap(),
            EntityDataValue::typed(1, EntityMetadataValue::VarInt(300)).unwrap(),
            EntityDataValue::typed(8, EntityMetadataValue::Boolean(true)).unwrap(),
            EntityDataValue::typed(19, EntityMetadataValue::OptionalUnsignedInt(Some(4))).unwrap(),
            EntityDataValue::raw(254, 127, vec![0xaa, 0xbb]),
        ],
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        vec![
            0xac, 0x02, // entity id
            0, 0, 0xff, // byte metadata
            1, 1, 0xac, 0x02, // VarInt metadata
            8, 8, 1, // boolean metadata
            19, 19, 5, // OptionalInt encodes value + 1, absent as zero
            254, 127, 0xaa, 0xbb, // highest legal metadata id before EOF
            0xff, // EOF marker
        ]
    );
}

#[test]
fn clientbound_set_entity_data_packet_writes_empty_list_eof_marker() {
    let mut payload = Vec::new();
    ClientboundSetEntityDataPacket {
        id: 1,
        packed_items: Vec::new(),
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(payload, vec![1, 0xff]);
}

#[test]
fn clientbound_set_entity_data_packet_rejects_reserved_eof_index() {
    let err = ClientboundSetEntityDataPacket {
        id: 1,
        packed_items: vec![EntityDataValue::raw(0xff, 0, vec![0])],
    }
    .write(&mut Vec::new())
    .unwrap_err();

    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn entity_data_accessor_and_serializer_registry_match_java_syncher_contracts() {
    const ACCESSOR_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/syncher/EntityDataAccessor.java");
    const SERIALIZER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/syncher/EntityDataSerializer.java");
    const SERIALIZERS_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/syncher/EntityDataSerializers.java");

    assert!(ACCESSOR_JAVA.contains("return this.id == that.id;"));
    assert!(ACCESSOR_JAVA.contains("return this.id;"));
    assert!(ACCESSOR_JAVA.contains("return \"<entity data: \" + this.id + \">\";"));
    assert!(SERIALIZER_JAVA
        .contains("return new EntityDataAccessor<>(id, this);"));
    assert!(SERIALIZER_JAVA.contains("static <T> EntityDataSerializer<T> forValueType"));
    assert!(SERIALIZER_JAVA.contains("default T copy(final T value)"));

    for serializer in JAVA_ENTITY_DATA_SERIALIZER_ORDER {
        assert!(
            SERIALIZERS_JAVA.contains(&format!("registerSerializer({serializer});")),
            "missing Java serializer registration for {serializer}"
        );
    }
    assert_eq!(JAVA_ENTITY_DATA_SERIALIZER_ORDER.len(), 43);
}

const JAVA_ENTITY_DATA_SERIALIZER_ORDER: &[&str] = &[
    "BYTE",
    "INT",
    "LONG",
    "FLOAT",
    "STRING",
    "COMPONENT",
    "OPTIONAL_COMPONENT",
    "ITEM_STACK",
    "BOOLEAN",
    "ROTATIONS",
    "BLOCK_POS",
    "OPTIONAL_BLOCK_POS",
    "DIRECTION",
    "OPTIONAL_LIVING_ENTITY_REFERENCE",
    "BLOCK_STATE",
    "OPTIONAL_BLOCK_STATE",
    "PARTICLE",
    "PARTICLES",
    "VILLAGER_DATA",
    "OPTIONAL_UNSIGNED_INT",
    "POSE",
    "CAT_VARIANT",
    "CAT_SOUND_VARIANT",
    "COW_VARIANT",
    "COW_SOUND_VARIANT",
    "WOLF_VARIANT",
    "WOLF_SOUND_VARIANT",
    "FROG_VARIANT",
    "PIG_VARIANT",
    "PIG_SOUND_VARIANT",
    "CHICKEN_VARIANT",
    "CHICKEN_SOUND_VARIANT",
    "ZOMBIE_NAUTILUS_VARIANT",
    "OPTIONAL_GLOBAL_POS",
    "PAINTING_VARIANT",
    "SNIFFER_STATE",
    "ARMADILLO_STATE",
    "COPPER_GOLEM_STATE",
    "WEATHERING_COPPER_STATE",
    "VECTOR3",
    "QUATERNION",
    "RESOLVABLE_PROFILE",
    "HUMANOID_ARM",
];
