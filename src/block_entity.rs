#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::block_update::{BlockPos, Direction};
use crate::map_state::DyeColor;
use crate::redstone::{comparator_output, ComparatorMode, MAX_SIGNAL};
use crate::special_block::{
    command_block_tick, CommandBlockMode, CommandBlockState, SpecialBlockAction,
};
use crate::storage::datafix::require_current_world_data_version;
use crate::storage::nbt::Tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockEntityTypeId {
    Furnace,
    Chest,
    TrappedChest,
    EnderChest,
    Jukebox,
    Dispenser,
    Dropper,
    Sign,
    HangingSign,
    MobSpawner,
    CreakingHeart,
    Piston,
    BrewingStand,
    EnchantingTable,
    EndPortal,
    Beacon,
    Skull,
    DaylightDetector,
    Hopper,
    Comparator,
    Banner,
    StructureBlock,
    EndGateway,
    CommandBlock,
    ShulkerBox,
    Bed,
    Conduit,
    Barrel,
    Smoker,
    BlastFurnace,
    Lectern,
    Bell,
    Jigsaw,
    Campfire,
    Beehive,
    SculkSensor,
    CalibratedSculkSensor,
    SculkCatalyst,
    SculkShrieker,
    ChiseledBookshelf,
    Shelf,
    BrushableBlock,
    DecoratedPot,
    Crafter,
    TrialSpawner,
    Vault,
    TestBlock,
    TestInstanceBlock,
    CopperGolemStatue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockEntityTickKind {
    None,
    Server,
    Client,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityTypeInfo {
    pub id: BlockEntityTypeId,
    pub key: &'static str,
    pub valid_blocks: &'static [&'static str],
    pub tick_kind: BlockEntityTickKind,
    pub op_only_custom_data: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockEntity {
    pub ty: BlockEntityTypeId,
    pub pos: BlockPos,
    pub block_state: String,
    pub custom_data: BTreeMap<String, Tag>,
    pub components: BTreeMap<String, Tag>,
    pub has_level: bool,
    pub removed: bool,
    pub changed: bool,
    pub tick_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundBlockEntityDataPacket {
    pub pos: BlockPos,
    pub ty: BlockEntityTypeId,
    pub tag: Tag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TickingBlockEntity {
    pub entity: BlockEntity,
    pub client_side: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestBlockEntityState {
    pub mode: TestBlockMode,
    pub message: String,
    pub powered: bool,
    pub triggered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestInstanceStatus {
    Cleared,
    Running,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestInstanceBlockEntityData {
    pub test: Option<String>,
    pub size: (i32, i32, i32),
    pub rotation: String,
    pub ignore_entities: bool,
    pub status: TestInstanceStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestInstanceErrorMarker {
    pub pos: BlockPos,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestInstanceBlockEntityState {
    pub data: TestInstanceBlockEntityData,
    pub errors: Vec<TestInstanceErrorMarker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BedBlockEntity {
    pub color: DyeColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndPortalBlockEntity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheEndGatewayBlockEntity {
    pub age: i64,
    pub teleport_cooldown: i32,
    pub exit_portal: Option<BlockPos>,
    pub exact_teleport: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawJointType {
    Rollable,
    Aligned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawGenerationPlan {
    pub pool: String,
    pub target: String,
    pub levels: i32,
    pub keep_jigsaws: bool,
    pub start_pos: BlockPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawBlockEntity {
    pub name: String,
    pub target: String,
    pub pool: String,
    pub joint: JigsawJointType,
    pub final_state: String,
    pub placement_priority: i32,
    pub selection_priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComparatorBlockEntity {
    pub mode: ComparatorMode,
    pub output_signal: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaylightDetectorBlockEntity {
    pub inverted: bool,
    pub power: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBlockEntity {
    pub command: String,
    pub success_count: i32,
    pub custom_name: Option<String>,
    pub track_output: bool,
    pub last_output: Option<String>,
    pub update_last_execution: bool,
    pub last_execution: i64,
    pub powered: bool,
    pub automatic: bool,
    pub condition_met: bool,
    pub mode: CommandBlockMode,
    pub conditional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JukeboxSongEvent {
    Started,
    Stopped,
    ItemChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JukeboxBlockEntity {
    pub item: Option<PotItemStack>,
    pub is_playing: bool,
    pub ticks_since_song_started: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnchantingTableBlockEntity {
    pub custom_name: Option<String>,
    pub time: i32,
    pub flip: f32,
    pub o_flip: f32,
    pub flip_t: f32,
    pub flip_a: f32,
    pub open: f32,
    pub o_open: f32,
    pub rot: f32,
    pub o_rot: f32,
    pub t_rot: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShelfBlockEntity {
    pub items: Vec<Option<PotItemStack>>,
    pub align_items_to_bottom: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeaconBeamBlock {
    Transparent,
    Blocking,
    Bedrock,
    TintedGlass(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconBeamSection {
    pub color: i32,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconEffectApplication {
    pub effect: String,
    pub duration_ticks: i32,
    pub amplifier: i32,
    pub range: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconBlockEntity {
    pub levels: i32,
    pub primary_power: Option<String>,
    pub secondary_power: Option<String>,
    pub custom_name: Option<String>,
    pub lock_key: Option<String>,
    pub payment_item: Option<PotItemStack>,
    pub beam_sections: Vec<BeaconBeamSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerPatternLayer {
    pub pattern: String,
    pub color: DyeColor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerBlockEntity {
    pub base_color: DyeColor,
    pub patterns: Vec<BannerPatternLayer>,
    pub custom_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotDecorations {
    pub back: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
    pub front: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotItemStack {
    pub item_id: String,
    pub count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoratedPotWobbleStyle {
    Positive,
    Negative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoratedPotBlockEntity {
    pub decorations: PotDecorations,
    pub item: Option<PotItemStack>,
    pub loot_table: Option<String>,
    pub loot_table_seed: i64,
    pub wobble_started_at_tick: i64,
    pub last_wobble_style: Option<DecoratedPotWobbleStyle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoratedPotDrops {
    pub decoration_items: Vec<String>,
    pub stored_item: Option<PotItemStack>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopperWeatherState {
    Unaffected,
    Exposed,
    Weathered,
    Oxidized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopperGolemStatuePose {
    Standing,
    Sitting,
    Running,
    Star,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopperGolemStatueBlockEntity {
    pub weather_state: CopperWeatherState,
    pub waxed: bool,
    pub pose: CopperGolemStatuePose,
    pub custom_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkullBlockEntity {
    pub profile: Option<Tag>,
    pub note_block_sound: Option<String>,
    pub custom_name: Option<String>,
    pub animation_tick_count: i32,
    pub is_animating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BellBlockEvent {
    pub event_id: i32,
    pub event_param: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BellTickEffects {
    pub play_resonate_sound: bool,
    pub glowing_raiders: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BellBlockEntity {
    pub last_ring_timestamp: u64,
    pub ticks: i32,
    pub shaking: bool,
    pub click_direction: Option<Direction>,
    pub heard_bell_entities: usize,
    pub nearby_raiders_within_hear_radius: usize,
    pub nearby_raiders_within_highlight_radius: usize,
    pub resonating: bool,
    pub resonation_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushResult {
    CoolingDown,
    InProgress { dusted: i32 },
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushableBlockEntity {
    pub brush_count: i32,
    pub brush_count_resets_at_tick: u64,
    pub cooldown_ends_at_tick: u64,
    pub item: Option<PotItemStack>,
    pub hit_direction: Option<Direction>,
    pub loot_table: Option<String>,
    pub loot_table_seed: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockEntityError {
    UnknownType(String),
    MissingId,
    InvalidBlockState {
        ty: BlockEntityTypeId,
        block_state: String,
    },
    UnsupportedDataVersion(String),
}

impl TestBlockMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Log => "log",
            Self::Fail => "fail",
            Self::Accept => "accept",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "log" => Some(Self::Log),
            "fail" => Some(Self::Fail),
            "accept" => Some(Self::Accept),
            _ => None,
        }
    }
}

impl Default for TestBlockEntityState {
    fn default() -> Self {
        Self {
            mode: TestBlockMode::Fail,
            message: String::new(),
            powered: false,
            triggered: false,
        }
    }
}

impl TestBlockEntityState {
    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "mode".to_string(),
                Tag::String(self.mode.as_str().to_string()),
            ),
            ("message".to_string(), Tag::String(self.message.clone())),
            ("powered".to_string(), Tag::Byte(i8::from(self.powered))),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let entries = compound_entries(tag);
        Self {
            mode: entries
                .and_then(|entries| get_string(entries, "mode"))
                .and_then(TestBlockMode::from_str)
                .unwrap_or(TestBlockMode::Fail),
            message: entries
                .and_then(|entries| get_string(entries, "message"))
                .unwrap_or("")
                .to_string(),
            powered: entries
                .and_then(|entries| get_byte(entries, "powered"))
                .unwrap_or(0)
                != 0,
            triggered: false,
        }
    }

    pub fn reset(&mut self) {
        self.triggered = false;
        if self.mode == TestBlockMode::Start {
            self.powered = false;
        }
    }

    pub fn trigger(&mut self) {
        if self.mode == TestBlockMode::Start {
            self.powered = true;
        } else {
            self.triggered = true;
        }
    }
}

impl TestInstanceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cleared => "cleared",
            Self::Running => "running",
            Self::Finished => "finished",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "cleared" => Some(Self::Cleared),
            "running" => Some(Self::Running),
            "finished" => Some(Self::Finished),
            _ => None,
        }
    }
}

impl Default for TestInstanceBlockEntityData {
    fn default() -> Self {
        Self {
            test: None,
            size: (0, 0, 0),
            rotation: "none".to_string(),
            ignore_entities: false,
            status: TestInstanceStatus::Cleared,
            error_message: None,
        }
    }
}

impl TestInstanceBlockEntityData {
    pub fn with_status(&self, status: TestInstanceStatus) -> Self {
        Self {
            status,
            error_message: None,
            ..self.clone()
        }
    }

    pub fn with_error(&self, error: impl Into<String>) -> Self {
        Self {
            status: TestInstanceStatus::Finished,
            error_message: Some(error.into()),
            ..self.clone()
        }
    }

    fn to_tag(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(test) = &self.test {
            fields.push(("test".to_string(), Tag::String(test.clone())));
        }
        fields.push((
            "size".to_string(),
            Tag::List(vec![
                Tag::Int(self.size.0),
                Tag::Int(self.size.1),
                Tag::Int(self.size.2),
            ]),
        ));
        fields.push(("rotation".to_string(), Tag::String(self.rotation.clone())));
        fields.push((
            "ignore_entities".to_string(),
            Tag::Byte(i8::from(self.ignore_entities)),
        ));
        fields.push((
            "status".to_string(),
            Tag::String(self.status.as_str().to_string()),
        ));
        if let Some(error_message) = &self.error_message {
            fields.push((
                "error_message".to_string(),
                Tag::String(error_message.clone()),
            ));
        }
        Tag::Compound(fields)
    }

    fn from_tag(tag: &Tag) -> Self {
        let entries = compound_entries(tag);
        let size = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "size"))
            .and_then(|(_, tag)| match tag {
                Tag::List(values) if values.len() == 3 => Some((
                    tag_int_or_zero(&values[0]),
                    tag_int_or_zero(&values[1]),
                    tag_int_or_zero(&values[2]),
                )),
                _ => None,
            })
            .unwrap_or((0, 0, 0));
        Self {
            test: entries
                .and_then(|entries| get_string(entries, "test"))
                .map(ToString::to_string),
            size,
            rotation: entries
                .and_then(|entries| get_string(entries, "rotation"))
                .unwrap_or("none")
                .to_string(),
            ignore_entities: entries
                .and_then(|entries| get_byte(entries, "ignore_entities"))
                .unwrap_or(0)
                != 0,
            status: entries
                .and_then(|entries| get_string(entries, "status"))
                .and_then(TestInstanceStatus::from_str)
                .unwrap_or(TestInstanceStatus::Cleared),
            error_message: entries
                .and_then(|entries| get_string(entries, "error_message"))
                .map(ToString::to_string),
        }
    }
}

impl Default for TestInstanceBlockEntityState {
    fn default() -> Self {
        Self {
            data: TestInstanceBlockEntityData::default(),
            errors: Vec::new(),
        }
    }
}

impl TestInstanceBlockEntityState {
    pub fn save_additional(&self) -> Tag {
        let mut fields = vec![("data".to_string(), self.data.to_tag())];
        if !self.errors.is_empty() {
            fields.push((
                "errors".to_string(),
                Tag::List(
                    self.errors
                        .iter()
                        .map(TestInstanceErrorMarker::to_tag)
                        .collect(),
                ),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let entries = compound_entries(tag);
        let data = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "data"))
            .map(|(_, tag)| TestInstanceBlockEntityData::from_tag(tag))
            .unwrap_or_default();
        let errors = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "errors"))
            .and_then(|(_, tag)| match tag {
                Tag::List(values) => Some(
                    values
                        .iter()
                        .filter_map(TestInstanceErrorMarker::from_tag)
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        Self { data, errors }
    }

    pub fn set_running(&mut self) {
        self.data = self.data.with_status(TestInstanceStatus::Running);
    }

    pub fn set_success(&mut self) {
        self.data = self.data.with_status(TestInstanceStatus::Finished);
    }

    pub fn set_error_message(&mut self, message: impl Into<String>) {
        self.data = self.data.with_error(message);
    }

    pub fn mark_error(&mut self, pos: BlockPos, text: impl Into<String>) {
        self.errors.push(TestInstanceErrorMarker {
            pos,
            text: text.into(),
        });
    }

    pub fn clear_error_markers(&mut self) {
        self.errors.clear();
    }
}

impl TestInstanceErrorMarker {
    fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            (
                "pos".to_string(),
                Tag::List(vec![
                    Tag::Int(self.pos.x),
                    Tag::Int(self.pos.y),
                    Tag::Int(self.pos.z),
                ]),
            ),
            ("text".to_string(), Tag::String(self.text.clone())),
        ])
    }

    fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        let pos =
            entries
                .iter()
                .find(|(name, _)| name == "pos")
                .and_then(|(_, tag)| match tag {
                    Tag::List(values) if values.len() == 3 => Some(BlockPos {
                        x: tag_int_or_zero(&values[0]),
                        y: tag_int_or_zero(&values[1]),
                        z: tag_int_or_zero(&values[2]),
                    }),
                    _ => None,
                })?;
        let text = get_string(entries, "text")?.to_string();
        Some(Self { pos, text })
    }
}

impl BedBlockEntity {
    pub fn from_block_state(block_state: &str) -> Option<Self> {
        let color = match block_state.strip_prefix("minecraft:")? {
            "white_bed" => DyeColor::White,
            "orange_bed" => DyeColor::Orange,
            "magenta_bed" => DyeColor::Magenta,
            "light_blue_bed" => DyeColor::LightBlue,
            "yellow_bed" => DyeColor::Yellow,
            "lime_bed" => DyeColor::Lime,
            "pink_bed" => DyeColor::Pink,
            "gray_bed" => DyeColor::Gray,
            "light_gray_bed" => DyeColor::LightGray,
            "cyan_bed" => DyeColor::Cyan,
            "purple_bed" => DyeColor::Purple,
            "blue_bed" => DyeColor::Blue,
            "brown_bed" => DyeColor::Brown,
            "green_bed" => DyeColor::Green,
            "red_bed" => DyeColor::Red,
            "black_bed" => DyeColor::Black,
            _ => return None,
        };
        Some(Self { color })
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(Vec::new())
    }
}

impl EndPortalBlockEntity {
    pub fn save_additional(&self) -> Tag {
        Tag::Compound(Vec::new())
    }
}

impl TheEndGatewayBlockEntity {
    pub const SPAWN_TIME: i64 = 200;
    pub const COOLDOWN_TIME: i32 = 40;
    pub const ATTENTION_INTERVAL: i64 = 2400;
    pub const EVENT_COOLDOWN: i32 = 1;
    pub const GATEWAY_HEIGHT_ABOVE_SURFACE: i32 = 10;

    pub fn new() -> Self {
        Self {
            age: 0,
            teleport_cooldown: 0,
            exit_portal: None,
            exact_teleport: false,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = vec![("Age".to_string(), Tag::Long(self.age))];
        if let Some(exit_portal) = self.exit_portal {
            fields.push(("exit_portal".to_string(), block_pos_to_tag(exit_portal)));
        }
        if self.exact_teleport {
            fields.push(("ExactTeleport".to_string(), Tag::Byte(1)));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        Self {
            age: entries
                .iter()
                .find(|(name, _)| name == "Age")
                .map(|(_, tag)| tag_long_or_zero(tag))
                .unwrap_or(0),
            teleport_cooldown: 0,
            exit_portal: entries
                .iter()
                .find(|(name, _)| name == "exit_portal")
                .and_then(|(_, tag)| block_pos_from_tag(tag)),
            exact_teleport: get_bool(entries, "ExactTeleport").unwrap_or(false),
        }
    }

    pub fn beam_animation_tick(&mut self) {
        self.age += 1;
        if self.is_cooling_down() {
            self.teleport_cooldown -= 1;
        }
    }

    pub fn portal_tick(&mut self) -> bool {
        let was_spawning = self.is_spawning();
        let was_cooling_down = self.is_cooling_down();
        self.age += 1;
        if was_cooling_down {
            self.teleport_cooldown -= 1;
        } else if self.age % Self::ATTENTION_INTERVAL == 0 {
            self.trigger_cooldown();
        }
        was_spawning != self.is_spawning() || was_cooling_down != self.is_cooling_down()
    }

    pub fn is_spawning(&self) -> bool {
        self.age < Self::SPAWN_TIME
    }

    pub fn is_cooling_down(&self) -> bool {
        self.teleport_cooldown > 0
    }

    pub fn spawn_percent(&self, partial_tick: f32) -> f32 {
        ((self.age as f32 + partial_tick) / Self::SPAWN_TIME as f32).clamp(0.0, 1.0)
    }

    pub fn cooldown_percent(&self, partial_tick: f32) -> f32 {
        1.0 - ((self.teleport_cooldown as f32 - partial_tick) / Self::COOLDOWN_TIME as f32)
            .clamp(0.0, 1.0)
    }

    pub fn trigger_cooldown(&mut self) {
        self.teleport_cooldown = Self::COOLDOWN_TIME;
    }

    pub fn trigger_event(&mut self, event: i32) -> bool {
        if event == Self::EVENT_COOLDOWN {
            self.trigger_cooldown();
            true
        } else {
            false
        }
    }

    pub fn set_exit_position(&mut self, exit_portal: BlockPos, exact_teleport: bool) {
        self.exit_portal = Some(exit_portal);
        self.exact_teleport = exact_teleport;
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }
}

impl JigsawJointType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rollable => "rollable",
            Self::Aligned => "aligned",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "rollable" => Some(Self::Rollable),
            "aligned" => Some(Self::Aligned),
            _ => None,
        }
    }
}

impl JigsawBlockEntity {
    pub const EMPTY_ID: &'static str = "minecraft:empty";
    pub const DEFAULT_FINAL_STATE: &'static str = "minecraft:air";

    pub fn new() -> Self {
        Self {
            name: Self::EMPTY_ID.to_string(),
            target: Self::EMPTY_ID.to_string(),
            pool: Self::EMPTY_ID.to_string(),
            joint: JigsawJointType::Rollable,
            final_state: Self::DEFAULT_FINAL_STATE.to_string(),
            placement_priority: 0,
            selection_priority: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            ("name".to_string(), Tag::String(self.name.clone())),
            ("target".to_string(), Tag::String(self.target.clone())),
            ("pool".to_string(), Tag::String(self.pool.clone())),
            (
                "final_state".to_string(),
                Tag::String(self.final_state.clone()),
            ),
            (
                "joint".to_string(),
                Tag::String(self.joint.as_str().to_string()),
            ),
            (
                "placement_priority".to_string(),
                Tag::Int(self.placement_priority),
            ),
            (
                "selection_priority".to_string(),
                Tag::Int(self.selection_priority),
            ),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        Self {
            name: get_string(entries, "name")
                .unwrap_or(Self::EMPTY_ID)
                .to_string(),
            target: get_string(entries, "target")
                .unwrap_or(Self::EMPTY_ID)
                .to_string(),
            pool: get_string(entries, "pool")
                .unwrap_or(Self::EMPTY_ID)
                .to_string(),
            final_state: get_string(entries, "final_state")
                .unwrap_or(Self::DEFAULT_FINAL_STATE)
                .to_string(),
            joint: get_string(entries, "joint")
                .and_then(JigsawJointType::from_str)
                .unwrap_or(JigsawJointType::Rollable),
            placement_priority: get_int(entries, "placement_priority").unwrap_or(0),
            selection_priority: get_int(entries, "selection_priority").unwrap_or(0),
        }
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn generation_plan(
        &self,
        block_pos: BlockPos,
        orientation_front: Direction,
        levels: i32,
        keep_jigsaws: bool,
    ) -> JigsawGenerationPlan {
        JigsawGenerationPlan {
            pool: self.pool.clone(),
            target: self.target.clone(),
            levels,
            keep_jigsaws,
            start_pos: block_pos.relative(orientation_front),
        }
    }
}

impl ComparatorBlockEntity {
    pub fn new(mode: ComparatorMode) -> Self {
        Self {
            mode,
            output_signal: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![(
            "OutputSignal".to_string(),
            Tag::Int(self.output_signal),
        )])
    }

    pub fn load_additional(mode: ComparatorMode, tag: &Tag) -> Self {
        let output_signal = compound_entries(tag)
            .and_then(|entries| get_int(entries, "OutputSignal"))
            .unwrap_or(0);
        Self {
            mode,
            output_signal,
        }
    }

    pub fn calculate_output(&self, rear_input: u8, side_input: u8) -> u8 {
        comparator_output(self.mode, rear_input, side_input)
    }

    pub fn update_output(&mut self, rear_input: u8, side_input: u8) -> bool {
        let next = self.calculate_output(rear_input, side_input);
        let next = i32::from(next);
        let changed = self.output_signal != next;
        self.output_signal = next;
        changed
    }
}

impl DaylightDetectorBlockEntity {
    pub const TICK_INTERVAL: u64 = 20;

    pub fn new(inverted: bool) -> Self {
        Self { inverted, power: 0 }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(Vec::new())
    }

    pub fn calculate_power(
        inverted: bool,
        effective_sky_brightness: i32,
        sun_angle_degrees: f32,
    ) -> u8 {
        let mut target = effective_sky_brightness;
        if inverted {
            target = i32::from(MAX_SIGNAL) - target;
        } else if target > 0 {
            let mut sun_angle = sun_angle_degrees.to_radians();
            let offset = if sun_angle < std::f32::consts::PI {
                0.0
            } else {
                std::f32::consts::TAU
            };
            sun_angle += (offset - sun_angle) * 0.2;
            target = ((target as f32) * sun_angle.cos()).round() as i32;
        }

        target.clamp(0, i32::from(MAX_SIGNAL)) as u8
    }

    pub fn update_signal(&mut self, effective_sky_brightness: i32, sun_angle_degrees: f32) -> bool {
        let next =
            Self::calculate_power(self.inverted, effective_sky_brightness, sun_angle_degrees);
        let changed = self.power != next;
        self.power = next;
        changed
    }

    pub fn tick(
        &mut self,
        game_time: u64,
        effective_sky_brightness: i32,
        sun_angle_degrees: f32,
    ) -> bool {
        if game_time % Self::TICK_INTERVAL != 0 {
            return false;
        }
        self.update_signal(effective_sky_brightness, sun_angle_degrees)
    }
}

impl CommandBlockEntity {
    pub const NO_LAST_EXECUTION: i64 = -1;
    pub const PERMISSION_LEVEL: &'static str = "gamemaster";
    pub const SEARGE_COMMAND: &'static str = "Searge";
    pub const SEARGE_OUTPUT: &'static str = "#itzlipofutzli";

    pub fn new(mode: CommandBlockMode, conditional: bool) -> Self {
        Self {
            command: String::new(),
            success_count: 0,
            custom_name: None,
            track_output: true,
            last_output: None,
            update_last_execution: true,
            last_execution: Self::NO_LAST_EXECUTION,
            powered: false,
            automatic: false,
            condition_met: false,
            mode,
            conditional,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = vec![
            ("Command".to_string(), Tag::String(self.command.clone())),
            ("SuccessCount".to_string(), Tag::Int(self.success_count)),
            (
                "TrackOutput".to_string(),
                Tag::Byte(self.track_output as i8),
            ),
            (
                "UpdateLastExecution".to_string(),
                Tag::Byte(self.update_last_execution as i8),
            ),
            ("powered".to_string(), Tag::Byte(self.powered as i8)),
            (
                "conditionMet".to_string(),
                Tag::Byte(self.condition_met as i8),
            ),
            ("auto".to_string(), Tag::Byte(self.automatic as i8)),
        ];

        if let Some(custom_name) = &self.custom_name {
            entries.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        if self.track_output {
            if let Some(last_output) = &self.last_output {
                entries.push(("LastOutput".to_string(), Tag::String(last_output.clone())));
            }
        }
        if self.update_last_execution && self.last_execution != Self::NO_LAST_EXECUTION {
            entries.push(("LastExecution".to_string(), Tag::Long(self.last_execution)));
        }

        Tag::Compound(entries)
    }

    pub fn load_additional(mode: CommandBlockMode, conditional: bool, tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new(mode, conditional);
        };
        let track_output = get_bool(entries, "TrackOutput").unwrap_or(true);
        let update_last_execution = get_bool(entries, "UpdateLastExecution").unwrap_or(true);
        Self {
            command: get_string(entries, "Command").unwrap_or("").to_string(),
            success_count: get_int(entries, "SuccessCount").unwrap_or(0),
            custom_name: get_string(entries, "CustomName").map(ToString::to_string),
            track_output,
            last_output: if track_output {
                get_string(entries, "LastOutput").map(ToString::to_string)
            } else {
                None
            },
            update_last_execution,
            last_execution: if update_last_execution {
                get_long(entries, "LastExecution").unwrap_or(Self::NO_LAST_EXECUTION)
            } else {
                Self::NO_LAST_EXECUTION
            },
            powered: get_bool(entries, "powered").unwrap_or(false),
            automatic: get_bool(entries, "auto").unwrap_or(false),
            condition_met: get_bool(entries, "conditionMet").unwrap_or(false),
            mode,
            conditional,
        }
    }

    pub fn set_command(&mut self, command: impl Into<String>) {
        self.command = command.into();
        self.success_count = 0;
    }

    pub fn set_automatic(&mut self, automatic: bool, has_level: bool) -> bool {
        let previous = self.automatic;
        self.automatic = automatic;
        !previous
            && automatic
            && !self.powered
            && has_level
            && self.mode != CommandBlockMode::Sequence
    }

    pub fn mark_condition_met(&mut self, previous_command_success: bool) -> bool {
        self.condition_met = !self.conditional || previous_command_success;
        self.condition_met
    }

    pub fn can_use(&self, player_can_use_gamemaster_blocks: bool) -> bool {
        player_can_use_gamemaster_blocks
    }

    pub fn execution_action(
        &self,
        has_permission: bool,
        previous_success: bool,
    ) -> SpecialBlockAction {
        command_block_tick(
            &CommandBlockState {
                command: self.command.clone(),
                mode: self.mode.clone(),
                powered: self.powered,
                previously_powered: false,
                conditional: self.conditional,
                previous_success,
            },
            has_permission,
        )
    }

    pub fn perform_command(
        &mut self,
        game_time: i64,
        command_blocks_enabled: bool,
        has_permission: bool,
        previous_success: bool,
    ) -> bool {
        if self.update_last_execution && self.last_execution == game_time {
            return false;
        }
        if self.command.eq_ignore_ascii_case(Self::SEARGE_COMMAND) {
            self.last_output = Some(Self::SEARGE_OUTPUT.to_string());
            self.success_count = 1;
            return true;
        }

        self.success_count = 0;
        let executed = match self
            .execution_action(has_permission && command_blocks_enabled, previous_success)
        {
            SpecialBlockAction::ExecuteCommand { success_count } => {
                self.success_count = success_count;
                true
            }
            SpecialBlockAction::Noop => false,
            _ => false,
        };

        if executed {
            if self.update_last_execution {
                self.last_execution = game_time;
            } else {
                self.last_execution = Self::NO_LAST_EXECUTION;
            }
        }
        executed
    }
}

impl JukeboxBlockEntity {
    pub const RECORD_ITEM_TAG_ID: &'static str = "RecordItem";
    pub const TICKS_SINCE_SONG_STARTED_TAG_ID: &'static str = "ticks_since_song_started";
    pub const STOP_LEVEL_EVENT: i32 = 1011;

    pub fn new() -> Self {
        Self {
            item: None,
            is_playing: false,
            ticks_since_song_started: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(item) = self.item.as_ref().filter(|item| !item.is_empty()) {
            entries.push((Self::RECORD_ITEM_TAG_ID.to_string(), item.to_tag()));
        }
        if self.song_item().is_some() {
            entries.push((
                Self::TICKS_SINCE_SONG_STARTED_TAG_ID.to_string(),
                Tag::Long(self.ticks_since_song_started),
            ));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        let item = entries
            .iter()
            .find(|(key, _)| key == Self::RECORD_ITEM_TAG_ID)
            .and_then(|(_, tag)| PotItemStack::from_tag(tag));
        let mut jukebox = Self {
            item,
            is_playing: false,
            ticks_since_song_started: 0,
        };
        if jukebox.song_item().is_some() {
            jukebox.ticks_since_song_started =
                get_long(entries, Self::TICKS_SINCE_SONG_STARTED_TAG_ID).unwrap_or(0);
        }
        jukebox
    }

    pub fn set_the_item(&mut self, item: Option<PotItemStack>) -> JukeboxSongEvent {
        self.item = item.filter(|item| !item.is_empty());
        self.ticks_since_song_started = 0;
        if self.song_item().is_some() {
            self.is_playing = true;
            JukeboxSongEvent::Started
        } else {
            self.is_playing = false;
            JukeboxSongEvent::Stopped
        }
    }

    pub fn set_song_item_without_playing(&mut self, item: PotItemStack) -> JukeboxSongEvent {
        self.item = (!item.is_empty()).then_some(item);
        self.is_playing = false;
        self.ticks_since_song_started = 0;
        JukeboxSongEvent::ItemChanged
    }

    pub fn remove_the_item(&mut self) -> Option<PotItemStack> {
        self.is_playing = false;
        self.ticks_since_song_started = 0;
        self.item.take()
    }

    pub fn pop_out_the_item(&mut self) -> Option<PotItemStack> {
        self.remove_the_item()
    }

    pub fn tick(&mut self) -> bool {
        if self.is_playing {
            self.ticks_since_song_started += 1;
            true
        } else {
            false
        }
    }

    pub fn redstone_signal(&self) -> u8 {
        if self.is_playing {
            MAX_SIGNAL
        } else {
            0
        }
    }

    pub fn comparator_output(&self) -> u8 {
        self.song_item()
            .map(jukebox_song_comparator_output)
            .unwrap_or(0)
    }

    pub fn can_place_item(&self, item: &PotItemStack) -> bool {
        self.item.is_none() && jukebox_song_item_id(&item.item_id).is_some()
    }

    pub fn can_take_item(&self, destination_has_empty_slot: bool) -> bool {
        destination_has_empty_slot
    }

    fn song_item(&self) -> Option<&str> {
        self.item
            .as_ref()
            .and_then(|item| jukebox_song_item_id(&item.item_id))
    }
}

fn jukebox_song_item_id(item_id: &str) -> Option<&str> {
    item_id.strip_prefix("minecraft:music_disc_")
}

fn jukebox_song_comparator_output(song_id: &str) -> u8 {
    match song_id {
        "13" => 1,
        "cat" => 2,
        "blocks" => 3,
        "chirp" => 4,
        "far" => 5,
        "mall" => 6,
        "mellohi" => 7,
        "stal" => 8,
        "strad" | "lava_chicken" => 9,
        "ward" | "tears" => 10,
        "11" | "creator_music_box" => 11,
        "wait" | "creator" => 12,
        "pigstep" | "precipice" => 13,
        "otherside" | "relic" => 14,
        "5" => 15,
        _ => 0,
    }
}

impl EnchantingTableBlockEntity {
    pub const DEFAULT_NAME: &'static str = "container.enchant";

    pub fn new() -> Self {
        Self {
            custom_name: None,
            time: 0,
            flip: 0.0,
            o_flip: 0.0,
            flip_t: 0.0,
            flip_a: 0.0,
            open: 0.0,
            o_open: 0.0,
            rot: 0.0,
            o_rot: 0.0,
            t_rot: 0.0,
        }
    }

    pub fn display_name(&self) -> &str {
        self.custom_name.as_deref().unwrap_or(Self::DEFAULT_NAME)
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(custom_name) = &self.custom_name {
            entries.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut table = Self::new();
        if let Some(entries) = compound_entries(tag) {
            table.custom_name = get_string(entries, "CustomName").map(ToString::to_string);
        }
        table
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn bookshelf_offsets() -> Vec<BlockPos> {
        let mut offsets = Vec::new();
        for x in -2i32..=2 {
            for y in 0i32..=1 {
                for z in -2i32..=2 {
                    if x.abs() == 2 || z.abs() == 2 {
                        offsets.push(BlockPos { x, y, z });
                    }
                }
            }
        }
        offsets
    }

    pub fn count_valid_bookshelves(
        is_power_provider: impl Fn(BlockPos) -> bool,
        is_power_transmitter: impl Fn(BlockPos) -> bool,
    ) -> usize {
        Self::bookshelf_offsets()
            .into_iter()
            .filter(|offset| {
                is_power_provider(*offset)
                    && is_power_transmitter(BlockPos {
                        x: offset.x / 2,
                        y: offset.y,
                        z: offset.z / 2,
                    })
            })
            .take(15)
            .count()
    }

    pub fn book_animation_tick(
        &mut self,
        player_offset_xz: Option<(f64, f64)>,
        next_flip_delta: Option<f32>,
    ) {
        self.o_open = self.open;
        self.o_rot = self.rot;
        if let Some((xd, zd)) = player_offset_xz {
            self.t_rot = (zd.atan2(xd)) as f32;
            self.open += 0.1;
            if self.open < 0.5 {
                if let Some(delta) = next_flip_delta {
                    let old = self.flip_t;
                    if delta != 0.0 {
                        self.flip_t += delta;
                    } else {
                        self.flip_t += 1.0;
                    }
                    if self.flip_t == old {
                        self.flip_t += 1.0;
                    }
                }
            }
        } else {
            self.t_rot += 0.02;
            self.open -= 0.1;
        }

        self.rot = wrap_radians(self.rot);
        self.t_rot = wrap_radians(self.t_rot);
        let rot_dir = wrap_radians(self.t_rot - self.rot);
        self.rot += rot_dir * 0.4;
        self.open = self.open.clamp(0.0, 1.0);
        self.time += 1;
        self.o_flip = self.flip;
        let diff = ((self.flip_t - self.flip) * 0.4).clamp(-0.2, 0.2);
        self.flip_a += (diff - self.flip_a) * 0.9;
        self.flip += self.flip_a;
    }
}

fn wrap_radians(mut value: f32) -> f32 {
    while value >= std::f32::consts::PI {
        value -= std::f32::consts::TAU;
    }
    while value < -std::f32::consts::PI {
        value += std::f32::consts::TAU;
    }
    value
}

impl ShelfBlockEntity {
    pub const MAX_ITEMS: usize = 3;
    pub const ALIGN_ITEMS_TO_BOTTOM_TAG: &'static str = "align_items_to_bottom";

    pub fn new() -> Self {
        Self {
            items: vec![None; Self::MAX_ITEMS],
            align_items_to_bottom: false,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = vec![(
            "Items".to_string(),
            Tag::List(
                self.items
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, item)| {
                        let item = item.as_ref().filter(|item| !item.is_empty())?;
                        let mut tag = match item.to_tag() {
                            Tag::Compound(entries) => entries,
                            _ => return None,
                        };
                        tag.push(("Slot".to_string(), Tag::Byte(slot as i8)));
                        Some(Tag::Compound(tag))
                    })
                    .collect(),
            ),
        )];
        entries.push((
            Self::ALIGN_ITEMS_TO_BOTTOM_TAG.to_string(),
            Tag::Byte(self.align_items_to_bottom as i8),
        ));
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut shelf = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return shelf;
        };
        shelf.align_items_to_bottom =
            get_bool(entries, Self::ALIGN_ITEMS_TO_BOTTOM_TAG).unwrap_or(false);
        if let Some(Tag::List(items)) = entries
            .iter()
            .find(|(key, _)| key == "Items")
            .map(|(_, tag)| tag)
        {
            for item_tag in items {
                let Some(item_entries) = compound_entries(item_tag) else {
                    continue;
                };
                let Some(slot) = get_byte(item_entries, "Slot") else {
                    continue;
                };
                if let Some(slot) = usize::try_from(slot)
                    .ok()
                    .filter(|slot| *slot < Self::MAX_ITEMS)
                {
                    shelf.items[slot] = PotItemStack::from_tag(item_tag);
                }
            }
        }
        shelf
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn get_item(&self, slot: usize) -> Option<&PotItemStack> {
        self.items.get(slot).and_then(Option::as_ref)
    }

    pub fn set_item_no_update(&mut self, slot: usize, item: Option<PotItemStack>) -> bool {
        let Some(target) = self.items.get_mut(slot) else {
            return false;
        };
        *target = item.filter(|item| !item.is_empty());
        true
    }

    pub fn remove_item_no_update(&mut self, slot: usize) -> Option<PotItemStack> {
        self.items.get_mut(slot).and_then(Option::take)
    }

    pub fn swap_item_no_update(
        &mut self,
        slot: usize,
        held_item_stack: Option<PotItemStack>,
    ) -> Option<PotItemStack> {
        let retrieved = self.remove_item_no_update(slot);
        self.set_item_no_update(slot, held_item_stack);
        retrieved
    }

    pub fn filled_slot_count(&self) -> usize {
        self.items.iter().filter(|item| item.is_some()).count()
    }

    pub fn comparator_output(&self) -> u8 {
        self.filled_slot_count() as u8
    }
}

impl BeaconBeamSection {
    pub fn new(color: i32) -> Self {
        Self { color, height: 1 }
    }

    pub fn increase_height(&mut self) {
        self.height += 1;
    }
}

impl BeaconBlockEntity {
    pub const MAX_LEVELS: i32 = 4;
    pub const BLOCKS_CHECK_PER_TICK: i32 = 10;
    pub const DEFAULT_NAME: &'static str = "container.beacon";

    pub fn new() -> Self {
        Self {
            levels: 0,
            primary_power: None,
            secondary_power: None,
            custom_name: None,
            lock_key: None,
            payment_item: None,
            beam_sections: Vec::new(),
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(primary) = &self.primary_power {
            entries.push(("primary_effect".to_string(), Tag::String(primary.clone())));
        }
        if let Some(secondary) = &self.secondary_power {
            entries.push((
                "secondary_effect".to_string(),
                Tag::String(secondary.clone()),
            ));
        }
        entries.push(("Levels".to_string(), Tag::Int(self.levels)));
        if let Some(custom_name) = &self.custom_name {
            entries.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        if let Some(lock_key) = &self.lock_key {
            entries.push(("Lock".to_string(), Tag::String(lock_key.clone())));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut beacon = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return beacon;
        };
        beacon.primary_power = get_string(entries, "primary_effect").and_then(filter_beacon_effect);
        beacon.secondary_power =
            get_string(entries, "secondary_effect").and_then(filter_beacon_effect);
        beacon.levels = get_int(entries, "Levels")
            .unwrap_or(0)
            .clamp(0, Self::MAX_LEVELS);
        beacon.custom_name = get_string(entries, "CustomName").map(ToString::to_string);
        beacon.lock_key = get_string(entries, "Lock").map(ToString::to_string);
        beacon
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn display_name(&self) -> &str {
        self.custom_name.as_deref().unwrap_or(Self::DEFAULT_NAME)
    }

    pub fn set_primary_power(&mut self, effect: Option<&str>) {
        self.primary_power = effect.and_then(filter_beacon_effect);
    }

    pub fn set_secondary_power(&mut self, effect: Option<&str>) {
        self.secondary_power = effect.and_then(filter_beacon_effect);
    }

    pub fn can_pay_with(item: &PotItemStack) -> bool {
        matches!(
            item.item_id.as_str(),
            "minecraft:netherite_ingot"
                | "minecraft:emerald"
                | "minecraft:diamond"
                | "minecraft:gold_ingot"
                | "minecraft:iron_ingot"
                | "minecraft:amethyst_shard"
        )
    }

    pub fn set_payment_item(&mut self, item: Option<PotItemStack>) -> bool {
        if item.as_ref().is_some_and(|item| !Self::can_pay_with(item)) {
            return false;
        }
        self.payment_item = item.filter(|item| !item.is_empty());
        true
    }

    pub fn comparator_output(&self) -> u8 {
        self.levels.clamp(0, Self::MAX_LEVELS) as u8
    }

    pub fn update_base(
        beacon_pos: BlockPos,
        min_y: i32,
        is_base_block: impl Fn(BlockPos) -> bool,
    ) -> i32 {
        let mut levels = 0;
        for step in 1..=Self::MAX_LEVELS {
            let y = beacon_pos.y - step;
            if y < min_y {
                break;
            }
            let mut ok = true;
            'layer: for x in beacon_pos.x - step..=beacon_pos.x + step {
                for z in beacon_pos.z - step..=beacon_pos.z + step {
                    if !is_base_block(BlockPos { x, y, z }) {
                        ok = false;
                        break 'layer;
                    }
                }
            }
            if !ok {
                break;
            }
            levels = step;
        }
        levels
    }

    pub fn scan_beam(blocks: impl IntoIterator<Item = BeaconBeamBlock>) -> Vec<BeaconBeamSection> {
        let mut sections: Vec<BeaconBeamSection> = Vec::new();
        let mut last: Option<BeaconBeamSection> = None;
        for block in blocks {
            match block {
                BeaconBeamBlock::TintedGlass(color) => {
                    if sections.len() <= 1 {
                        let section = BeaconBeamSection::new(color);
                        sections.push(section.clone());
                        last = Some(section);
                    } else if let Some(current) = last.as_mut() {
                        if current.color == color {
                            current.increase_height();
                            if let Some(stored) = sections.last_mut() {
                                stored.increase_height();
                            }
                        } else {
                            let averaged = average_argb(current.color, color);
                            let section = BeaconBeamSection::new(averaged);
                            sections.push(section.clone());
                            last = Some(section);
                        }
                    }
                }
                BeaconBeamBlock::Transparent | BeaconBeamBlock::Bedrock => {
                    if let Some(stored) = sections.last_mut() {
                        stored.increase_height();
                    } else {
                        let mut section = BeaconBeamSection::new(0xFFFF_FFFFu32 as i32);
                        section.increase_height();
                        sections.push(section.clone());
                        last = Some(section);
                    }
                }
                BeaconBeamBlock::Blocking => return Vec::new(),
            }
        }
        sections
    }

    pub fn effect_applications(&self) -> Vec<BeaconEffectApplication> {
        if self.levels <= 0 {
            return Vec::new();
        }
        let Some(primary) = self.primary_power.as_ref() else {
            return Vec::new();
        };
        let range = self.levels * 10 + 10;
        let duration_ticks = (9 + self.levels * 2) * 20;
        let mut out = vec![BeaconEffectApplication {
            effect: primary.clone(),
            duration_ticks,
            amplifier: if self.levels >= 4 && self.secondary_power.as_ref() == Some(primary) {
                1
            } else {
                0
            },
            range,
        }];
        if self.levels >= 4 {
            if let Some(secondary) = self.secondary_power.as_ref() {
                if secondary != primary {
                    out.push(BeaconEffectApplication {
                        effect: secondary.clone(),
                        duration_ticks,
                        amplifier: 0,
                        range,
                    });
                }
            }
        }
        out
    }
}

fn filter_beacon_effect(effect: &str) -> Option<String> {
    matches!(
        effect,
        "minecraft:speed"
            | "minecraft:haste"
            | "minecraft:resistance"
            | "minecraft:jump_boost"
            | "minecraft:strength"
            | "minecraft:regeneration"
    )
    .then(|| effect.to_string())
}

fn average_argb(left: i32, right: i32) -> i32 {
    let left = left as u32;
    let right = right as u32;
    let avg = |shift| (((left >> shift) & 0xFFu32) + ((right >> shift) & 0xFFu32)) / 2u32;
    ((avg(24) << 24) | (avg(16) << 16) | (avg(8) << 8) | avg(0)) as i32
}

impl DyeColor {
    fn vanilla_name(self) -> &'static str {
        match self {
            DyeColor::White => "white",
            DyeColor::Orange => "orange",
            DyeColor::Magenta => "magenta",
            DyeColor::LightBlue => "light_blue",
            DyeColor::Yellow => "yellow",
            DyeColor::Lime => "lime",
            DyeColor::Pink => "pink",
            DyeColor::Gray => "gray",
            DyeColor::LightGray => "light_gray",
            DyeColor::Cyan => "cyan",
            DyeColor::Purple => "purple",
            DyeColor::Blue => "blue",
            DyeColor::Brown => "brown",
            DyeColor::Green => "green",
            DyeColor::Red => "red",
            DyeColor::Black => "black",
        }
    }

    fn from_vanilla_name(value: &str) -> Option<Self> {
        match value {
            "white" => Some(DyeColor::White),
            "orange" => Some(DyeColor::Orange),
            "magenta" => Some(DyeColor::Magenta),
            "light_blue" => Some(DyeColor::LightBlue),
            "yellow" => Some(DyeColor::Yellow),
            "lime" => Some(DyeColor::Lime),
            "pink" => Some(DyeColor::Pink),
            "gray" => Some(DyeColor::Gray),
            "light_gray" => Some(DyeColor::LightGray),
            "cyan" => Some(DyeColor::Cyan),
            "purple" => Some(DyeColor::Purple),
            "blue" => Some(DyeColor::Blue),
            "brown" => Some(DyeColor::Brown),
            "green" => Some(DyeColor::Green),
            "red" => Some(DyeColor::Red),
            "black" => Some(DyeColor::Black),
            _ => None,
        }
    }
}

impl BannerPatternLayer {
    fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("pattern".to_string(), Tag::String(self.pattern.clone())),
            (
                "color".to_string(),
                Tag::String(self.color.vanilla_name().to_string()),
            ),
        ])
    }

    fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        Some(Self {
            pattern: get_string(entries, "pattern")?.to_string(),
            color: DyeColor::from_vanilla_name(get_string(entries, "color")?)?,
        })
    }
}

impl BannerBlockEntity {
    pub const MAX_PATTERNS: usize = 6;

    pub fn from_block_state(block_state: &str) -> Option<Self> {
        let name = block_state.strip_prefix("minecraft:")?;
        let color_name = name
            .strip_suffix("_wall_banner")
            .or_else(|| name.strip_suffix("_banner"))?;
        Some(Self {
            base_color: DyeColor::from_vanilla_name(color_name)?,
            patterns: Vec::new(),
            custom_name: None,
        })
    }

    pub fn add_pattern(&mut self, pattern: impl Into<String>, color: DyeColor) -> bool {
        if self.patterns.len() >= Self::MAX_PATTERNS {
            return false;
        }
        self.patterns.push(BannerPatternLayer {
            pattern: pattern.into(),
            color,
        });
        true
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if !self.patterns.is_empty() {
            fields.push((
                "patterns".to_string(),
                Tag::List(
                    self.patterns
                        .iter()
                        .map(BannerPatternLayer::to_tag)
                        .collect(),
                ),
            ));
        }
        if let Some(custom_name) = &self.custom_name {
            fields.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(block_state: &str, tag: &Tag) -> Option<Self> {
        let mut banner = Self::from_block_state(block_state)?;
        let entries = compound_entries(tag);
        banner.custom_name = entries
            .and_then(|entries| get_string(entries, "CustomName"))
            .map(ToString::to_string);
        banner.patterns = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "patterns"))
            .and_then(|(_, tag)| match tag {
                Tag::List(values) => Some(
                    values
                        .iter()
                        .filter_map(BannerPatternLayer::from_tag)
                        .take(Self::MAX_PATTERNS)
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        Some(banner)
    }
}

impl Default for PotDecorations {
    fn default() -> Self {
        Self {
            back: None,
            left: None,
            right: None,
            front: None,
        }
    }
}

impl PotDecorations {
    const BRICK: &'static str = "minecraft:brick";

    pub fn new(
        back: Option<String>,
        left: Option<String>,
        right: Option<String>,
        front: Option<String>,
    ) -> Self {
        Self {
            back: Self::normalize_side(back),
            left: Self::normalize_side(left),
            right: Self::normalize_side(right),
            front: Self::normalize_side(front),
        }
    }

    fn normalize_side(side: Option<String>) -> Option<String> {
        side.filter(|item| item != Self::BRICK)
    }

    pub fn ordered(&self) -> Vec<String> {
        [&self.back, &self.left, &self.right, &self.front]
            .into_iter()
            .map(|side| side.clone().unwrap_or_else(|| Self::BRICK.to_string()))
            .collect()
    }

    fn is_empty(&self) -> bool {
        self.back.is_none() && self.left.is_none() && self.right.is_none() && self.front.is_none()
    }

    fn to_tag(&self) -> Tag {
        Tag::List(self.ordered().into_iter().map(Tag::String).collect())
    }

    fn from_tag(tag: &Tag) -> Self {
        let Tag::List(values) = tag else {
            return Self::default();
        };
        let item = |index: usize| -> Option<String> {
            values.get(index).and_then(|tag| match tag {
                Tag::String(item) if item != Self::BRICK => Some(item.clone()),
                _ => None,
            })
        };
        Self::new(item(0), item(1), item(2), item(3))
    }
}

impl PotItemStack {
    fn is_empty(&self) -> bool {
        self.item_id == "minecraft:air" || self.count <= 0
    }

    fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("id".to_string(), Tag::String(self.item_id.clone())),
            ("count".to_string(), Tag::Int(self.count)),
        ])
    }

    fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        let stack = Self {
            item_id: get_string(entries, "id")?.to_string(),
            count: get_int(entries, "count").unwrap_or(1),
        };
        (!stack.is_empty()).then_some(stack)
    }
}

impl DecoratedPotWobbleStyle {
    pub fn id(self) -> i32 {
        match self {
            Self::Positive => 0,
            Self::Negative => 1,
        }
    }

    pub fn duration(self) -> i32 {
        match self {
            Self::Positive => 7,
            Self::Negative => 10,
        }
    }

    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Positive),
            1 => Some(Self::Negative),
            _ => None,
        }
    }
}

impl Default for DecoratedPotBlockEntity {
    fn default() -> Self {
        Self {
            decorations: PotDecorations::default(),
            item: None,
            loot_table: None,
            loot_table_seed: 0,
            wobble_started_at_tick: 0,
            last_wobble_style: None,
        }
    }
}

impl DecoratedPotBlockEntity {
    pub const EVENT_POT_WOBBLES: i32 = 1;

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if !self.decorations.is_empty() {
            fields.push(("sherds".to_string(), self.decorations.to_tag()));
        }
        if let Some(loot_table) = &self.loot_table {
            fields.push(("LootTable".to_string(), Tag::String(loot_table.clone())));
            if self.loot_table_seed != 0 {
                fields.push(("LootTableSeed".to_string(), Tag::Long(self.loot_table_seed)));
            }
        } else if let Some(item) = &self.item {
            if !item.is_empty() {
                fields.push(("item".to_string(), item.to_tag()));
            }
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::default();
        };
        let decorations = entries
            .iter()
            .find(|(name, _)| name == "sherds")
            .map(|(_, tag)| PotDecorations::from_tag(tag))
            .unwrap_or_default();
        let loot_table = get_string(entries, "LootTable").map(ToString::to_string);
        let loot_table_seed = entries
            .iter()
            .find_map(|(name, tag)| match tag {
                Tag::Long(seed) if name == "LootTableSeed" => Some(*seed),
                _ => None,
            })
            .unwrap_or(0);
        let item = if loot_table.is_some() {
            None
        } else {
            entries
                .iter()
                .find(|(name, _)| name == "item")
                .and_then(|(_, tag)| PotItemStack::from_tag(tag))
        };
        Self {
            decorations,
            item,
            loot_table,
            loot_table_seed,
            wobble_started_at_tick: 0,
            last_wobble_style: None,
        }
    }

    pub fn trigger_event(&mut self, event: i32, data: i32, game_time: i64) -> bool {
        let Some(style) = DecoratedPotWobbleStyle::from_id(data) else {
            return false;
        };
        if event != Self::EVENT_POT_WOBBLES {
            return false;
        }
        self.wobble_started_at_tick = game_time;
        self.last_wobble_style = Some(style);
        true
    }

    pub fn destruction_drops(&self) -> DecoratedPotDrops {
        DecoratedPotDrops {
            decoration_items: self.decorations.ordered(),
            stored_item: self.item.clone().filter(|item| !item.is_empty()),
        }
    }
}

impl CopperWeatherState {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Unaffected => "unaffected",
            Self::Exposed => "exposed",
            Self::Weathered => "weathered",
            Self::Oxidized => "oxidized",
        }
    }
}

impl CopperGolemStatuePose {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Standing => "standing",
            Self::Sitting => "sitting",
            Self::Running => "running",
            Self::Star => "star",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Standing => Self::Sitting,
            Self::Sitting => Self::Running,
            Self::Running => Self::Star,
            Self::Star => Self::Standing,
        }
    }

    pub fn comparator_output(self) -> u8 {
        match self {
            Self::Standing => 1,
            Self::Sitting => 2,
            Self::Running => 3,
            Self::Star => 4,
        }
    }
}

impl CopperGolemStatueBlockEntity {
    pub fn from_block_state(block_state: &str, pose: CopperGolemStatuePose) -> Option<Self> {
        let id = block_state.strip_prefix("minecraft:")?;
        let (waxed, id) = id
            .strip_prefix("waxed_")
            .map(|id| (true, id))
            .unwrap_or((false, id));
        let weather_state = match id {
            "copper_golem_statue" => CopperWeatherState::Unaffected,
            "exposed_copper_golem_statue" => CopperWeatherState::Exposed,
            "weathered_copper_golem_statue" => CopperWeatherState::Weathered,
            "oxidized_copper_golem_statue" => CopperWeatherState::Oxidized,
            _ => return None,
        };
        Some(Self {
            weather_state,
            waxed,
            pose,
            custom_name: None,
        })
    }

    pub fn update_pose(&mut self) {
        self.pose = self.pose.next();
    }

    pub fn comparator_output(&self) -> u8 {
        self.pose.comparator_output()
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(custom_name) = &self.custom_name {
            fields.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn clone_item_components(&self) -> Tag {
        let mut fields = vec![(
            "minecraft:block_state".to_string(),
            Tag::Compound(vec![(
                "copper_golem_pose".to_string(),
                Tag::String(self.pose.serialized_name().to_string()),
            )]),
        )];
        if let Some(custom_name) = &self.custom_name {
            fields.push((
                "minecraft:custom_name".to_string(),
                Tag::String(custom_name.clone()),
            ));
        }
        Tag::Compound(fields)
    }
}

impl SkullBlockEntity {
    pub fn new() -> Self {
        Self {
            profile: None,
            note_block_sound: None,
            custom_name: None,
            animation_tick_count: 0,
            is_animating: false,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(profile) = &self.profile {
            fields.push(("profile".to_string(), profile.clone()));
        }
        if let Some(note_block_sound) = &self.note_block_sound {
            fields.push((
                "note_block_sound".to_string(),
                Tag::String(note_block_sound.clone()),
            ));
        }
        if let Some(custom_name) = &self.custom_name {
            fields.push(("custom_name".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Tag::Compound(entries) = tag else {
            return Self::new();
        };
        Self {
            profile: entries
                .iter()
                .find(|(name, _)| name == "profile")
                .map(|(_, tag)| tag.clone()),
            note_block_sound: get_string(entries, "note_block_sound").map(ToString::to_string),
            custom_name: get_string(entries, "custom_name").map(ToString::to_string),
            animation_tick_count: 0,
            is_animating: false,
        }
    }

    pub fn apply_implicit_components(&mut self, components: &BTreeMap<String, Tag>) {
        self.profile = components.get("minecraft:profile").cloned();
        self.note_block_sound = components
            .get("minecraft:note_block_sound")
            .and_then(|tag| match tag {
                Tag::String(id) => Some(id.clone()),
                _ => None,
            });
        self.custom_name = components
            .get("minecraft:custom_name")
            .and_then(|tag| match tag {
                Tag::String(name) => Some(name.clone()),
                _ => None,
            });
    }

    pub fn collect_implicit_components(&self) -> BTreeMap<String, Tag> {
        let mut components = BTreeMap::new();
        if let Some(profile) = &self.profile {
            components.insert("minecraft:profile".to_string(), profile.clone());
        }
        if let Some(note_block_sound) = &self.note_block_sound {
            components.insert(
                "minecraft:note_block_sound".to_string(),
                Tag::String(note_block_sound.clone()),
            );
        }
        if let Some(custom_name) = &self.custom_name {
            components.insert(
                "minecraft:custom_name".to_string(),
                Tag::String(custom_name.clone()),
            );
        }
        components
    }

    pub fn remove_components_from_tag(tag: &mut Tag) {
        if let Tag::Compound(entries) = tag {
            entries.retain(|(name, _)| {
                name != "profile" && name != "note_block_sound" && name != "custom_name"
            });
        }
    }

    pub fn animation_tick(&mut self, powered: bool) {
        if powered {
            self.is_animating = true;
            self.animation_tick_count += 1;
        } else {
            self.is_animating = false;
        }
    }

    pub fn animation(&self, partial_tick: f32) -> f32 {
        if self.is_animating {
            self.animation_tick_count as f32 + partial_tick
        } else {
            self.animation_tick_count as f32
        }
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }
}

impl BellBlockEntity {
    pub const EVENT_RING: i32 = 1;
    pub const DURATION: i32 = 50;
    pub const GLOW_DURATION: i32 = 60;
    pub const MIN_TICKS_BETWEEN_SEARCHES: u64 = 60;
    pub const MAX_RESONATION_TICKS: i32 = 40;
    pub const TICKS_BEFORE_RESONATION: i32 = 5;
    pub const SEARCH_RADIUS: f64 = 48.0;
    pub const HEAR_BELL_RADIUS: f64 = 32.0;
    pub const HIGHLIGHT_RAIDERS_RADIUS: f64 = 48.0;

    pub fn new() -> Self {
        Self {
            last_ring_timestamp: 0,
            ticks: 0,
            shaking: false,
            click_direction: None,
            heard_bell_entities: 0,
            nearby_raiders_within_hear_radius: 0,
            nearby_raiders_within_highlight_radius: 0,
            resonating: false,
            resonation_ticks: 0,
        }
    }

    pub fn direction_3d_data_value(direction: Direction) -> i32 {
        match direction {
            Direction::Down => 0,
            Direction::Up => 1,
            Direction::North => 2,
            Direction::South => 3,
            Direction::West => 4,
            Direction::East => 5,
        }
    }

    pub fn direction_from_3d_data_value(value: i32) -> Direction {
        match value {
            0 => Direction::Down,
            1 => Direction::Up,
            2 => Direction::North,
            3 => Direction::South,
            4 => Direction::West,
            5 => Direction::East,
            _ => Direction::Down,
        }
    }

    pub fn on_hit(&mut self, click_direction: Direction) -> BellBlockEvent {
        self.click_direction = Some(click_direction);
        if self.shaking {
            self.ticks = 0;
        } else {
            self.shaking = true;
        }
        BellBlockEvent {
            event_id: Self::EVENT_RING,
            event_param: Self::direction_3d_data_value(click_direction),
        }
    }

    pub fn trigger_event(
        &mut self,
        event_id: i32,
        event_param: i32,
        game_time: u64,
        nearby_living_within_hear_radius: usize,
        nearby_raiders_within_hear_radius: usize,
        nearby_raiders_within_highlight_radius: usize,
    ) -> bool {
        if event_id != Self::EVENT_RING {
            return false;
        }
        self.update_entities(
            game_time,
            nearby_living_within_hear_radius,
            nearby_raiders_within_hear_radius,
            nearby_raiders_within_highlight_radius,
        );
        self.resonation_ticks = 0;
        self.click_direction = Some(Self::direction_from_3d_data_value(event_param));
        self.ticks = 0;
        self.shaking = true;
        true
    }

    pub fn update_entities(
        &mut self,
        game_time: u64,
        nearby_living_within_hear_radius: usize,
        nearby_raiders_within_hear_radius: usize,
        nearby_raiders_within_highlight_radius: usize,
    ) {
        if game_time > self.last_ring_timestamp + Self::MIN_TICKS_BETWEEN_SEARCHES
            || self.last_ring_timestamp == 0
        {
            self.last_ring_timestamp = game_time;
            self.heard_bell_entities = nearby_living_within_hear_radius;
            self.nearby_raiders_within_hear_radius = nearby_raiders_within_hear_radius;
            self.nearby_raiders_within_highlight_radius = nearby_raiders_within_highlight_radius;
        }
    }

    pub fn tick(&mut self) -> BellTickEffects {
        let mut effects = BellTickEffects {
            play_resonate_sound: false,
            glowing_raiders: 0,
        };

        if self.shaking {
            self.ticks += 1;
        }

        if self.ticks >= Self::DURATION {
            self.shaking = false;
            self.ticks = 0;
        }

        if self.ticks >= Self::TICKS_BEFORE_RESONATION
            && self.resonation_ticks == 0
            && self.nearby_raiders_within_hear_radius > 0
        {
            self.resonating = true;
            effects.play_resonate_sound = true;
        }

        if self.resonating {
            if self.resonation_ticks < Self::MAX_RESONATION_TICKS {
                self.resonation_ticks += 1;
            } else {
                effects.glowing_raiders = self.nearby_raiders_within_highlight_radius;
                self.resonating = false;
            }
        }

        effects
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![])
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }
}

impl BrushableBlockEntity {
    pub const BRUSH_COOLDOWN_TICKS: u64 = 10;
    pub const BRUSH_RESET_TICKS: u64 = 40;
    pub const REQUIRED_BRUSHES_TO_BREAK: i32 = 10;
    pub const RETRACTION_SPEED: i32 = 2;
    pub const RETRACTION_TICKS: u64 = 4;

    pub fn new() -> Self {
        Self {
            brush_count: 0,
            brush_count_resets_at_tick: 0,
            cooldown_ends_at_tick: 0,
            item: None,
            hit_direction: None,
            loot_table: None,
            loot_table_seed: 0,
        }
    }

    pub fn set_loot_table(&mut self, loot_table: impl Into<String>, seed: i64) {
        self.loot_table = Some(loot_table.into());
        self.loot_table_seed = seed;
        self.item = None;
    }

    pub fn unpack_loot_table(&mut self, generated_item: Option<PotItemStack>) -> bool {
        if self.loot_table.take().is_some() {
            self.loot_table_seed = 0;
            self.item = generated_item;
            true
        } else {
            false
        }
    }

    pub fn brush(
        &mut self,
        game_time: u64,
        direction: Direction,
        generated_loot_item: Option<PotItemStack>,
    ) -> BrushResult {
        if self.hit_direction.is_none() {
            self.hit_direction = Some(direction);
        }
        self.brush_count_resets_at_tick = game_time + Self::BRUSH_RESET_TICKS;
        if game_time < self.cooldown_ends_at_tick {
            return BrushResult::CoolingDown;
        }

        self.cooldown_ends_at_tick = game_time + Self::BRUSH_COOLDOWN_TICKS;
        self.unpack_loot_table(generated_loot_item);
        self.brush_count += 1;
        if self.brush_count >= Self::REQUIRED_BRUSHES_TO_BREAK {
            self.brush_count = Self::REQUIRED_BRUSHES_TO_BREAK;
            return BrushResult::Completed;
        }

        BrushResult::InProgress {
            dusted: self.completion_state(),
        }
    }

    pub fn check_reset(&mut self, game_time: u64) -> Option<i32> {
        if self.brush_count != 0 && game_time >= self.brush_count_resets_at_tick {
            let previous = self.completion_state();
            self.brush_count = (self.brush_count - Self::RETRACTION_SPEED).max(0);
            let current = self.completion_state();
            if self.brush_count == 0 {
                self.hit_direction = None;
                self.brush_count_resets_at_tick = 0;
                self.cooldown_ends_at_tick = 0;
            } else {
                self.brush_count_resets_at_tick = game_time + Self::RETRACTION_TICKS;
            }
            return (previous != current).then_some(current);
        }

        None
    }

    pub fn completion_state(&self) -> i32 {
        if self.brush_count == 0 {
            0
        } else if self.brush_count < 3 {
            1
        } else if self.brush_count < 6 {
            2
        } else {
            3
        }
    }

    pub fn drop_content(&mut self) -> Option<(PotItemStack, Direction)> {
        let item = self.item.take()?;
        Some((item, self.hit_direction.unwrap_or(Direction::Up)))
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(loot_table) = &self.loot_table {
            fields.push(("LootTable".to_string(), Tag::String(loot_table.clone())));
            if self.loot_table_seed != 0 {
                fields.push(("LootTableSeed".to_string(), Tag::Long(self.loot_table_seed)));
            }
        } else if let Some(item) = &self.item {
            fields.push(("item".to_string(), item.to_tag()));
        }
        if let Some(direction) = self.hit_direction {
            fields.push((
                "hit_direction".to_string(),
                Tag::String(direction_name(direction).to_string()),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut brushable = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return brushable;
        };
        brushable.loot_table = get_string(entries, "LootTable").map(ToString::to_string);
        brushable.loot_table_seed = entries
            .iter()
            .find(|(name, _)| name == "LootTableSeed")
            .map(|(_, tag)| tag_long_or_zero(tag))
            .unwrap_or(0);
        if brushable.loot_table.is_none() {
            brushable.item = entries
                .iter()
                .find(|(name, _)| name == "item")
                .and_then(|(_, tag)| PotItemStack::from_tag(tag));
        }
        brushable.hit_direction =
            get_string(entries, "hit_direction").and_then(direction_from_name);
        brushable
    }

    pub fn get_update_tag(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(direction) = self.hit_direction {
            fields.push((
                "hit_direction".to_string(),
                Tag::String(direction_name(direction).to_string()),
            ));
        }
        if let Some(item) = &self.item {
            fields.push(("item".to_string(), item.to_tag()));
        }
        Tag::Compound(fields)
    }
}

// Source: decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlockEntityType.java
pub const BLOCK_ENTITY_TYPES: &[BlockEntityTypeInfo] = &[
    info(
        BlockEntityTypeId::Furnace,
        "furnace",
        &["minecraft:furnace"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Chest,
        "chest",
        &[
            "minecraft:chest",
            "minecraft:copper_chest",
            "minecraft:exposed_copper_chest",
            "minecraft:weathered_copper_chest",
            "minecraft:oxidized_copper_chest",
            "minecraft:waxed_copper_chest",
            "minecraft:waxed_exposed_copper_chest",
            "minecraft:waxed_weathered_copper_chest",
            "minecraft:waxed_oxidized_copper_chest",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::TrappedChest,
        "trapped_chest",
        &["minecraft:trapped_chest"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::EnderChest,
        "ender_chest",
        &["minecraft:ender_chest"],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::Jukebox,
        "jukebox",
        &["minecraft:jukebox"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Dispenser,
        "dispenser",
        &["minecraft:dispenser"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Dropper,
        "dropper",
        &["minecraft:dropper"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Sign,
        "sign",
        &[
            "minecraft:oak_sign",
            "minecraft:spruce_sign",
            "minecraft:birch_sign",
            "minecraft:acacia_sign",
            "minecraft:cherry_sign",
            "minecraft:jungle_sign",
            "minecraft:dark_oak_sign",
            "minecraft:pale_oak_sign",
            "minecraft:mangrove_sign",
            "minecraft:crimson_sign",
            "minecraft:warped_sign",
            "minecraft:bamboo_sign",
            "minecraft:oak_wall_sign",
            "minecraft:spruce_wall_sign",
            "minecraft:birch_wall_sign",
            "minecraft:acacia_wall_sign",
            "minecraft:cherry_wall_sign",
            "minecraft:jungle_wall_sign",
            "minecraft:dark_oak_wall_sign",
            "minecraft:pale_oak_wall_sign",
            "minecraft:mangrove_wall_sign",
            "minecraft:crimson_wall_sign",
            "minecraft:warped_wall_sign",
            "minecraft:bamboo_wall_sign",
        ],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::HangingSign,
        "hanging_sign",
        &[
            "minecraft:oak_hanging_sign",
            "minecraft:spruce_hanging_sign",
            "minecraft:birch_hanging_sign",
            "minecraft:acacia_hanging_sign",
            "minecraft:cherry_hanging_sign",
            "minecraft:jungle_hanging_sign",
            "minecraft:dark_oak_hanging_sign",
            "minecraft:pale_oak_hanging_sign",
            "minecraft:crimson_hanging_sign",
            "minecraft:warped_hanging_sign",
            "minecraft:mangrove_hanging_sign",
            "minecraft:bamboo_hanging_sign",
            "minecraft:oak_wall_hanging_sign",
            "minecraft:spruce_wall_hanging_sign",
            "minecraft:birch_wall_hanging_sign",
            "minecraft:acacia_wall_hanging_sign",
            "minecraft:cherry_wall_hanging_sign",
            "minecraft:jungle_wall_hanging_sign",
            "minecraft:dark_oak_wall_hanging_sign",
            "minecraft:pale_oak_wall_hanging_sign",
            "minecraft:crimson_wall_hanging_sign",
            "minecraft:warped_wall_hanging_sign",
            "minecraft:mangrove_wall_hanging_sign",
            "minecraft:bamboo_wall_hanging_sign",
        ],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::MobSpawner,
        "mob_spawner",
        &["minecraft:spawner"],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::CreakingHeart,
        "creaking_heart",
        &["minecraft:creaking_heart"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Piston,
        "piston",
        &["minecraft:moving_piston"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::BrewingStand,
        "brewing_stand",
        &["minecraft:brewing_stand"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::EnchantingTable,
        "enchanting_table",
        &["minecraft:enchanting_table"],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::EndPortal,
        "end_portal",
        &["minecraft:end_portal"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Beacon,
        "beacon",
        &["minecraft:beacon"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Skull,
        "skull",
        &[
            "minecraft:skeleton_wall_skull",
            "minecraft:creeper_head",
            "minecraft:creeper_wall_head",
            "minecraft:dragon_wall_head",
            "minecraft:skeleton_skull",
            "minecraft:player_head",
            "minecraft:player_wall_head",
            "minecraft:dragon_head",
            "minecraft:zombie_head",
            "minecraft:zombie_wall_head",
            "minecraft:wither_skeleton_skull",
            "minecraft:wither_skeleton_wall_skull",
            "minecraft:piglin_head",
            "minecraft:piglin_wall_head",
        ],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::DaylightDetector,
        "daylight_detector",
        &["minecraft:daylight_detector"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Hopper,
        "hopper",
        &["minecraft:hopper"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Comparator,
        "comparator",
        &["minecraft:comparator"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Banner,
        "banner",
        &[
            "minecraft:white_banner",
            "minecraft:orange_banner",
            "minecraft:magenta_banner",
            "minecraft:light_blue_banner",
            "minecraft:yellow_banner",
            "minecraft:lime_banner",
            "minecraft:pink_banner",
            "minecraft:gray_banner",
            "minecraft:light_gray_banner",
            "minecraft:cyan_banner",
            "minecraft:purple_banner",
            "minecraft:blue_banner",
            "minecraft:brown_banner",
            "minecraft:green_banner",
            "minecraft:red_banner",
            "minecraft:black_banner",
            "minecraft:white_wall_banner",
            "minecraft:orange_wall_banner",
            "minecraft:magenta_wall_banner",
            "minecraft:light_blue_wall_banner",
            "minecraft:yellow_wall_banner",
            "minecraft:lime_wall_banner",
            "minecraft:pink_wall_banner",
            "minecraft:gray_wall_banner",
            "minecraft:light_gray_wall_banner",
            "minecraft:cyan_wall_banner",
            "minecraft:purple_wall_banner",
            "minecraft:blue_wall_banner",
            "minecraft:brown_wall_banner",
            "minecraft:green_wall_banner",
            "minecraft:red_wall_banner",
            "minecraft:black_wall_banner",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::StructureBlock,
        "structure_block",
        &["minecraft:structure_block"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::EndGateway,
        "end_gateway",
        &["minecraft:end_gateway"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::CommandBlock,
        "command_block",
        &[
            "minecraft:command_block",
            "minecraft:chain_command_block",
            "minecraft:repeating_command_block",
        ],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::ShulkerBox,
        "shulker_box",
        &[
            "minecraft:shulker_box",
            "minecraft:black_shulker_box",
            "minecraft:blue_shulker_box",
            "minecraft:brown_shulker_box",
            "minecraft:cyan_shulker_box",
            "minecraft:gray_shulker_box",
            "minecraft:green_shulker_box",
            "minecraft:light_blue_shulker_box",
            "minecraft:light_gray_shulker_box",
            "minecraft:lime_shulker_box",
            "minecraft:magenta_shulker_box",
            "minecraft:orange_shulker_box",
            "minecraft:pink_shulker_box",
            "minecraft:purple_shulker_box",
            "minecraft:red_shulker_box",
            "minecraft:white_shulker_box",
            "minecraft:yellow_shulker_box",
        ],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::Bed,
        "bed",
        &[
            "minecraft:black_bed",
            "minecraft:blue_bed",
            "minecraft:brown_bed",
            "minecraft:cyan_bed",
            "minecraft:gray_bed",
            "minecraft:green_bed",
            "minecraft:light_blue_bed",
            "minecraft:light_gray_bed",
            "minecraft:lime_bed",
            "minecraft:magenta_bed",
            "minecraft:orange_bed",
            "minecraft:pink_bed",
            "minecraft:purple_bed",
            "minecraft:red_bed",
            "minecraft:white_bed",
            "minecraft:yellow_bed",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Conduit,
        "conduit",
        &["minecraft:conduit"],
        BlockEntityTickKind::Both,
        false,
    ),
    info(
        BlockEntityTypeId::Barrel,
        "barrel",
        &["minecraft:barrel"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Smoker,
        "smoker",
        &["minecraft:smoker"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::BlastFurnace,
        "blast_furnace",
        &["minecraft:blast_furnace"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Lectern,
        "lectern",
        &["minecraft:lectern"],
        BlockEntityTickKind::None,
        true,
    ),
    info(
        BlockEntityTypeId::Bell,
        "bell",
        &["minecraft:bell"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Jigsaw,
        "jigsaw",
        &["minecraft:jigsaw"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Campfire,
        "campfire",
        &["minecraft:campfire", "minecraft:soul_campfire"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Beehive,
        "beehive",
        &["minecraft:bee_nest", "minecraft:beehive"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::SculkSensor,
        "sculk_sensor",
        &["minecraft:sculk_sensor"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::CalibratedSculkSensor,
        "calibrated_sculk_sensor",
        &["minecraft:calibrated_sculk_sensor"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::SculkCatalyst,
        "sculk_catalyst",
        &["minecraft:sculk_catalyst"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::SculkShrieker,
        "sculk_shrieker",
        &["minecraft:sculk_shrieker"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::ChiseledBookshelf,
        "chiseled_bookshelf",
        &["minecraft:chiseled_bookshelf"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Shelf,
        "shelf",
        &[
            "minecraft:acacia_shelf",
            "minecraft:bamboo_shelf",
            "minecraft:birch_shelf",
            "minecraft:cherry_shelf",
            "minecraft:crimson_shelf",
            "minecraft:dark_oak_shelf",
            "minecraft:jungle_shelf",
            "minecraft:mangrove_shelf",
            "minecraft:oak_shelf",
            "minecraft:pale_oak_shelf",
            "minecraft:spruce_shelf",
            "minecraft:warped_shelf",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::BrushableBlock,
        "brushable_block",
        &["minecraft:suspicious_sand", "minecraft:suspicious_gravel"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::DecoratedPot,
        "decorated_pot",
        &["minecraft:decorated_pot"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Crafter,
        "crafter",
        &["minecraft:crafter"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::TrialSpawner,
        "trial_spawner",
        &["minecraft:trial_spawner"],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::Vault,
        "vault",
        &["minecraft:vault"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::TestBlock,
        "test_block",
        &["minecraft:test_block"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::TestInstanceBlock,
        "test_instance_block",
        &["minecraft:test_instance_block"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::CopperGolemStatue,
        "copper_golem_statue",
        &[
            "minecraft:copper_golem_statue",
            "minecraft:exposed_copper_golem_statue",
            "minecraft:weathered_copper_golem_statue",
            "minecraft:oxidized_copper_golem_statue",
            "minecraft:waxed_copper_golem_statue",
            "minecraft:waxed_exposed_copper_golem_statue",
            "minecraft:waxed_weathered_copper_golem_statue",
            "minecraft:waxed_oxidized_copper_golem_statue",
        ],
        BlockEntityTickKind::Server,
        false,
    ),
];

const fn info(
    id: BlockEntityTypeId,
    key: &'static str,
    valid_blocks: &'static [&'static str],
    tick_kind: BlockEntityTickKind,
    op_only_custom_data: bool,
) -> BlockEntityTypeInfo {
    BlockEntityTypeInfo {
        id,
        key,
        valid_blocks,
        tick_kind,
        op_only_custom_data,
    }
}

pub fn type_info(ty: BlockEntityTypeId) -> &'static BlockEntityTypeInfo {
    BLOCK_ENTITY_TYPES
        .iter()
        .find(|info| info.id == ty)
        .expect("block entity type table covers every id")
}

pub fn type_by_key(key: &str) -> Option<BlockEntityTypeId> {
    BLOCK_ENTITY_TYPES
        .iter()
        .find(|info| info.key == key || format!("minecraft:{}", info.key) == key)
        .map(|info| info.id)
}

pub fn is_valid_block_state(ty: BlockEntityTypeId, block_state: &str) -> bool {
    type_info(ty).valid_blocks.contains(&block_state)
}

pub fn only_op_can_set_nbt(ty: BlockEntityTypeId) -> bool {
    type_info(ty).op_only_custom_data
}

pub fn has_block_entity_for_block(registry_id: &str) -> bool {
    BLOCK_ENTITY_TYPES
        .iter()
        .any(|entry| entry.valid_blocks.contains(&registry_id))
}

impl BlockEntity {
    pub fn new(
        ty: BlockEntityTypeId,
        pos: BlockPos,
        block_state: &str,
    ) -> Result<Self, BlockEntityError> {
        if !is_valid_block_state(ty, block_state) {
            return Err(BlockEntityError::InvalidBlockState {
                ty,
                block_state: block_state.to_string(),
            });
        }

        Ok(Self {
            ty,
            pos,
            block_state: block_state.to_string(),
            custom_data: BTreeMap::new(),
            components: BTreeMap::new(),
            has_level: false,
            removed: false,
            changed: false,
            tick_count: 0,
        })
    }

    pub fn set_level(&mut self) {
        self.has_level = true;
    }

    pub fn set_removed(&mut self) {
        self.removed = true;
    }

    pub fn clear_removed(&mut self) {
        self.removed = false;
    }

    pub fn set_changed(&mut self) {
        if self.has_level {
            self.changed = true;
        }
    }

    pub fn save_custom_only(&self) -> Tag {
        compound_from_map(&self.custom_data)
    }

    pub fn save_without_metadata(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "components".to_string(),
            compound_from_map(&self.components),
        );
        compound_from_map(&values)
    }

    pub fn save_with_id(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "id".to_string(),
            Tag::String(type_info(self.ty).key.to_string()),
        );
        compound_from_map(&values)
    }

    pub fn save_with_full_metadata(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "id".to_string(),
            Tag::String(type_info(self.ty).key.to_string()),
        );
        values.insert("x".to_string(), Tag::Int(self.pos.x));
        values.insert("y".to_string(), Tag::Int(self.pos.y));
        values.insert("z".to_string(), Tag::Int(self.pos.z));
        values.insert(
            "components".to_string(),
            compound_from_map(&self.components),
        );
        compound_from_map(&values)
    }

    pub fn get_update_tag(&self) -> Tag {
        match self.ty {
            BlockEntityTypeId::Chest
            | BlockEntityTypeId::TrappedChest
            | BlockEntityTypeId::Barrel
            | BlockEntityTypeId::Hopper
            | BlockEntityTypeId::Dispenser
            | BlockEntityTypeId::Dropper => Tag::Compound(Vec::new()),
            _ => self.save_without_metadata(),
        }
    }

    pub fn get_update_packet(&self) -> ClientboundBlockEntityDataPacket {
        ClientboundBlockEntityDataPacket {
            pos: self.pos,
            ty: self.ty,
            tag: self.get_update_tag(),
        }
    }

    pub fn handle_update_tag(&mut self, tag: &Tag) {
        let Some(entries) = compound_entries(tag) else {
            return;
        };

        self.custom_data.clear();
        self.components.clear();
        for (key, value) in entries {
            match key.as_str() {
                "id" | "x" | "y" | "z" => {}
                "components" => {
                    self.components = map_from_compound(value);
                }
                _ => {
                    self.custom_data.insert(key.clone(), value.clone());
                }
            }
        }
    }

    pub fn tick(&mut self, client_side: bool) -> bool {
        let tick_kind = type_info(self.ty).tick_kind;
        let should_tick = matches!(
            (tick_kind, client_side),
            (BlockEntityTickKind::Both, _)
                | (BlockEntityTickKind::Server, false)
                | (BlockEntityTickKind::Client, true)
        );

        if should_tick && !self.removed && self.has_level {
            self.tick_count += 1;
            true
        } else {
            false
        }
    }
}

impl TickingBlockEntity {
    pub fn new(entity: BlockEntity, client_side: bool) -> Self {
        Self {
            entity,
            client_side,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.entity.tick(self.client_side)
    }

    pub fn is_removed(&self) -> bool {
        self.entity.removed
    }

    pub fn pos(&self) -> BlockPos {
        self.entity.pos
    }

    pub fn type_key(&self) -> &'static str {
        type_info(self.entity.ty).key
    }
}

pub fn load_static(
    pos: BlockPos,
    block_state: &str,
    tag: &Tag,
) -> Result<BlockEntity, BlockEntityError> {
    let values = compound_entries(tag);
    let id = values
        .and_then(|entries| get_string(entries, "id"))
        .ok_or(BlockEntityError::MissingId)?;
    let ty = type_by_key(id).ok_or_else(|| BlockEntityError::UnknownType(id.to_string()))?;
    let mut entity = BlockEntity::new(ty, pos, block_state)?;

    if let Some(entries) = values {
        for (key, value) in entries {
            match key.as_str() {
                "id" | "x" | "y" | "z" => {}
                "components" => {
                    entity.components = map_from_compound(value);
                }
                _ => {
                    entity.custom_data.insert(key.clone(), value.clone());
                }
            }
        }
    }

    Ok(entity)
}

pub fn load_static_with_data_version(
    pos: BlockPos,
    block_state: &str,
    tag: &Tag,
    data_version: i32,
) -> Result<BlockEntity, BlockEntityError> {
    require_current_world_data_version(data_version)
        .map_err(BlockEntityError::UnsupportedDataVersion)?;
    load_static(pos, block_state, tag)
}

pub fn corrected_pos_from_chunk(base_chunk_x: i32, base_chunk_z: i32, tag: &Tag) -> BlockPos {
    let entries = compound_entries(tag);
    let x = entries
        .and_then(|entries| get_int(entries, "x"))
        .unwrap_or(0);
    let y = entries
        .and_then(|entries| get_int(entries, "y"))
        .unwrap_or(0);
    let z = entries
        .and_then(|entries| get_int(entries, "z"))
        .unwrap_or(0);
    let section_x = x.div_euclid(16);
    let section_z = z.div_euclid(16);

    if section_x == base_chunk_x && section_z == base_chunk_z {
        BlockPos { x, y, z }
    } else {
        BlockPos {
            x: base_chunk_x * 16 + x.rem_euclid(16),
            y,
            z: base_chunk_z * 16 + z.rem_euclid(16),
        }
    }
}

pub fn block_entity_packet_from_chunk(
    entity: &BlockEntity,
    chunk_min_y: i32,
) -> (u8, i16, BlockEntityTypeId, Tag) {
    let packed_xz = ((entity.pos.x & 15) << 4) | (entity.pos.z & 15);
    let section_y = (entity.pos.y - chunk_min_y) as i16;
    (
        packed_xz as u8,
        section_y,
        entity.ty,
        entity.get_update_tag(),
    )
}

fn compound_from_map(values: &BTreeMap<String, Tag>) -> Tag {
    Tag::Compound(
        values
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    )
}

fn map_from_compound(tag: &Tag) -> BTreeMap<String, Tag> {
    compound_entries(tag)
        .map(|entries| entries.iter().cloned().collect())
        .unwrap_or_default()
}

fn compound_entries(tag: &Tag) -> Option<&Vec<(String, Tag)>> {
    match tag {
        Tag::Compound(entries) => Some(entries),
        _ => None,
    }
}

fn get_string<'a>(entries: &'a [(String, Tag)], key: &str) -> Option<&'a str> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::String(value) if name == key => Some(value.as_str()),
        _ => None,
    })
}

fn get_int(entries: &[(String, Tag)], key: &str) -> Option<i32> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Int(value) if name == key => Some(*value),
        _ => None,
    })
}

fn get_long(entries: &[(String, Tag)], key: &str) -> Option<i64> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Long(value) if name == key => Some(*value),
        _ => None,
    })
}

fn get_byte(entries: &[(String, Tag)], key: &str) -> Option<i8> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Byte(value) if name == key => Some(*value),
        _ => None,
    })
}

fn get_bool(entries: &[(String, Tag)], key: &str) -> Option<bool> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Byte(value) if name == key => Some(*value != 0),
        _ => None,
    })
}

fn block_pos_to_tag(pos: BlockPos) -> Tag {
    Tag::List(vec![Tag::Int(pos.x), Tag::Int(pos.y), Tag::Int(pos.z)])
}

fn block_pos_from_tag(tag: &Tag) -> Option<BlockPos> {
    match tag {
        Tag::List(values) if values.len() == 3 => Some(BlockPos {
            x: tag_int_or_zero(&values[0]),
            y: tag_int_or_zero(&values[1]),
            z: tag_int_or_zero(&values[2]),
        }),
        _ => None,
    }
}

fn tag_int_or_zero(tag: &Tag) -> i32 {
    match tag {
        Tag::Byte(value) => *value as i32,
        Tag::Short(value) => *value as i32,
        Tag::Int(value) => *value,
        Tag::Long(value) => *value as i32,
        _ => 0,
    }
}

fn tag_long_or_zero(tag: &Tag) -> i64 {
    match tag {
        Tag::Byte(value) => *value as i64,
        Tag::Short(value) => *value as i64,
        Tag::Int(value) => *value as i64,
        Tag::Long(value) => *value,
        _ => 0,
    }
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}

fn direction_from_name(value: &str) -> Option<Direction> {
    match value {
        "down" => Some(Direction::Down),
        "up" => Some(Direction::Up),
        "north" => Some(Direction::North),
        "south" => Some(Direction::South),
        "west" => Some(Direction::West),
        "east" => Some(Direction::East),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos() -> BlockPos {
        BlockPos {
            x: 18,
            y: 64,
            z: 35,
        }
    }

    #[test]
    fn block_entity_registry_matches_26_1_2_type_surface() {
        assert_eq!(BLOCK_ENTITY_TYPES.len(), 49);
        assert_eq!(type_info(BlockEntityTypeId::Furnace).key, "furnace");
        assert_eq!(
            type_info(BlockEntityTypeId::CopperGolemStatue).key,
            "copper_golem_statue"
        );
        assert_eq!(
            type_info(BlockEntityTypeId::CopperGolemStatue).valid_blocks,
            &[
                "minecraft:copper_golem_statue",
                "minecraft:exposed_copper_golem_statue",
                "minecraft:weathered_copper_golem_statue",
                "minecraft:oxidized_copper_golem_statue",
                "minecraft:waxed_copper_golem_statue",
                "minecraft:waxed_exposed_copper_golem_statue",
                "minecraft:waxed_weathered_copper_golem_statue",
                "minecraft:waxed_oxidized_copper_golem_statue",
            ]
        );
        assert!(is_valid_block_state(
            BlockEntityTypeId::Sign,
            "minecraft:oak_wall_sign"
        ));
        assert!(!is_valid_block_state(
            BlockEntityTypeId::Sign,
            "minecraft:stone"
        ));
    }

    #[test]
    fn op_only_custom_data_matches_vanilla_guarded_types() {
        assert!(only_op_can_set_nbt(BlockEntityTypeId::CommandBlock));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::Lectern));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::Sign));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::HangingSign));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::MobSpawner));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::TrialSpawner));
        assert!(!only_op_can_set_nbt(BlockEntityTypeId::Vault));
    }

    #[test]
    fn validates_block_state_on_creation_and_load() {
        assert!(BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:chest").is_ok());
        assert_eq!(
            BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:furnace"),
            Err(BlockEntityError::InvalidBlockState {
                ty: BlockEntityTypeId::Chest,
                block_state: "minecraft:furnace".to_string()
            })
        );
    }

    #[test]
    fn detects_block_entity_support_for_block_states() {
        assert!(has_block_entity_for_block("minecraft:chest"));
        assert!(has_block_entity_for_block("minecraft:oak_sign"));
        assert!(has_block_entity_for_block("minecraft:oak_hanging_sign"));
        assert!(has_block_entity_for_block("minecraft:lectern"));
        assert!(has_block_entity_for_block("minecraft:command_block"));
        assert!(has_block_entity_for_block(
            "minecraft:wither_skeleton_skull"
        ));
        assert!(has_block_entity_for_block("minecraft:red_banner"));
        assert!(has_block_entity_for_block("minecraft:conduit"));
        assert!(has_block_entity_for_block("minecraft:bell"));
        assert!(has_block_entity_for_block("minecraft:crimson_hanging_sign"));
        assert!(has_block_entity_for_block("minecraft:brown_banner"));
        assert!(has_block_entity_for_block("minecraft:waxed_copper_chest"));
        assert!(has_block_entity_for_block("minecraft:dark_oak_wall_sign"));
        assert!(has_block_entity_for_block("minecraft:spawner"));
        assert!(has_block_entity_for_block("minecraft:vault"));
        assert!(has_block_entity_for_block("minecraft:trial_spawner"));
        assert!(has_block_entity_for_block(
            "minecraft:calibrated_sculk_sensor"
        ));
        assert!(has_block_entity_for_block("minecraft:chiseled_bookshelf"));
        assert!(has_block_entity_for_block("minecraft:suspicious_sand"));
        assert!(has_block_entity_for_block("minecraft:green_bed"));
        assert!(has_block_entity_for_block("minecraft:black_shulker_box"));
        assert!(has_block_entity_for_block("minecraft:crimson_shelf"));
        assert!(has_block_entity_for_block(
            "minecraft:waxed_oxidized_copper_golem_statue"
        ));
        assert!(has_block_entity_for_block(
            "minecraft:warped_wall_hanging_sign"
        ));
        assert!(has_block_entity_for_block("minecraft:campfire"));
        assert!(!has_block_entity_for_block("minecraft:candle"));
        assert!(!has_block_entity_for_block("minecraft:cauldron"));
        assert!(!has_block_entity_for_block("minecraft:stone"));
        assert!(!has_block_entity_for_block("minecraft:dirt"));
    }

    #[test]
    fn bed_block_entity_is_color_only_placeholder() {
        assert_eq!(
            BedBlockEntity::from_block_state("minecraft:white_bed"),
            Some(BedBlockEntity {
                color: DyeColor::White
            })
        );
        assert_eq!(
            BedBlockEntity::from_block_state("minecraft:light_blue_bed"),
            Some(BedBlockEntity {
                color: DyeColor::LightBlue
            })
        );
        assert_eq!(
            BedBlockEntity::from_block_state("minecraft:black_bed"),
            Some(BedBlockEntity {
                color: DyeColor::Black
            })
        );
        assert_eq!(BedBlockEntity::from_block_state("minecraft:stone"), None);

        let bed = BlockEntity::new(BlockEntityTypeId::Bed, pos(), "minecraft:red_bed").unwrap();
        assert_eq!(bed.ty, BlockEntityTypeId::Bed);
        assert_eq!(
            BedBlockEntity::from_block_state(&bed.block_state),
            Some(BedBlockEntity {
                color: DyeColor::Red
            })
        );
        assert_eq!(
            BedBlockEntity::from_block_state(&bed.block_state)
                .unwrap()
                .save_additional(),
            Tag::Compound(Vec::new())
        );
        assert_eq!(
            bed.save_with_full_metadata(),
            Tag::Compound(vec![
                ("components".to_string(), Tag::Compound(Vec::new())),
                ("id".to_string(), Tag::String("bed".to_string())),
                ("x".to_string(), Tag::Int(pos().x)),
                ("y".to_string(), Tag::Int(pos().y)),
                ("z".to_string(), Tag::Int(pos().z)),
            ])
        );
    }

    #[test]
    fn end_portal_block_entity_is_zero_data_portal_placeholder() {
        let portal =
            BlockEntity::new(BlockEntityTypeId::EndPortal, pos(), "minecraft:end_portal").unwrap();
        assert_eq!(portal.ty, BlockEntityTypeId::EndPortal);
        assert_eq!(type_info(BlockEntityTypeId::EndPortal).key, "end_portal");
        assert_eq!(
            type_info(BlockEntityTypeId::EndPortal).valid_blocks,
            &["minecraft:end_portal"]
        );
        assert_eq!(
            EndPortalBlockEntity.save_additional(),
            Tag::Compound(Vec::new())
        );
        assert_eq!(
            portal.save_with_full_metadata(),
            Tag::Compound(vec![
                ("components".to_string(), Tag::Compound(Vec::new())),
                ("id".to_string(), Tag::String("end_portal".to_string())),
                ("x".to_string(), Tag::Int(pos().x)),
                ("y".to_string(), Tag::Int(pos().y)),
                ("z".to_string(), Tag::Int(pos().z)),
            ])
        );
    }

    #[test]
    fn end_gateway_block_entity_saves_ticks_cooldown_and_exit_like_java() {
        let mut gateway = TheEndGatewayBlockEntity::new();
        assert!(gateway.is_spawning());
        assert!(!gateway.is_cooling_down());
        assert_eq!(gateway.spawn_percent(0.0), 0.0);
        assert_eq!(TheEndGatewayBlockEntity::SPAWN_TIME, 200);
        assert_eq!(TheEndGatewayBlockEntity::COOLDOWN_TIME, 40);
        assert_eq!(TheEndGatewayBlockEntity::ATTENTION_INTERVAL, 2400);
        assert_eq!(TheEndGatewayBlockEntity::GATEWAY_HEIGHT_ABOVE_SURFACE, 10);

        gateway.age = 199;
        assert_eq!(gateway.spawn_percent(0.5), 0.9975);
        assert!(gateway.portal_tick());
        assert_eq!(gateway.age, 200);
        assert!(!gateway.is_spawning());

        gateway.set_exit_position(
            BlockPos {
                x: 12,
                y: 80,
                z: -7,
            },
            true,
        );
        let saved = gateway.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                ("Age".to_string(), Tag::Long(200)),
                (
                    "exit_portal".to_string(),
                    Tag::List(vec![Tag::Int(12), Tag::Int(80), Tag::Int(-7)])
                ),
                ("ExactTeleport".to_string(), Tag::Byte(1)),
            ])
        );
        assert_eq!(TheEndGatewayBlockEntity::load_additional(&saved), gateway);
        assert_eq!(gateway.get_update_tag(), saved);

        gateway.trigger_cooldown();
        assert!(gateway.is_cooling_down());
        assert_eq!(gateway.teleport_cooldown, 40);
        assert_eq!(gateway.cooldown_percent(0.0), 0.0);
        gateway.beam_animation_tick();
        assert_eq!(gateway.age, 201);
        assert_eq!(gateway.teleport_cooldown, 39);
        assert!((gateway.cooldown_percent(0.0) - 0.025).abs() < f32::EPSILON * 4.0);
        assert!(gateway.trigger_event(TheEndGatewayBlockEntity::EVENT_COOLDOWN));
        assert_eq!(gateway.teleport_cooldown, 40);
        assert!(!gateway.trigger_event(99));

        let mut attention = TheEndGatewayBlockEntity {
            age: 2399,
            ..TheEndGatewayBlockEntity::new()
        };
        assert!(attention.portal_tick());
        assert_eq!(attention.age, 2400);
        assert_eq!(attention.teleport_cooldown, 40);
    }

    #[test]
    fn jigsaw_block_entity_saves_priorities_joint_and_generation_plan_like_java() {
        let mut jigsaw = JigsawBlockEntity::new();
        assert_eq!(jigsaw.name, JigsawBlockEntity::EMPTY_ID);
        assert_eq!(jigsaw.target, JigsawBlockEntity::EMPTY_ID);
        assert_eq!(jigsaw.pool, JigsawBlockEntity::EMPTY_ID);
        assert_eq!(jigsaw.final_state, JigsawBlockEntity::DEFAULT_FINAL_STATE);
        assert_eq!(jigsaw.joint, JigsawJointType::Rollable);

        jigsaw.name = "minecraft:house/start".to_string();
        jigsaw.target = "minecraft:house/door".to_string();
        jigsaw.pool = "minecraft:village/plains/houses".to_string();
        jigsaw.final_state = "minecraft:oak_planks".to_string();
        jigsaw.joint = JigsawJointType::Aligned;
        jigsaw.placement_priority = 7;
        jigsaw.selection_priority = -3;

        let saved = jigsaw.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                (
                    "name".to_string(),
                    Tag::String("minecraft:house/start".to_string())
                ),
                (
                    "target".to_string(),
                    Tag::String("minecraft:house/door".to_string())
                ),
                (
                    "pool".to_string(),
                    Tag::String("minecraft:village/plains/houses".to_string())
                ),
                (
                    "final_state".to_string(),
                    Tag::String("minecraft:oak_planks".to_string())
                ),
                ("joint".to_string(), Tag::String("aligned".to_string())),
                ("placement_priority".to_string(), Tag::Int(7)),
                ("selection_priority".to_string(), Tag::Int(-3)),
            ])
        );
        assert_eq!(JigsawBlockEntity::load_additional(&saved), jigsaw);
        assert_eq!(jigsaw.get_update_tag(), saved);

        let plan =
            jigsaw.generation_plan(BlockPos { x: 4, y: 70, z: 8 }, Direction::North, 5, true);
        assert_eq!(
            plan,
            JigsawGenerationPlan {
                pool: "minecraft:village/plains/houses".to_string(),
                target: "minecraft:house/door".to_string(),
                levels: 5,
                keep_jigsaws: true,
                start_pos: BlockPos { x: 4, y: 70, z: 7 },
            }
        );

        let defaults = JigsawBlockEntity::load_additional(&Tag::Compound(vec![(
            "joint".to_string(),
            Tag::String("unknown".to_string()),
        )]));
        assert_eq!(defaults.name, JigsawBlockEntity::EMPTY_ID);
        assert_eq!(defaults.joint, JigsawJointType::Rollable);
        assert_eq!(defaults.placement_priority, 0);
        assert_eq!(defaults.selection_priority, 0);
    }

    #[test]
    fn comparator_block_entity_persists_output_and_uses_compare_subtract_logic() {
        let mut compare = ComparatorBlockEntity::new(ComparatorMode::Compare);
        assert_eq!(
            compare.save_additional(),
            Tag::Compound(vec![("OutputSignal".to_string(), Tag::Int(0))])
        );
        assert!(compare.update_output(12, 7));
        assert_eq!(compare.output_signal, 12);
        assert!(!compare.update_output(12, 7));
        assert_eq!(compare.calculate_output(3, 10), 0);

        let saved = compare.save_additional();
        assert_eq!(
            ComparatorBlockEntity::load_additional(ComparatorMode::Compare, &saved),
            compare
        );
        assert_eq!(
            ComparatorBlockEntity::load_additional(
                ComparatorMode::Subtract,
                &Tag::Compound(vec![("OutputSignal".to_string(), Tag::Int(99))]),
            )
            .output_signal,
            99
        );

        let mut subtract = ComparatorBlockEntity::new(ComparatorMode::Subtract);
        assert!(subtract.update_output(12, 7));
        assert_eq!(subtract.output_signal, 5);
        assert!(subtract.update_output(3, 10));
        assert_eq!(subtract.output_signal, 0);
    }

    #[test]
    fn daylight_detector_updates_power_with_vanilla_solar_math_and_tick_cadence() {
        let mut normal = DaylightDetectorBlockEntity::new(false);
        assert_eq!(normal.save_additional(), Tag::Compound(Vec::new()));
        assert!(!normal.tick(19, 15, 0.0));
        assert_eq!(normal.power, 0);
        assert!(normal.tick(20, 15, 0.0));
        assert_eq!(normal.power, 15);
        assert!(normal.update_signal(15, 180.0));
        assert_eq!(normal.power, 0);
        assert!(!normal.update_signal(-4, 0.0));
        assert_eq!(normal.power, 0);

        assert_eq!(
            DaylightDetectorBlockEntity::calculate_power(false, 10, 90.0),
            3
        );
        assert_eq!(
            DaylightDetectorBlockEntity::calculate_power(false, 99, 0.0),
            15
        );

        let mut inverted = DaylightDetectorBlockEntity::new(true);
        assert!(inverted.update_signal(4, 90.0));
        assert_eq!(inverted.power, 11);
        assert!(inverted.update_signal(99, 0.0));
        assert_eq!(inverted.power, 0);
    }

    #[test]
    fn command_block_entity_persists_base_fields_and_models_execution_gate() {
        let mut command = CommandBlockEntity::new(CommandBlockMode::Redstone, true);
        command.set_command("say hello");
        command.custom_name = Some("\"Runner\"".to_string());
        command.track_output = true;
        command.last_output = Some("\"previous\"".to_string());
        command.last_execution = 41;
        command.powered = true;
        command.automatic = true;
        command.mark_condition_met(true);

        let saved = command.save_additional();
        assert_eq!(
            CommandBlockEntity::load_additional(CommandBlockMode::Redstone, true, &saved),
            command
        );
        assert_eq!(
            command.execution_action(true, true),
            SpecialBlockAction::ExecuteCommand { success_count: 1 }
        );
        assert_eq!(command.perform_command(42, true, true, true), true);
        assert_eq!(command.success_count, 1);
        assert_eq!(command.last_execution, 42);
        assert_eq!(command.perform_command(42, true, true, true), false);

        let mut blocked = CommandBlockEntity::new(CommandBlockMode::Auto, false);
        blocked.set_command("say no");
        assert_eq!(
            blocked.execution_action(false, true),
            SpecialBlockAction::ExecuteCommand { success_count: 0 }
        );
        assert!(blocked.set_automatic(true, true));
        assert!(!blocked.set_automatic(true, true));
        assert!(blocked.can_use(true));
        assert!(!blocked.can_use(false));

        let mut searge = CommandBlockEntity::new(CommandBlockMode::Auto, false);
        searge.set_command("Searge");
        assert!(searge.perform_command(9, false, false, false));
        assert_eq!(searge.success_count, 1);
        assert_eq!(
            searge.last_output.as_deref(),
            Some(CommandBlockEntity::SEARGE_OUTPUT)
        );

        let loaded_without_tracking = CommandBlockEntity::load_additional(
            CommandBlockMode::Auto,
            false,
            &Tag::Compound(vec![
                ("TrackOutput".to_string(), Tag::Byte(0)),
                (
                    "LastOutput".to_string(),
                    Tag::String("\"ignored\"".to_string()),
                ),
                ("UpdateLastExecution".to_string(), Tag::Byte(0)),
                ("LastExecution".to_string(), Tag::Long(99)),
            ]),
        );
        assert_eq!(loaded_without_tracking.last_output, None);
        assert_eq!(
            loaded_without_tracking.last_execution,
            CommandBlockEntity::NO_LAST_EXECUTION
        );
    }

    #[test]
    fn jukebox_block_entity_tracks_disc_playback_ticks_and_outputs() {
        let mut jukebox = JukeboxBlockEntity::new();
        let disc = PotItemStack {
            item_id: "minecraft:music_disc_13".to_string(),
            count: 1,
        };
        assert!(jukebox.can_place_item(&disc));
        assert_eq!(
            jukebox.set_the_item(Some(disc.clone())),
            JukeboxSongEvent::Started
        );
        assert!(jukebox.is_playing);
        assert_eq!(jukebox.redstone_signal(), 15);
        assert_eq!(jukebox.comparator_output(), 1);
        assert!(jukebox.tick());
        assert_eq!(jukebox.ticks_since_song_started, 1);

        let saved = jukebox.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                ("RecordItem".to_string(), disc.to_tag()),
                ("ticks_since_song_started".to_string(), Tag::Long(1)),
            ])
        );
        let loaded = JukeboxBlockEntity::load_additional(&saved);
        assert_eq!(loaded.item, Some(disc.clone()));
        assert_eq!(loaded.ticks_since_song_started, 1);
        assert!(!loaded.is_playing);
        assert_eq!(loaded.comparator_output(), 1);

        assert!(!jukebox.can_place_item(&PotItemStack {
            item_id: "minecraft:music_disc_5".to_string(),
            count: 1,
        }));
        assert_eq!(jukebox.pop_out_the_item(), Some(disc));
        assert!(!jukebox.is_playing);
        assert_eq!(jukebox.redstone_signal(), 0);
        assert_eq!(jukebox.comparator_output(), 0);
        assert_eq!(jukebox.save_additional(), Tag::Compound(Vec::new()));

        let mut inert = JukeboxBlockEntity::new();
        assert_eq!(
            inert.set_the_item(Some(PotItemStack {
                item_id: "minecraft:diamond".to_string(),
                count: 1,
            })),
            JukeboxSongEvent::Stopped
        );
        assert!(!inert.tick());
        assert_eq!(inert.comparator_output(), 0);
        assert!(!inert.can_place_item(&PotItemStack {
            item_id: "minecraft:diamond".to_string(),
            count: 1,
        }));
        assert!(inert.can_take_item(true));
        assert!(!inert.can_take_item(false));

        let mut without_playing = JukeboxBlockEntity::new();
        assert_eq!(
            without_playing.set_song_item_without_playing(PotItemStack {
                item_id: "minecraft:music_disc_5".to_string(),
                count: 1,
            }),
            JukeboxSongEvent::ItemChanged
        );
        assert!(!without_playing.is_playing);
        assert_eq!(without_playing.redstone_signal(), 0);
        assert_eq!(without_playing.comparator_output(), 15);
    }

    #[test]
    fn enchanting_table_saves_name_scans_bookshelves_and_animates_book_like_java() {
        let mut table = EnchantingTableBlockEntity::new();
        assert_eq!(
            table.display_name(),
            EnchantingTableBlockEntity::DEFAULT_NAME
        );
        table.custom_name = Some("\"Arcana\"".to_string());
        let saved = table.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![(
                "CustomName".to_string(),
                Tag::String("\"Arcana\"".to_string())
            )])
        );
        assert_eq!(
            EnchantingTableBlockEntity::load_additional(&saved).custom_name,
            Some("\"Arcana\"".to_string())
        );
        assert_eq!(table.get_update_tag(), saved);

        let offsets = EnchantingTableBlockEntity::bookshelf_offsets();
        assert_eq!(offsets.len(), 32);
        assert!(offsets.contains(&BlockPos { x: -2, y: 0, z: 0 }));
        assert!(offsets.contains(&BlockPos { x: 2, y: 1, z: 2 }));
        let power = |pos: BlockPos| pos.x == 2 || pos.z == -2;
        let transmit = |_pos: BlockPos| true;
        assert_eq!(
            EnchantingTableBlockEntity::count_valid_bookshelves(power, transmit),
            15
        );

        table.book_animation_tick(Some((1.0, 0.0)), Some(2.0));
        assert_eq!(table.time, 1);
        assert_eq!(table.o_open, 0.0);
        assert_eq!(table.open, 0.1);
        assert_eq!(table.t_rot, 0.0);
        assert!(table.flip > 0.0);
        let previous_flip = table.flip;
        table.book_animation_tick(None, None);
        assert_eq!(table.time, 2);
        assert_eq!(table.o_open, 0.1);
        assert_eq!(table.open, 0.0);
        assert!(table.t_rot > 0.0);
        assert!(table.flip >= previous_flip);

        table.rot = std::f32::consts::TAU;
        table.t_rot = -std::f32::consts::TAU;
        table.book_animation_tick(None, None);
        assert!(table.rot < std::f32::consts::PI);
        assert!(table.t_rot > -std::f32::consts::PI);
    }

    #[test]
    fn shelf_block_entity_saves_three_items_align_flag_and_swaps_slots() {
        let mut shelf = ShelfBlockEntity::new();
        assert_eq!(shelf.items.len(), ShelfBlockEntity::MAX_ITEMS);
        assert_eq!(shelf.comparator_output(), 0);
        assert!(shelf.set_item_no_update(
            0,
            Some(PotItemStack {
                item_id: "minecraft:book".to_string(),
                count: 1,
            })
        ));
        assert!(shelf.set_item_no_update(
            2,
            Some(PotItemStack {
                item_id: "minecraft:diamond".to_string(),
                count: 3,
            })
        ));
        assert!(!shelf.set_item_no_update(
            3,
            Some(PotItemStack {
                item_id: "minecraft:apple".to_string(),
                count: 1,
            })
        ));
        shelf.align_items_to_bottom = true;
        assert_eq!(shelf.filled_slot_count(), 2);
        assert_eq!(shelf.comparator_output(), 2);

        let saved = shelf.save_additional();
        assert_eq!(ShelfBlockEntity::load_additional(&saved), shelf);
        assert_eq!(shelf.get_update_tag(), saved);
        assert_eq!(
            shelf.swap_item_no_update(
                0,
                Some(PotItemStack {
                    item_id: "minecraft:stick".to_string(),
                    count: 4,
                })
            ),
            Some(PotItemStack {
                item_id: "minecraft:book".to_string(),
                count: 1,
            })
        );
        assert_eq!(
            shelf.get_item(0),
            Some(&PotItemStack {
                item_id: "minecraft:stick".to_string(),
                count: 4,
            })
        );
        assert_eq!(
            shelf.remove_item_no_update(2),
            Some(PotItemStack {
                item_id: "minecraft:diamond".to_string(),
                count: 3,
            })
        );
        assert_eq!(shelf.comparator_output(), 1);

        let out_of_range = Tag::Compound(vec![(
            "Items".to_string(),
            Tag::List(vec![Tag::Compound(vec![
                ("id".to_string(), Tag::String("minecraft:apple".to_string())),
                ("count".to_string(), Tag::Int(1)),
                ("Slot".to_string(), Tag::Byte(9)),
            ])]),
        )]);
        assert_eq!(
            ShelfBlockEntity::load_additional(&out_of_range),
            ShelfBlockEntity::new()
        );
    }

    #[test]
    fn beacon_tier_effect_payment_and_beam_state_match_vanilla_rules() {
        let pos = BlockPos { x: 0, y: 64, z: 0 };
        let full_four_tier = |block: BlockPos| {
            let dy = pos.y - block.y;
            (1..=4).contains(&dy) && block.x.abs() <= dy && block.z.abs() <= dy
        };
        assert_eq!(BeaconBlockEntity::update_base(pos, 0, full_four_tier), 4);
        let broken_second_tier = |block: BlockPos| {
            let dy = pos.y - block.y;
            (1..=4).contains(&dy)
                && block.x.abs() <= dy
                && block.z.abs() <= dy
                && !(dy == 2 && block.x == 2 && block.z == 0)
        };
        assert_eq!(
            BeaconBlockEntity::update_base(pos, 0, broken_second_tier),
            1
        );
        assert_eq!(
            BeaconBlockEntity::update_base(BlockPos { x: 0, y: 1, z: 0 }, 1, |_| true),
            0
        );

        let mut beacon = BeaconBlockEntity::new();
        beacon.levels = 4;
        beacon.set_primary_power(Some("minecraft:speed"));
        beacon.set_secondary_power(Some("minecraft:speed"));
        assert_eq!(beacon.comparator_output(), 4);
        assert_eq!(
            beacon.effect_applications(),
            vec![BeaconEffectApplication {
                effect: "minecraft:speed".to_string(),
                duration_ticks: 340,
                amplifier: 1,
                range: 50,
            }]
        );
        beacon.set_secondary_power(Some("minecraft:regeneration"));
        assert_eq!(
            beacon.effect_applications(),
            vec![
                BeaconEffectApplication {
                    effect: "minecraft:speed".to_string(),
                    duration_ticks: 340,
                    amplifier: 0,
                    range: 50,
                },
                BeaconEffectApplication {
                    effect: "minecraft:regeneration".to_string(),
                    duration_ticks: 340,
                    amplifier: 0,
                    range: 50,
                },
            ]
        );

        beacon.set_primary_power(Some("minecraft:night_vision"));
        assert_eq!(beacon.primary_power, None);
        assert!(BeaconBlockEntity::can_pay_with(&PotItemStack {
            item_id: "minecraft:amethyst_shard".to_string(),
            count: 1,
        }));
        assert!(!beacon.set_payment_item(Some(PotItemStack {
            item_id: "minecraft:apple".to_string(),
            count: 1,
        })));
        assert!(beacon.set_payment_item(Some(PotItemStack {
            item_id: "minecraft:emerald".to_string(),
            count: 1,
        })));

        beacon.primary_power = Some("minecraft:haste".to_string());
        beacon.secondary_power = Some("minecraft:regeneration".to_string());
        beacon.custom_name = Some("\"Beacon\"".to_string());
        beacon.lock_key = Some("secret".to_string());
        let saved = beacon.save_additional();
        let loaded = BeaconBlockEntity::load_additional(&saved);
        assert_eq!(loaded.levels, 4);
        assert_eq!(loaded.primary_power.as_deref(), Some("minecraft:haste"));
        assert_eq!(
            loaded.secondary_power.as_deref(),
            Some("minecraft:regeneration")
        );
        assert_eq!(loaded.display_name(), "\"Beacon\"");
        assert_eq!(loaded.lock_key.as_deref(), Some("secret"));
        assert_eq!(loaded.get_update_tag(), loaded.save_additional());

        let sections = BeaconBlockEntity::scan_beam([
            BeaconBeamBlock::TintedGlass(0xFFFF_0000u32 as i32),
            BeaconBeamBlock::TintedGlass(0xFFFF_0000u32 as i32),
            BeaconBeamBlock::TintedGlass(0xFF00_00FFu32 as i32),
            BeaconBeamBlock::Transparent,
        ]);
        assert_eq!(
            sections,
            vec![
                BeaconBeamSection {
                    color: 0xFFFF_0000u32 as i32,
                    height: 1,
                },
                BeaconBeamSection {
                    color: 0xFFFF_0000u32 as i32,
                    height: 1,
                },
                BeaconBeamSection {
                    color: 0xFF7F_007Fu32 as i32,
                    height: 2,
                },
            ]
        );
        assert_eq!(
            BeaconBlockEntity::scan_beam([BeaconBeamBlock::Blocking]),
            Vec::<BeaconBeamSection>::new()
        );
    }

    #[test]
    fn banner_block_entity_tracks_color_patterns_and_update_tag_shape() {
        let mut banner =
            BannerBlockEntity::from_block_state("minecraft:light_blue_wall_banner").unwrap();
        assert_eq!(banner.base_color, DyeColor::LightBlue);
        assert!(banner.add_pattern("minecraft:stripe_bottom", DyeColor::Red));
        assert!(banner.add_pattern("minecraft:flower", DyeColor::Yellow));
        assert!(banner.add_pattern("minecraft:creeper", DyeColor::Green));
        assert!(banner.add_pattern("minecraft:skull", DyeColor::Black));
        assert!(banner.add_pattern("minecraft:mojang", DyeColor::Purple));
        assert!(banner.add_pattern("minecraft:globe", DyeColor::White));
        assert!(!banner.add_pattern("minecraft:extra", DyeColor::Orange));
        banner.custom_name = Some("{\"text\":\"Marker\"}".to_string());

        let saved = banner.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                (
                    "patterns".to_string(),
                    Tag::List(vec![
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:stripe_bottom".to_string())
                            ),
                            ("color".to_string(), Tag::String("red".to_string())),
                        ]),
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:flower".to_string())
                            ),
                            ("color".to_string(), Tag::String("yellow".to_string())),
                        ]),
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:creeper".to_string())
                            ),
                            ("color".to_string(), Tag::String("green".to_string())),
                        ]),
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:skull".to_string())
                            ),
                            ("color".to_string(), Tag::String("black".to_string())),
                        ]),
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:mojang".to_string())
                            ),
                            ("color".to_string(), Tag::String("purple".to_string())),
                        ]),
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:globe".to_string())
                            ),
                            ("color".to_string(), Tag::String("white".to_string())),
                        ]),
                    ])
                ),
                (
                    "CustomName".to_string(),
                    Tag::String("{\"text\":\"Marker\"}".to_string())
                ),
            ])
        );

        let loaded =
            BannerBlockEntity::load_additional("minecraft:light_blue_wall_banner", &saved).unwrap();
        assert_eq!(loaded, banner);
        assert_eq!(BannerBlockEntity::from_block_state("minecraft:stone"), None);

        let mut entity = BlockEntity::new(
            BlockEntityTypeId::Banner,
            pos(),
            "minecraft:light_blue_banner",
        )
        .unwrap();
        entity.custom_data.insert(
            "patterns".to_string(),
            match saved {
                Tag::Compound(fields) => fields
                    .into_iter()
                    .find(|(key, _)| key == "patterns")
                    .map(|(_, value)| value)
                    .unwrap(),
                _ => unreachable!(),
            },
        );
        assert!(
            matches!(entity.get_update_tag(), Tag::Compound(fields) if fields.iter().any(|(key, _)| key == "patterns") && fields.iter().all(|(key, _)| key != "id"))
        );
    }

    #[test]
    fn banner_pattern_layers_preserve_order_and_enforce_six_layer_cap() {
        let mut banner = BannerBlockEntity::from_block_state("minecraft:red_banner").unwrap();
        let layers = [
            ("minecraft:stripe_bottom", DyeColor::White),
            ("minecraft:stripe_top", DyeColor::Black),
            ("minecraft:stripe_left", DyeColor::Blue),
            ("minecraft:stripe_right", DyeColor::Yellow),
            ("minecraft:diagonal_left", DyeColor::Green),
            ("minecraft:diagonal_right", DyeColor::Purple),
        ];
        for (pattern, color) in layers {
            assert!(banner.add_pattern(pattern, color));
        }
        assert!(!banner.add_pattern("minecraft:globe", DyeColor::Cyan));
        banner.custom_name = Some("{\"text\":\"Six Layers\"}".to_string());

        let saved = banner.save_additional();
        let Tag::Compound(fields) = &saved else {
            panic!("banner save_additional should produce a compound");
        };
        assert_eq!(fields[0].0, "patterns");
        assert_eq!(fields[1].0, "CustomName");

        let Tag::List(saved_layers) = &fields[0].1 else {
            panic!("patterns should be a list");
        };
        assert_eq!(saved_layers.len(), BannerBlockEntity::MAX_PATTERNS);
        for (idx, (expected_pattern, expected_color)) in layers.iter().enumerate() {
            assert_eq!(
                saved_layers[idx],
                Tag::Compound(vec![
                    (
                        "pattern".to_string(),
                        Tag::String((*expected_pattern).to_string())
                    ),
                    (
                        "color".to_string(),
                        Tag::String(expected_color.vanilla_name().to_string())
                    ),
                ])
            );
        }

        let loaded = BannerBlockEntity::load_additional("minecraft:red_wall_banner", &saved)
            .expect("saved red banner should load");
        assert_eq!(loaded.base_color, DyeColor::Red);
        assert_eq!(loaded.patterns, banner.patterns);
        assert_eq!(loaded.custom_name, banner.custom_name);

        let overlong = Tag::Compound(vec![(
            "patterns".to_string(),
            Tag::List(
                (0..8)
                    .map(|idx| {
                        BannerPatternLayer {
                            pattern: format!("minecraft:test_{idx}"),
                            color: DyeColor::White,
                        }
                        .to_tag()
                    })
                    .collect(),
            ),
        )]);
        let truncated =
            BannerBlockEntity::load_additional("minecraft:white_banner", &overlong).unwrap();
        assert_eq!(truncated.patterns.len(), BannerBlockEntity::MAX_PATTERNS);
        assert_eq!(truncated.patterns[5].pattern, "minecraft:test_5");
    }

    #[test]
    fn decorated_pot_saves_sherds_item_loot_and_wobble_like_java() {
        let mut pot = DecoratedPotBlockEntity {
            decorations: PotDecorations::new(
                Some("minecraft:angler_pottery_sherd".to_string()),
                None,
                Some("minecraft:arms_up_pottery_sherd".to_string()),
                Some("minecraft:brick".to_string()),
            ),
            item: Some(PotItemStack {
                item_id: "minecraft:diamond".to_string(),
                count: 2,
            }),
            ..DecoratedPotBlockEntity::default()
        };

        assert_eq!(
            pot.decorations.ordered(),
            vec![
                "minecraft:angler_pottery_sherd".to_string(),
                "minecraft:brick".to_string(),
                "minecraft:arms_up_pottery_sherd".to_string(),
                "minecraft:brick".to_string(),
            ]
        );
        let saved = pot.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                (
                    "sherds".to_string(),
                    Tag::List(vec![
                        Tag::String("minecraft:angler_pottery_sherd".to_string()),
                        Tag::String("minecraft:brick".to_string()),
                        Tag::String("minecraft:arms_up_pottery_sherd".to_string()),
                        Tag::String("minecraft:brick".to_string()),
                    ]),
                ),
                (
                    "item".to_string(),
                    Tag::Compound(vec![
                        (
                            "id".to_string(),
                            Tag::String("minecraft:diamond".to_string())
                        ),
                        ("count".to_string(), Tag::Int(2)),
                    ]),
                ),
            ])
        );
        assert_eq!(DecoratedPotBlockEntity::load_additional(&saved), pot);

        pot.loot_table = Some("minecraft:chests/trial_chambers/reward".to_string());
        pot.loot_table_seed = 123;
        let loot_saved = pot.save_additional();
        assert!(
            matches!(&loot_saved, Tag::Compound(fields) if fields.iter().any(|(key, _)| key == "LootTable") && fields.iter().all(|(key, _)| key != "item"))
        );
        let loaded_loot = DecoratedPotBlockEntity::load_additional(&loot_saved);
        assert_eq!(loaded_loot.loot_table, pot.loot_table);
        assert_eq!(loaded_loot.loot_table_seed, 123);
        assert_eq!(loaded_loot.item, None);

        assert_eq!(DecoratedPotWobbleStyle::Positive.duration(), 7);
        assert_eq!(DecoratedPotWobbleStyle::Negative.duration(), 10);
        assert!(pot.trigger_event(
            DecoratedPotBlockEntity::EVENT_POT_WOBBLES,
            DecoratedPotWobbleStyle::Negative.id(),
            42,
        ));
        assert_eq!(pot.wobble_started_at_tick, 42);
        assert_eq!(
            pot.last_wobble_style,
            Some(DecoratedPotWobbleStyle::Negative)
        );
        assert!(!pot.trigger_event(99, DecoratedPotWobbleStyle::Positive.id(), 43));
        assert!(!pot.trigger_event(DecoratedPotBlockEntity::EVENT_POT_WOBBLES, 99, 43));
    }

    #[test]
    fn decorated_pot_wobble_and_destruction_drops_preserve_sherds_and_item() {
        let mut pot = DecoratedPotBlockEntity {
            decorations: PotDecorations::new(
                Some("minecraft:arms_up_pottery_sherd".to_string()),
                Some("minecraft:blade_pottery_sherd".to_string()),
                None,
                Some("minecraft:brewer_pottery_sherd".to_string()),
            ),
            ..DecoratedPotBlockEntity::default()
        };

        pot.item = Some(PotItemStack {
            item_id: "minecraft:emerald".to_string(),
            count: 3,
        });
        assert!(pot.trigger_event(
            DecoratedPotBlockEntity::EVENT_POT_WOBBLES,
            DecoratedPotWobbleStyle::Positive.id(),
            2400,
        ));
        assert_eq!(pot.wobble_started_at_tick, 2400);
        assert_eq!(
            pot.last_wobble_style,
            Some(DecoratedPotWobbleStyle::Positive)
        );
        assert_eq!(DecoratedPotWobbleStyle::Positive.duration(), 7);

        let drops = pot.destruction_drops();
        assert_eq!(
            drops.decoration_items,
            vec![
                "minecraft:arms_up_pottery_sherd".to_string(),
                "minecraft:blade_pottery_sherd".to_string(),
                "minecraft:brick".to_string(),
                "minecraft:brewer_pottery_sherd".to_string(),
            ]
        );
        assert_eq!(
            drops.stored_item,
            Some(PotItemStack {
                item_id: "minecraft:emerald".to_string(),
                count: 3,
            })
        );

        pot.item = Some(PotItemStack {
            item_id: "minecraft:air".to_string(),
            count: 0,
        });
        assert_eq!(pot.destruction_drops().stored_item, None);
    }

    #[test]
    fn brushable_block_entity_brushes_resets_loot_and_update_tag_like_java() {
        assert_eq!(BrushableBlockEntity::BRUSH_COOLDOWN_TICKS, 10);
        assert_eq!(BrushableBlockEntity::BRUSH_RESET_TICKS, 40);
        assert_eq!(BrushableBlockEntity::REQUIRED_BRUSHES_TO_BREAK, 10);

        let mut brushable = BrushableBlockEntity::new();
        brushable.set_loot_table("minecraft:archaeology/desert_pyramid", 99);
        assert_eq!(
            brushable.save_additional(),
            Tag::Compound(vec![
                (
                    "LootTable".to_string(),
                    Tag::String("minecraft:archaeology/desert_pyramid".to_string())
                ),
                ("LootTableSeed".to_string(), Tag::Long(99)),
            ])
        );

        let generated_item = PotItemStack {
            item_id: "minecraft:diamond".to_string(),
            count: 1,
        };
        assert_eq!(
            brushable.brush(100, Direction::North, Some(generated_item.clone())),
            BrushResult::InProgress { dusted: 1 }
        );
        assert_eq!(brushable.hit_direction, Some(Direction::North));
        assert_eq!(brushable.brush_count, 1);
        assert_eq!(brushable.brush_count_resets_at_tick, 140);
        assert_eq!(brushable.cooldown_ends_at_tick, 110);
        assert_eq!(brushable.item, Some(generated_item.clone()));
        assert_eq!(brushable.loot_table, None);
        assert_eq!(
            brushable.get_update_tag(),
            Tag::Compound(vec![
                (
                    "hit_direction".to_string(),
                    Tag::String("north".to_string())
                ),
                ("item".to_string(), generated_item.to_tag()),
            ])
        );
        assert_eq!(
            brushable.brush(105, Direction::South, None),
            BrushResult::CoolingDown
        );
        assert_eq!(brushable.hit_direction, Some(Direction::North));

        assert_eq!(
            brushable.brush(110, Direction::South, None),
            BrushResult::InProgress { dusted: 1 }
        );
        assert_eq!(
            brushable.brush(120, Direction::South, None),
            BrushResult::InProgress { dusted: 2 }
        );
        assert_eq!(
            brushable.brush(130, Direction::South, None),
            BrushResult::InProgress { dusted: 2 }
        );
        assert_eq!(
            brushable.brush(140, Direction::South, None),
            BrushResult::InProgress { dusted: 2 }
        );
        assert_eq!(
            brushable.brush(150, Direction::South, None),
            BrushResult::InProgress { dusted: 3 }
        );
        assert_eq!(brushable.brush_count, 6);

        assert_eq!(brushable.check_reset(189), None);
        assert_eq!(brushable.check_reset(190), Some(2));
        assert_eq!(brushable.brush_count, 4);
        assert_eq!(brushable.brush_count_resets_at_tick, 194);
        assert_eq!(brushable.check_reset(194), Some(1));
        assert_eq!(brushable.brush_count, 2);
        assert_eq!(brushable.check_reset(198), Some(0));
        assert_eq!(brushable.brush_count, 0);
        assert_eq!(brushable.hit_direction, None);
        assert_eq!(brushable.cooldown_ends_at_tick, 0);

        brushable.hit_direction = Some(Direction::East);
        brushable.item = Some(generated_item.clone());
        let saved_item = brushable.save_additional();
        assert_eq!(
            BrushableBlockEntity::load_additional(&saved_item),
            brushable
        );
        assert_eq!(
            brushable.drop_content(),
            Some((generated_item, Direction::East))
        );
        assert_eq!(brushable.item, None);

        let mut completing = BrushableBlockEntity::new();
        for step in 0..9 {
            assert!(matches!(
                completing.brush(step * 10, Direction::Up, None),
                BrushResult::InProgress { .. }
            ));
        }
        assert_eq!(
            completing.brush(90, Direction::Up, None),
            BrushResult::Completed
        );
        assert_eq!(completing.brush_count, 10);
    }

    #[test]
    fn copper_golem_statue_tracks_weather_pose_comparator_and_clone_components() {
        let mut statue = CopperGolemStatueBlockEntity::from_block_state(
            "minecraft:waxed_weathered_copper_golem_statue",
            CopperGolemStatuePose::Standing,
        )
        .unwrap();
        assert_eq!(statue.weather_state, CopperWeatherState::Weathered);
        assert_eq!(statue.weather_state.serialized_name(), "weathered");
        assert!(statue.waxed);
        assert_eq!(statue.comparator_output(), 1);

        statue.update_pose();
        assert_eq!(statue.pose, CopperGolemStatuePose::Sitting);
        assert_eq!(statue.comparator_output(), 2);
        statue.update_pose();
        assert_eq!(statue.pose, CopperGolemStatuePose::Running);
        assert_eq!(statue.comparator_output(), 3);
        statue.update_pose();
        assert_eq!(statue.pose, CopperGolemStatuePose::Star);
        assert_eq!(statue.comparator_output(), 4);
        statue.update_pose();
        assert_eq!(statue.pose, CopperGolemStatuePose::Standing);

        statue.custom_name = Some("{\"text\":\"Copper Buddy\"}".to_string());
        assert_eq!(
            statue.save_additional(),
            Tag::Compound(vec![(
                "CustomName".to_string(),
                Tag::String("{\"text\":\"Copper Buddy\"}".to_string())
            )])
        );
        assert_eq!(
            statue.clone_item_components(),
            Tag::Compound(vec![
                (
                    "minecraft:block_state".to_string(),
                    Tag::Compound(vec![(
                        "copper_golem_pose".to_string(),
                        Tag::String("standing".to_string())
                    )])
                ),
                (
                    "minecraft:custom_name".to_string(),
                    Tag::String("{\"text\":\"Copper Buddy\"}".to_string())
                ),
            ])
        );

        let oxidized = CopperGolemStatueBlockEntity::from_block_state(
            "minecraft:oxidized_copper_golem_statue",
            CopperGolemStatuePose::Star,
        )
        .unwrap();
        assert_eq!(oxidized.weather_state, CopperWeatherState::Oxidized);
        assert!(!oxidized.waxed);
        assert_eq!(oxidized.comparator_output(), 4);
        assert_eq!(
            CopperGolemStatueBlockEntity::from_block_state(
                "minecraft:copper_block",
                CopperGolemStatuePose::Standing,
            ),
            None
        );
    }

    #[test]
    fn skull_block_entity_saves_profile_components_and_animation_like_java() {
        let profile = Tag::Compound(vec![
            ("name".to_string(), Tag::String("Steve".to_string())),
            (
                "id".to_string(),
                Tag::String("8667ba71-b85a-4004-af54-457a9734eed7".to_string()),
            ),
        ]);
        let mut skull = SkullBlockEntity::new();
        skull.profile = Some(profile.clone());
        skull.note_block_sound = Some("minecraft:block.note_block.basedrum".to_string());
        skull.custom_name = Some("{\"text\":\"Head\"}".to_string());

        let saved = skull.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                ("profile".to_string(), profile.clone()),
                (
                    "note_block_sound".to_string(),
                    Tag::String("minecraft:block.note_block.basedrum".to_string())
                ),
                (
                    "custom_name".to_string(),
                    Tag::String("{\"text\":\"Head\"}".to_string())
                ),
            ])
        );
        assert_eq!(SkullBlockEntity::load_additional(&saved), skull);
        assert_eq!(skull.get_update_tag(), saved);

        skull.animation_tick(true);
        skull.animation_tick(true);
        assert!(skull.is_animating);
        assert_eq!(skull.animation_tick_count, 2);
        assert_eq!(skull.animation(0.5), 2.5);
        skull.animation_tick(false);
        assert!(!skull.is_animating);
        assert_eq!(skull.animation(0.5), 2.0);

        let mut tag_with_components = saved.clone();
        SkullBlockEntity::remove_components_from_tag(&mut tag_with_components);
        assert_eq!(tag_with_components, Tag::Compound(vec![]));

        let mut from_components = SkullBlockEntity::new();
        from_components.apply_implicit_components(&BTreeMap::from([
            ("minecraft:profile".to_string(), profile.clone()),
            (
                "minecraft:note_block_sound".to_string(),
                Tag::String("minecraft:block.note_block.harp".to_string()),
            ),
            (
                "minecraft:custom_name".to_string(),
                Tag::String("{\"text\":\"Component Head\"}".to_string()),
            ),
        ]));
        assert_eq!(from_components.profile, Some(profile.clone()));
        assert_eq!(
            from_components.note_block_sound,
            Some("minecraft:block.note_block.harp".to_string())
        );
        assert_eq!(
            from_components.custom_name,
            Some("{\"text\":\"Component Head\"}".to_string())
        );
        assert_eq!(
            from_components.collect_implicit_components(),
            BTreeMap::from([
                ("minecraft:profile".to_string(), profile),
                (
                    "minecraft:note_block_sound".to_string(),
                    Tag::String("minecraft:block.note_block.harp".to_string())
                ),
                (
                    "minecraft:custom_name".to_string(),
                    Tag::String("{\"text\":\"Component Head\"}".to_string())
                ),
            ])
        );
    }

    #[test]
    fn bell_block_entity_tracks_ring_resonation_and_raider_glow_like_java() {
        assert_eq!(BellBlockEntity::DURATION, 50);
        assert_eq!(BellBlockEntity::GLOW_DURATION, 60);
        assert_eq!(BellBlockEntity::MIN_TICKS_BETWEEN_SEARCHES, 60);
        assert_eq!(BellBlockEntity::MAX_RESONATION_TICKS, 40);
        assert_eq!(BellBlockEntity::TICKS_BEFORE_RESONATION, 5);
        assert_eq!(BellBlockEntity::SEARCH_RADIUS, 48.0);
        assert_eq!(BellBlockEntity::HEAR_BELL_RADIUS, 32.0);
        assert_eq!(BellBlockEntity::HIGHLIGHT_RAIDERS_RADIUS, 48.0);

        let mut bell = BellBlockEntity::new();
        let block_event = bell.on_hit(Direction::North);
        assert_eq!(
            block_event,
            BellBlockEvent {
                event_id: BellBlockEntity::EVENT_RING,
                event_param: 2,
            }
        );
        assert!(bell.shaking);
        assert_eq!(bell.click_direction, Some(Direction::North));

        bell.ticks = 12;
        assert_eq!(bell.on_hit(Direction::East).event_param, 5);
        assert_eq!(bell.ticks, 0);
        assert_eq!(bell.click_direction, Some(Direction::East));

        assert!(bell.trigger_event(1, 3, 100, 4, 1, 2));
        assert_eq!(bell.click_direction, Some(Direction::South));
        assert_eq!(bell.last_ring_timestamp, 100);
        assert_eq!(bell.heard_bell_entities, 4);
        assert_eq!(bell.nearby_raiders_within_hear_radius, 1);
        assert_eq!(bell.nearby_raiders_within_highlight_radius, 2);
        assert_eq!(bell.ticks, 0);
        assert!(bell.shaking);
        assert!(!bell.trigger_event(99, 0, 100, 0, 0, 0));

        for _ in 0..4 {
            assert_eq!(
                bell.tick(),
                BellTickEffects {
                    play_resonate_sound: false,
                    glowing_raiders: 0,
                }
            );
        }
        assert_eq!(bell.ticks, 4);
        assert_eq!(
            bell.tick(),
            BellTickEffects {
                play_resonate_sound: true,
                glowing_raiders: 0,
            }
        );
        assert!(bell.resonating);
        assert_eq!(bell.resonation_ticks, 1);

        for _ in 0..39 {
            let effects = bell.tick();
            assert!(!effects.play_resonate_sound);
            assert_eq!(effects.glowing_raiders, 0);
        }
        assert_eq!(bell.resonation_ticks, 40);
        assert_eq!(
            bell.tick(),
            BellTickEffects {
                play_resonate_sound: false,
                glowing_raiders: 2,
            }
        );
        assert!(!bell.resonating);

        while bell.shaking {
            bell.tick();
        }
        assert_eq!(bell.ticks, 0);
        assert_eq!(bell.save_additional(), Tag::Compound(vec![]));
        assert_eq!(bell.get_update_tag(), Tag::Compound(vec![]));

        let mut cached = bell.clone();
        cached.update_entities(120, 7, 3, 5);
        assert_eq!(cached.heard_bell_entities, 4);
        cached.update_entities(161, 7, 3, 5);
        assert_eq!(cached.heard_bell_entities, 7);
        assert_eq!(cached.nearby_raiders_within_highlight_radius, 5);
    }

    #[test]
    fn save_modes_match_metadata_and_custom_data_boundaries() {
        let mut entity =
            BlockEntity::new(BlockEntityTypeId::Sign, pos(), "minecraft:oak_sign").unwrap();
        entity
            .custom_data
            .insert("front_text".to_string(), Tag::String("hello".to_string()));
        entity.components.insert(
            "minecraft:custom_name".to_string(),
            Tag::String("\"Name\"".to_string()),
        );

        assert_eq!(
            entity.save_custom_only(),
            Tag::Compound(vec![(
                "front_text".to_string(),
                Tag::String("hello".to_string())
            )])
        );
        assert!(
            matches!(entity.save_without_metadata(), Tag::Compound(values) if values.iter().any(|(k, _)| k == "components") && values.iter().all(|(k, _)| k != "id"))
        );
        assert!(
            matches!(entity.save_with_id(), Tag::Compound(values) if values.iter().any(|(k, v)| k == "id" && *v == Tag::String("sign".to_string())) && values.iter().all(|(k, _)| k != "x"))
        );
        assert!(
            matches!(entity.save_with_full_metadata(), Tag::Compound(values) if values.iter().any(|(k, v)| k == "x" && *v == Tag::Int(18)))
        );
    }

    #[test]
    fn load_static_reads_id_components_and_custom_payload() {
        let tag = Tag::Compound(vec![
            (
                "id".to_string(),
                Tag::String("minecraft:campfire".to_string()),
            ),
            ("x".to_string(), Tag::Int(18)),
            ("y".to_string(), Tag::Int(64)),
            ("z".to_string(), Tag::Int(35)),
            ("CookingTimes".to_string(), Tag::List(vec![Tag::Int(10)])),
            (
                "components".to_string(),
                Tag::Compound(vec![(
                    "minecraft:lore".to_string(),
                    Tag::String("[]".to_string()),
                )]),
            ),
        ]);
        let entity = load_static(pos(), "minecraft:campfire", &tag).unwrap();
        assert_eq!(entity.ty, BlockEntityTypeId::Campfire);
        assert!(entity.custom_data.contains_key("CookingTimes"));
        assert!(entity.components.contains_key("minecraft:lore"));
    }

    #[test]
    fn load_static_with_data_version_refuses_unsafe_migrations() {
        let tag = Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:campfire".to_string()),
        )]);

        let entity = load_static_with_data_version(
            pos(),
            "minecraft:campfire",
            &tag,
            crate::storage::datafix::TARGET_DATA_VERSION,
        )
        .unwrap();
        assert_eq!(entity.ty, BlockEntityTypeId::Campfire);

        assert!(matches!(
            load_static_with_data_version(
                pos(),
                "minecraft:campfire",
                &tag,
                crate::storage::datafix::TARGET_DATA_VERSION - 1,
            ),
            Err(BlockEntityError::UnsupportedDataVersion(message))
                if message.contains("unsafe migrations")
        ));
    }

    #[test]
    fn save_load_round_trip_preserves_generic_fields_for_every_type() {
        for info in BLOCK_ENTITY_TYPES {
            let block_state = info
                .valid_blocks
                .first()
                .expect("every block entity type has at least one valid block");
            let mut entity = BlockEntity::new(info.id, pos(), block_state).unwrap();
            entity
                .custom_data
                .insert("CustomInt".to_string(), Tag::Int(42));
            entity.components.insert(
                "minecraft:custom_name".to_string(),
                Tag::String("\"Round Trip\"".to_string()),
            );

            let tag = entity.save_with_full_metadata();
            let loaded = load_static(pos(), block_state, &tag).unwrap();

            assert_eq!(loaded.ty, entity.ty, "type failed for {}", info.key);
            assert_eq!(loaded.pos, entity.pos, "position failed for {}", info.key);
            assert_eq!(
                loaded.block_state, entity.block_state,
                "block state failed for {}",
                info.key
            );
            assert_eq!(
                loaded.custom_data, entity.custom_data,
                "custom data failed for {}",
                info.key
            );
            assert_eq!(
                loaded.components, entity.components,
                "components failed for {}",
                info.key
            );
            assert!(
                !loaded.has_level,
                "level attachment leaked for {}",
                info.key
            );
            assert!(!loaded.removed, "removed flag leaked for {}", info.key);
            assert!(!loaded.changed, "changed flag leaked for {}", info.key);
            assert_eq!(loaded.tick_count, 0, "tick count leaked for {}", info.key);
        }
    }

    #[test]
    fn update_tag_subset_is_stable_for_every_type() {
        for info in BLOCK_ENTITY_TYPES {
            let block_state = info
                .valid_blocks
                .first()
                .expect("every block entity type has at least one valid block");
            let mut entity = BlockEntity::new(info.id, pos(), block_state).unwrap();
            entity
                .custom_data
                .insert("CustomInt".to_string(), Tag::Int(42));
            entity.components.insert(
                "minecraft:custom_name".to_string(),
                Tag::String("\"Update Tag\"".to_string()),
            );

            let tag = entity.get_update_tag();
            let entries = compound_entries(&tag).expect("update tag is compound");

            assert!(
                entries
                    .iter()
                    .all(|(key, _)| key != "id" && key != "x" && key != "y" && key != "z"),
                "metadata leaked into update tag for {}",
                info.key
            );

            match info.id {
                BlockEntityTypeId::Chest
                | BlockEntityTypeId::TrappedChest
                | BlockEntityTypeId::Barrel
                | BlockEntityTypeId::Hopper
                | BlockEntityTypeId::Dispenser
                | BlockEntityTypeId::Dropper => {
                    assert!(
                        entries.is_empty(),
                        "container inventory data leaked into update tag for {}",
                        info.key
                    );
                }
                _ => {
                    assert!(
                        entries
                            .iter()
                            .any(|(key, value)| key == "CustomInt" && *value == Tag::Int(42)),
                        "custom data missing from update tag for {}",
                        info.key
                    );
                    assert!(
                        entries.iter().any(|(key, _)| key == "components"),
                        "components missing from update tag for {}",
                        info.key
                    );
                }
            }
        }
    }

    #[test]
    fn wrong_chunk_positions_are_corrected_like_vanilla() {
        let tag = Tag::Compound(vec![
            ("x".to_string(), Tag::Int(34)),
            ("y".to_string(), Tag::Int(-20)),
            ("z".to_string(), Tag::Int(-17)),
        ]);
        assert_eq!(
            corrected_pos_from_chunk(0, 0, &tag),
            BlockPos {
                x: 2,
                y: -20,
                z: 15
            }
        );
    }

    #[test]
    fn ticking_requires_level_side_match_and_not_removed() {
        let mut furnace =
            BlockEntity::new(BlockEntityTypeId::Furnace, pos(), "minecraft:furnace").unwrap();
        assert!(!furnace.tick(false));
        furnace.set_level();
        assert!(furnace.tick(false));
        assert!(!furnace.tick(true));
        furnace.set_removed();
        assert!(!furnace.tick(false));

        let mut conduit =
            BlockEntity::new(BlockEntityTypeId::Conduit, pos(), "minecraft:conduit").unwrap();
        conduit.set_level();
        assert!(conduit.tick(false));
        assert!(conduit.tick(true));
    }

    #[test]
    fn ticking_block_entity_wrapper_exposes_scheduler_shape() {
        let mut furnace =
            BlockEntity::new(BlockEntityTypeId::Furnace, pos(), "minecraft:furnace").unwrap();
        furnace.set_level();
        let mut ticker = TickingBlockEntity::new(furnace, false);

        assert_eq!(ticker.pos(), pos());
        assert_eq!(ticker.type_key(), "furnace");
        assert!(!ticker.is_removed());
        assert!(ticker.tick());
        assert_eq!(ticker.entity.tick_count, 1);

        ticker.entity.set_removed();
        assert!(ticker.is_removed());
        assert!(!ticker.tick());
    }

    #[test]
    fn changed_flag_only_sets_when_attached_to_level() {
        let mut entity =
            BlockEntity::new(BlockEntityTypeId::Bell, pos(), "minecraft:bell").unwrap();
        entity.set_changed();
        assert!(!entity.changed);
        entity.set_level();
        entity.set_changed();
        assert!(entity.changed);
    }

    #[test]
    fn update_packets_use_position_type_and_update_tag() {
        let mut sign =
            BlockEntity::new(BlockEntityTypeId::Sign, pos(), "minecraft:oak_sign").unwrap();
        sign.custom_data
            .insert("front_text".to_string(), Tag::String("hi".to_string()));
        let packet = sign.get_update_packet();
        assert_eq!(packet.pos, pos());
        assert_eq!(packet.ty, BlockEntityTypeId::Sign);
        assert!(
            matches!(packet.tag, Tag::Compound(values) if values.iter().any(|(k, _)| k == "front_text"))
        );

        let chest = BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:chest").unwrap();
        assert_eq!(chest.get_update_tag(), Tag::Compound(Vec::new()));
    }

    #[test]
    fn handle_update_tag_applies_network_subset_without_metadata() {
        let mut entity =
            BlockEntity::new(BlockEntityTypeId::Sign, pos(), "minecraft:oak_sign").unwrap();
        entity
            .custom_data
            .insert("old_text".to_string(), Tag::String("stale".to_string()));
        entity
            .components
            .insert("old_component".to_string(), Tag::Int(1));

        entity.handle_update_tag(&Tag::Compound(vec![
            ("x".to_string(), Tag::Int(999)),
            ("id".to_string(), Tag::String("minecraft:chest".to_string())),
            ("front_text".to_string(), Tag::String("hello".to_string())),
            (
                "components".to_string(),
                Tag::Compound(vec![(
                    "minecraft:custom_name".to_string(),
                    Tag::String("Sign".to_string()),
                )]),
            ),
        ]));

        assert_eq!(entity.ty, BlockEntityTypeId::Sign);
        assert_eq!(entity.pos, pos());
        assert!(!entity.custom_data.contains_key("old_text"));
        assert_eq!(
            entity.custom_data.get("front_text"),
            Some(&Tag::String("hello".to_string()))
        );
        assert!(!entity.components.contains_key("old_component"));
        assert_eq!(
            entity.components.get("minecraft:custom_name"),
            Some(&Tag::String("Sign".to_string()))
        );
    }

    #[test]
    fn test_block_entity_state_saves_loads_and_tracks_triggers_like_java() {
        let mut state = TestBlockEntityState {
            mode: TestBlockMode::Start,
            message: "begin".to_string(),
            powered: false,
            triggered: true,
        };
        assert_eq!(
            state.save_additional(),
            Tag::Compound(vec![
                ("mode".to_string(), Tag::String("start".to_string())),
                ("message".to_string(), Tag::String("begin".to_string())),
                ("powered".to_string(), Tag::Byte(0)),
            ])
        );

        state.trigger();
        assert!(state.powered);
        assert!(state.triggered);
        state.reset();
        assert!(!state.powered);
        assert!(!state.triggered);

        let loaded = TestBlockEntityState::load_additional(&Tag::Compound(vec![
            ("mode".to_string(), Tag::String("accept".to_string())),
            ("message".to_string(), Tag::String("done".to_string())),
            ("powered".to_string(), Tag::Byte(1)),
        ]));
        assert_eq!(loaded.mode, TestBlockMode::Accept);
        assert_eq!(loaded.message, "done");
        assert!(loaded.powered);
        assert!(!loaded.triggered);
        assert_eq!(
            TestBlockEntityState::load_additional(&Tag::Compound(Vec::new())).mode,
            TestBlockMode::Fail
        );
    }

    #[test]
    fn test_instance_block_entity_state_saves_loads_status_and_errors() {
        let mut state = TestInstanceBlockEntityState {
            data: TestInstanceBlockEntityData {
                test: Some("minecraft:always_pass".to_string()),
                size: (3, 4, 5),
                rotation: "clockwise_90".to_string(),
                ignore_entities: true,
                status: TestInstanceStatus::Cleared,
                error_message: None,
            },
            errors: Vec::new(),
        };
        state.set_running();
        state.mark_error(BlockPos { x: 1, y: 2, z: 3 }, "bad block");
        state.set_error_message("failed");

        let saved = state.save_additional();
        let loaded = TestInstanceBlockEntityState::load_additional(&saved);
        assert_eq!(loaded.data.test.as_deref(), Some("minecraft:always_pass"));
        assert_eq!(loaded.data.size, (3, 4, 5));
        assert_eq!(loaded.data.rotation, "clockwise_90");
        assert!(loaded.data.ignore_entities);
        assert_eq!(loaded.data.status, TestInstanceStatus::Finished);
        assert_eq!(loaded.data.error_message.as_deref(), Some("failed"));
        assert_eq!(
            loaded.errors,
            vec![TestInstanceErrorMarker {
                pos: BlockPos { x: 1, y: 2, z: 3 },
                text: "bad block".to_string(),
            }]
        );

        let mut success = loaded.clone();
        success.set_success();
        assert_eq!(success.data.status, TestInstanceStatus::Finished);
        assert_eq!(success.data.error_message, None);
        success.clear_error_markers();
        assert!(success.errors.is_empty());
    }

    #[test]
    fn chunk_packet_data_packs_local_xz_y_type_and_tag() {
        let entity = BlockEntity::new(BlockEntityTypeId::Vault, pos(), "minecraft:vault").unwrap();
        let (packed_xz, y, ty, tag) = block_entity_packet_from_chunk(&entity, -64);
        assert_eq!(packed_xz, 0x23);
        assert_eq!(y, 128);
        assert_eq!(ty, BlockEntityTypeId::Vault);
        assert_eq!(
            tag,
            Tag::Compound(vec![("components".to_string(), Tag::Compound(Vec::new()))])
        );
    }
}
