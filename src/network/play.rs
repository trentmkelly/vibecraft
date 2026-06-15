#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Read, Write};
use std::sync::Arc;

use crate::block_entity::BLOCK_ENTITY_TYPES;
use crate::inventory::{Menu, Slot};
use crate::inventory_transactions::{
    apply_scripted_packet, InventoryTransactionResult, ScriptedContainerClickPacket, SlotCorrection,
};
use crate::item_catalog::item_protocol_id;
use crate::item_properties::ItemComponent;
use crate::item_stack::ItemStack;
use crate::network::codec::{
    read_collection, read_enum_index, read_identifier, read_optional, read_string,
    read_trusted_component, read_uuid, write_bitset, write_collection, write_enum_index,
    write_identifier, write_optional, write_string, write_trusted_component, write_uuid,
    ComponentJson, Uuid,
};
use crate::network::common::ServerboundResourcePackPacket;
use crate::network::dispatch::{DecodedPacket, DispatchOutcome, PacketDirection, ProtocolState};
use crate::network::varint::{read_var_i32, read_var_i64, write_var_i32, write_var_i64};
use crate::player_inventory::InventoryMenu;
use crate::registry::Identifier;
use crate::storage::chunk::{
    pack_chunk_pos_as_long, unpack_chunk_pos_from_long, ChunkSection, LevelChunk,
    PalettedContainer,
};
use crate::storage::nbt::Tag;
use crate::storage::region::ChunkPos;
use crate::world_time::ClockNetworkState;
#[cfg(test)]
use crate::world_time::OVERWORLD_CLOCK_ID;

pub const SERVERBOUND_PLAY_PACKET_COUNT_26_1_2: usize = 69;
pub const CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2: usize = 141;
pub const OVERWORLD_MIN_SECTION_Y: i32 = -4;
pub const OVERWORLD_SECTION_COUNT: usize = 24;

pub const SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID: i32 = 0;
pub const SERVERBOUND_ATTACK_PACKET_ID: i32 = 1;
pub const SERVERBOUND_BLOCK_ENTITY_TAG_QUERY_PACKET_ID: i32 = 2;
pub const SERVERBOUND_SELECT_BUNDLE_ITEM_PACKET_ID: i32 = 3;
pub const SERVERBOUND_CHANGE_DIFFICULTY_PACKET_ID: i32 = 4;
pub const SERVERBOUND_CHANGE_GAME_MODE_PACKET_ID: i32 = 5;
pub const SERVERBOUND_CHAT_ACK_PACKET_ID: i32 = 6;
pub const SERVERBOUND_CHAT_COMMAND_PACKET_ID: i32 = 7;
pub const SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID: i32 = 8;
pub const SERVERBOUND_CHAT_PACKET_ID: i32 = 9;
pub const SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID: i32 = 10;
pub const SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID: i32 = 11;
pub const SERVERBOUND_CLIENT_COMMAND_PACKET_ID: i32 = 12;
pub const SERVERBOUND_CLIENT_TICK_END_PACKET_ID: i32 = 13;
pub const SERVERBOUND_CLIENT_INFORMATION_PACKET_ID: i32 = 14;
pub const SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID: i32 = 15;
pub const SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID: i32 = 16;
pub const SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID: i32 = 17;
pub const SERVERBOUND_CONTAINER_CLICK_PACKET_ID: i32 = 18;
pub const SERVERBOUND_CONTAINER_CLOSE_PACKET_ID: i32 = 19;
pub const SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED_PACKET_ID: i32 = 20;
pub const SERVERBOUND_CUSTOM_PAYLOAD_PACKET_ID: i32 = 22;
pub const SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST_PACKET_ID: i32 = 23;
pub const SERVERBOUND_EDIT_BOOK_PACKET_ID: i32 = 24;
pub const SERVERBOUND_ENTITY_TAG_QUERY_PACKET_ID: i32 = 25;
pub const SERVERBOUND_INTERACT_PACKET_ID: i32 = 26;
pub const SERVERBOUND_JIGSAW_GENERATE_PACKET_ID: i32 = 27;
pub const SERVERBOUND_LOCK_DIFFICULTY_PACKET_ID: i32 = 29;
pub const SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID: i32 = 30;
pub const SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID: i32 = 31;
pub const SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID: i32 = 32;
pub const SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID: i32 = 33;
pub const SERVERBOUND_MOVE_VEHICLE_PACKET_ID: i32 = 34;
pub const SERVERBOUND_KEEP_ALIVE_PACKET_ID: i32 = 28;
pub const SERVERBOUND_PLAYER_ACTION_PACKET_ID: i32 = 41;
pub const SERVERBOUND_PLAYER_COMMAND_PACKET_ID: i32 = 42;
pub const SERVERBOUND_PADDLE_BOAT_PACKET_ID: i32 = 35;
pub const SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID: i32 = 36;
pub const SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID: i32 = 37;
pub const SERVERBOUND_PLACE_RECIPE_PACKET_ID: i32 = 39;
pub const SERVERBOUND_PLAYER_ABILITIES_PACKET_ID: i32 = 40;
pub const SERVERBOUND_PLAYER_INPUT_PACKET_ID: i32 = 43;
pub const SERVERBOUND_PLAYER_LOADED_PACKET_ID: i32 = 44;
pub const SERVERBOUND_PONG_PACKET_ID: i32 = 45;
pub const SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID: i32 = 46;
pub const SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID: i32 = 47;
pub const SERVERBOUND_RENAME_ITEM_PACKET_ID: i32 = 48;
pub const SERVERBOUND_RESOURCE_PACK_PACKET_ID: i32 = 49;
pub const SERVERBOUND_SEEN_ADVANCEMENTS_PACKET_ID: i32 = 50;
pub const SERVERBOUND_SELECT_TRADE_PACKET_ID: i32 = 51;
pub const SERVERBOUND_SET_BEACON_PACKET_ID: i32 = 52;
pub const SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID: i32 = 53;
pub const SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID: i32 = 54;
pub const SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID: i32 = 55;
pub const SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID: i32 = 56;
pub const SERVERBOUND_SET_GAME_RULE_PACKET_ID: i32 = 57;
pub const SERVERBOUND_SET_JIGSAW_BLOCK_PACKET_ID: i32 = 58;
pub const SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID: i32 = 59;
pub const SERVERBOUND_SET_TEST_BLOCK_PACKET_ID: i32 = 60;
pub const SERVERBOUND_SIGN_UPDATE_PACKET_ID: i32 = 61;
pub const SERVERBOUND_SPECTATE_ENTITY_PACKET_ID: i32 = 62;
pub const SERVERBOUND_SWING_PACKET_ID: i32 = 63;
pub const SERVERBOUND_TELEPORT_TO_ENTITY_PACKET_ID: i32 = 64;
pub const SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION_PACKET_ID: i32 = 65;
pub const SERVERBOUND_USE_ITEM_ON_PACKET_ID: i32 = 66;
pub const SERVERBOUND_USE_ITEM_PACKET_ID: i32 = 67;

pub const CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID: i32 = 0;
pub const CLIENTBOUND_LOGIN_PACKET_ID: i32 = 49;
pub const CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID: i32 = 11;
pub const CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID: i32 = 12;
pub const CLIENTBOUND_CHUNKS_BIOMES_PACKET_ID: i32 = 13;
pub const CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID: i32 = 10;
pub const CLIENTBOUND_ADD_ENTITY_PACKET_ID: i32 = 1;
pub const CLIENTBOUND_ANIMATE_PACKET_ID: i32 = 2;
pub const CLIENTBOUND_AWARD_STATS_PACKET_ID: i32 = 3;
pub const CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID: i32 = 4;
pub const CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID: i32 = 6;
pub const CLIENTBOUND_BLOCK_UPDATE_PACKET_ID: i32 = 8;
pub const CLIENTBOUND_BOSS_EVENT_PACKET_ID: i32 = 9;
pub const CLIENTBOUND_CLEAR_TITLES_PACKET_ID: i32 = 14;
pub const CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID: i32 = 15;
pub const CLIENTBOUND_COMMANDS_PACKET_ID: i32 = 16;
pub const CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID: i32 = 17;
pub const CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID: i32 = 18;
pub const CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID: i32 = 19;
pub const CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID: i32 = 20;
pub const CLIENTBOUND_COOLDOWN_PACKET_ID: i32 = 22;
pub const CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS_PACKET_ID: i32 = 23;
pub const CLIENTBOUND_CUSTOM_PAYLOAD_PACKET_ID: i32 = 24;
pub const CLIENTBOUND_DAMAGE_EVENT_PACKET_ID: i32 = 25;
pub const CLIENTBOUND_DEBUG_BLOCK_VALUE_PACKET_ID: i32 = 26;
pub const CLIENTBOUND_DEBUG_CHUNK_VALUE_PACKET_ID: i32 = 27;
pub const CLIENTBOUND_DEBUG_ENTITY_VALUE_PACKET_ID: i32 = 28;
pub const CLIENTBOUND_DEBUG_EVENT_PACKET_ID: i32 = 29;
pub const CLIENTBOUND_DEBUG_SAMPLE_PACKET_ID: i32 = 30;
pub const CLIENTBOUND_DELETE_CHAT_PACKET_ID: i32 = 31;
pub const CLIENTBOUND_ENTITY_EVENT_PACKET_ID: i32 = 34;
pub const CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID: i32 = 37;
pub const CLIENTBOUND_GAME_RULE_VALUES_PACKET_ID: i32 = 39;
pub const CLIENTBOUND_GAME_EVENT_PACKET_ID: i32 = 38;
pub const CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS_PACKET_ID: i32 = 40;
pub const CLIENTBOUND_HURT_ANIMATION_PACKET_ID: i32 = 42;
pub const CLIENTBOUND_MOUNT_SCREEN_OPEN_PACKET_ID: i32 = 41;
pub const CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID: i32 = 43;
pub const CLIENTBOUND_KEEP_ALIVE_PACKET_ID: i32 = 44;
pub const CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID: i32 = 45;
pub const CLIENTBOUND_LEVEL_EVENT_PACKET_ID: i32 = 46;
pub const CLIENTBOUND_LEVEL_PARTICLES_PACKET_ID: i32 = 47;
pub const CLIENTBOUND_LIGHT_UPDATE_PACKET_ID: i32 = 48;
pub const CLIENTBOUND_LOW_DISK_SPACE_WARNING_PACKET_ID: i32 = 50;
pub const CLIENTBOUND_MAP_ITEM_DATA_PACKET_ID: i32 = 51;
pub const CLIENTBOUND_MERCHANT_OFFERS_PACKET_ID: i32 = 52;
pub const CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID: i32 = 53;
pub const CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID: i32 = 54;
pub const CLIENTBOUND_MOVE_MINECART_PACKET_ID: i32 = 55;
pub const CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID: i32 = 56;
pub const CLIENTBOUND_MOVE_VEHICLE_PACKET_ID: i32 = 57;
pub const CLIENTBOUND_OPEN_BOOK_PACKET_ID: i32 = 58;
pub const CLIENTBOUND_OPEN_SCREEN_PACKET_ID: i32 = 59;
pub const CLIENTBOUND_OPEN_SIGN_EDITOR_PACKET_ID: i32 = 60;
pub const CLIENTBOUND_PING_PACKET_ID: i32 = 61;
pub const CLIENTBOUND_PLACE_GHOST_RECIPE_PACKET_ID: i32 = 63;
pub const CLIENTBOUND_PLAYER_CHAT_PACKET_ID: i32 = 65;
pub const CLIENTBOUND_PLAYER_COMBAT_END_PACKET_ID: i32 = 66;
pub const CLIENTBOUND_PLAYER_COMBAT_ENTER_PACKET_ID: i32 = 67;
pub const CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID: i32 = 68;
pub const CLIENTBOUND_PLAYER_INFO_REMOVE_PACKET_ID: i32 = 69;
pub const CLIENTBOUND_PLAYER_LOOK_AT_PACKET_ID: i32 = 71;
pub const CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID: i32 = 74;
pub const CLIENTBOUND_RECIPE_BOOK_REMOVE_PACKET_ID: i32 = 75;
pub const CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID: i32 = 76;
pub const CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID: i32 = 77;
pub const CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID: i32 = 78;
pub const CLIENTBOUND_RESET_SCORE_PACKET_ID: i32 = 79;
pub const CLIENTBOUND_RESOURCE_PACK_POP_PACKET_ID: i32 = 80;
pub const CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID: i32 = 81;
pub const CLIENTBOUND_PLAYER_POSITION_PACKET_ID: i32 = 72;
pub const CLIENTBOUND_PLAYER_ROTATION_PACKET_ID: i32 = 73;
pub const CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID: i32 = 64;
pub const CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID: i32 = 70;
pub const CLIENTBOUND_RESPAWN_PACKET_ID: i32 = 82;
pub const CLIENTBOUND_ROTATE_HEAD_PACKET_ID: i32 = 83;
pub const CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID: i32 = 84;
pub const CLIENTBOUND_SELECT_ADVANCEMENTS_TAB_PACKET_ID: i32 = 85;
pub const CLIENTBOUND_SERVER_DATA_PACKET_ID: i32 = 86;
pub const CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID: i32 = 87;
pub const CLIENTBOUND_SET_BORDER_CENTER_PACKET_ID: i32 = 88;
pub const CLIENTBOUND_SET_BORDER_LERP_SIZE_PACKET_ID: i32 = 89;
pub const CLIENTBOUND_SET_BORDER_SIZE_PACKET_ID: i32 = 90;
pub const CLIENTBOUND_SET_BORDER_WARNING_DELAY_PACKET_ID: i32 = 91;
pub const CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_PACKET_ID: i32 = 92;
pub const CLIENTBOUND_SET_CAMERA_PACKET_ID: i32 = 93;
pub const CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID: i32 = 94;
pub const CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID: i32 = 95;
pub const CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID: i32 = 96;
pub const CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID: i32 = 97;
pub const CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID: i32 = 98;
pub const CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID: i32 = 99;
pub const CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID: i32 = 100;
pub const CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID: i32 = 101;
pub const CLIENTBOUND_SET_EQUIPMENT_PACKET_ID: i32 = 102;
pub const CLIENTBOUND_SET_EXPERIENCE_PACKET_ID: i32 = 103;
pub const CLIENTBOUND_SET_HEALTH_PACKET_ID: i32 = 104;
pub const CLIENTBOUND_SET_HELD_SLOT_PACKET_ID: i32 = 105;
pub const CLIENTBOUND_SET_OBJECTIVE_PACKET_ID: i32 = 106;
pub const CLIENTBOUND_SET_PASSENGERS_PACKET_ID: i32 = 107;
/// Java: `net/minecraft/network/protocol/game/ClientboundSetPlayerInventoryPacket`
pub const CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID: i32 = 108;
pub const CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID: i32 = 109;
pub const CLIENTBOUND_SET_SCORE_PACKET_ID: i32 = 110;
pub const CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID: i32 = 112;
pub const CLIENTBOUND_SET_TIME_PACKET_ID: i32 = 113;
pub const CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID: i32 = 114;
pub const CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID: i32 = 115;
pub const CLIENTBOUND_SOUND_ENTITY_PACKET_ID: i32 = 116;
pub const CLIENTBOUND_SOUND_PACKET_ID: i32 = 117;
pub const CLIENTBOUND_START_CONFIGURATION_PACKET_ID: i32 = 118;
pub const CLIENTBOUND_STOP_SOUND_PACKET_ID: i32 = 119;
pub const CLIENTBOUND_SYSTEM_CHAT_PACKET_ID: i32 = 121;
pub const CLIENTBOUND_TAB_LIST_PACKET_ID: i32 = 122;
pub const CLIENTBOUND_TAG_QUERY_PACKET_ID: i32 = 123;
pub const CLIENTBOUND_DISCONNECT_PACKET_ID: i32 = 32;
pub const CLIENTBOUND_DISGUISED_CHAT_PACKET_ID: i32 = 33;
pub const CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID: i32 = 35;
pub const CLIENTBOUND_EXPLODE_PACKET_ID: i32 = 36;
/// Java: `net/minecraft/network/protocol/game/ClientboundTakeItemEntityPacket`
pub const CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID: i32 = 124;
pub const CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID: i32 = 125;
pub const CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID: i32 = 130;
pub const CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID: i32 = 131;
pub const CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID: i32 = 132;
pub const CLIENTBOUND_UPDATE_RECIPES_PACKET_ID: i32 = 133;
pub const CLIENTBOUND_PROJECTILE_POWER_PACKET_ID: i32 = 135;
pub const CLIENTBOUND_WAYPOINT_PACKET_ID: i32 = 138;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientGamePacketListenerHandler {
    pub method: &'static str,
    pub packet: &'static str,
}

pub const CLIENT_GAME_PACKET_LISTENER_PROTOCOL: ProtocolState = ProtocolState::Play;
pub const CLIENT_GAME_PACKET_LISTENER_HANDLERS: &[ClientGamePacketListenerHandler] = &[
    client_game_handler("handleAddEntity", "ClientboundAddEntityPacket"),
    client_game_handler("handleAddObjective", "ClientboundSetObjectivePacket"),
    client_game_handler("handleAnimate", "ClientboundAnimatePacket"),
    client_game_handler("handleHurtAnimation", "ClientboundHurtAnimationPacket"),
    client_game_handler("handleAwardStats", "ClientboundAwardStatsPacket"),
    client_game_handler("handleRecipeBookAdd", "ClientboundRecipeBookAddPacket"),
    client_game_handler("handleRecipeBookRemove", "ClientboundRecipeBookRemovePacket"),
    client_game_handler("handleRecipeBookSettings", "ClientboundRecipeBookSettingsPacket"),
    client_game_handler("handleBlockDestruction", "ClientboundBlockDestructionPacket"),
    client_game_handler("handleOpenSignEditor", "ClientboundOpenSignEditorPacket"),
    client_game_handler("handleBlockEntityData", "ClientboundBlockEntityDataPacket"),
    client_game_handler("handleBlockEvent", "ClientboundBlockEventPacket"),
    client_game_handler("handleBlockUpdate", "ClientboundBlockUpdatePacket"),
    client_game_handler("handleSystemChat", "ClientboundSystemChatPacket"),
    client_game_handler("handlePlayerChat", "ClientboundPlayerChatPacket"),
    client_game_handler("handleDisguisedChat", "ClientboundDisguisedChatPacket"),
    client_game_handler("handleDeleteChat", "ClientboundDeleteChatPacket"),
    client_game_handler("handleChunkBlocksUpdate", "ClientboundSectionBlocksUpdatePacket"),
    client_game_handler("handleMapItemData", "ClientboundMapItemDataPacket"),
    client_game_handler("handleContainerClose", "ClientboundContainerClosePacket"),
    client_game_handler("handleContainerContent", "ClientboundContainerSetContentPacket"),
    client_game_handler("handleMountScreenOpen", "ClientboundMountScreenOpenPacket"),
    client_game_handler("handleContainerSetData", "ClientboundContainerSetDataPacket"),
    client_game_handler("handleContainerSetSlot", "ClientboundContainerSetSlotPacket"),
    client_game_handler("handleEntityEvent", "ClientboundEntityEventPacket"),
    client_game_handler("handleEntityLinkPacket", "ClientboundSetEntityLinkPacket"),
    client_game_handler("handleSetEntityPassengersPacket", "ClientboundSetPassengersPacket"),
    client_game_handler("handleExplosion", "ClientboundExplodePacket"),
    client_game_handler("handleGameEvent", "ClientboundGameEventPacket"),
    client_game_handler("handleLevelChunkWithLight", "ClientboundLevelChunkWithLightPacket"),
    client_game_handler("handleChunksBiomes", "ClientboundChunksBiomesPacket"),
    client_game_handler("handleForgetLevelChunk", "ClientboundForgetLevelChunkPacket"),
    client_game_handler("handleLevelEvent", "ClientboundLevelEventPacket"),
    client_game_handler("handleLogin", "ClientboundLoginPacket"),
    client_game_handler("handleMoveEntity", "ClientboundMoveEntityPacket"),
    client_game_handler("handleMinecartAlongTrack", "ClientboundMoveMinecartPacket"),
    client_game_handler("handleMovePlayer", "ClientboundPlayerPositionPacket"),
    client_game_handler("handleRotatePlayer", "ClientboundPlayerRotationPacket"),
    client_game_handler("handleParticleEvent", "ClientboundLevelParticlesPacket"),
    client_game_handler("handlePlayerAbilities", "ClientboundPlayerAbilitiesPacket"),
    client_game_handler("handleGameRuleValues", "ClientboundGameRuleValuesPacket"),
    client_game_handler("handlePlayerInfoRemove", "ClientboundPlayerInfoRemovePacket"),
    client_game_handler("handlePlayerInfoUpdate", "ClientboundPlayerInfoUpdatePacket"),
    client_game_handler("handleRemoveEntities", "ClientboundRemoveEntitiesPacket"),
    client_game_handler("handleRemoveMobEffect", "ClientboundRemoveMobEffectPacket"),
    client_game_handler("handleRespawn", "ClientboundRespawnPacket"),
    client_game_handler("handleRotateMob", "ClientboundRotateHeadPacket"),
    client_game_handler("handleSetHeldSlot", "ClientboundSetHeldSlotPacket"),
    client_game_handler("handleSetDisplayObjective", "ClientboundSetDisplayObjectivePacket"),
    client_game_handler("handleSetEntityData", "ClientboundSetEntityDataPacket"),
    client_game_handler("handleSetEntityMotion", "ClientboundSetEntityMotionPacket"),
    client_game_handler("handleSetEquipment", "ClientboundSetEquipmentPacket"),
    client_game_handler("handleSetExperience", "ClientboundSetExperiencePacket"),
    client_game_handler("handleSetHealth", "ClientboundSetHealthPacket"),
    client_game_handler("handleSetPlayerTeamPacket", "ClientboundSetPlayerTeamPacket"),
    client_game_handler("handleSetScore", "ClientboundSetScorePacket"),
    client_game_handler("handleResetScore", "ClientboundResetScorePacket"),
    client_game_handler("handleSetSpawn", "ClientboundSetDefaultSpawnPositionPacket"),
    client_game_handler("handleSetTime", "ClientboundSetTimePacket"),
    client_game_handler("handleSoundEvent", "ClientboundSoundPacket"),
    client_game_handler("handleSoundEntityEvent", "ClientboundSoundEntityPacket"),
    client_game_handler("handleTakeItemEntity", "ClientboundTakeItemEntityPacket"),
    client_game_handler("handleEntityPositionSync", "ClientboundEntityPositionSyncPacket"),
    client_game_handler("handleTeleportEntity", "ClientboundTeleportEntityPacket"),
    client_game_handler("handleTickingState", "ClientboundTickingStatePacket"),
    client_game_handler("handleTickingStep", "ClientboundTickingStepPacket"),
    client_game_handler("handleUpdateAttributes", "ClientboundUpdateAttributesPacket"),
    client_game_handler("handleUpdateMobEffect", "ClientboundUpdateMobEffectPacket"),
    client_game_handler("handlePlayerCombatEnd", "ClientboundPlayerCombatEndPacket"),
    client_game_handler("handlePlayerCombatEnter", "ClientboundPlayerCombatEnterPacket"),
    client_game_handler("handlePlayerCombatKill", "ClientboundPlayerCombatKillPacket"),
    client_game_handler("handleChangeDifficulty", "ClientboundChangeDifficultyPacket"),
    client_game_handler("handleSetCamera", "ClientboundSetCameraPacket"),
    client_game_handler("handleInitializeBorder", "ClientboundInitializeBorderPacket"),
    client_game_handler("handleSetBorderLerpSize", "ClientboundSetBorderLerpSizePacket"),
    client_game_handler("handleSetBorderSize", "ClientboundSetBorderSizePacket"),
    client_game_handler("handleSetBorderWarningDelay", "ClientboundSetBorderWarningDelayPacket"),
    client_game_handler(
        "handleSetBorderWarningDistance",
        "ClientboundSetBorderWarningDistancePacket",
    ),
    client_game_handler("handleSetBorderCenter", "ClientboundSetBorderCenterPacket"),
    client_game_handler("handleTabListCustomisation", "ClientboundTabListPacket"),
    client_game_handler("handleBossUpdate", "ClientboundBossEventPacket"),
    client_game_handler("handleItemCooldown", "ClientboundCooldownPacket"),
    client_game_handler("handleMoveVehicle", "ClientboundMoveVehiclePacket"),
    client_game_handler("handleUpdateAdvancementsPacket", "ClientboundUpdateAdvancementsPacket"),
    client_game_handler("handleSelectAdvancementsTab", "ClientboundSelectAdvancementsTabPacket"),
    client_game_handler("handlePlaceRecipe", "ClientboundPlaceGhostRecipePacket"),
    client_game_handler("handleCommands", "ClientboundCommandsPacket"),
    client_game_handler("handleStopSoundEvent", "ClientboundStopSoundPacket"),
    client_game_handler("handleCommandSuggestions", "ClientboundCommandSuggestionsPacket"),
    client_game_handler("handleUpdateRecipes", "ClientboundUpdateRecipesPacket"),
    client_game_handler("handleLookAt", "ClientboundPlayerLookAtPacket"),
    client_game_handler("handleTagQueryPacket", "ClientboundTagQueryPacket"),
    client_game_handler("handleLightUpdatePacket", "ClientboundLightUpdatePacket"),
    client_game_handler("handleOpenBook", "ClientboundOpenBookPacket"),
    client_game_handler("handleOpenScreen", "ClientboundOpenScreenPacket"),
    client_game_handler("handleMerchantOffers", "ClientboundMerchantOffersPacket"),
    client_game_handler("handleSetChunkCacheRadius", "ClientboundSetChunkCacheRadiusPacket"),
    client_game_handler("handleSetSimulationDistance", "ClientboundSetSimulationDistancePacket"),
    client_game_handler("handleSetChunkCacheCenter", "ClientboundSetChunkCacheCenterPacket"),
    client_game_handler("handleBlockChangedAck", "ClientboundBlockChangedAckPacket"),
    client_game_handler("setActionBarText", "ClientboundSetActionBarTextPacket"),
    client_game_handler("setSubtitleText", "ClientboundSetSubtitleTextPacket"),
    client_game_handler("setTitleText", "ClientboundSetTitleTextPacket"),
    client_game_handler("setTitlesAnimation", "ClientboundSetTitlesAnimationPacket"),
    client_game_handler("handleTitlesClear", "ClientboundClearTitlesPacket"),
    client_game_handler("handleServerData", "ClientboundServerDataPacket"),
    client_game_handler("handleCustomChatCompletions", "ClientboundCustomChatCompletionsPacket"),
    client_game_handler("handleBundlePacket", "ClientboundBundlePacket"),
    client_game_handler("handleDamageEvent", "ClientboundDamageEventPacket"),
    client_game_handler("handleConfigurationStart", "ClientboundStartConfigurationPacket"),
    client_game_handler("handleChunkBatchStart", "ClientboundChunkBatchStartPacket"),
    client_game_handler("handleChunkBatchFinished", "ClientboundChunkBatchFinishedPacket"),
    client_game_handler("handleDebugSample", "ClientboundDebugSamplePacket"),
    client_game_handler("handleProjectilePowerPacket", "ClientboundProjectilePowerPacket"),
    client_game_handler("handleSetCursorItem", "ClientboundSetCursorItemPacket"),
    client_game_handler("handleSetPlayerInventory", "ClientboundSetPlayerInventoryPacket"),
    client_game_handler("handleTestInstanceBlockStatus", "ClientboundTestInstanceBlockStatus"),
    client_game_handler("handleWaypoint", "ClientboundTrackedWaypointPacket"),
    client_game_handler("handleDebugChunkValue", "ClientboundDebugChunkValuePacket"),
    client_game_handler("handleDebugBlockValue", "ClientboundDebugBlockValuePacket"),
    client_game_handler("handleDebugEntityValue", "ClientboundDebugEntityValuePacket"),
    client_game_handler("handleDebugEvent", "ClientboundDebugEventPacket"),
    client_game_handler("handleGameTestHighlightPos", "ClientboundGameTestHighlightPosPacket"),
    client_game_handler("handleLowDiskSpaceWarning", "ClientboundLowDiskSpaceWarningPacket"),
];

const fn client_game_handler(
    method: &'static str,
    packet: &'static str,
) -> ClientGamePacketListenerHandler {
    ClientGamePacketListenerHandler { method, packet }
}

mod chunk_types_a;
pub use chunk_types_a::*;

mod chunk_types_b;
pub use chunk_types_b::*;

mod chunk_types_c;

mod chunk_impl_early;
pub use chunk_impl_early::*;

mod chunk_a;
pub use chunk_a::*;

mod chunk_b;

mod chunk_move_entity;

mod chunk_move_minecart;

mod chunk_c;
use chunk_c::*;

mod chunk_c2;
use chunk_c2::*;

mod chunk_d;
pub use chunk_d::*;

mod chunk_d2;
use chunk_d2::*;

mod chunk_e;
pub use chunk_e::*;

mod chunk_set_default_spawn_position;

mod chunk_e2;
mod chunk_e3;
use chunk_e2::*;

mod chunk_waypoint;

#[cfg(test)]
mod tests;
