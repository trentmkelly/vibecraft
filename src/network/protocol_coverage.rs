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

macro_rules! sb {
    ($id:literal, $name:literal, $java:literal, $fields:literal) => {
        PlayPacketSpec {
            id: $id,
            direction: crate::network::dispatch::PacketDirection::Serverbound,
            wire_name: $name,
            java_class: $java,
            field_order: $fields,
        }
    };
}

macro_rules! cb {
    ($id:literal, $name:literal, $java:literal, $fields:literal) => {
        PlayPacketSpec {
            id: $id,
            direction: crate::network::dispatch::PacketDirection::Clientbound,
            wire_name: $name,
            java_class: $java,
            field_order: $fields,
        }
    };
}

pub const PLAY_PACKET_SPECS_26_1_2: &[PlayPacketSpec] = &[
    sb!(0, "accept_teleportation", "ServerboundAcceptTeleportationPacket", "teleport_id:var_int"),
    sb!(1, "attack", "ServerboundAttackPacket", "entity_id:VarInt"),
    sb!(2, "block_entity_tag_query", "ServerboundBlockEntityTagQueryPacket", "transaction_id:VarInt, pos:BlockPos"),
    sb!(3, "bundle_item_selected", "ServerboundSelectBundleItemPacket", "slot_id:VarInt, selected_item_index:VarInt (-1 or >=0)"),
    sb!(4, "change_difficulty", "ServerboundChangeDifficultyPacket", "difficulty:difficulty_enum"),
    sb!(5, "change_game_mode", "ServerboundChangeGameModePacket", "mode:GameType enum VarInt"),
    sb!(6, "chat_ack", "ServerboundChatAckPacket", "offset:VarInt"),
    sb!(7, "chat_command", "ServerboundChatCommandPacket", "command:String max 32767"),
    sb!(8, "chat_command_signed", "ServerboundChatCommandSignedPacket", "command:String max 32767, timestamp:Instant i64 epoch millis, salt:i64, argument_signatures:collection max 8 of name:String max 16 + 256-byte signature, last_seen:offset VarInt + fixed 20-bit acknowledged bitset + checksum byte"),
    sb!(9, "chat", "ServerboundChatPacket", "message:String max 256, timestamp:Instant i64 epoch millis, salt:i64, nullable 256-byte signature, last_seen:offset VarInt + fixed 20-bit acknowledged bitset + checksum byte"),
    sb!(10, "chat_session_update", "ServerboundChatSessionUpdatePacket", "session_id:UUID, expires_at:Instant i64 epoch millis, public_key:byte array max 512, key_signature:byte array max 4096"),
    sb!(11, "chunk_batch_received", "ServerboundChunkBatchReceivedPacket", "desired_chunks_per_tick:f32"),
    sb!(12, "client_command", "ServerboundClientCommandPacket", "action:enum"),
    sb!(13, "client_tick_end", "ServerboundClientTickEndPacket", "empty_payload"),
    sb!(14, "client_information", "ServerboundClientInformationPacket", "language:String max 16, view_distance:i8, chat_visibility:enum VarInt, chat_colors:bool, model_customisation:u8, main_hand:enum VarInt, text_filtering_enabled:bool, allows_listing:bool, particle_status:enum VarInt"),
    sb!(15, "command_suggestion", "ServerboundCommandSuggestionPacket", "id:VarInt, command:utf8 max 32500"),
    sb!(16, "configuration_acknowledged", "ServerboundConfigurationAcknowledgedPacket", "empty_payload"),
    sb!(17, "container_button_click", "ServerboundContainerButtonClickPacket", "container_id:CONTAINER_ID, button_id:VarInt"),
    sb!(18, "container_click", "ServerboundContainerClickPacket", "container_id:CONTAINER_ID VarInt, state_id:VarInt, slot_num:i16, button_num:i8, container_input:VarInt id mapper, changed_slots:map max 128 of slot i16 to HashedStack, carried_item:HashedStack"),
    sb!(19, "container_close", "ServerboundContainerClosePacket", "container_id:VarInt"),
    sb!(20, "container_slot_state_changed", "ServerboundContainerSlotStateChangedPacket", "slot_id:VarInt, container_id:CONTAINER_ID, new_state:bool"),
    sb!(21, "cookie_response", "ServerboundCookieResponsePacket", "key:Identifier, payload:Optional<bytes VarInt length max 5120>"),
    sb!(22, "custom_payload", "ServerboundCustomPayloadPacket", "payload:CustomPacketPayload (channel Identifier, minecraft:brand string or unknown payload up to 32767 bytes)"),
    sb!(23, "debug_subscription_request", "ServerboundDebugSubscriptionRequestPacket", "subscriptions:Set<DebugSubscription registry holder VarInt>"),
    sb!(24, "edit_book", "ServerboundEditBookPacket", "slot:VarInt, pages:list max 100 of utf8 max 1024, title:optional utf8 max 32"),
    sb!(25, "entity_tag_query", "ServerboundEntityTagQueryPacket", "transaction_id:VarInt, entity_id:VarInt"),
    sb!(26, "interact", "ServerboundInteractPacket", "entity_id:VarInt, hand:InteractionHand enum VarInt, location:LpVec3, using_secondary_action:bool"),
    sb!(27, "jigsaw_generate", "ServerboundJigsawGeneratePacket", "pos:BlockPos, levels:VarInt, keep_jigsaws:bool"),
    sb!(28, "keep_alive", "ServerboundKeepAlivePacket", "id:i64_be"),
    sb!(29, "lock_difficulty", "ServerboundLockDifficultyPacket", "locked:bool"),
    sb!(30, "move_player_pos", "ServerboundMovePlayerPacket.Pos", "x:f64, y:f64, z:f64, flags:u8"),
    sb!(31, "move_player_pos_rot", "ServerboundMovePlayerPacket.PosRot", "x:f64, y:f64, z:f64, y_rot:f32, x_rot:f32, flags:u8"),
    sb!(32, "move_player_rot", "ServerboundMovePlayerPacket.Rot", "y_rot:f32, x_rot:f32, flags:u8"),
    sb!(33, "move_player_status_only", "ServerboundMovePlayerPacket.StatusOnly", "flags:u8"),
    sb!(34, "move_vehicle", "ServerboundMoveVehiclePacket", "position:Vec3, y_rot:f32, x_rot:f32, on_ground:bool"),
    sb!(35, "paddle_boat", "ServerboundPaddleBoatPacket", "left:bool, right:bool"),
    sb!(36, "pick_item_from_block", "ServerboundPickItemFromBlockPacket", "pos:BlockPos, include_data:bool"),
    sb!(37, "pick_item_from_entity", "ServerboundPickItemFromEntityPacket", "id:VarInt, include_data:bool"),
    sb!(38, "ping_request", "ServerboundPingRequestPacket", "time:i64_be"),
    sb!(39, "place_recipe", "ServerboundPlaceRecipePacket", "container_id:CONTAINER_ID, recipe:RecipeDisplayId VarInt, use_max_items:bool"),
    sb!(40, "player_abilities", "ServerboundPlayerAbilitiesPacket", "flags:u8 bit1 flying"),
    sb!(41, "player_action", "ServerboundPlayerActionPacket", "action:enum VarInt, pos:BlockPos, direction:u8, sequence:VarInt"),
    sb!(42, "player_command", "ServerboundPlayerCommandPacket", "id:VarInt, action:enum VarInt, data:VarInt"),
    sb!(43, "player_input", "ServerboundPlayerInputPacket", "input:Input.STREAM_CODEC"),
    sb!(44, "player_loaded", "ServerboundPlayerLoadedPacket", "empty_payload"),
    sb!(45, "pong", "ServerboundPongPacket", "id:i32_be"),
    sb!(46, "recipe_book_change_settings", "ServerboundRecipeBookChangeSettingsPacket", "book_type:RecipeBookType enum VarInt, is_open:bool, is_filtering:bool"),
    sb!(47, "recipe_book_seen_recipe", "ServerboundRecipeBookSeenRecipePacket", "recipe:RecipeDisplayId VarInt index"),
    sb!(48, "rename_item", "ServerboundRenameItemPacket", "name:utf(32767)"),
    sb!(49, "resource_pack", "ServerboundResourcePackPacket", "id:UUID, action:ResourcePackAction enum VarInt"),
    sb!(50, "seen_advancements", "ServerboundSeenAdvancementsPacket", "action:enum VarInt, tab:Identifier only when action OPENED_TAB"),
    sb!(51, "select_trade", "ServerboundSelectTradePacket", "item:VarInt"),
    sb!(52, "set_beacon", "ServerboundSetBeaconPacket", "primary:optional MobEffect.STREAM_CODEC, secondary:optional MobEffect.STREAM_CODEC"),
    sb!(53, "set_carried_item", "ServerboundSetCarriedItemPacket", "slot:i16_be"),
    sb!(54, "set_command_block", "ServerboundSetCommandBlockPacket", "pos:BlockPos, command:utf8 max 32767, mode:CommandBlockEntity.Mode enum VarInt, flags:u8(track_output=1, conditional=2, automatic=4)"),
    sb!(55, "set_command_minecart", "ServerboundSetCommandMinecartPacket", "entity:VarInt, command:utf8 max 32767, track_output:bool"),
    sb!(56, "set_creative_mode_slot", "ServerboundSetCreativeModeSlotPacket", "slot_num:i16, item_stack:validated ItemStack.OPTIONAL_UNTRUSTED_STREAM_CODEC encoded as count VarInt, optional item registry id, delimited data component patch"),
    sb!(57, "set_game_rule", "ServerboundSetGameRulePacket", "entries:List(game_rule:ResourceKey<GameRule> Identifier, value:String UTF-8)"),
    sb!(58, "set_jigsaw_block", "ServerboundSetJigsawBlockPacket", "pos:BlockPos, name:Identifier, target:Identifier, pool:Identifier, final_state:String, joint:String, selection_priority:VarInt, placement_priority:VarInt"),
    sb!(59, "set_structure_block", "ServerboundSetStructureBlockPacket", "pos:BlockPos, update_type:enum VarInt, mode:StructureMode enum VarInt, name:utf8, offset:3 i8 clamped -48..48, size:3 i8 clamped 0..48, mirror:enum VarInt, rotation:enum VarInt, data:utf8 max 128, integrity:f32 clamped 0..1, seed:VarLong, flags:u8(ignore_entities=1, show_air=2, show_bounding_box=4, strict=8)"),
    sb!(60, "set_test_block", "ServerboundSetTestBlockPacket", "position:BlockPos, mode:TestBlockMode, message:String UTF-8"),
    sb!(61, "sign_update", "ServerboundSignUpdatePacket", "pos:BlockPos, is_front_text:bool, lines:[utf(384);4]"),
    sb!(62, "spectate_entity", "ServerboundSpectateEntityPacket", "entity_id:VarInt"),
    sb!(63, "swing", "ServerboundSwingPacket", "hand:enum"),
    sb!(64, "teleport_to_entity", "ServerboundTeleportToEntityPacket", "uuid:UUID"),
    sb!(65, "test_instance_block_action", "ServerboundTestInstanceBlockActionPacket", "pos:BlockPos, action:enum VarInt, data:TestInstanceBlockEntity.Data"),
    sb!(66, "use_item_on", "ServerboundUseItemOnPacket", "hand:enum VarInt, block_hit:BlockHitResult, sequence:VarInt"),
    sb!(67, "use_item", "ServerboundUseItemPacket", "hand:enum VarInt, sequence:VarInt, y_rot:f32, x_rot:f32"),
    sb!(68, "custom_click_action", "ServerboundCustomClickActionPacket", "id:Identifier, payload:Optional<Tag> length-prefixed max 65536 with NBT accounter 32768"),
    cb!(0, "bundle", "ClientboundBundlePacket", "bundle delimiter packet; payload is nested packet sequence handled by BundlePacket"),
    cb!(1, "add_entity", "ClientboundAddEntityPacket", "id:VarInt, uuid:UUID, type:EntityType registry VarInt, x:double, y:double, z:double, movement:Vec3.LP_STREAM_CODEC, xRot:byte, yRot:byte, yHeadRot:byte, data:VarInt"),
    cb!(2, "animate", "ClientboundAnimatePacket", "id:var_int, action:u8"),
    cb!(3, "award_stats", "ClientboundAwardStatsPacket", "stats:Map<Stat(stat_type:VarInt, stat_value:VarInt), value:VarInt>"),
    cb!(4, "block_changed_ack", "ClientboundBlockChangedAckPacket", "sequence:VarInt"),
    cb!(5, "block_destruction", "ClientboundBlockDestructionPacket", "id:VarInt, pos:BlockPos, progress:u8"),
    cb!(6, "block_entity_data", "ClientboundBlockEntityDataPacket", "pos:BlockPos, type:registry VarInt, tag:trusted compound tag"),
    cb!(7, "block_event", "ClientboundBlockEventPacket", "pos:BlockPos, action:u8, param:u8, block:registry VarInt"),
    cb!(8, "block_update", "ClientboundBlockUpdatePacket", "pos:BlockPos, block_state:Block.BLOCK_STATE_REGISTRY VarInt"),
    cb!(9, "boss_event", "ClientboundBossEventPacket", "id:UUID, operation:OperationType enum VarInt, payload by operation: add(name:trusted Component network NBT tag, progress:float, color enum VarInt, overlay enum VarInt, flags byte), remove(), update_progress(float), update_name(trusted Component network NBT tag), update_style(color enum VarInt, overlay enum VarInt), update_properties(flags byte)"),
    cb!(10, "change_difficulty", "ClientboundChangeDifficultyPacket", "difficulty:difficulty_enum, locked:bool"),
    cb!(11, "chunk_batch_finished", "ClientboundChunkBatchFinishedPacket", "batch_size:VarInt"),
    cb!(12, "chunk_batch_start", "ClientboundChunkBatchStartPacket", "empty_payload"),
    cb!(13, "chunks_biomes", "ClientboundChunksBiomesPacket", "chunk_biome_data:List(pos:ChunkPos i64, buffer:byte_array max 2097152)"),
    cb!(14, "clear_titles", "ClientboundClearTitlesPacket", "reset_times:bool"),
    cb!(15, "command_suggestions", "ClientboundCommandSuggestionsPacket", "id:VarInt, start:VarInt, length:VarInt, suggestions:List<Entry(text:String, tooltip:Optional<trusted Component network NBT tag>)>"),
    cb!(16, "commands", "ClientboundCommandsPacket", "entries:List(Entry(flags:byte type/root/literal/argument + executable + redirect + custom_suggestions + restricted, children:VarIntArray, redirect:optional VarInt, literal:name String or argument:name String + command_argument_type registry VarInt + type payload + optional suggestion Identifier)), root_index:VarInt"),
    cb!(17, "container_close", "ClientboundContainerClosePacket", "container_id:CONTAINER_ID VarInt"),
    cb!(18, "container_set_content", "ClientboundContainerSetContentPacket", "containerId:CONTAINER_ID VarInt, stateId:VarInt, items:ItemStack.OPTIONAL_LIST_STREAM_CODEC, carriedItem:ItemStack.OPTIONAL_STREAM_CODEC"),
    cb!(19, "container_set_data", "ClientboundContainerSetDataPacket", "container_id:CONTAINER_ID VarInt, id:short, value:short"),
    cb!(20, "container_set_slot", "ClientboundContainerSetSlotPacket", "containerId:CONTAINER_ID VarInt, stateId:VarInt, slot:short, itemStack:ItemStack.OPTIONAL_STREAM_CODEC"),
    cb!(21, "cookie_request", "ClientboundCookieRequestPacket", "key:Identifier"),
    cb!(22, "cooldown", "ClientboundCooldownPacket", "cooldown_group:Identifier, duration:VarInt"),
    cb!(23, "custom_chat_completions", "ClientboundCustomChatCompletionsPacket", "action:enum VarInt, entries:List<String>"),
    cb!(24, "custom_payload", "ClientboundCustomPayloadPacket", "payload:CustomPacketPayload (channel Identifier, minecraft:brand string or unknown payload up to 1 MiB)"),
    cb!(25, "damage_event", "ClientboundDamageEventPacket", "entity_id:VarInt, source_type:DamageType holder, source_cause_id:VarInt id+1, source_direct_id:VarInt id+1, source_position:Optional<Vec3 double>"),
    cb!(26, "debug/block_value", "ClientboundDebugBlockValuePacket", "block_pos:BlockPos, update:DebugSubscription.Update"),
    cb!(27, "debug/chunk_value", "ClientboundDebugChunkValuePacket", "chunk_pos:ChunkPos i64, update:DebugSubscription.Update"),
    cb!(28, "debug/entity_value", "ClientboundDebugEntityValuePacket", "entity_id:VarInt, update:DebugSubscription.Update"),
    cb!(29, "debug/event", "ClientboundDebugEventPacket", "event:DebugSubscription.Event"),
    cb!(30, "debug_sample", "ClientboundDebugSamplePacket", "sample:long_array(VarInt length + i64 entries), debug_sample_type:enum VarInt"),
    cb!(31, "delete_chat", "ClientboundDeleteChatPacket", "message_signature:MessageSignature.Packed (VarInt id+1, full 256-byte signature when id is -1)"),
    cb!(32, "disconnect", "ClientboundDisconnectPacket", "reason:ComponentSerialization.TRUSTED_CONTEXT_FREE_STREAM_CODEC as network NBT tag"),
    cb!(33, "disguised_chat", "ClientboundDisguisedChatPacket", "message:ComponentSerialization.TRUSTED_STREAM_CODEC as network NBT tag, chatType:ChatType.Bound(chatType:Holder<ChatType> id+1, name:trusted Component network NBT tag, targetName:Optional<trusted Component network NBT tag>)"),
    cb!(34, "entity_event", "ClientboundEntityEventPacket", "entity_id:int, event_id:byte"),
    cb!(35, "entity_position_sync", "ClientboundEntityPositionSyncPacket", "id:VarInt, values:PositionMoveRotation(position Vec3, deltaMovement Vec3, yRot float, xRot float), on_ground:bool"),
    cb!(36, "explode", "ClientboundExplodePacket", "center:Vec3, radius:float, block_count:int, player_knockback:Optional<Vec3>, explosion_particle:ParticleTypes.STREAM_CODEC, explosion_sound:SoundEvent.STREAM_CODEC, block_particles:WeightedList<ExplosionParticleInfo(particle, scaling:float, speed:float)>"),
    cb!(37, "forget_level_chunk", "ClientboundForgetLevelChunkPacket", "pos:ChunkPos i64"),
    cb!(38, "game_event", "ClientboundGameEventPacket", "event:u8, param:f32"),
    cb!(39, "game_rule_values", "ClientboundGameRuleValuesPacket", "values:Map<GameRule ResourceKey Identifier, String UTF-8>"),
    cb!(40, "game_test_highlight_pos", "ClientboundGameTestHighlightPosPacket", "absolute_pos:BlockPos, relative_pos:BlockPos"),
    cb!(41, "mount_screen_open", "ClientboundMountScreenOpenPacket", "container_id:VarInt, inventory_columns:VarInt, entity_id:int"),
    cb!(42, "hurt_animation", "ClientboundHurtAnimationPacket", "id:var_int, yaw:f32"),
    cb!(43, "initialize_border", "ClientboundInitializeBorderPacket", "new_center_x:f64, new_center_z:f64, old_size:f64, new_size:f64, lerp_time:VarLong, new_absolute_max_size:VarInt, warning_blocks:VarInt, warning_time:VarInt"),
    cb!(44, "keep_alive", "ClientboundKeepAlivePacket", "id:i64_be"),
    cb!(45, "level_chunk_with_light", "ClientboundLevelChunkWithLightPacket", "x:int, z:int, chunk_data(heightmaps:Map<Heightmap.Types, long[]>, buffer:VarInt length+bytes up to 2MiB, block_entities:List(packed_xz:byte, y:short, type:registry VarInt, tag:nullable compound NBT)), light_data(sky_y_mask, block_y_mask, empty_sky_y_mask, empty_block_y_mask bitsets, sky_updates/block_updates lists of 2048-byte arrays)"),
    cb!(46, "level_event", "ClientboundLevelEventPacket", "type:int, pos:BlockPos, data:int, global_event:bool"),
    cb!(47, "level_particles", "ClientboundLevelParticlesPacket", "override_limiter:bool, always_show:bool, x/y/z:double, x/y/z_dist:float, max_speed:float, count:int, particle:ParticleTypes.STREAM_CODEC (registry id VarInt + particle-specific payload)"),
    cb!(48, "light_update", "ClientboundLightUpdatePacket", "x:VarInt, z:VarInt, light_data(sky_y_mask, block_y_mask, empty_sky_y_mask, empty_block_y_mask bitsets, sky_updates/block_updates lists of 2048-byte arrays)"),
    cb!(49, "login", "ClientboundLoginPacket", "player_id:int, hardcore:bool, levels:Collection<ResourceKey<Level>>, max_players:VarInt, chunk_radius:VarInt, simulation_distance:VarInt, reduced_debug_info:bool, show_death_screen:bool, do_limited_crafting:bool, common_spawn_info, enforces_secure_chat:bool"),
    cb!(50, "low_disk_space_warning", "ClientboundLowDiskSpaceWarningPacket", "empty_payload"),
    cb!(51, "map_item_data", "ClientboundMapItemDataPacket", "mapId:VarInt, scale:byte, locked:bool, decorations:Optional<List(type:Holder<MapDecorationType> registry VarInt, x:byte, y:byte, rot:byte, name:Optional<Component network NBT tag>)>, colorPatch:width byte (0 absent) then height/startX/startY bytes and colors byte array"),
    cb!(52, "merchant_offers", "ClientboundMerchantOffersPacket", "container_id:ContainerId VarInt, offers:List<MerchantOffer(ItemCost A, result ItemStack.STREAM_CODEC, Optional<ItemCost B>, out_of_stock:bool, uses:int, max_uses:int, xp:int, special_price_diff:int, price_multiplier:float, demand:int)>, villager_level:VarInt, villager_xp:VarInt, show_progress:bool, can_restock:bool"),
    cb!(53, "move_entity_pos", "ClientboundMoveEntityPacket.Pos", "entity_id:var_int, xa:short, ya:short, za:short, on_ground:bool"),
    cb!(54, "move_entity_pos_rot", "ClientboundMoveEntityPacket.PosRot", "entity_id:var_int, xa:short, ya:short, za:short, y_rot:byte, x_rot:byte, on_ground:bool"),
    cb!(55, "move_minecart_along_track", "ClientboundMoveMinecartPacket", "entity_id:VarInt, lerp_steps:List<NewMinecartBehavior.MinecartStep>"),
    cb!(56, "move_entity_rot", "ClientboundMoveEntityPacket.Rot", "entity_id:var_int, y_rot:byte, x_rot:byte, on_ground:bool"),
    cb!(57, "move_vehicle", "ClientboundMoveVehiclePacket", "position:Vec3.STREAM_CODEC, y_rot:f32, x_rot:f32"),
    cb!(58, "open_book", "ClientboundOpenBookPacket", "hand:InteractionHand enum VarInt"),
    cb!(59, "open_screen", "ClientboundOpenScreenPacket", "containerId:ContainerId VarInt, type:MenuType registry VarInt, title:trusted Component network NBT tag"),
    cb!(60, "open_sign_editor", "ClientboundOpenSignEditorPacket", "pos:BlockPos, is_front_text:bool"),
    cb!(61, "ping", "ClientboundPingPacket", "id:i32_be"),
    cb!(62, "pong_response", "ClientboundPongResponsePacket", "time:i64_be"),
    cb!(63, "place_ghost_recipe", "ClientboundPlaceGhostRecipePacket", "container_id:CONTAINER_ID, recipe_display:RecipeDisplay"),
    cb!(64, "player_abilities", "ClientboundPlayerAbilitiesPacket", "flags:byte, flying_speed:f32, walking_speed:f32"),
    cb!(65, "player_chat", "ClientboundPlayerChatPacket", "global_index:VarInt, sender:UUID, index:VarInt, signature:nullable 256-byte MessageSignature, body:SignedMessageBody.Packed(content:utf max 256, timestamp:Instant epoch millis long, salt:long, last_seen:list MessageSignature.Packed), unsigned_content:nullable trusted Component, filter_mask:FilterMask enum VarInt plus optional BitSet, chat_type:Bound(chat_type holder registry VarInt, name trusted Component, target_name optional trusted Component)"),
    cb!(66, "player_combat_end", "ClientboundPlayerCombatEndPacket", "duration:VarInt"),
    cb!(67, "player_combat_enter", "ClientboundPlayerCombatEnterPacket", "empty_payload"),
    cb!(68, "player_combat_kill", "ClientboundPlayerCombatKillPacket", "player_id:VarInt, message:ComponentSerialization.TRUSTED_STREAM_CODEC"),
    cb!(69, "player_info_remove", "ClientboundPlayerInfoRemovePacket", "profile_ids:list UUID"),
    cb!(70, "player_info_update", "ClientboundPlayerInfoUpdatePacket", "actions:fixed BitSet over Action enum(ADD_PLAYER, INITIALIZE_CHAT, UPDATE_GAME_MODE, UPDATE_LISTED, UPDATE_LATENCY, UPDATE_DISPLAY_NAME, UPDATE_LIST_ORDER, UPDATE_HAT), entries:List(UUID + selected action payloads: profile name/properties, nullable RemoteChatSession.Data, game_mode VarInt, listed bool, latency VarInt, nullable trusted Component, list_order VarInt, show_hat bool)"),
    cb!(71, "player_look_at", "ClientboundPlayerLookAtPacket", "from_anchor:enum VarInt, x:double, y:double, z:double, at_entity:bool, optional entity:VarInt + to_anchor:enum VarInt"),
    cb!(72, "player_position", "ClientboundPlayerPositionPacket", "id:VarInt, change:PositionMoveRotation(position Vec3, deltaMovement Vec3, yRot float, xRot float), relatives:Set<Relative> as fixed int"),
    cb!(73, "player_rotation", "ClientboundPlayerRotationPacket", "y_rot:f32, relative_y:bool, x_rot:f32, relative_x:bool"),
    cb!(74, "recipe_book_add", "ClientboundRecipeBookAddPacket", "entries:list(contents:RecipeDisplayEntry(id:RecipeDisplayId VarInt, display:registry-dispatched RecipeDisplay, group:OptionalVarInt, category:recipe_book_category registry VarInt, crafting_requirements:optional list Ingredient holder-set), flags:byte), replace:bool"),
    cb!(75, "recipe_book_remove", "ClientboundRecipeBookRemovePacket", "recipes:list RecipeDisplayId(index:VarInt)"),
    cb!(76, "recipe_book_settings", "ClientboundRecipeBookSettingsPacket", "crafting(open:bool, filtering:bool), furnace(open:bool, filtering:bool), blast_furnace(open:bool, filtering:bool), smoker(open:bool, filtering:bool)"),
    cb!(77, "remove_entities", "ClientboundRemoveEntitiesPacket", "entity_ids:int_id_list"),
    cb!(78, "remove_mob_effect", "ClientboundRemoveMobEffectPacket", "entity_id:VarInt, effect:MobEffect registry holder VarInt"),
    cb!(79, "reset_score", "ClientboundResetScorePacket", "owner:utf, objective_name:nullable utf"),
    cb!(80, "resource_pack_pop", "ClientboundResourcePackPopPacket", "id:optional UUID"),
    cb!(81, "resource_pack_push", "ClientboundResourcePackPushPacket", "id:UUID, url:String, hash:String(max 40), required:bool, prompt:Optional<trusted context-free Component network NBT tag>"),
    cb!(82, "respawn", "ClientboundRespawnPacket", "common_spawn_info(dimension_type:DimensionType.STREAM_CODEC, dimension:ResourceKey<Level>, seed:long, game_type:byte, previous_game_type:nullable byte, is_debug:bool, is_flat:bool, last_death_location:Optional<GlobalPos>, portal_cooldown:VarInt, sea_level:VarInt), data_to_keep:byte"),
    cb!(83, "rotate_head", "ClientboundRotateHeadPacket", "entity_id:var_int, y_head_rot:byte"),
    cb!(84, "section_blocks_update", "ClientboundSectionBlocksUpdatePacket", "section_pos:long, updates:VarInt count then VarLong(block_state_id << 12 | packed_section_pos)"),
    cb!(85, "select_advancements_tab", "ClientboundSelectAdvancementsTabPacket", "tab:nullable Identifier"),
    cb!(86, "server_data", "ClientboundServerDataPacket", "motd:trusted context-free Component, icon_bytes:Optional<byte_array>"),
    cb!(87, "set_action_bar_text", "ClientboundSetActionBarTextPacket", "text:ComponentSerialization.TRUSTED_STREAM_CODEC as network NBT tag"),
    cb!(88, "set_border_center", "ClientboundSetBorderCenterPacket", "new_center_x:f64, new_center_z:f64"),
    cb!(89, "set_border_lerp_size", "ClientboundSetBorderLerpSizePacket", "old_size:f64, new_size:f64, lerp_time:VarLong"),
    cb!(90, "set_border_size", "ClientboundSetBorderSizePacket", "size:f64"),
    cb!(91, "set_border_warning_delay", "ClientboundSetBorderWarningDelayPacket", "warning_delay:VarInt"),
    cb!(92, "set_border_warning_distance", "ClientboundSetBorderWarningDistancePacket", "warning_blocks:VarInt"),
    cb!(93, "set_camera", "ClientboundSetCameraPacket", "camera_id:VarInt"),
    cb!(94, "set_chunk_cache_center", "ClientboundSetChunkCacheCenterPacket", "x:var_int, z:var_int"),
    cb!(95, "set_chunk_cache_radius", "ClientboundSetChunkCacheRadiusPacket", "radius:var_int"),
    cb!(96, "set_cursor_item", "ClientboundSetCursorItemPacket", "contents:ItemStack.OPTIONAL_STREAM_CODEC"),
    cb!(97, "set_default_spawn_position", "ClientboundSetDefaultSpawnPositionPacket", "respawn_data:LevelData.RespawnData.STREAM_CODEC"),
    cb!(98, "set_display_objective", "ClientboundSetDisplayObjectivePacket", "slot:DisplaySlot enum VarInt, objective_name:utf"),
    cb!(99, "set_entity_data", "ClientboundSetEntityDataPacket", "id:var_int, packed_items:(index:u8, serializer_id:var_int, serializer payload)*, eof:0xff"),
    cb!(100, "set_entity_link", "ClientboundSetEntityLinkPacket", "source_id:int, dest_id:int"),
    cb!(101, "set_entity_motion", "ClientboundSetEntityMotionPacket", "id:var_int, movement:Vec3.LP_STREAM_CODEC"),
    cb!(102, "set_equipment", "ClientboundSetEquipmentPacket", "entity:VarInt, repeated slot byte (high bit continues, low 7 bits EquipmentSlot ordinal) + ItemStack.OPTIONAL_STREAM_CODEC"),
    cb!(103, "set_experience", "ClientboundSetExperiencePacket", "experience_progress:f32, experience_level:var_int, total_experience:var_int"),
    cb!(104, "set_health", "ClientboundSetHealthPacket", "health:f32, food:var_int, saturation:f32"),
    cb!(105, "set_held_slot", "ClientboundSetHeldSlotPacket", "slot:var_int"),
    cb!(106, "set_objective", "ClientboundSetObjectivePacket", "objectiveName:String, method:byte, if add/change displayName:trusted Component network NBT tag, renderType:ObjectiveCriteria.RenderType enum VarInt, numberFormat:Optional<NumberFormatTypes.STREAM_CODEC>"),
    cb!(107, "set_passengers", "ClientboundSetPassengersPacket", "vehicle:var_int, passengers:var_int_array"),
    cb!(108, "set_player_inventory", "ClientboundSetPlayerInventoryPacket", "slot:VarInt, contents:ItemStack.OPTIONAL_STREAM_CODEC"),
    cb!(109, "set_player_team", "ClientboundSetPlayerTeamPacket", "name:String, method:byte, if create/update parameters(displayName:trusted Component network NBT tag, options:byte, nametagVisibility:Team.Visibility VarInt, collisionRule:Team.CollisionRule VarInt, color:ChatFormatting enum VarInt, prefix:trusted Component network NBT tag, suffix:trusted Component network NBT tag), if create/join/leave players:List<String>"),
    cb!(110, "set_score", "ClientboundSetScorePacket", "owner:String, objectiveName:String, score:VarInt, display:Optional<trusted Component network NBT tag>, numberFormat:Optional<NumberFormatTypes.STREAM_CODEC>"),
    cb!(111, "set_simulation_distance", "ClientboundSetSimulationDistancePacket", "simulation_distance:var_int"),
    cb!(112, "set_subtitle_text", "ClientboundSetSubtitleTextPacket", "text:ComponentSerialization.TRUSTED_STREAM_CODEC as network NBT tag"),
    cb!(113, "set_time", "ClientboundSetTimePacket", "game_time:i64, clock_updates:map"),
    cb!(114, "set_title_text", "ClientboundSetTitleTextPacket", "text:ComponentSerialization.TRUSTED_STREAM_CODEC as network NBT tag"),
    cb!(115, "set_titles_animation", "ClientboundSetTitlesAnimationPacket", "fade_in:int, stay:int, fade_out:int"),
    cb!(116, "sound_entity", "ClientboundSoundEntityPacket", "sound:Holder<SoundEvent> (registered id+1 or direct id 0 + Identifier + optional fixed_range), source:SoundSource enum VarInt, entity_id:VarInt, volume:float, pitch:float, seed:long"),
    cb!(117, "sound", "ClientboundSoundPacket", "sound:Holder<SoundEvent> (registered id+1 or direct id 0 + Identifier + optional fixed_range), source:SoundSource enum VarInt, x/y/z:int fixed point (*8), volume:float, pitch:float, seed:long"),
    cb!(118, "start_configuration", "ClientboundStartConfigurationPacket", "unit/no fields; terminal play packet"),
    cb!(119, "stop_sound", "ClientboundStopSoundPacket", "flags:byte bit0 has_source bit1 has_sound, optional source:SoundSource enum VarInt, optional name:Identifier"),
    cb!(120, "store_cookie", "ClientboundStoreCookiePacket", "key:Identifier, payload:bytes VarInt length max 5120"),
    cb!(121, "system_chat", "ClientboundSystemChatPacket", "content:ComponentSerialization.TRUSTED_STREAM_CODEC as network NBT tag, overlay:bool"),
    cb!(122, "tab_list", "ClientboundTabListPacket", "header:trusted Component network NBT tag, footer:trusted Component network NBT tag"),
    cb!(123, "tag_query", "ClientboundTagQueryPacket", "transaction_id:VarInt, tag:nullable CompoundTag"),
    cb!(124, "take_item_entity", "ClientboundTakeItemEntityPacket", "item_id:VarInt, player_id:VarInt, amount:VarInt"),
    cb!(125, "teleport_entity", "ClientboundTeleportEntityPacket", "id:var_int, change:PositionMoveRotation.STREAM_CODEC, relatives:Set<Relative>, on_ground:bool"),
    cb!(126, "test_instance_block_status", "ClientboundTestInstanceBlockStatus", "status:ComponentSerialization.STREAM_CODEC, size:Optional<Vec3i>"),
    cb!(127, "ticking_state", "ClientboundTickingStatePacket", "tick_rate:f32, is_frozen:bool"),
    cb!(128, "ticking_step", "ClientboundTickingStepPacket", "tick_steps:var_int"),
    cb!(129, "transfer", "ClientboundTransferPacket", "host:String max 32767, port:VarInt"),
    cb!(130, "update_advancements", "ClientboundUpdateAdvancementsPacket", "reset:bool, added:List(AdvancementHolder id:Identifier + Advancement(parent:optional Identifier, display:optional DisplayInfo, requirements:list list string, sends_telemetry_event:bool)), removed:LinkedHashSet<Identifier>, progress:Map(Identifier -> criteria map string -> nullable Instant epoch millis), show_advancements:bool"),
    cb!(131, "update_attributes", "ClientboundUpdateAttributesPacket", "entity_id:VarInt, attributes:List(attribute:Holder<Attribute> registry VarInt, base:double, modifiers:List(id:Identifier, amount:double, operation:VarInt))"),
    cb!(132, "update_mob_effect", "ClientboundUpdateMobEffectPacket", "entity_id:VarInt, effect:MobEffect registry holder VarInt, amplifier:VarInt, duration_ticks:VarInt, flags:byte"),
    cb!(133, "update_recipes", "ClientboundUpdateRecipesPacket", "item_sets:Map<ResourceKey<RecipePropertySet>, RecipePropertySet>, stonecutter_recipes:SelectableRecipe.SingleInputSet<StonecutterRecipe>"),
    cb!(134, "update_tags", "ClientboundUpdateTagsPacket", "tags:Map<registry ResourceKey, NetworkPayload(tags:List<Identifier, int id list>)>"),
    cb!(135, "projectile_power", "ClientboundProjectilePowerPacket", "id:VarInt, acceleration_power:f64"),
    cb!(136, "custom_report_details", "ClientboundCustomReportDetailsPacket", "details:List max 32 of key:String max 128, value:String max 4096"),
    cb!(137, "server_links", "ClientboundServerLinksPacket", "links:List(label:known bool + known enum VarInt or custom Component, link:String max 32767)"),
    cb!(138, "waypoint", "ClientboundTrackedWaypointPacket", "operation:enum VarInt, waypoint:TrackedWaypoint"),
    cb!(139, "clear_dialog", "ClientboundClearDialogPacket", "empty_payload"),
    cb!(140, "show_dialog", "ClientboundShowDialogPacket", "payload:remaining bytes max 1 MiB"),
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

    type FieldOrderCheck = (PacketDirection, &'static str, &'static str);

    const SCALAR_FIELD_ORDER_CHECKS: &[FieldOrderCheck] = &[
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

    const COMMON_FIELD_ORDER_CHECKS: &[FieldOrderCheck] = &[
        (
            PacketDirection::Serverbound,
            "client_information",
            "language:String max 16, view_distance:i8, chat_visibility:enum VarInt, chat_colors:bool, model_customisation:u8, main_hand:enum VarInt, text_filtering_enabled:bool, allows_listing:bool, particle_status:enum VarInt",
        ),
        (
            PacketDirection::Serverbound,
            "cookie_response",
            "key:Identifier, payload:Optional<bytes VarInt length max 5120>",
        ),
        (
            PacketDirection::Serverbound,
            "custom_payload",
            "payload:CustomPacketPayload (channel Identifier, minecraft:brand string or unknown payload up to 32767 bytes)",
        ),
        (PacketDirection::Serverbound, "keep_alive", "id:i64_be"),
        (PacketDirection::Serverbound, "ping_request", "time:i64_be"),
        (PacketDirection::Clientbound, "cookie_request", "key:Identifier"),
        (PacketDirection::Clientbound, "keep_alive", "id:i64_be"),
        (PacketDirection::Clientbound, "pong_response", "time:i64_be"),
        (
            PacketDirection::Clientbound,
            "store_cookie",
            "key:Identifier, payload:bytes VarInt length max 5120",
        ),
        (
            PacketDirection::Clientbound,
            "transfer",
            "host:String max 32767, port:VarInt",
        ),
        (
            PacketDirection::Clientbound,
            "custom_report_details",
            "details:List max 32 of key:String max 128, value:String max 4096",
        ),
        (
            PacketDirection::Clientbound,
            "server_links",
            "links:List(label:known bool + known enum VarInt or custom Component, link:String max 32767)",
        ),
        (PacketDirection::Clientbound, "clear_dialog", "empty_payload"),
        (
            PacketDirection::Clientbound,
            "show_dialog",
            "payload:remaining bytes max 1 MiB",
        ),
    ];

    const GAME_FIELD_ORDER_CHECKS: &[FieldOrderCheck] = &[
        (PacketDirection::Serverbound, "attack", "entity_id:VarInt"),
        (
            PacketDirection::Serverbound,
            "block_entity_tag_query",
            "transaction_id:VarInt, pos:BlockPos",
        ),
        (
            PacketDirection::Serverbound,
            "bundle_item_selected",
            "slot_id:VarInt, selected_item_index:VarInt (-1 or >=0)",
        ),
        (
            PacketDirection::Serverbound,
            "change_game_mode",
            "mode:GameType enum VarInt",
        ),
        (
            PacketDirection::Serverbound,
            "container_slot_state_changed",
            "slot_id:VarInt, container_id:CONTAINER_ID, new_state:bool",
        ),
        (
            PacketDirection::Serverbound,
            "debug_subscription_request",
            "subscriptions:Set<DebugSubscription registry holder VarInt>",
        ),
        (
            PacketDirection::Serverbound,
            "entity_tag_query",
            "transaction_id:VarInt, entity_id:VarInt",
        ),
        (
            PacketDirection::Serverbound,
            "place_recipe",
            "container_id:CONTAINER_ID, recipe:RecipeDisplayId VarInt, use_max_items:bool",
        ),
        (
            PacketDirection::Serverbound,
            "player_abilities",
            "flags:u8 bit1 flying",
        ),
        (
            PacketDirection::Serverbound,
            "seen_advancements",
            "action:enum VarInt, tab:Identifier only when action OPENED_TAB",
        ),
        (
            PacketDirection::Serverbound,
            "set_game_rule",
            "entries:List(game_rule:ResourceKey<GameRule> Identifier, value:String UTF-8)",
        ),
        (
            PacketDirection::Serverbound,
            "set_jigsaw_block",
            "pos:BlockPos, name:Identifier, target:Identifier, pool:Identifier, final_state:String, joint:String, selection_priority:VarInt, placement_priority:VarInt",
        ),
        (
            PacketDirection::Serverbound,
            "set_test_block",
            "position:BlockPos, mode:TestBlockMode, message:String UTF-8",
        ),
        (
            PacketDirection::Serverbound,
            "spectate_entity",
            "entity_id:VarInt",
        ),
        (PacketDirection::Serverbound, "teleport_to_entity", "uuid:UUID"),
        (
            PacketDirection::Serverbound,
            "test_instance_block_action",
            "pos:BlockPos, action:enum VarInt, data:TestInstanceBlockEntity.Data",
        ),
        (
            PacketDirection::Serverbound,
            "custom_click_action",
            "id:Identifier, payload:Optional<Tag> length-prefixed max 65536 with NBT accounter 32768",
        ),
        (
            PacketDirection::Clientbound,
            "block_changed_ack",
            "sequence:VarInt",
        ),
        (
            PacketDirection::Clientbound,
            "chunk_batch_start",
            "empty_payload",
        ),
        (
            PacketDirection::Clientbound,
            "forget_level_chunk",
            "pos:ChunkPos i64",
        ),
        (
            PacketDirection::Clientbound,
            "game_rule_values",
            "values:Map<GameRule ResourceKey Identifier, String UTF-8>",
        ),
        (
            PacketDirection::Clientbound,
            "open_book",
            "hand:InteractionHand enum VarInt",
        ),
        (
            PacketDirection::Clientbound,
            "open_sign_editor",
            "pos:BlockPos, is_front_text:bool",
        ),
        (
            PacketDirection::Clientbound,
            "player_combat_kill",
            "player_id:VarInt, message:ComponentSerialization.TRUSTED_STREAM_CODEC",
        ),
        (
            PacketDirection::Clientbound,
            "set_camera",
            "camera_id:VarInt",
        ),
        (
            PacketDirection::Clientbound,
            "set_player_inventory",
            "slot:VarInt, contents:ItemStack.OPTIONAL_STREAM_CODEC",
        ),
        (
            PacketDirection::Clientbound,
            "tag_query",
            "transaction_id:VarInt, tag:nullable CompoundTag",
        ),
        (
            PacketDirection::Clientbound,
            "take_item_entity",
            "item_id:VarInt, player_id:VarInt, amount:VarInt",
        ),
    ];

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
        assert_eq!(keep_alive.id, 44);
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_KEEP_ALIVE_PACKET_ID),
            Some("keep_alive")
        );

        let set_time = specs
            .iter()
            .find(|entry| {
                entry.direction == PacketDirection::Clientbound && entry.wire_name == "set_time"
            })
            .unwrap();
        assert_eq!(set_time.id, 113);
        assert_eq!(registry.clientbound_name(113), Some("set_time"));
        assert_ne!(CLIENTBOUND_KEEP_ALIVE_PACKET_ID, set_time.id);

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
    fn play_packet_specification_has_no_unparsed_field_orders() {
        let specs = play_packet_specs_26_1_2();
        for spec in specs {
            assert_ne!(
                spec.field_order, "unparsed",
                "{:?} {} {} still needs concrete field order",
                spec.direction, spec.id, spec.wire_name
            );
        }
    }

    #[test]
    fn play_packet_specification_scalar_fields_are_concretely_decoded() {
        let specs = play_packet_specs_26_1_2();
        assert_field_order_checks(specs, SCALAR_FIELD_ORDER_CHECKS, false);
    }

    #[test]
    fn play_packet_specification_common_packets_have_concrete_field_orders() {
        let specs = play_packet_specs_26_1_2();
        assert_field_order_checks(specs, COMMON_FIELD_ORDER_CHECKS, true);
    }

    #[test]
    fn play_packet_specification_decompiled_game_packets_have_concrete_field_orders() {
        let specs = play_packet_specs_26_1_2();
        assert_field_order_checks(specs, GAME_FIELD_ORDER_CHECKS, true);
    }

    fn assert_field_order_checks(
        specs: &[super::PlayPacketSpec],
        checks: &[FieldOrderCheck],
        require_concrete: bool,
    ) {
        for (direction, wire_name, expected) in checks {
            let spec = specs
                .iter()
                .find(|entry| entry.direction == *direction && entry.wire_name == *wire_name)
                .unwrap_or_else(|| panic!("missing manifest entry for {direction:?} {wire_name}"));
            if require_concrete {
                assert_ne!(
                    spec.field_order, "unparsed",
                    "{direction:?} {wire_name} must stay concretely documented"
                );
            }
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
