#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketPackageCoverage {
    pub java_package: &'static str,
    pub packet_class_count: usize,
    pub rust_modules: &'static [&'static str],
}

pub const TOTAL_PACKET_CLASSES_26_1_2: usize = 227;
pub const PLAY_PACKET_SPEC_COUNT_26_1_2: usize = 210;
pub const PLAYBOUND_PACKET_SPEC_COUNT_26_1_2: usize = 69;
pub const CLIENTBOUND_PACKET_SPEC_COUNT_26_1_2: usize = 141;

// Source: net.minecraft protocol packet interfaces under decompiled-server-26.1.2/net/minecraft/network/protocol
pub const PACKET_PACKAGE_COVERAGE_26_1_2: &[PacketPackageCoverage] = &[
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol",
        packet_class_count: 3,
        rust_modules: &["network::bundle", "network::dispatch", "network::pipeline"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/common",
        packet_class_count: 19,
        rust_modules: &[
            "network::common",
            "network::cookie",
            "network::ping",
            "network::transfer",
        ],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/configuration",
        packet_class_count: 7,
        rust_modules: &["network::configuration"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/cookie",
        packet_class_count: 2,
        rust_modules: &["network::cookie"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/game",
        packet_class_count: 182,
        rust_modules: &["network::play"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/handshake",
        packet_class_count: 1,
        rust_modules: &["network::handshake"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/login",
        packet_class_count: 9,
        rust_modules: &["network::login"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/ping",
        packet_class_count: 2,
        rust_modules: &["network::ping"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/status",
        packet_class_count: 2,
        rust_modules: &["network::status"],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayPacketSpec {
    pub id: i32,
    pub direction: crate::network::dispatch::PacketDirection,
    pub wire_name: &'static str,
    pub java_class: &'static str,
    pub field_order: &'static str,
}

pub const PLAY_PACKET_SPECS_26_1_2: &[PlayPacketSpec] = &[
    PlayPacketSpec {
        id: 0,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "accept_teleportation",
        java_class: "ServerboundAcceptTeleportationPacket",
        field_order: "teleport_id:var_int",
    },
    PlayPacketSpec {
        id: 1,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "attack",
        java_class: "ServerboundAttackPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 2,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "block_entity_tag_query",
        java_class: "ServerboundBlockEntityTagQueryPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 3,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "bundle_item_selected",
        java_class: "ServerboundSelectBundleItemPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 4,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "change_difficulty",
        java_class: "ServerboundChangeDifficultyPacket",
        field_order: "difficulty:difficulty_enum",
    },
    PlayPacketSpec {
        id: 5,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "change_game_mode",
        java_class: "ServerboundChangeGameModePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 6,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "chat_ack",
        java_class: "ServerboundChatAckPacket",
        field_order: "offset:VarInt",
    },
    PlayPacketSpec {
        id: 7,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "chat_command",
        java_class: "ServerboundChatCommandPacket",
        field_order: "command:String max 32767",
    },
    PlayPacketSpec {
        id: 8,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "chat_command_signed",
        java_class: "ServerboundChatCommandSignedPacket",
        field_order: "command:String max 32767, timestamp:Instant i64 epoch millis, salt:i64, argument_signatures:collection max 8 of name:String max 16 + 256-byte signature, last_seen:offset VarInt + fixed 20-bit acknowledged bitset + checksum byte",
    },
    PlayPacketSpec {
        id: 9,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "chat",
        java_class: "ServerboundChatPacket",
        field_order: "message:String max 256, timestamp:Instant i64 epoch millis, salt:i64, nullable 256-byte signature, last_seen:offset VarInt + fixed 20-bit acknowledged bitset + checksum byte",
    },
    PlayPacketSpec {
        id: 10,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "chat_session_update",
        java_class: "ServerboundChatSessionUpdatePacket",
        field_order: "session_id:UUID, expires_at:Instant i64 epoch millis, public_key:byte array max 512, key_signature:byte array max 4096",
    },
    PlayPacketSpec {
        id: 11,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "chunk_batch_received",
        java_class: "ServerboundChunkBatchReceivedPacket",
        field_order: "desired_chunks_per_tick:f32",
    },
    PlayPacketSpec {
        id: 12,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "client_command",
        java_class: "ServerboundClientCommandPacket",
        field_order: "action:enum",
    },
    PlayPacketSpec {
        id: 13,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "client_tick_end",
        java_class: "ServerboundClientTickEndPacket",
        field_order: "empty_payload",
    },
    PlayPacketSpec {
        id: 14,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "client_information",
        java_class: "ServerboundClientInformationPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 15,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "command_suggestion",
        java_class: "ServerboundCommandSuggestionPacket",
        field_order: "id:VarInt, command:utf8 max 32500",
    },
    PlayPacketSpec {
        id: 16,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "configuration_acknowledged",
        java_class: "ServerboundConfigurationAcknowledgedPacket",
        field_order: "empty_payload",
    },
    PlayPacketSpec {
        id: 17,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "container_button_click",
        java_class: "ServerboundContainerButtonClickPacket",
        field_order: "container_id:CONTAINER_ID, button_id:VarInt",
    },
    PlayPacketSpec {
        id: 18,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "container_click",
        java_class: "ServerboundContainerClickPacket",
        field_order: "container_id:CONTAINER_ID VarInt, state_id:VarInt, slot_num:i16, button_num:i8, container_input:VarInt id mapper, changed_slots:map max 128 of slot i16 to HashedStack, carried_item:HashedStack",
    },
    PlayPacketSpec {
        id: 19,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "container_close",
        java_class: "ServerboundContainerClosePacket",
        field_order: "container_id:VarInt",
    },
    PlayPacketSpec {
        id: 20,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "container_slot_state_changed",
        java_class: "ServerboundContainerSlotStateChangedPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 21,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "cookie_response",
        java_class: "ServerboundCookieResponsePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 22,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "custom_payload",
        java_class: "ServerboundCustomPayloadPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 23,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "debug_subscription_request",
        java_class: "ServerboundDebugSubscriptionRequestPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 24,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "edit_book",
        java_class: "ServerboundEditBookPacket",
        field_order: "slot:VarInt, pages:list max 100 of utf8 max 1024, title:optional utf8 max 32",
    },
    PlayPacketSpec {
        id: 25,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "entity_tag_query",
        java_class: "ServerboundEntityTagQueryPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 26,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "interact",
        java_class: "ServerboundInteractPacket",
        field_order: "entity_id:VarInt, hand:InteractionHand enum VarInt, location:LpVec3, using_secondary_action:bool",
    },
    PlayPacketSpec {
        id: 27,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "jigsaw_generate",
        java_class: "ServerboundJigsawGeneratePacket",
        field_order: "pos:BlockPos, levels:VarInt, keep_jigsaws:bool",
    },
    PlayPacketSpec {
        id: 28,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "keep_alive",
        java_class: "ServerboundKeepAlivePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 29,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "lock_difficulty",
        java_class: "ServerboundLockDifficultyPacket",
        field_order: "locked:bool",
    },
    PlayPacketSpec {
        id: 30,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "move_player_pos",
        java_class: "ServerboundMovePlayerPacket.Pos",
        field_order: "x:f64, y:f64, z:f64, flags:u8",
    },
    PlayPacketSpec {
        id: 31,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "move_player_pos_rot",
        java_class: "ServerboundMovePlayerPacket.PosRot",
        field_order: "x:f64, y:f64, z:f64, y_rot:f32, x_rot:f32, flags:u8",
    },
    PlayPacketSpec {
        id: 32,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "move_player_rot",
        java_class: "ServerboundMovePlayerPacket.Rot",
        field_order: "y_rot:f32, x_rot:f32, flags:u8",
    },
    PlayPacketSpec {
        id: 33,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "move_player_status_only",
        java_class: "ServerboundMovePlayerPacket.StatusOnly",
        field_order: "flags:u8",
    },
    PlayPacketSpec {
        id: 34,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "move_vehicle",
        java_class: "ServerboundMoveVehiclePacket",
        field_order: "position:Vec3, y_rot:f32, x_rot:f32, on_ground:bool",
    },
    PlayPacketSpec {
        id: 35,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "paddle_boat",
        java_class: "ServerboundPaddleBoatPacket",
        field_order: "left:bool, right:bool",
    },
    PlayPacketSpec {
        id: 36,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "pick_item_from_block",
        java_class: "ServerboundPickItemFromBlockPacket",
        field_order: "pos:BlockPos, include_data:bool",
    },
    PlayPacketSpec {
        id: 37,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "pick_item_from_entity",
        java_class: "ServerboundPickItemFromEntityPacket",
        field_order: "id:VarInt, include_data:bool",
    },
    PlayPacketSpec {
        id: 38,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "ping_request",
        java_class: "ServerboundPingRequestPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 39,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "place_recipe",
        java_class: "ServerboundPlaceRecipePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 40,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "player_abilities",
        java_class: "ServerboundPlayerAbilitiesPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 41,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "player_action",
        java_class: "ServerboundPlayerActionPacket",
        field_order: "action:enum VarInt, pos:BlockPos, direction:u8, sequence:VarInt",
    },
    PlayPacketSpec {
        id: 42,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "player_command",
        java_class: "ServerboundPlayerCommandPacket",
        field_order: "id:VarInt, action:enum VarInt, data:VarInt",
    },
    PlayPacketSpec {
        id: 43,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "player_input",
        java_class: "ServerboundPlayerInputPacket",
        field_order: "input:Input.STREAM_CODEC",
    },
    PlayPacketSpec {
        id: 44,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "player_loaded",
        java_class: "ServerboundPlayerLoadedPacket",
        field_order: "empty_payload",
    },
    PlayPacketSpec {
        id: 45,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "pong",
        java_class: "ServerboundPongPacket",
        field_order: "id:i32_be",
    },
    PlayPacketSpec {
        id: 46,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "recipe_book_change_settings",
        java_class: "ServerboundRecipeBookChangeSettingsPacket",
        field_order: "book_type:RecipeBookType enum VarInt, is_open:bool, is_filtering:bool",
    },
    PlayPacketSpec {
        id: 47,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "recipe_book_seen_recipe",
        java_class: "ServerboundRecipeBookSeenRecipePacket",
        field_order: "recipe:RecipeDisplayId VarInt index",
    },
    PlayPacketSpec {
        id: 48,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "rename_item",
        java_class: "ServerboundRenameItemPacket",
        field_order: "name:utf(32767)",
    },
    PlayPacketSpec {
        id: 49,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "resource_pack",
        java_class: "ServerboundResourcePackPacket",
        field_order: "id:UUID, action:ResourcePackAction enum VarInt",
    },
    PlayPacketSpec {
        id: 50,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "seen_advancements",
        java_class: "ServerboundSeenAdvancementsPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 51,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "select_trade",
        java_class: "ServerboundSelectTradePacket",
        field_order: "item:VarInt",
    },
    PlayPacketSpec {
        id: 52,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "set_beacon",
        java_class: "ServerboundSetBeaconPacket",
        field_order: "primary:optional MobEffect.STREAM_CODEC, secondary:optional MobEffect.STREAM_CODEC",
    },
    PlayPacketSpec {
        id: 53,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "set_carried_item",
        java_class: "ServerboundSetCarriedItemPacket",
        field_order: "slot:i16_be",
    },
    PlayPacketSpec {
        id: 54,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "set_command_block",
        java_class: "ServerboundSetCommandBlockPacket",
        field_order: "pos:BlockPos, command:utf8 max 32767, mode:CommandBlockEntity.Mode enum VarInt, flags:u8(track_output=1, conditional=2, automatic=4)",
    },
    PlayPacketSpec {
        id: 55,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "set_command_minecart",
        java_class: "ServerboundSetCommandMinecartPacket",
        field_order: "entity:VarInt, command:utf8 max 32767, track_output:bool",
    },
    PlayPacketSpec {
        id: 56,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "set_creative_mode_slot",
        java_class: "ServerboundSetCreativeModeSlotPacket",
        field_order: "slot_num:i16, item_stack:validated ItemStack.OPTIONAL_UNTRUSTED_STREAM_CODEC encoded as count VarInt, optional item registry id, delimited data component patch",
    },
    PlayPacketSpec {
        id: 57,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "set_game_rule",
        java_class: "ServerboundSetGameRulePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 58,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "set_jigsaw_block",
        java_class: "ServerboundSetJigsawBlockPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 59,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "set_structure_block",
        java_class: "ServerboundSetStructureBlockPacket",
        field_order: "pos:BlockPos, update_type:enum VarInt, mode:StructureMode enum VarInt, name:utf8, offset:3 i8 clamped -48..48, size:3 i8 clamped 0..48, mirror:enum VarInt, rotation:enum VarInt wrap, data:utf8 max 128, integrity:f32 clamped 0..1, seed:VarLong, flags:u8(ignore_entities=1, show_air=2, show_bounding_box=4, strict=8)",
    },
    PlayPacketSpec {
        id: 60,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "set_test_block",
        java_class: "ServerboundSetTestBlockPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 61,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "sign_update",
        java_class: "ServerboundSignUpdatePacket",
        field_order: "pos:BlockPos, is_front_text:bool, lines:[utf(384);4]",
    },
    PlayPacketSpec {
        id: 62,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "spectate_entity",
        java_class: "ServerboundSpectateEntityPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 63,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "swing",
        java_class: "ServerboundSwingPacket",
        field_order: "hand:enum",
    },
    PlayPacketSpec {
        id: 64,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "teleport_to_entity",
        java_class: "ServerboundTeleportToEntityPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 65,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "test_instance_block_action",
        java_class: "ServerboundTestInstanceBlockActionPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 66,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "use_item_on",
        java_class: "ServerboundUseItemOnPacket",
        field_order: "hand:enum VarInt, block_hit:BlockHitResult, sequence:VarInt",
    },
    PlayPacketSpec {
        id: 67,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "use_item",
        java_class: "ServerboundUseItemPacket",
        field_order: "hand:enum VarInt, sequence:VarInt, y_rot:f32, x_rot:f32",
    },
    PlayPacketSpec {
        id: 68,
        direction: crate::network::dispatch::PacketDirection::Serverbound,
        wire_name: "custom_click_action",
        java_class: "ServerboundCustomClickActionPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 0,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "bundle",
        java_class: "ClientboundBundlePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 1,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "add_entity",
        java_class: "ClientboundAddEntityPacket",
        field_order:
            "id:VarInt, uuid:UUID, type:EntityType registry VarInt, x:double, y:double, z:double, movement:Vec3.LP_STREAM_CODEC, xRot:byte, yRot:byte, yHeadRot:byte, data:VarInt",
    },
    PlayPacketSpec {
        id: 2,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "animate",
        java_class: "ClientboundAnimatePacket",
        field_order: "id:var_int, action:u8",
    },
    PlayPacketSpec {
        id: 3,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "award_stats",
        java_class: "ClientboundAwardStatsPacket",
        field_order: "stats:Map<Stat(stat_type:VarInt, stat_value:VarInt), value:VarInt>",
    },
    PlayPacketSpec {
        id: 4,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "block_changed_ack",
        java_class: "ClientboundBlockChangedAckPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 5,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "block_destruction",
        java_class: "ClientboundBlockDestructionPacket",
        field_order: "id:VarInt, pos:BlockPos, progress:u8",
    },
    PlayPacketSpec {
        id: 6,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "block_entity_data",
        java_class: "ClientboundBlockEntityDataPacket",
        field_order: "pos:BlockPos, type:registry VarInt, tag:trusted compound tag",
    },
    PlayPacketSpec {
        id: 7,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "block_event",
        java_class: "ClientboundBlockEventPacket",
        field_order: "pos:BlockPos, action:u8, param:u8, block:registry VarInt",
    },
    PlayPacketSpec {
        id: 8,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "block_update",
        java_class: "ClientboundBlockUpdatePacket",
        field_order: "pos:BlockPos, block_state:Block.BLOCK_STATE_REGISTRY VarInt",
    },
    PlayPacketSpec {
        id: 9,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "boss_event",
        java_class: "ClientboundBossEventPacket",
        field_order: "id:UUID, operation:OperationType enum VarInt, payload by operation: add(name:trusted Component network NBT tag, progress:float, color enum VarInt, overlay enum VarInt, flags byte), remove(), update_progress(float), update_name(trusted Component network NBT tag), update_style(color enum VarInt, overlay enum VarInt), update_properties(flags byte)",
    },
    PlayPacketSpec {
        id: 10,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "change_difficulty",
        java_class: "ClientboundChangeDifficultyPacket",
        field_order: "difficulty:difficulty_enum, locked:bool",
    },
    PlayPacketSpec {
        id: 11,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "chunk_batch_finished",
        java_class: "ClientboundChunkBatchFinishedPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 12,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "chunk_batch_start",
        java_class: "ClientboundChunkBatchStartPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 13,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "chunks_biomes",
        java_class: "ClientboundChunksBiomesPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 14,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "clear_titles",
        java_class: "ClientboundClearTitlesPacket",
        field_order: "reset_times:bool",
    },
    PlayPacketSpec {
        id: 15,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "command_suggestions",
        java_class: "ClientboundCommandSuggestionsPacket",
        field_order: "id:VarInt, start:VarInt, length:VarInt, suggestions:List<Entry(text:String, tooltip:Optional<trusted Component network NBT tag>)>",
    },
    PlayPacketSpec {
        id: 16,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "commands",
        java_class: "ClientboundCommandsPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 17,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "container_close",
        java_class: "ClientboundContainerClosePacket",
        field_order: "container_id:CONTAINER_ID VarInt",
    },
    PlayPacketSpec {
        id: 18,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "container_set_content",
        java_class: "ClientboundContainerSetContentPacket",
        field_order: "containerId:CONTAINER_ID VarInt, stateId:VarInt, items:ItemStack.OPTIONAL_LIST_STREAM_CODEC, carriedItem:ItemStack.OPTIONAL_STREAM_CODEC",
    },
    PlayPacketSpec {
        id: 19,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "container_set_data",
        java_class: "ClientboundContainerSetDataPacket",
        field_order: "container_id:CONTAINER_ID VarInt, id:short, value:short",
    },
    PlayPacketSpec {
        id: 20,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "container_set_slot",
        java_class: "ClientboundContainerSetSlotPacket",
        field_order: "containerId:CONTAINER_ID VarInt, stateId:VarInt, slot:short, itemStack:ItemStack.OPTIONAL_STREAM_CODEC",
    },
    PlayPacketSpec {
        id: 21,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "cookie_request",
        java_class: "ClientboundCookieRequestPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 22,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "cooldown",
        java_class: "ClientboundCooldownPacket",
        field_order: "cooldown_group:Identifier, duration:VarInt",
    },
    PlayPacketSpec {
        id: 23,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "custom_chat_completions",
        java_class: "ClientboundCustomChatCompletionsPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 24,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "custom_payload",
        java_class: "ClientboundCustomPayloadPacket",
        field_order: "payload:CustomPacketPayload (channel Identifier, minecraft:brand string or unknown payload up to 1 MiB)",
    },
    PlayPacketSpec {
        id: 25,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "damage_event",
        java_class: "ClientboundDamageEventPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 26,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "debug_block_value",
        java_class: "ClientboundDebugBlockValuePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 27,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "debug_chunk_value",
        java_class: "ClientboundDebugChunkValuePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 28,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "debug_entity_value",
        java_class: "ClientboundDebugEntityValuePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 29,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "debug_event",
        java_class: "ClientboundDebugEventPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 30,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "debug_sample",
        java_class: "ClientboundDebugSamplePacket",
        field_order: "sample:long_array(VarInt length + i64 entries), debug_sample_type:enum VarInt",
    },
    PlayPacketSpec {
        id: 31,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "delete_chat",
        java_class: "ClientboundDeleteChatPacket",
        field_order: "message_signature:MessageSignature.Packed (VarInt id+1, full 256-byte signature when id is -1)",
    },
    PlayPacketSpec {
        id: 32,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "disconnect",
        java_class: "ClientboundDisconnectPacket",
        field_order: "reason:ComponentSerialization.TRUSTED_CONTEXT_FREE_STREAM_CODEC as network NBT tag",
    },
    PlayPacketSpec {
        id: 33,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "disguised_chat",
        java_class: "ClientboundDisguisedChatPacket",
        field_order: "message:ComponentSerialization.TRUSTED_STREAM_CODEC as network NBT tag, chatType:ChatType.Bound(chatType:Holder<ChatType> id+1, name:trusted Component network NBT tag, targetName:Optional<trusted Component network NBT tag>)",
    },
    PlayPacketSpec {
        id: 34,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "entity_event",
        java_class: "ClientboundEntityEventPacket",
        field_order: "entity_id:int, event_id:byte",
    },
    PlayPacketSpec {
        id: 35,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "entity_position_sync",
        java_class: "ClientboundEntityPositionSyncPacket",
        field_order: "id:VarInt, values:PositionMoveRotation(position Vec3, deltaMovement Vec3, yRot float, xRot float), on_ground:bool",
    },
    PlayPacketSpec {
        id: 36,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "explode",
        java_class: "ClientboundExplodePacket",
        field_order: "center:Vec3, radius:float, block_count:int, player_knockback:Optional<Vec3>, explosion_particle:ParticleTypes.STREAM_CODEC, explosion_sound:SoundEvent.STREAM_CODEC, block_particles:WeightedList<ExplosionParticleInfo(particle, scaling:float, speed:float)>",
    },
    PlayPacketSpec {
        id: 37,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "forget_level_chunk",
        java_class: "ClientboundForgetLevelChunkPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 38,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "game_event",
        java_class: "ClientboundGameEventPacket",
        field_order: "event:u8, param:f32",
    },
    PlayPacketSpec {
        id: 39,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "game_rule_values",
        java_class: "ClientboundGameRuleValuesPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 40,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "game_test_highlight_pos",
        java_class: "ClientboundGameTestHighlightPosPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 41,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "mount_screen_open",
        java_class: "ClientboundMountScreenOpenPacket",
        field_order: "container_id:VarInt, inventory_columns:VarInt, entity_id:int",
    },
    PlayPacketSpec {
        id: 42,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "hurt_animation",
        java_class: "ClientboundHurtAnimationPacket",
        field_order: "id:var_int, yaw:f32",
    },
    PlayPacketSpec {
        id: 43,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "initialize_border",
        java_class: "ClientboundInitializeBorderPacket",
        field_order: "new_center_x:f64, new_center_z:f64, old_size:f64, new_size:f64, lerp_time:VarLong, new_absolute_max_size:VarInt, warning_blocks:VarInt, warning_time:VarInt",
    },
    PlayPacketSpec {
        id: 44,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "keep_alive",
        java_class: "ClientboundKeepAlivePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 45,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "level_chunk_with_light",
        java_class: "ClientboundLevelChunkWithLightPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 46,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "level_event",
        java_class: "ClientboundLevelEventPacket",
        field_order: "type:int, pos:BlockPos, data:int, global_event:bool",
    },
    PlayPacketSpec {
        id: 47,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "level_particles",
        java_class: "ClientboundLevelParticlesPacket",
        field_order: "override_limiter:bool, always_show:bool, x/y/z:double, x/y/z_dist:float, max_speed:float, count:int, particle:ParticleTypes.STREAM_CODEC (registry id VarInt + particle-specific payload)",
    },
    PlayPacketSpec {
        id: 48,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "light_update",
        java_class: "ClientboundLightUpdatePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 49,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "login",
        java_class: "ClientboundLoginPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 50,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "low_disk_space_warning",
        java_class: "ClientboundLowDiskSpaceWarningPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 51,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "map_item_data",
        java_class: "ClientboundMapItemDataPacket",
        field_order: "mapId:VarInt, scale:byte, locked:bool, decorations:Optional<List(type:Holder<MapDecorationType> registry VarInt, x:byte, y:byte, rot:byte, name:Optional<Component network NBT tag>)>, colorPatch:width byte (0 absent) then height/startX/startY bytes and colors byte array",
    },
    PlayPacketSpec {
        id: 52,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "merchant_offers",
        java_class: "ClientboundMerchantOffersPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 53,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "move_entity_pos",
        java_class: "ClientboundMoveEntityPacket.Pos",
        field_order: "entity_id:var_int, xa:short, ya:short, za:short, on_ground:bool",
    },
    PlayPacketSpec {
        id: 54,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "move_entity_pos_rot",
        java_class: "ClientboundMoveEntityPacket.PosRot",
        field_order:
            "entity_id:var_int, xa:short, ya:short, za:short, y_rot:byte, x_rot:byte, on_ground:bool",
    },
    PlayPacketSpec {
        id: 55,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "move_minecart_along_track",
        java_class: "ClientboundMoveMinecartPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 56,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "move_entity_rot",
        java_class: "ClientboundMoveEntityPacket.Rot",
        field_order: "entity_id:var_int, y_rot:byte, x_rot:byte, on_ground:bool",
    },
    PlayPacketSpec {
        id: 57,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "move_vehicle",
        java_class: "ClientboundMoveVehiclePacket",
        field_order: "position:Vec3.STREAM_CODEC, y_rot:f32, x_rot:f32",
    },
    PlayPacketSpec {
        id: 58,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "open_book",
        java_class: "ClientboundOpenBookPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 59,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "open_screen",
        java_class: "ClientboundOpenScreenPacket",
        field_order: "containerId:ContainerId VarInt, type:MenuType registry VarInt, title:trusted Component network NBT tag",
    },
    PlayPacketSpec {
        id: 60,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "open_sign_editor",
        java_class: "ClientboundOpenSignEditorPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 61,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "ping",
        java_class: "ClientboundPingPacket",
        field_order: "id:i32_be",
    },
    PlayPacketSpec {
        id: 62,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "pong_response",
        java_class: "ClientboundPongResponsePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 63,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "place_ghost_recipe",
        java_class: "ClientboundPlaceGhostRecipePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 64,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "player_abilities",
        java_class: "ClientboundPlayerAbilitiesPacket",
        field_order: "flags:byte, flying_speed:f32, walking_speed:f32",
    },
    PlayPacketSpec {
        id: 65,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "player_chat",
        java_class: "ClientboundPlayerChatPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 66,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "player_combat_end",
        java_class: "ClientboundPlayerCombatEndPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 67,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "player_combat_enter",
        java_class: "ClientboundPlayerCombatEnterPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 68,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "player_combat_kill",
        java_class: "ClientboundPlayerCombatKillPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 69,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "player_info_remove",
        java_class: "ClientboundPlayerInfoRemovePacket",
        field_order: "profile_ids:list UUID",
    },
    PlayPacketSpec {
        id: 70,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "player_info_update",
        java_class: "ClientboundPlayerInfoUpdatePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 71,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "player_look_at",
        java_class: "ClientboundPlayerLookAtPacket",
        field_order: "from_anchor:enum VarInt, x:double, y:double, z:double, at_entity:bool, optional entity:VarInt + to_anchor:enum VarInt",
    },
    PlayPacketSpec {
        id: 72,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "player_position",
        java_class: "ClientboundPlayerPositionPacket",
        field_order: "id:VarInt, change:PositionMoveRotation(position Vec3, deltaMovement Vec3, yRot float, xRot float), relatives:Set<Relative> as fixed int",
    },
    PlayPacketSpec {
        id: 73,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "player_rotation",
        java_class: "ClientboundPlayerRotationPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 74,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "recipe_book_add",
        java_class: "ClientboundRecipeBookAddPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 75,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "recipe_book_remove",
        java_class: "ClientboundRecipeBookRemovePacket",
        field_order: "recipes:list RecipeDisplayId(index:VarInt)",
    },
    PlayPacketSpec {
        id: 76,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "recipe_book_settings",
        java_class: "ClientboundRecipeBookSettingsPacket",
        field_order: "crafting(open:bool, filtering:bool), furnace(open:bool, filtering:bool), blast_furnace(open:bool, filtering:bool), smoker(open:bool, filtering:bool)",
    },
    PlayPacketSpec {
        id: 77,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "remove_entities",
        java_class: "ClientboundRemoveEntitiesPacket",
        field_order: "entity_ids:int_id_list",
    },
    PlayPacketSpec {
        id: 78,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "remove_mob_effect",
        java_class: "ClientboundRemoveMobEffectPacket",
        field_order: "entity_id:VarInt, effect:MobEffect registry holder VarInt",
    },
    PlayPacketSpec {
        id: 79,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "reset_score",
        java_class: "ClientboundResetScorePacket",
        field_order: "owner:utf, objective_name:nullable utf",
    },
    PlayPacketSpec {
        id: 80,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "resource_pack_pop",
        java_class: "ClientboundResourcePackPopPacket",
        field_order: "id:optional UUID",
    },
    PlayPacketSpec {
        id: 81,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "resource_pack_push",
        java_class: "ClientboundResourcePackPushPacket",
        field_order: "id:UUID, url:String, hash:String(max 40), required:bool, prompt:Optional<trusted context-free Component network NBT tag>",
    },
    PlayPacketSpec {
        id: 82,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "respawn",
        java_class: "ClientboundRespawnPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 83,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "rotate_head",
        java_class: "ClientboundRotateHeadPacket",
        field_order: "entity_id:var_int, y_head_rot:byte",
    },
    PlayPacketSpec {
        id: 84,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "section_blocks_update",
        java_class: "ClientboundSectionBlocksUpdatePacket",
        field_order: "section_pos:long, updates:VarInt count then VarLong(block_state_id << 12 | packed_section_pos)",
    },
    PlayPacketSpec {
        id: 85,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "select_advancements_tab",
        java_class: "ClientboundSelectAdvancementsTabPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 86,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "server_data",
        java_class: "ClientboundServerDataPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 87,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_action_bar_text",
        java_class: "ClientboundSetActionBarTextPacket",
        field_order: "text:ComponentSerialization.TRUSTED_STREAM_CODEC as network NBT tag",
    },
    PlayPacketSpec {
        id: 88,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_border_center",
        java_class: "ClientboundSetBorderCenterPacket",
        field_order: "new_center_x:f64, new_center_z:f64",
    },
    PlayPacketSpec {
        id: 89,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_border_lerp_size",
        java_class: "ClientboundSetBorderLerpSizePacket",
        field_order: "old_size:f64, new_size:f64, lerp_time:VarLong",
    },
    PlayPacketSpec {
        id: 90,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_border_size",
        java_class: "ClientboundSetBorderSizePacket",
        field_order: "size:f64",
    },
    PlayPacketSpec {
        id: 91,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_border_warning_delay",
        java_class: "ClientboundSetBorderWarningDelayPacket",
        field_order: "warning_delay:VarInt",
    },
    PlayPacketSpec {
        id: 92,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_border_warning_distance",
        java_class: "ClientboundSetBorderWarningDistancePacket",
        field_order: "warning_blocks:VarInt",
    },
    PlayPacketSpec {
        id: 93,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_camera",
        java_class: "ClientboundSetCameraPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 94,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_chunk_cache_center",
        java_class: "ClientboundSetChunkCacheCenterPacket",
        field_order: "x:var_int, z:var_int",
    },
    PlayPacketSpec {
        id: 95,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_chunk_cache_radius",
        java_class: "ClientboundSetChunkCacheRadiusPacket",
        field_order: "radius:var_int",
    },
    PlayPacketSpec {
        id: 96,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_cursor_item",
        java_class: "ClientboundSetCursorItemPacket",
        field_order: "contents:ItemStack.OPTIONAL_STREAM_CODEC",
    },
    PlayPacketSpec {
        id: 97,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_default_spawn_position",
        java_class: "ClientboundSetDefaultSpawnPositionPacket",
        field_order: "respawn_data:LevelData.RespawnData.STREAM_CODEC",
    },
    PlayPacketSpec {
        id: 98,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_display_objective",
        java_class: "ClientboundSetDisplayObjectivePacket",
        field_order: "slot:DisplaySlot enum VarInt, objective_name:utf",
    },
    PlayPacketSpec {
        id: 99,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_entity_data",
        java_class: "ClientboundSetEntityDataPacket",
        field_order:
            "id:var_int, packed_items:(index:u8, serializer_id:var_int, serializer payload)*, eof:0xff",
    },
    PlayPacketSpec {
        id: 100,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_entity_link",
        java_class: "ClientboundSetEntityLinkPacket",
        field_order: "source_id:int, dest_id:int",
    },
    PlayPacketSpec {
        id: 101,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_entity_motion",
        java_class: "ClientboundSetEntityMotionPacket",
        field_order: "id:var_int, movement:Vec3.LP_STREAM_CODEC",
    },
    PlayPacketSpec {
        id: 102,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_equipment",
        java_class: "ClientboundSetEquipmentPacket",
        field_order: "entity:VarInt, repeated slot byte (high bit continues, low 7 bits EquipmentSlot ordinal) + ItemStack.OPTIONAL_STREAM_CODEC",
    },
    PlayPacketSpec {
        id: 103,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_experience",
        java_class: "ClientboundSetExperiencePacket",
        field_order: "experience_progress:f32, experience_level:var_int, total_experience:var_int",
    },
    PlayPacketSpec {
        id: 104,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_health",
        java_class: "ClientboundSetHealthPacket",
        field_order: "health:f32, food:var_int, saturation:f32",
    },
    PlayPacketSpec {
        id: 105,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_held_slot",
        java_class: "ClientboundSetHeldSlotPacket",
        field_order: "slot:var_int",
    },
    PlayPacketSpec {
        id: 106,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_objective",
        java_class: "ClientboundSetObjectivePacket",
        field_order: "objectiveName:String, method:byte, if add/change displayName:trusted Component network NBT tag, renderType:ObjectiveCriteria.RenderType enum VarInt, numberFormat:Optional<NumberFormatTypes.STREAM_CODEC>",
    },
    PlayPacketSpec {
        id: 107,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_passengers",
        java_class: "ClientboundSetPassengersPacket",
        field_order: "vehicle:var_int, passengers:var_int_array",
    },
    PlayPacketSpec {
        id: 108,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_player_inventory",
        java_class: "ClientboundSetPlayerInventoryPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 109,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_player_team",
        java_class: "ClientboundSetPlayerTeamPacket",
        field_order: "name:String, method:byte, if create/update parameters(displayName:trusted Component network NBT tag, options:byte, nametagVisibility:Team.Visibility VarInt, collisionRule:Team.CollisionRule VarInt, color:ChatFormatting enum VarInt, prefix:trusted Component network NBT tag, suffix:trusted Component network NBT tag), if create/join/leave players:List<String>",
    },
    PlayPacketSpec {
        id: 110,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_score",
        java_class: "ClientboundSetScorePacket",
        field_order: "owner:String, objectiveName:String, score:VarInt, display:Optional<trusted Component network NBT tag>, numberFormat:Optional<NumberFormatTypes.STREAM_CODEC>",
    },
    PlayPacketSpec {
        id: 111,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_simulation_distance",
        java_class: "ClientboundSetSimulationDistancePacket",
        field_order: "simulation_distance:var_int",
    },
    PlayPacketSpec {
        id: 112,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_subtitle_text",
        java_class: "ClientboundSetSubtitleTextPacket",
        field_order: "text:ComponentSerialization.TRUSTED_STREAM_CODEC as network NBT tag",
    },
    PlayPacketSpec {
        id: 113,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_time",
        java_class: "ClientboundSetTimePacket",
        field_order: "game_time:i64, clock_updates:map",
    },
    PlayPacketSpec {
        id: 114,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_title_text",
        java_class: "ClientboundSetTitleTextPacket",
        field_order: "text:ComponentSerialization.TRUSTED_STREAM_CODEC as network NBT tag",
    },
    PlayPacketSpec {
        id: 115,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "set_titles_animation",
        java_class: "ClientboundSetTitlesAnimationPacket",
        field_order: "fade_in:int, stay:int, fade_out:int",
    },
    PlayPacketSpec {
        id: 116,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "sound_entity",
        java_class: "ClientboundSoundEntityPacket",
        field_order: "sound:Holder<SoundEvent> (registered id+1 or direct id 0 + Identifier + optional fixed_range), source:SoundSource enum VarInt, entity_id:VarInt, volume:float, pitch:float, seed:long",
    },
    PlayPacketSpec {
        id: 117,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "sound",
        java_class: "ClientboundSoundPacket",
        field_order: "sound:Holder<SoundEvent> (registered id+1 or direct id 0 + Identifier + optional fixed_range), source:SoundSource enum VarInt, x/y/z:int fixed point (*8), volume:float, pitch:float, seed:long",
    },
    PlayPacketSpec {
        id: 118,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "start_configuration",
        java_class: "ClientboundStartConfigurationPacket",
        field_order: "unit/no fields; terminal play packet",
    },
    PlayPacketSpec {
        id: 119,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "stop_sound",
        java_class: "ClientboundStopSoundPacket",
        field_order: "flags:byte bit0 has_source bit1 has_sound, optional source:SoundSource enum VarInt, optional name:Identifier",
    },
    PlayPacketSpec {
        id: 120,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "store_cookie",
        java_class: "ClientboundStoreCookiePacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 121,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "system_chat",
        java_class: "ClientboundSystemChatPacket",
        field_order: "content:ComponentSerialization.TRUSTED_STREAM_CODEC as network NBT tag, overlay:bool",
    },
    PlayPacketSpec {
        id: 122,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "tab_list",
        java_class: "ClientboundTabListPacket",
        field_order: "header:trusted Component network NBT tag, footer:trusted Component network NBT tag",
    },
    PlayPacketSpec {
        id: 123,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "tag_query",
        java_class: "ClientboundTagQueryPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 124,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "take_item_entity",
        java_class: "ClientboundTakeItemEntityPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 125,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "teleport_entity",
        java_class: "ClientboundTeleportEntityPacket",
        field_order:
            "id:var_int, change:PositionMoveRotation.STREAM_CODEC, relatives:Set<Relative>, on_ground:bool",
    },
    PlayPacketSpec {
        id: 126,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "test_instance_block_status",
        java_class: "ClientboundTestInstanceBlockStatus",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 127,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "ticking_state",
        java_class: "ClientboundTickingStatePacket",
        field_order: "tick_rate:f32, is_frozen:bool",
    },
    PlayPacketSpec {
        id: 128,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "ticking_step",
        java_class: "ClientboundTickingStepPacket",
        field_order: "tick_steps:var_int",
    },
    PlayPacketSpec {
        id: 129,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "transfer",
        java_class: "ClientboundTransferPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 130,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "update_advancements",
        java_class: "ClientboundUpdateAdvancementsPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 131,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "update_attributes",
        java_class: "ClientboundUpdateAttributesPacket",
        field_order: "entity_id:VarInt, attributes:List(attribute:Holder<Attribute> registry VarInt, base:double, modifiers:List(id:Identifier, amount:double, operation:VarInt))",
    },
    PlayPacketSpec {
        id: 132,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "update_mob_effect",
        java_class: "ClientboundUpdateMobEffectPacket",
        field_order: "entity_id:VarInt, effect:MobEffect registry holder VarInt, amplifier:VarInt, duration_ticks:VarInt, flags:byte",
    },
    PlayPacketSpec {
        id: 133,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "update_recipes",
        java_class: "ClientboundUpdateRecipesPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 134,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "update_tags",
        java_class: "ClientboundUpdateTagsPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 135,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "projectile_power",
        java_class: "ClientboundProjectilePowerPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 136,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "custom_report_details",
        java_class: "ClientboundCustomReportDetailsPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 137,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "server_links",
        java_class: "ClientboundServerLinksPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 138,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "waypoint",
        java_class: "ClientboundTrackedWaypointPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 139,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "clear_dialog",
        java_class: "ClientboundClearDialogPacket",
        field_order: "unparsed",
    },
    PlayPacketSpec {
        id: 140,
        direction: crate::network::dispatch::PacketDirection::Clientbound,
        wire_name: "show_dialog",
        java_class: "ClientboundShowDialogPacket",
        field_order: "unparsed",
    },
];

pub fn play_packet_specs_26_1_2() -> &'static [PlayPacketSpec] {
    PLAY_PACKET_SPECS_26_1_2
}

pub fn covered_packet_class_count() -> usize {
    PACKET_PACKAGE_COVERAGE_26_1_2
        .iter()
        .map(|entry| entry.packet_class_count)
        .sum()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        covered_packet_class_count, play_packet_specs_26_1_2, CLIENTBOUND_PACKET_SPEC_COUNT_26_1_2,
        PACKET_PACKAGE_COVERAGE_26_1_2, PLAYBOUND_PACKET_SPEC_COUNT_26_1_2,
        PLAY_PACKET_SPEC_COUNT_26_1_2, TOTAL_PACKET_CLASSES_26_1_2,
    };
    use crate::network::configuration::{
        ClientboundCodeOfConductPacket, ClientboundFinishConfigurationPacket,
        ClientboundRegistryDataPacket, ClientboundResetChatPacket,
        ClientboundUpdateEnabledFeaturesPacket, ServerboundAcceptCodeOfConductPacket,
        ServerboundFinishConfigurationPacket,
    };
    use crate::network::dispatch::PacketDirection;
    use crate::network::login::{
        CLIENTBOUND_COOKIE_REQUEST_PACKET_ID, CLIENTBOUND_LOGIN_FINISHED_PACKET_ID,
        SERVERBOUND_COOKIE_RESPONSE_PACKET_ID, SERVERBOUND_HELLO_PACKET_ID,
    };
    use crate::network::play::{
        PlayProtocolRegistry, CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
        CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2, SERVERBOUND_PLAY_PACKET_COUNT_26_1_2,
    };
    use crate::registry::Identifier;

    #[test]
    fn protocol_packet_class_manifest_matches_decompiled_26_1_2_tree() {
        assert_eq!(covered_packet_class_count(), TOTAL_PACKET_CLASSES_26_1_2);
        assert_eq!(PACKET_PACKAGE_COVERAGE_26_1_2.len(), 9);
        assert!(PACKET_PACKAGE_COVERAGE_26_1_2.iter().any(|entry| entry
            .java_package
            .ends_with("/game")
            && entry.packet_class_count == 182
            && entry.rust_modules == ["network::play"]));
    }

    #[test]
    fn protocol_state_registries_back_the_packet_class_manifest() {
        assert_eq!(SERVERBOUND_PLAY_PACKET_COUNT_26_1_2, 69);
        assert_eq!(CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2, 141);
        let _configuration_packets = (
            ClientboundCodeOfConductPacket {
                code_of_conduct: String::new(),
            },
            ClientboundFinishConfigurationPacket,
            ClientboundRegistryDataPacket {
                registry: Identifier::parse("minecraft:root").unwrap(),
                entries: Vec::new(),
            },
            ClientboundResetChatPacket,
            ClientboundUpdateEnabledFeaturesPacket {
                features: Vec::new(),
            },
            ServerboundAcceptCodeOfConductPacket,
            ServerboundFinishConfigurationPacket,
        );
        assert_eq!(SERVERBOUND_HELLO_PACKET_ID, 0);
        assert_eq!(SERVERBOUND_COOKIE_RESPONSE_PACKET_ID, 4);
        assert_eq!(CLIENTBOUND_LOGIN_FINISHED_PACKET_ID, 2);
        assert_eq!(CLIENTBOUND_COOKIE_REQUEST_PACKET_ID, 5);
    }

    #[test]
    fn play_packet_specification_is_complete_and_contiguous() {
        let specs = play_packet_specs_26_1_2();
        assert_eq!(specs.len(), PLAY_PACKET_SPEC_COUNT_26_1_2);
        assert_eq!(
            specs.len(),
            PLAYBOUND_PACKET_SPEC_COUNT_26_1_2 + CLIENTBOUND_PACKET_SPEC_COUNT_26_1_2
        );

        let serverbound_ids: Vec<_> = specs
            .iter()
            .filter(|entry| entry.direction == PacketDirection::Serverbound)
            .map(|entry| entry.id)
            .collect();
        assert_eq!(serverbound_ids.len(), PLAYBOUND_PACKET_SPEC_COUNT_26_1_2);
        let expected_serverbound =
            (0..(PLAYBOUND_PACKET_SPEC_COUNT_26_1_2 as i32)).collect::<Vec<_>>();
        let mut serverbound_ids = serverbound_ids;
        serverbound_ids.sort_unstable();
        assert_eq!(serverbound_ids, expected_serverbound);

        let clientbound_ids: Vec<_> = specs
            .iter()
            .filter(|entry| entry.direction == PacketDirection::Clientbound)
            .map(|entry| entry.id)
            .collect();
        assert_eq!(clientbound_ids.len(), CLIENTBOUND_PACKET_SPEC_COUNT_26_1_2);
        let expected_clientbound =
            (0..(CLIENTBOUND_PACKET_SPEC_COUNT_26_1_2 as i32)).collect::<Vec<_>>();
        let mut clientbound_ids = clientbound_ids;
        clientbound_ids.sort_unstable();
        assert_eq!(clientbound_ids, expected_clientbound);
    }

    #[test]
    fn play_packet_specification_has_no_duplicates_per_direction() {
        let specs = play_packet_specs_26_1_2();
        let mut seen_serverbound = HashSet::new();
        let mut seen_clientbound = HashSet::new();
        for spec in specs {
            match spec.direction {
                PacketDirection::Serverbound => {
                    assert!(
                        seen_serverbound.insert(spec.id),
                        "duplicate serverbound packet id {}",
                        spec.id
                    );
                }
                PacketDirection::Clientbound => {
                    assert!(
                        seen_clientbound.insert(spec.id),
                        "duplicate clientbound packet id {}",
                        spec.id
                    );
                }
            }
        }
        assert_eq!(seen_serverbound.len(), PLAYBOUND_PACKET_SPEC_COUNT_26_1_2);
        assert_eq!(seen_clientbound.len(), CLIENTBOUND_PACKET_SPEC_COUNT_26_1_2);
    }

    #[test]
    fn play_packet_specification_matches_play_registries() {
        let specs = play_packet_specs_26_1_2();
        let registry = PlayProtocolRegistry::new();
        for spec in specs {
            match spec.direction {
                PacketDirection::Serverbound => {
                    assert_eq!(Some(spec.wire_name), registry.serverbound_name(spec.id));
                }
                PacketDirection::Clientbound => {
                    assert_eq!(Some(spec.wire_name), registry.clientbound_name(spec.id));
                }
            }
        }
    }

    #[test]
    fn play_packet_specification_keeps_critical_ids_fixed() {
        let specs = play_packet_specs_26_1_2();
        let registry = PlayProtocolRegistry::new();
        let keep_alive = specs
            .iter()
            .find(|entry| {
                entry.direction == PacketDirection::Clientbound && entry.wire_name == "keep_alive"
            })
            .unwrap();
        assert_eq!(keep_alive.id, CLIENTBOUND_KEEP_ALIVE_PACKET_ID);

        let level_chunk = specs
            .iter()
            .find(|entry| {
                entry.direction == PacketDirection::Clientbound
                    && entry.wire_name == "level_chunk_with_light"
            })
            .unwrap();
        let level_chunk_id = (0..(CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2 as i32))
            .find(|id| registry.clientbound_name(*id) == Some("level_chunk_with_light"))
            .expect("clientbound level_chunk_with_light should exist in play registry");
        assert_eq!(level_chunk.id, level_chunk_id);
    }

    #[test]
    fn play_packet_specification_entry_fields_are_present() {
        let specs = play_packet_specs_26_1_2();
        for spec in specs {
            assert!(
                !spec.wire_name.is_empty(),
                "missing wire name for {:?}",
                spec.direction
            );
            assert!(
                !spec.java_class.is_empty(),
                "missing java class for {spec:?}"
            );
            assert!(
                !spec.field_order.is_empty(),
                "missing field order for {spec:?}"
            );
        }
    }

    #[test]
    fn play_packet_specification_scalar_fields_are_concretely_decoded() {
        let specs = play_packet_specs_26_1_2();
        let checks: &[(PacketDirection, &str, &str)] = &[
            (
                PacketDirection::Serverbound,
                "change_difficulty",
                "difficulty:difficulty_enum",
            ),
            (
                PacketDirection::Serverbound,
                "chunk_batch_received",
                "desired_chunks_per_tick:f32",
            ),
            (
                PacketDirection::Serverbound,
                "client_command",
                "action:enum",
            ),
            (
                PacketDirection::Serverbound,
                "client_tick_end",
                "empty_payload",
            ),
            (
                PacketDirection::Serverbound,
                "lock_difficulty",
                "locked:bool",
            ),
            (
                PacketDirection::Serverbound,
                "paddle_boat",
                "left:bool, right:bool",
            ),
            (
                PacketDirection::Serverbound,
                "player_input",
                "input:Input.STREAM_CODEC",
            ),
            (
                PacketDirection::Serverbound,
                "player_loaded",
                "empty_payload",
            ),
            (
                PacketDirection::Serverbound,
                "set_carried_item",
                "slot:i16_be",
            ),
            (PacketDirection::Serverbound, "swing", "hand:enum"),
            (
                PacketDirection::Clientbound,
                "change_difficulty",
                "difficulty:difficulty_enum, locked:bool",
            ),
            (
                PacketDirection::Clientbound,
                "set_chunk_cache_center",
                "x:var_int, z:var_int",
            ),
            (
                PacketDirection::Clientbound,
                "set_chunk_cache_radius",
                "radius:var_int",
            ),
            (
                PacketDirection::Clientbound,
                "set_default_spawn_position",
                "respawn_data:LevelData.RespawnData.STREAM_CODEC",
            ),
            (
                PacketDirection::Clientbound,
                "set_experience",
                "experience_progress:f32, experience_level:var_int, total_experience:var_int",
            ),
            (
                PacketDirection::Clientbound,
                "set_health",
                "health:f32, food:var_int, saturation:f32",
            ),
            (
                PacketDirection::Clientbound,
                "set_held_slot",
                "slot:var_int",
            ),
            (
                PacketDirection::Clientbound,
                "set_simulation_distance",
                "simulation_distance:var_int",
            ),
            (
                PacketDirection::Clientbound,
                "set_time",
                "game_time:i64, clock_updates:map",
            ),
            (
                PacketDirection::Clientbound,
                "game_event",
                "event:u8, param:f32",
            ),
            (
                PacketDirection::Clientbound,
                "ticking_state",
                "tick_rate:f32, is_frozen:bool",
            ),
            (
                PacketDirection::Clientbound,
                "ticking_step",
                "tick_steps:var_int",
            ),
        ];

        for (direction, wire_name, expected) in checks {
            let spec = specs
                .iter()
                .find(|entry| entry.direction == *direction && entry.wire_name == *wire_name)
                .unwrap_or_else(|| panic!("missing manifest entry for {direction:?} {wire_name}"));
            assert_eq!(
                spec.field_order, *expected,
                "{direction:?} {wire_name} field-order drift"
            );
        }
    }

    #[test]
    fn protocol_package_manifest_is_stable_and_non_empty() {
        assert_eq!(
            PACKET_PACKAGE_COVERAGE_26_1_2.first().unwrap().java_package,
            "net/minecraft/network/protocol",
        );
        assert_eq!(
            PACKET_PACKAGE_COVERAGE_26_1_2.last().unwrap().java_package,
            "net/minecraft/network/protocol/status",
        );

        let mut package_names = HashSet::new();
        for entry in PACKET_PACKAGE_COVERAGE_26_1_2 {
            assert!(
                package_names.insert(entry.java_package),
                "duplicated package in PACKET_PACKAGE_COVERAGE_26_1_2: {}",
                entry.java_package
            );
            assert_eq!(entry.java_package, entry.java_package.trim());
            assert!(
                entry.packet_class_count > 0,
                "zero packet count for {}",
                entry.java_package
            );
            assert!(
                !entry.rust_modules.is_empty(),
                "no rust modules for {}",
                entry.java_package
            );
        }
    }
}
