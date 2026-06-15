use super::*;

const CLIENTBOUND_INITIALIZE_BORDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundInitializeBorderPacket.java");
const CLIENTBOUND_LEVEL_EVENT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundLevelEventPacket.java");
const CLIENTBOUND_LOW_DISK_SPACE_WARNING_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundLowDiskSpaceWarningPacket.java");
const CLIENTBOUND_MOUNT_SCREEN_OPEN_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundMountScreenOpenPacket.java");
const CLIENTBOUND_OPEN_BOOK_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundOpenBookPacket.java");
const CLIENTBOUND_OPEN_SCREEN_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundOpenScreenPacket.java");
const CLIENTBOUND_OPEN_SIGN_EDITOR_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundOpenSignEditorPacket.java");

#[test]
fn ui_world_clientbound_packet_codecs_match_java_sources() {
    assert_initialize_border();
    assert_level_event();
    assert_low_disk_space_warning();
    assert_mount_screen_open();
    assert_open_book();
    assert_open_screen();
    assert_open_sign_editor();
}

fn assert_initialize_border() {
    assert_java_contains(
        CLIENTBOUND_INITIALIZE_BORDER_JAVA,
        &[
            "this.newCenterX = input.readDouble();",
            "this.lerpTime = input.readVarLong();",
            "output.writeVarInt(this.warningTime);",
            "return GamePacketTypes.CLIENTBOUND_INITIALIZE_BORDER;",
            "listener.handleInitializeBorder(this);",
        ],
    );
    assert_registry(CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID, "initialize_border");

    let mut payload = Vec::new();
    ClientboundInitializeBorderPacket {
        new_center_x: 12.5,
        new_center_z: -34.25,
        old_size: 100.0,
        new_size: 200.0,
        lerp_time: 300,
        new_absolute_max_size: 400,
        warning_blocks: 5,
        warning_time: 6,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(&payload[..8], &12.5_f64.to_be_bytes());
    assert_eq!(&payload[32..], &[0xac, 0x02, 0x90, 0x03, 5, 6]);
}

fn assert_level_event() {
    assert_java_contains(
        CLIENTBOUND_LEVEL_EVENT_JAVA,
        &[
            "this.type = input.readInt();",
            "this.pos = input.readBlockPos();",
            "output.writeBlockPos(this.pos);",
            "this.globalEvent = input.readBoolean();",
            "listener.handleLevelEvent(this);",
        ],
    );
    assert_registry(CLIENTBOUND_LEVEL_EVENT_PACKET_ID, "level_event");

    let mut payload = Vec::new();
    ClientboundLevelEventPacket {
        event_type: 2001,
        x: -2,
        y: 64,
        z: 3,
        data: 42,
        global_event: true,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(&payload[..4], &2001_i32.to_be_bytes());
    assert_eq!(
        &payload[4..12],
        &pack_block_position(-2, 64, 3).to_be_bytes()
    );
    assert_eq!(&payload[12..16], &42_i32.to_be_bytes());
    assert_eq!(payload[16], 1);
}

fn assert_low_disk_space_warning() {
    assert_java_contains(
        CLIENTBOUND_LOW_DISK_SPACE_WARNING_JAVA,
        &[
            "public static final ClientboundLowDiskSpaceWarningPacket INSTANCE",
            "StreamCodec.unit(INSTANCE)",
            "return GamePacketTypes.CLIENTBOUND_LOW_DISK_SPACE_WARNING;",
            "listener.handleLowDiskSpaceWarning(this);",
        ],
    );
    assert_registry(
        CLIENTBOUND_LOW_DISK_SPACE_WARNING_PACKET_ID,
        "low_disk_space_warning",
    );

    let mut payload = Vec::new();
    ClientboundLowDiskSpaceWarningPacket
        .write(&mut payload)
        .unwrap();
    assert!(payload.is_empty());
    assert_eq!(
        ClientboundLowDiskSpaceWarningPacket::read(&mut cursor(payload)).unwrap(),
        ClientboundLowDiskSpaceWarningPacket
    );
}

fn assert_mount_screen_open() {
    assert_java_contains(
        CLIENTBOUND_MOUNT_SCREEN_OPEN_JAVA,
        &[
            "this.containerId = input.readContainerId();",
            "this.inventoryColumns = input.readVarInt();",
            "this.entityId = input.readInt();",
            "return GamePacketTypes.CLIENTBOUND_MOUNT_SCREEN_OPEN;",
            "listener.handleMountScreenOpen(this);",
        ],
    );
    assert_registry(CLIENTBOUND_MOUNT_SCREEN_OPEN_PACKET_ID, "mount_screen_open");

    let mut payload = Vec::new();
    ClientboundMountScreenOpenPacket {
        container_id: 128,
        inventory_columns: 3,
        entity_id: -123,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![0x80, 0x01, 3, 0xff, 0xff, 0xff, 0x85]);
}

fn assert_open_book() {
    assert_java_contains(
        CLIENTBOUND_OPEN_BOOK_JAVA,
        &[
            "this.hand = input.readEnum(InteractionHand.class);",
            "output.writeEnum(this.hand);",
            "return GamePacketTypes.CLIENTBOUND_OPEN_BOOK;",
            "listener.handleOpenBook(this);",
            "public InteractionHand getHand()",
        ],
    );
    assert_registry(CLIENTBOUND_OPEN_BOOK_PACKET_ID, "open_book");

    let packet = ClientboundOpenBookPacket {
        hand: ClientboundInteractionHand::OffHand,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![1]);
    assert_eq!(
        ClientboundOpenBookPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
    assert!(ClientboundOpenBookPacket::read(&mut cursor(vec![2])).is_err());
}

fn assert_open_screen() {
    assert_java_contains(
        CLIENTBOUND_OPEN_SCREEN_JAVA,
        &[
            "ByteBufCodecs.CONTAINER_ID",
            "ByteBufCodecs.registry(Registries.MENU)",
            "ComponentSerialization.TRUSTED_STREAM_CODEC",
            "return GamePacketTypes.CLIENTBOUND_OPEN_SCREEN;",
            "listener.handleOpenScreen(this);",
        ],
    );
    assert_registry(CLIENTBOUND_OPEN_SCREEN_PACKET_ID, "open_screen");

    let mut payload = Vec::new();
    ClientboundOpenScreenPacket {
        container_id: 300,
        menu_type_id: 12,
        title: Tag::Compound(vec![(
            "translate".to_string(),
            Tag::String("container.crafting".to_string()),
        )]),
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(&payload[..3], &[0xac, 0x02, 12]);
    assert!(payload.ends_with(&[0]));
}

fn assert_open_sign_editor() {
    assert_java_contains(
        CLIENTBOUND_OPEN_SIGN_EDITOR_JAVA,
        &[
            "this.pos = input.readBlockPos();",
            "this.isFrontText = input.readBoolean();",
            "output.writeBlockPos(this.pos);",
            "output.writeBoolean(this.isFrontText);",
            "listener.handleOpenSignEditor(this);",
        ],
    );
    assert_registry(CLIENTBOUND_OPEN_SIGN_EDITOR_PACKET_ID, "open_sign_editor");

    let packet = ClientboundOpenSignEditorPacket {
        x: -4,
        y: 70,
        z: 8,
        is_front_text: false,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        &payload[..8],
        &pack_block_position(-4, 70, 8).to_be_bytes()
    );
    assert_eq!(payload[8], 0);
    assert_eq!(
        ClientboundOpenSignEditorPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_registry(packet_id: i32, name: &str) {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(registry.clientbound_name(packet_id), Some(name));
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "Java source missing sentinel: {sentinel}"
        );
    }
}
