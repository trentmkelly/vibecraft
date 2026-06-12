use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundChangeGameModePacket {
    pub mode: GameMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundDebugSubscriptionRequestPacket {
    pub subscriptions: BTreeSet<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundEntityTagQueryPacket {
    pub transaction_id: i32,
    pub entity_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundBlockEntityTagQueryPacket {
    pub transaction_id: i32,
    pub pos: crate::block_update::BlockPos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundTagQueryPacket {
    pub transaction_id: i32,
    pub tag: Option<Tag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundGameTestHighlightPosPacket {
    pub absolute_pos: crate::block_update::BlockPos,
    pub relative_pos: crate::block_update::BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundRotateHeadPacket {
    pub id: i32,
    pub y_head_rot: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetPassengersPacket {
    pub vehicle: i32,
    pub passengers: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundEntityEventPacket {
    pub entity_id: i32,
    pub event_id: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetEntityLinkPacket {
    pub source_id: i32,
    pub dest_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetEquipmentPacket {
    pub entity: i32,
    pub slots: Vec<EquipmentEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentEntry {
    pub slot: EquipmentSlotKind,
    pub item_stack: RawItemStack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlotKind {
    MainHand = 0,
    OffHand = 1,
    Feet = 2,
    Legs = 3,
    Chest = 4,
    Head = 5,
    Body = 6,
    Saddle = 7,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundUpdateAttributesPacket {
    pub entity_id: i32,
    pub attributes: Vec<AttributeSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSnapshot {
    pub attribute_id: i32,
    pub base: f64,
    pub modifiers: Vec<AttributeModifierSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeModifierSnapshot {
    pub id: Identifier,
    pub amount: f64,
    pub operation: AttributeModifierOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeModifierOperation {
    Value = 0,
    MultipliedBase = 1,
    MultipliedTotal = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundUpdateMobEffectPacket {
    pub entity_id: i32,
    pub effect_id: i32,
    pub amplifier: i32,
    pub duration_ticks: i32,
    pub flags: MobEffectFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobEffectFlags(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundRemoveMobEffectPacket {
    pub entity_id: i32,
    pub effect_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundAnimatePacket {
    pub id: i32,
    pub action: EntityAnimation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAnimation {
    SwingMainHand = 0,
    WakeUp = 2,
    SwingOffHand = 3,
    CriticalHit = 4,
    MagicCriticalHit = 5,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundInitializeBorderPacket {
    pub new_center_x: f64,
    pub new_center_z: f64,
    pub old_size: f64,
    pub new_size: f64,
    pub lerp_time: i64,
    pub new_absolute_max_size: i32,
    pub warning_blocks: i32,
    pub warning_time: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetBorderCenterPacket {
    pub new_center_x: f64,
    pub new_center_z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetBorderLerpSizePacket {
    pub old_size: f64,
    pub new_size: f64,
    pub lerp_time: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetBorderSizePacket {
    pub size: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetBorderWarningDelayPacket {
    pub warning_delay: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetBorderWarningDistancePacket {
    pub warning_blocks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetCameraPacket {
    pub camera_id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntitySpawnBundle {
    pub spawn: ClientboundAddEntityPacket,
    pub metadata: Option<ClientboundSetEntityDataPacket>,
    pub velocity: Option<ClientboundSetEntityMotionPacket>,
    pub equipment: Option<ClientboundSetEquipmentPacket>,
    pub attributes: Option<ClientboundUpdateAttributesPacket>,
    pub effects: Vec<ClientboundUpdateMobEffectPacket>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundContainerPacket {
    pub container_id: i32,
    pub state_id: i32,
    pub slots: Vec<RawItemStack>,
    pub carried_item: RawItemStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundContainerSetSlotPacket {
    pub container_id: i32,
    pub state_id: i32,
    pub slot: i16,
    pub item_stack: RawItemStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetCursorItemPacket {
    pub item_stack: RawItemStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipePacket {
    pub recipes: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipeBookAddPacket {
    pub entries: Vec<RecipeBookAddEntry>,
    pub replace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeBookAddEntry {
    pub contents: RecipeDisplayEntryData,
    pub flags: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDisplayEntryData {
    pub id: i32,
    pub display: RecipeDisplayData,
    pub group: Option<i32>,
    pub category_id: i32,
    pub crafting_requirements: Option<Vec<RecipeIngredientData>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeDisplayData {
    CraftingShapeless {
        ingredients: Vec<SlotDisplayData>,
        result: SlotDisplayData,
        crafting_station: SlotDisplayData,
    },
    CraftingShaped {
        width: i32,
        height: i32,
        ingredients: Vec<SlotDisplayData>,
        result: SlotDisplayData,
        crafting_station: SlotDisplayData,
    },
    Furnace {
        ingredient: SlotDisplayData,
        fuel: SlotDisplayData,
        result: SlotDisplayData,
        crafting_station: SlotDisplayData,
        duration: i32,
        experience_bits: u32,
    },
    Stonecutter {
        ingredient: SlotDisplayData,
        result: SlotDisplayData,
        crafting_station: SlotDisplayData,
    },
    Smithing {
        template: SlotDisplayData,
        base: SlotDisplayData,
        addition: SlotDisplayData,
        result: SlotDisplayData,
        crafting_station: SlotDisplayData,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotDisplayData {
    Empty,
    AnyFuel,
    WithAnyPotion(Box<SlotDisplayData>),
    OnlyWithComponent {
        contents: Box<SlotDisplayData>,
        component_type_id: i32,
    },
    Item {
        item_id: i32,
    },
    ItemStack {
        stack: RawItemStack,
    },
    Tag {
        tag: Identifier,
    },
    Dyed {
        dye: Box<SlotDisplayData>,
        target: Box<SlotDisplayData>,
    },
    SmithingTrim {
        base: Box<SlotDisplayData>,
        material: Box<SlotDisplayData>,
        pattern_id: i32,
    },
    WithRemainder {
        input: Box<SlotDisplayData>,
        remainder: Box<SlotDisplayData>,
    },
    Composite(Vec<SlotDisplayData>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeIngredientData {
    DirectItems(Vec<i32>),
    Tag(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipeBookRemovePacket {
    pub recipe_display_ids: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeBookTypeSettings {
    pub open: bool,
    pub filtering: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundRecipeBookSettingsPacket {
    pub crafting: RecipeBookTypeSettings,
    pub furnace: RecipeBookTypeSettings,
    pub blast_furnace: RecipeBookTypeSettings,
    pub smoker: RecipeBookTypeSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundAdvancementsPacket {
    pub reset: bool,
    pub added: Vec<AdvancementHolderData>,
    pub removed: Vec<Identifier>,
    pub progress: Vec<(Identifier, AdvancementProgressData)>,
    pub show_advancements: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerInfoUpdatePacket {
    pub actions: Vec<PlayerInfoUpdateAction>,
    pub entries: Vec<PlayerInfoUpdateEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerChatPacket {
    pub global_index: i32,
    pub sender: Uuid,
    pub index: i32,
    pub signature: Option<Vec<u8>>,
    pub body: SignedMessageBodyPacked,
    pub unsigned_content_payload: Option<Vec<u8>>,
    pub filter_mask: FilterMaskData,
    pub chat_type: BoundChatTypeData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageBodyPacked {
    pub content: String,
    pub timestamp_epoch_millis: i64,
    pub salt: i64,
    pub last_seen: Vec<MessageSignaturePackedData>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageSignaturePackedData {
    Full(Vec<u8>),
    Id(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterMaskData {
    PassThrough,
    FullyFiltered,
    PartiallyFiltered(Vec<u64>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundChatTypeData {
    pub chat_type_id: i32,
    pub name_payload: Vec<u8>,
    pub target_name_payload: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerInfoUpdateAction {
    AddPlayer,
    InitializeChat,
    UpdateGameMode,
    UpdateListed,
    UpdateLatency,
    UpdateDisplayName,
    UpdateListOrder,
    UpdateHat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerInfoUpdateEntry {
    pub profile_id: Uuid,
    pub profile: Option<PlayerInfoProfile>,
    pub chat_session_payload: Option<Vec<u8>>,
    pub game_mode: i32,
    pub listed: bool,
    pub latency: i32,
    pub display_name_payload: Option<Vec<u8>>,
    pub list_order: i32,
    pub show_hat: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerInfoProfile {
    pub name: String,
    pub properties: Vec<GameProfileProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfileProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementHolderData {
    pub id: Identifier,
    pub value: AdvancementData,
}

impl AdvancementHolderData {
    pub fn java_equals_by_id(&self, other: &Self) -> bool {
        self.id == other.id
    }

    pub fn java_hash_key(&self) -> &Identifier {
        &self.id
    }

    pub fn java_to_string(&self) -> String {
        self.id.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementData {
    pub parent: Option<Identifier>,
    pub display_payload: Option<Vec<u8>>,
    pub requirements: Vec<Vec<String>>,
    pub sends_telemetry_event: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementProgressData {
    pub criteria: Vec<(String, CriterionProgressData)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CriterionProgressData {
    pub obtained_epoch_millis: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundAwardStatsPacket {
    pub stats: Vec<AwardedStat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AwardedStat {
    pub stat_type_id: i32,
    pub stat_value_id: i32,
    pub value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundPlayerLookAtPacket {
    pub from_anchor: EntityAnchor,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub target_entity: Option<(i32, EntityAnchor)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetTitleTextPacket {
    pub text: Tag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetSubtitleTextPacket {
    pub text: Tag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetActionBarTextPacket {
    pub text: Tag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSystemChatPacket {
    pub content: Tag,
    pub overlay: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundDisguisedChatPacket {
    pub message: Tag,
    pub chat_type: ChatTypeBound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomChatCompletionsPacket {
    pub action: CustomChatCompletionsAction,
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomChatCompletionsAction {
    Add,
    Remove,
    Set,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatTypeBound {
    pub chat_type_id: i32,
    pub name: Tag,
    pub target_name: Option<Tag>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundTabListPacket {
    pub header: Tag,
    pub footer: Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAnchor {
    Feet = 0,
    Eyes = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundResetScorePacket {
    pub owner: String,
    pub objective_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetDisplayObjectivePacket {
    pub slot: i32,
    pub objective_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetObjectivePacket {
    pub objective_name: String,
    pub method: ObjectiveMethod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveMethod {
    Add {
        display_name: Tag,
        render_type: ObjectiveRenderType,
        number_format: Option<NumberFormat>,
    },
    Remove,
    Change {
        display_name: Tag,
        render_type: ObjectiveRenderType,
        number_format: Option<NumberFormat>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectiveRenderType {
    Integer = 0,
    Hearts = 1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberFormat {
    Blank,
    Styled { style: Tag },
    Fixed { value: Tag },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetScorePacket {
    pub owner: String,
    pub objective_name: String,
    pub score: i32,
    pub display: Option<Tag>,
    pub number_format: Option<NumberFormat>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetPlayerTeamPacket {
    pub name: String,
    pub method: TeamPacketMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundLowDiskSpaceWarningPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundOpenBookPacket {
    pub hand: ClientboundInteractionHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientboundInteractionHand {
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundOpenScreenPacket {
    pub container_id: i32,
    pub menu_type_id: i32,
    pub title: Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundOpenSignEditorPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub is_front_text: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TeamPacketMethod {
    Create {
        parameters: TeamPacketParameters,
        players: Vec<String>,
    },
    Remove,
    Update {
        parameters: TeamPacketParameters,
    },
    AddPlayers {
        players: Vec<String>,
    },
    RemovePlayers {
        players: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TeamPacketParameters {
    pub display_name: Tag,
    pub options: u8,
    pub nametag_visibility: TeamVisibility,
    pub collision_rule: TeamCollisionRule,
    pub color_id: i32,
    pub prefix: Tag,
    pub suffix: Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamVisibility {
    Always = 0,
    Never = 1,
    HideForOtherTeams = 2,
    HideForOwnTeam = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamCollisionRule {
    Always = 0,
    Never = 1,
    PushOtherTeams = 2,
    PushOwnTeam = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundResourcePackPopPacket {
    pub id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDebugSamplePacket {
    pub sample: Vec<i64>,
    pub sample_type: RemoteDebugSampleType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteDebugSampleType {
    TickTime = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundStartConfigurationPacket;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundGameRuleValuesPacket {
    pub values: BTreeMap<Identifier, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundScoreboardPacket {
    pub objective: String,
    pub owner: Option<String>,
    pub score: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundBossEventPacket {
    pub event_id: Uuid,
    pub operation: BossEventOperation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BossEventOperation {
    Add {
        name: Tag,
        progress: f32,
        color: BossBarColor,
        overlay: BossBarOverlay,
        flags: BossEventFlags,
    },
    Remove,
    UpdateProgress {
        progress: f32,
    },
    UpdateName {
        name: Tag,
    },
    UpdateStyle {
        color: BossBarColor,
        overlay: BossBarOverlay,
    },
    UpdateProperties {
        flags: BossEventFlags,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarColor {
    Pink = 0,
    Blue = 1,
    Red = 2,
    Green = 3,
    Yellow = 4,
    Purple = 5,
    White = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarOverlay {
    Progress = 0,
    Notched6 = 1,
    Notched10 = 2,
    Notched12 = 3,
    Notched20 = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossEventFlags {
    pub darken_screen: bool,
    pub play_music: bool,
    pub create_world_fog: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTitlePacket {
    pub kind: TitlePacketKind,
    pub text: Option<String>,
    pub fade_in: Option<i32>,
    pub stay: Option<i32>,
    pub fade_out: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitlePacketKind {
    Title,
    Subtitle,
    ActionBar,
    Times,
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundClearTitlesPacket {
    pub reset_times: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetTitlesAnimationPacket {
    pub fade_in: i32,
    pub stay: i32,
    pub fade_out: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSoundPacket {
    pub sound: SoundEventHolder,
    pub source_id: i32,
    pub position: Vec3,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
    pub entity_id: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SoundEventHolder {
    Registered {
        id: i32,
    },
    Direct {
        location: Identifier,
        fixed_range: Option<f32>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundStopSoundPacket {
    pub source: Option<SoundSource>,
    pub name: Option<Identifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSource {
    Master = 0,
    Music = 1,
    Records = 2,
    Weather = 3,
    Blocks = 4,
    Hostile = 5,
    Neutral = 6,
    Players = 7,
    Ambient = 8,
    Voice = 9,
    Ui = 10,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundParticlePacket {
    pub particle_id: i32,
    pub override_limiter: bool,
    pub always_show: bool,
    pub position: Vec3,
    pub offset: Vec3,
    pub max_speed: f32,
    pub count: i32,
    pub particle_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundExplodePacket {
    pub center: Vec3,
    pub radius: f32,
    pub block_count: i32,
    pub player_knockback: Option<Vec3>,
    pub explosion_particle: RawParticleOptions,
    pub explosion_sound: SoundEventHolder,
    pub block_particles: Vec<WeightedExplosionParticle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawParticleOptions {
    pub particle_id: i32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionParticleInfo {
    pub particle: RawParticleOptions,
    pub scaling: f32,
    pub speed: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedExplosionParticle {
    pub value: ExplosionParticleInfo,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundMapItemDataPacket {
    pub map_id: i32,
    pub scale: u8,
    pub locked: bool,
    pub decorations: Option<Vec<MapDecorationData>>,
    pub color_patch: Option<MapPatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapDecorationData {
    pub decoration_type_id: i32,
    pub x: i8,
    pub y: i8,
    pub rotation: i8,
    pub name: Option<Tag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapPatch {
    pub width: u8,
    pub height: u8,
    pub start_x: u8,
    pub start_y: u8,
    pub colors: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundWorldBorderPacket {
    pub kind: WorldBorderPacketKind,
    pub center: Option<(f64, f64)>,
    pub old_size: Option<f64>,
    pub new_size: Option<f64>,
    pub lerp_time_ms: Option<i64>,
    pub warning_blocks: Option<i32>,
    pub warning_time: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldBorderPacketKind {
    Initialize,
    SetCenter,
    LerpSize,
    SetSize,
    SetWarningDelay,
    SetWarningDistance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCommandsPacket {
    pub root_index: i32,
    pub entries: Vec<CommandNodeEntryData>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandNodeEntryData {
    pub stub: CommandNodeStubData,
    pub executable: bool,
    pub restricted: bool,
    pub redirect: Option<i32>,
    pub children: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandNodeStubData {
    Root,
    Literal {
        name: String,
    },
    Argument {
        name: String,
        parser_type_id: i32,
        parser_payload: Vec<u8>,
        suggestion_id: Option<Identifier>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundCommandSuggestionsPacket {
    pub transaction_id: i32,
    pub start: i32,
    pub length: i32,
    pub suggestions: Vec<CommandSuggestionEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandSuggestionEntry {
    pub text: String,
    pub tooltip: Option<Tag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDeleteChatPacket {
    pub message_signature: PackedMessageSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackedMessageSignature {
    CacheId(i32),
    Full(Box<MessageSignature>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDebugPacket {
    pub kind: DebugPacketKind,
    pub payload_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugPacketKind {
    BlockValue,
    ChunkValue,
    EntityValue,
    Event,
    Sample,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundLevelChunkWithLightPacket {
    pub pos: ChunkPos,
    pub chunk_data: Option<ClientboundLevelChunkPacketData>,
    pub light_data: Option<ClientboundLightUpdatePacketData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundForgetLevelChunkPacket {
    pub pos: ChunkPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionBlockUpdate {
    pub packed_pos: u16,
    pub block_state_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSectionBlocksUpdatePacket {
    pub section_pos: SectionPos,
    pub updates: Vec<SectionBlockUpdate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundBlockEntityDataPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_entity_type_id: i32,
    pub tag: Tag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLightUpdatePacket {
    pub pos: ChunkPos,
    pub light_data: ClientboundLightUpdatePacketData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLightUpdatePacketData {
    pub sky_y_mask: Vec<u64>,
    pub block_y_mask: Vec<u64>,
    pub empty_sky_y_mask: Vec<u64>,
    pub empty_block_y_mask: Vec<u64>,
    pub sky_updates: Vec<Vec<i8>>,
    pub block_updates: Vec<Vec<i8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundLevelChunkPacketData {
    pub heightmaps: BTreeMap<String, Vec<i64>>,
    pub buffer: Vec<u8>,
    pub block_entity_count: usize,
    pub block_entities: Vec<LevelChunkBlockEntityInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelChunkBlockEntityInfo {
    pub packed_xz: u8,
    pub y: i16,
    pub block_entity_type_id: i32,
    pub tag: Tag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkChunkSection {
    pub non_empty_block_count: i16,
    pub fluid_count: i16,
    pub block_states: NetworkPalettedContainer,
    pub biomes: NetworkPalettedContainer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkPalettedContainer {
    pub bits_per_entry: u8,
    pub palette_ids: Vec<i32>,
    pub data: Vec<i64>,
    pub uses_global_palette: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRespawnPacket {
    pub spawn_info: CommonPlayerSpawnInfo,
    pub data_to_keep: RespawnDataToKeep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundPlayerCombatEndPacket {
    pub duration: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundPlayerCombatEnterPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RespawnDataToKeep {
    pub(super) bits: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerCombatKillPacket {
    pub player_id: i32,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundProjectilePowerPacket {
    pub id: i32,
    pub acceleration_power: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RespawnReason {
    Death,
    WonGameReturnToOverworld,
    DimensionChange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespawnRequest {
    pub reason: RespawnReason,
    pub keep_all_player_data: bool,
    pub missing_respawn_block: bool,
    pub hardcore: bool,
    pub active_effect_count: usize,
    pub respawn_anchor_depleted: bool,
    pub spawn_info: CommonPlayerSpawnInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameDifficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}
