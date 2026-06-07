use super::*;


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

    pub fn should_render_face(direction: Direction) -> bool {
        matches!(direction, Direction::Down | Direction::Up)
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

impl StructureBlockMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Save => "SAVE",
            Self::Load => "LOAD",
            Self::Corner => "CORNER",
            Self::Data => "DATA",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "SAVE" | "save" => Some(Self::Save),
            "LOAD" | "load" => Some(Self::Load),
            "CORNER" | "corner" => Some(Self::Corner),
            "DATA" | "data" => Some(Self::Data),
            _ => None,
        }
    }
}

impl StructureMirror {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::LeftRight => "LEFT_RIGHT",
            Self::FrontBack => "FRONT_BACK",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "NONE" | "none" => Some(Self::None),
            "LEFT_RIGHT" | "left_right" => Some(Self::LeftRight),
            "FRONT_BACK" | "front_back" => Some(Self::FrontBack),
            _ => None,
        }
    }
}

impl StructureRotation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Clockwise90 => "CLOCKWISE_90",
            Self::Clockwise180 => "CLOCKWISE_180",
            Self::Counterclockwise90 => "COUNTERCLOCKWISE_90",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "NONE" | "none" => Some(Self::None),
            "CLOCKWISE_90" | "clockwise_90" => Some(Self::Clockwise90),
            "CLOCKWISE_180" | "clockwise_180" => Some(Self::Clockwise180),
            "COUNTERCLOCKWISE_90" | "counterclockwise_90" => Some(Self::Counterclockwise90),
            _ => None,
        }
    }
}

impl StructureBlockEntity {
    pub const MAX_OFFSET_PER_AXIS: i32 = 48;
    pub const MAX_SIZE_PER_AXIS: i32 = 48;

    pub fn new(mode: StructureBlockMode) -> Self {
        Self {
            structure_name: None,
            author: String::new(),
            metadata: String::new(),
            structure_pos: BlockPos { x: 0, y: 1, z: 0 },
            structure_size: (0, 0, 0),
            mirror: StructureMirror::None,
            rotation: StructureRotation::None,
            mode,
            ignore_entities: true,
            strict: false,
            powered: false,
            show_air: false,
            show_bounding_box: true,
            integrity: 1.0,
            seed: 0,
        }
    }

    pub fn has_structure_name(&self) -> bool {
        self.structure_name.is_some()
    }

    pub fn structure_name(&self) -> &str {
        self.structure_name.as_deref().unwrap_or("")
    }

    pub fn set_structure_name(&mut self, structure_name: Option<&str>) {
        self.structure_name = structure_name
            .filter(|name| !name.is_empty())
            .map(ToString::to_string);
    }

    pub fn set_structure_pos(&mut self, pos: BlockPos) {
        self.structure_pos = Self::clamp_structure_pos(pos);
    }

    pub fn set_structure_size(&mut self, size: (i32, i32, i32)) {
        self.structure_size = Self::clamp_structure_size(size);
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "name".to_string(),
                Tag::String(self.structure_name().to_string()),
            ),
            ("author".to_string(), Tag::String(self.author.clone())),
            ("metadata".to_string(), Tag::String(self.metadata.clone())),
            ("posX".to_string(), Tag::Int(self.structure_pos.x)),
            ("posY".to_string(), Tag::Int(self.structure_pos.y)),
            ("posZ".to_string(), Tag::Int(self.structure_pos.z)),
            ("sizeX".to_string(), Tag::Int(self.structure_size.0)),
            ("sizeY".to_string(), Tag::Int(self.structure_size.1)),
            ("sizeZ".to_string(), Tag::Int(self.structure_size.2)),
            (
                "rotation".to_string(),
                Tag::String(self.rotation.as_str().to_string()),
            ),
            (
                "mirror".to_string(),
                Tag::String(self.mirror.as_str().to_string()),
            ),
            (
                "mode".to_string(),
                Tag::String(self.mode.as_str().to_string()),
            ),
            (
                "ignoreEntities".to_string(),
                Tag::Byte(i8::from(self.ignore_entities)),
            ),
            ("strict".to_string(), Tag::Byte(i8::from(self.strict))),
            ("powered".to_string(), Tag::Byte(i8::from(self.powered))),
            ("showair".to_string(), Tag::Byte(i8::from(self.show_air))),
            (
                "showboundingbox".to_string(),
                Tag::Byte(i8::from(self.show_bounding_box)),
            ),
            ("integrity".to_string(), Tag::Float(self.integrity)),
            ("seed".to_string(), Tag::Long(self.seed)),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new(StructureBlockMode::Data);
        };
        let mut entity = Self::new(
            get_string(entries, "mode")
                .and_then(StructureBlockMode::from_str)
                .unwrap_or(StructureBlockMode::Data),
        );
        entity.set_structure_name(get_string(entries, "name"));
        entity.author = get_string(entries, "author").unwrap_or("").to_string();
        entity.metadata = get_string(entries, "metadata").unwrap_or("").to_string();
        entity.structure_pos = Self::clamp_structure_pos(BlockPos {
            x: get_int(entries, "posX").unwrap_or(0),
            y: get_int(entries, "posY").unwrap_or(1),
            z: get_int(entries, "posZ").unwrap_or(0),
        });
        entity.structure_size = Self::clamp_structure_size((
            get_int(entries, "sizeX").unwrap_or(0),
            get_int(entries, "sizeY").unwrap_or(0),
            get_int(entries, "sizeZ").unwrap_or(0),
        ));
        entity.rotation = get_string(entries, "rotation")
            .and_then(StructureRotation::from_str)
            .unwrap_or(StructureRotation::None);
        entity.mirror = get_string(entries, "mirror")
            .and_then(StructureMirror::from_str)
            .unwrap_or(StructureMirror::None);
        entity.ignore_entities = get_bool(entries, "ignoreEntities").unwrap_or(true);
        entity.strict = get_bool(entries, "strict").unwrap_or(false);
        entity.powered = get_bool(entries, "powered").unwrap_or(false);
        entity.show_air = get_bool(entries, "showair").unwrap_or(false);
        entity.show_bounding_box = get_bool(entries, "showboundingbox").unwrap_or(true);
        entity.integrity = get_float(entries, "integrity").unwrap_or(1.0);
        entity.seed = get_long(entries, "seed").unwrap_or(0);
        entity
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn render_mode(&self) -> StructureRenderMode {
        if !matches!(
            self.mode,
            StructureBlockMode::Save | StructureBlockMode::Load
        ) {
            StructureRenderMode::None
        } else if self.mode == StructureBlockMode::Save && self.show_air {
            StructureRenderMode::BoxAndInvisibleBlocks
        } else if self.mode != StructureBlockMode::Save && !self.show_bounding_box {
            StructureRenderMode::None
        } else {
            StructureRenderMode::Box
        }
    }

    pub fn renderable_box(&self) -> StructureRenderableBox {
        let x_origin = self.structure_pos.x;
        let z_origin = self.structure_pos.z;
        let y0 = self.structure_pos.y;
        let y1 = y0 + self.structure_size.1;
        let (x_diff, z_diff) = match self.mirror {
            StructureMirror::LeftRight => (self.structure_size.0, -self.structure_size.2),
            StructureMirror::FrontBack => (-self.structure_size.0, self.structure_size.2),
            StructureMirror::None => (self.structure_size.0, self.structure_size.2),
        };
        let (x0, z0, x1, z1) = match self.rotation {
            StructureRotation::Clockwise90 => {
                let x0 = if z_diff < 0 { x_origin } else { x_origin + 1 };
                let z0 = if x_diff < 0 { z_origin + 1 } else { z_origin };
                (x0, z0, x0 - z_diff, z0 + x_diff)
            }
            StructureRotation::Clockwise180 => {
                let x0 = if x_diff < 0 { x_origin } else { x_origin + 1 };
                let z0 = if z_diff < 0 { z_origin } else { z_origin + 1 };
                (x0, z0, x0 - x_diff, z0 - z_diff)
            }
            StructureRotation::Counterclockwise90 => {
                let x0 = if z_diff < 0 { x_origin + 1 } else { x_origin };
                let z0 = if x_diff < 0 { z_origin } else { z_origin + 1 };
                (x0, z0, x0 + z_diff, z0 - x_diff)
            }
            StructureRotation::None => {
                let x0 = if x_diff < 0 { x_origin + 1 } else { x_origin };
                let z0 = if z_diff < 0 { z_origin + 1 } else { z_origin };
                (x0, z0, x0 + x_diff, z0 + z_diff)
            }
        };
        StructureRenderableBox {
            min: BlockPos {
                x: x0.min(x1),
                y: y0.min(y1),
                z: z0.min(z1),
            },
            max: BlockPos {
                x: x0.max(x1),
                y: y0.max(y1),
                z: z0.max(z1),
            },
        }
    }

    pub(super) fn clamp_structure_pos(pos: BlockPos) -> BlockPos {
        BlockPos {
            x: pos
                .x
                .clamp(-Self::MAX_OFFSET_PER_AXIS, Self::MAX_OFFSET_PER_AXIS),
            y: pos
                .y
                .clamp(-Self::MAX_OFFSET_PER_AXIS, Self::MAX_OFFSET_PER_AXIS),
            z: pos
                .z
                .clamp(-Self::MAX_OFFSET_PER_AXIS, Self::MAX_OFFSET_PER_AXIS),
        }
    }

    pub(super) fn clamp_structure_size(size: (i32, i32, i32)) -> (i32, i32, i32) {
        (
            size.0.clamp(0, Self::MAX_SIZE_PER_AXIS),
            size.1.clamp(0, Self::MAX_SIZE_PER_AXIS),
            size.2.clamp(0, Self::MAX_SIZE_PER_AXIS),
        )
    }
}

impl StructureRenderableBox {
    pub fn from_corners(
        x1: i32,
        y1: i32,
        z1: i32,
        x2: i32,
        y2: i32,
        z2: i32,
    ) -> Self {
        Self {
            min: BlockPos {
                x: x1.min(x2),
                y: y1.min(y2),
                z: z1.min(z2),
            },
            max: BlockPos {
                x: x1.max(x2),
                y: y1.max(y2),
                z: z1.max(z2),
            },
        }
    }

    pub fn local_pos(&self) -> BlockPos {
        self.min
    }

    pub fn size(&self) -> (i32, i32, i32) {
        (
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }
}
