use super::*;

pub struct PlayProtocolRegistry {
    pub(super) serverbound: Vec<&'static str>,
    pub(super) clientbound: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonPlayerSpawnInfo {
    pub dimension_type: Identifier,
    pub dimension: Identifier,
    pub seed: i64,
    pub game_mode: GameMode,
    pub previous_game_mode: Option<GameMode>,
    pub is_debug: bool,
    pub is_flat: bool,
    pub last_death_location: Option<(Identifier, [i32; 3])>,
    pub portal_cooldown: i32,
    pub sea_level: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLoginPacket {
    pub player_id: i32,
    pub hardcore: bool,
    pub levels: Vec<Identifier>,
    pub max_players: i32,
    pub chunk_radius: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
    pub do_limited_crafting: bool,
    pub spawn_info: CommonPlayerSpawnInfo,
    pub enforces_secure_chat: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundMovePlayerPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
    pub horizontal_collision: bool,
    pub has_position: bool,
    pub has_rotation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerboundMoveVehiclePacket {
    pub position: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundAcceptTeleportationPacket {
    pub teleport_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundSetCarriedItemPacket {
    pub slot: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundChangeDifficultyPacket {
    pub difficulty: GameDifficulty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundChatAckPacket {
    pub offset: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSignature(pub [u8; MessageSignature::BYTES]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenMessagesUpdate {
    pub offset: i32,
    pub acknowledged: Vec<u8>,
    pub checksum: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentSignature {
    pub name: String,
    pub signature: MessageSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatPacket {
    pub message: String,
    pub timestamp_epoch_millis: i64,
    pub salt: i64,
    pub signature: Option<MessageSignature>,
    pub last_seen_messages: LastSeenMessagesUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatCommandPacket {
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatCommandSignedPacket {
    pub command: String,
    pub timestamp_epoch_millis: i64,
    pub salt: i64,
    pub argument_signatures: Vec<ArgumentSignature>,
    pub last_seen_messages: LastSeenMessagesUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatSessionUpdatePacket {
    pub session_id: Uuid,
    pub expires_at_epoch_millis: i64,
    pub public_key: Vec<u8>,
    pub key_signature: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundClientTickEndPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundLockDifficultyPacket {
    pub locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPaddleBoatPacket {
    pub left: bool,
    pub right: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shift: bool,
    pub sprint: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerInputPacket {
    pub input: ServerboundPlayerInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerLoadedPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundPlayerCommandAction {
    StopSleeping,
    StartSprinting,
    StopSprinting,
    StartRidingJump,
    StopRidingJump,
    OpenInventory,
    StartFallFlying,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerCommandPacket {
    pub entity_id: i32,
    pub action: ServerboundPlayerCommandAction,
    pub data: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction3d {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundPlayerAction {
    StartDestroyBlock,
    AbortDestroyBlock,
    StopDestroyBlock,
    DropAllItems,
    DropItem,
    ReleaseUseItem,
    SwapItemWithOffhand,
    Stab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerActionPacket {
    pub action: ServerboundPlayerAction,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub direction: Direction3d,
    pub sequence: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerboundUseItemPacket {
    pub hand: ServerboundSwingHand,
    pub sequence: i32,
    pub y_rot: f32,
    pub x_rot: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockHitResultPacketData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub direction: Direction3d,
    pub click_x: f32,
    pub click_y: f32,
    pub click_z: f32,
    pub inside: bool,
    pub world_border_hit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerboundUseItemOnPacket {
    pub hand: ServerboundSwingHand,
    pub block_hit: BlockHitResultPacketData,
    pub sequence: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPongPacket {
    pub id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundJigsawGeneratePacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub levels: i32,
    pub keep_jigsaws: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSignUpdatePacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub is_front_text: bool,
    pub lines: [String; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundSetBeaconPacket {
    pub primary_effect_id: Option<i32>,
    pub secondary_effect_id: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBlockMode {
    Sequence,
    Auto,
    Redstone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetCommandBlockPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub command: String,
    pub mode: CommandBlockMode,
    pub track_output: bool,
    pub conditional: bool,
    pub automatic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetCommandMinecartPacket {
    pub entity_id: i32,
    pub command: String,
    pub track_output: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureBlockUpdateType {
    UpdateData,
    SaveArea,
    LoadArea,
    ScanArea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureBlockMode {
    Save,
    Load,
    Corner,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureMirror {
    None,
    LeftRight,
    FrontBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureRotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundSetStructureBlockPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub update_type: StructureBlockUpdateType,
    pub mode: StructureBlockMode,
    pub name: String,
    pub offset: [i8; 3],
    pub size: [u8; 3],
    pub mirror: StructureMirror,
    pub rotation: StructureRotation,
    pub data: String,
    pub integrity: f32,
    pub seed: i64,
    pub ignore_entities: bool,
    pub strict: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundSelectTradePacket {
    pub item: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundRenameItemPacket {
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundContainerClosePacket {
    pub container_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundContainerButtonClickPacket {
    pub container_id: i32,
    pub button_id: i32,
}

/// `ServerboundContainerSlotStateChangedPacket` — toggles a crafter grid slot
/// enabled/disabled (`slot_id`, `container_id`, `new_state`). 1:1 with the Java
/// record `(slotId: VarInt, containerId: ContainerId(VarInt), newState: bool)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundContainerSlotStateChangedPacket {
    pub slot_id: i32,
    pub container_id: i32,
    pub new_state: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundMerchantOffersPacket {
    pub container_id: i32,
    pub offers: Vec<MerchantOfferData>,
    pub villager_level: i32,
    pub villager_xp: i32,
    pub show_progress: bool,
    pub can_restock: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MerchantOfferData {
    pub base_cost_a: ItemCostData,
    pub result: RawItemStack,
    pub cost_b: Option<ItemCostData>,
    pub out_of_stock: bool,
    pub uses: i32,
    pub max_uses: i32,
    pub xp: i32,
    pub special_price_diff: i32,
    pub price_multiplier: f32,
    pub demand: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemCostData {
    pub item_id: i32,
    pub count: i32,
    pub components: RawDataComponentExactPredicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDataComponentPatch {
    pub added: Vec<(i32, Vec<u8>)>,
    pub removed: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDataComponentExactPredicate {
    pub expected_components: Vec<(i32, Vec<u8>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawItemStack {
    pub count: i32,
    pub item_id: Option<i32>,
    pub components: RawDataComponentPatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedPatchMap {
    pub added_component_hashes: Vec<(i32, i32)>,
    pub removed_components: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedStack {
    pub item_id: Option<i32>,
    pub count: i32,
    pub components: HashedPatchMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerInput {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundContainerClickPacket {
    pub container_id: i32,
    pub state_id: i32,
    pub slot_num: i16,
    pub button_num: i8,
    pub container_input: ContainerInput,
    pub changed_slots: BTreeMap<i32, HashedStack>,
    pub carried_item: HashedStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetCreativeModeSlotPacket {
    pub slot_num: i16,
    pub item_stack: RawItemStack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerAbilitiesPacket {
    pub is_flying: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCommandSuggestionPacket {
    pub id: i32,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundEditBookPacket {
    pub slot: i32,
    pub pages: Vec<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundAttackPacket {
    pub entity_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerboundInteractPacket {
    pub entity_id: i32,
    pub hand: ServerboundInteractionHand,
    pub location: Vec3,
    pub using_secondary_action: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundInteractionHand {
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPickItemFromBlockPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub include_data: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPickItemFromEntityPacket {
    pub entity_id: i32,
    pub include_data: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeBookType {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundRecipeBookChangeSettingsPacket {
    pub book_type: RecipeBookType,
    pub is_open: bool,
    pub is_filtering: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundRecipeBookSeenRecipePacket {
    pub recipe_index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlaceRecipePacket {
    pub container_id: i32,
    pub recipe_index: i32,
    pub use_max_items: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundConfigurationAcknowledgedPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundClientCommandAction {
    PerformRespawn,
    RequestStats,
    RequestGameruleValues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundClientCommandPacket {
    pub action: ServerboundClientCommandAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundSwingHand {
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundSwingPacket {
    pub hand: ServerboundSwingHand,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundChangeDifficultyPacket {
    pub difficulty: GameDifficulty,
    pub locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetChunkCacheCenterPacket {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetChunkCacheRadiusPacket {
    pub radius: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetDefaultSpawnPositionData {
    pub dimension: Identifier,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetDefaultSpawnPositionPacket {
    pub respawn_data: ClientboundSetDefaultSpawnPositionData,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetExperiencePacket {
    pub experience_progress: f32,
    pub experience_level: i32,
    pub total_experience: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetHealthPacket {
    pub health: f32,
    pub food: i32,
    pub saturation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetHeldSlotPacket {
    pub slot: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundBlockDestructionPacket {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub progress: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundBlockEventPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub action: u8,
    pub param: u8,
    pub block_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundBlockUpdatePacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_state_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundBlockChangedAckPacket {
    pub sequence: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundLevelEventPacket {
    pub event_type: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub data: i32,
    pub global_event: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerInfoRemovePacket {
    pub profile_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundContainerClosePacket {
    pub container_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundContainerSetDataPacket {
    pub container_id: i32,
    pub id: i16,
    pub value: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundMountScreenOpenPacket {
    pub container_id: i32,
    pub inventory_columns: i32,
    pub entity_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCooldownPacket {
    pub cooldown_group: Identifier,
    pub duration: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundPlayerAbilitiesPacket {
    pub invulnerable: bool,
    pub flying: bool,
    pub can_fly: bool,
    pub instant_build: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundPingPacket {
    pub id: i32,
}

/// Java: net/minecraft/network/protocol/game/ClientboundSetTimePacket.java
/// Wire: fixed i64 game_time (ByteBufCodecs.LONG), then VarInt map length followed by
/// (VarInt worldclock_registry_id, ClockNetworkState) pairs.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetTimePacket {
    pub game_time: i64,
    pub clock_updates: BTreeMap<i32, ClockNetworkState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientboundGameEventType {
    NoRespawnBlockAvailable,
    StartRaining,
    StopRaining,
    ChangeGameMode,
    WinGame,
    DemoEvent,
    PlayArrowHitSound,
    RainLevelChange,
    ThunderLevelChange,
    PufferFishSting,
    GuardianElderEffect,
    ImmediateRespawn,
    LimitedCrafting,
    LevelChunksLoadStart,
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundGameEventPacket {
    pub event: ClientboundGameEventType,
    pub param: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundHurtAnimationPacket {
    pub id: i32,
    pub yaw: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetSimulationDistancePacket {
    pub simulation_distance: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundTickingStatePacket {
    pub tick_rate: f32,
    pub is_frozen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundTickingStepPacket {
    pub tick_steps: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerboundChunkBatchReceivedPacket {
    pub desired_chunks_per_tick: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundChunkBatchFinishedPacket {
    pub batch_size: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundChunkBatchStartPacket;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundAddEntityPacket {
    pub id: i32,
    pub uuid: Uuid,
    pub entity_type: i32,
    pub position: Vec3,
    pub movement: Vec3,
    pub x_rot: u8,
    pub y_rot: u8,
    pub y_head_rot: u8,
    pub data: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddEntityPacketInput {
    pub id: i32,
    pub uuid: Uuid,
    pub entity_type: i32,
    pub position: Vec3,
    pub movement: Vec3,
    pub rotation: (f32, f32),
    pub y_head_rot: f32,
    pub data: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRemoveEntitiesPacket {
    pub entity_ids: Vec<i32>,
}

/// Triggers the item-pickup animation and sound on all clients tracking the item.
///
/// Java: `net/minecraft/network/protocol/game/ClientboundTakeItemEntityPacket`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTakeItemEntityPacket {
    /// Entity ID of the item entity being collected.
    pub item_entity_id: i32,
    /// Entity ID of the collecting player.
    pub collector_entity_id: i32,
    /// Number of items absorbed in this pickup event.
    pub amount: i32,
}

/// Synchronises a single player-inventory slot to the client.
/// Uses the player's own inventory numbering:
///   0–35  main inventory (hotbar at 0–8, storage at 9–35)
///   36–39 armour (boots/leggings/chestplate/helmet)
///   40    offhand
///
/// Java: `net/minecraft/network/protocol/game/ClientboundSetPlayerInventoryPacket`
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetPlayerInventoryPacket {
    pub slot: i32,
    pub contents: RawItemStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetEntityDataPacket {
    pub id: i32,
    pub packed_items: Vec<EntityDataValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDataValue {
    pub index: u8,
    pub serializer_id: i32,
    pub encoded_payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityMetadataValue {
    Byte(i8),
    VarInt(i32),
    VarLong(i64),
    Float(f32),
    String(String),
    Component(Vec<u8>),
    OptionalComponent(Option<Vec<u8>>),
    ItemStack(RawItemStack),
    Boolean(bool),
    Rotations(Rotations),
    BlockPos(BlockPosition),
    OptionalBlockPos(Option<BlockPosition>),
    Direction(DirectionData),
    OptionalLivingEntityReference(Option<i32>),
    BlockState(i32),
    OptionalBlockState(Option<i32>),
    Particle(RawParticleOptions),
    Particles(Vec<RawParticleOptions>),
    VillagerData(VillagerData),
    OptionalUnsignedInt(Option<i32>),
    Pose(PoseData),
    CatVariant(i32),
    CatSoundVariant(i32),
    CowVariant(i32),
    CowSoundVariant(i32),
    WolfVariant(i32),
    WolfSoundVariant(i32),
    FrogVariant(i32),
    PigVariant(i32),
    PigSoundVariant(i32),
    ChickenVariant(i32),
    ChickenSoundVariant(i32),
    ZombieNautilusVariant(i32),
    OptionalGlobalPos(Option<GlobalPosData>),
    PaintingVariant(i32),
    SnifferState(SnifferStateData),
    ArmadilloState(ArmadilloStateData),
    CopperGolemState(CopperGolemStateData),
    WeatheringCopperState(WeatheringCopperStateData),
    Vector3f(Vector3fData),
    Quaternionf(QuaternionfData),
    ResolvableProfile(Vec<u8>),
    HumanoidArm(HumanoidArmData),
    Raw {
        serializer_id: i32,
        encoded_payload: Vec<u8>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotations {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPosData {
    pub dimension: Identifier,
    pub pos: BlockPosition,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3fData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuaternionfData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VillagerData {
    pub villager_type: i32,
    pub profession: i32,
    pub level: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionData {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoseData {
    Standing,
    FallFlying,
    Sleeping,
    Swimming,
    SpinAttack,
    Crouching,
    LongJumping,
    Dying,
    Croaking,
    UsingTongue,
    Sitting,
    Roaring,
    Sniffing,
    Emerging,
    Digging,
    Sliding,
    Shooting,
    Inhaling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnifferStateData {
    Idling,
    FeelingHappy,
    Scenting,
    Sniffing,
    Searching,
    Digging,
    Rising,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmadilloStateData {
    Idle,
    Rolling,
    Scared,
    Unrolling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopperGolemStateData {
    Unoxidized,
    Exposed,
    Weathered,
    Oxidized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatheringCopperStateData {
    Unaffected,
    Exposed,
    Weathered,
    Oxidized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanoidArmData {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetEntityMotionPacket {
    pub id: i32,
    pub movement: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundMoveVehiclePacket {
    pub position: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundTeleportEntityPacket {
    pub id: i32,
    pub position: Vec3,
    pub movement: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub relative_flags: u32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundEntityPositionSyncPacket {
    pub id: i32,
    pub position: Vec3,
    pub movement: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundPlayerPositionPacket {
    pub id: i32,
    pub position: Vec3,
    pub movement: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub relative_flags: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundMoveEntityPacket {
    pub id: i32,
    pub delta: [i16; 3],
    pub y_rot: u8,
    pub x_rot: u8,
    pub on_ground: bool,
    pub has_position: bool,
    pub has_rotation: bool,
}
