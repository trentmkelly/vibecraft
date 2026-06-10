use super::*;

impl JigsawProjectionModel {
    pub fn id(self) -> &'static str {
        match self {
            JigsawProjectionModel::TerrainMatching => "terrain_matching",
            JigsawProjectionModel::Rigid => "rigid",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "terrain_matching" => Some(JigsawProjectionModel::TerrainMatching),
            "rigid" => Some(JigsawProjectionModel::Rigid),
            _ => None,
        }
    }

    pub fn processor_ids(self) -> &'static [&'static str] {
        match self {
            JigsawProjectionModel::TerrainMatching => &["minecraft:gravity"],
            JigsawProjectionModel::Rigid => &[],
        }
    }
}

impl JigsawDirectionModel {
    pub const ALL: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Up => "up",
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "down" => Some(Self::Down),
            "up" => Some(Self::Up),
            "north" => Some(Self::North),
            "south" => Some(Self::South),
            "west" => Some(Self::West),
            "east" => Some(Self::East),
            _ => None,
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    pub fn step(self) -> BlockPos {
        match self {
            Self::Down => BlockPos { x: 0, y: -1, z: 0 },
            Self::Up => BlockPos { x: 0, y: 1, z: 0 },
            Self::North => BlockPos { x: 0, y: 0, z: -1 },
            Self::South => BlockPos { x: 0, y: 0, z: 1 },
            Self::West => BlockPos { x: -1, y: 0, z: 0 },
            Self::East => BlockPos { x: 1, y: 0, z: 0 },
        }
    }
}

impl JigsawJointTypeModel {
    pub fn id(self) -> &'static str {
        match self {
            Self::Rollable => "rollable",
            Self::Aligned => "aligned",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "rollable" => Some(Self::Rollable),
            "aligned" => Some(Self::Aligned),
            _ => None,
        }
    }
}

impl JigsawConnectorModel {
    pub fn target_pos(&self, pos: BlockPos) -> BlockPos {
        let step = self.front.step();
        BlockPos {
            x: pos.x + step.x,
            y: pos.y + step.y,
            z: pos.z + step.z,
        }
    }
}

pub fn jigsaw_connectors_can_attach(
    source: &JigsawConnectorModel,
    target: &JigsawConnectorModel,
) -> bool {
    source.front == target.front.opposite()
        && (source.joint == JigsawJointTypeModel::Rollable || source.top == target.top)
        && source.target == target.name
}

impl<T> SequencedPriorityQueueModel<T> {
    pub fn new() -> Self {
        Self {
            queues_by_priority: BTreeMap::new(),
            highest_priority: None,
        }
    }

    pub fn add(&mut self, data: T, priority: i32) {
        let queue = self.queues_by_priority.entry(priority).or_default();
        queue.push_back(data);
        if self
            .highest_priority
            .is_none_or(|highest| priority >= highest)
        {
            self.highest_priority = Some(priority);
        }
    }

    pub fn next_item(&mut self) -> Option<T> {
        let priority = self.highest_priority?;
        let queue = self.queues_by_priority.get_mut(&priority)?;
        let result = queue.pop_front();
        if queue.is_empty() {
            self.switch_cache_to_next_highest_priority();
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        self.highest_priority.is_none()
    }

    pub fn highest_priority(&self) -> Option<i32> {
        self.highest_priority
    }

    fn switch_cache_to_next_highest_priority(&mut self) {
        self.highest_priority = self
            .queues_by_priority
            .iter()
            .rev()
            .find_map(|(priority, queue)| (!queue.is_empty()).then_some(*priority));
    }
}

impl<T> Default for SequencedPriorityQueueModel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl LiquidSettingsModel {
    pub fn id(self) -> &'static str {
        match self {
            LiquidSettingsModel::IgnoreWaterlogging => "ignore_waterlogging",
            LiquidSettingsModel::ApplyWaterlogging => "apply_waterlogging",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "ignore_waterlogging" => Some(LiquidSettingsModel::IgnoreWaterlogging),
            "apply_waterlogging" => Some(LiquidSettingsModel::ApplyWaterlogging),
            _ => None,
        }
    }

    pub fn should_apply_waterlogging(self) -> bool {
        self == LiquidSettingsModel::ApplyWaterlogging
    }
}

impl DimensionPaddingModel {
    pub const ZERO: Self = Self { bottom: 0, top: 0 };

    pub fn uniform(value: i32) -> Result<Self, String> {
        Self::new(value, value)
    }

    pub fn new(bottom: i32, top: i32) -> Result<Self, String> {
        if bottom < 0 || top < 0 {
            Err("dimension padding values must be non-negative".to_string())
        } else {
            Ok(Self { bottom, top })
        }
    }

    pub fn has_equal_top_and_bottom(self) -> bool {
        self.top == self.bottom
    }
}

impl JigsawMaxDistanceModel {
    pub const DEFAULT: Self = Self {
        horizontal: 80,
        vertical: 80,
    };

    pub fn uniform(value: i32) -> Result<Self, String> {
        Self::new(value, value)
    }

    pub fn new(horizontal: i32, vertical: i32) -> Result<Self, String> {
        if !(1..=128).contains(&horizontal) {
            return Err("jigsaw horizontal max distance must be in 1..=128".to_string());
        }
        if !(1..=384).contains(&vertical) {
            return Err("jigsaw vertical max distance must be in 1..=384".to_string());
        }
        Ok(Self {
            horizontal,
            vertical,
        })
    }

    pub fn can_encode_as_uniform(self) -> bool {
        self.horizontal == self.vertical
    }
}

impl JigsawJunctionModel {
    pub fn serialize(&self) -> JigsawJunctionTagModel {
        JigsawJunctionTagModel {
            source_x: self.source_x,
            source_ground_y: self.source_ground_y,
            source_z: self.source_z,
            delta_y: self.delta_y,
            dest_proj: self.dest_projection.id(),
        }
    }

    pub fn deserialize(tag: JigsawJunctionTagModel) -> Option<Self> {
        Some(Self {
            source_x: tag.source_x,
            source_ground_y: tag.source_ground_y,
            source_z: tag.source_z,
            delta_y: tag.delta_y,
            dest_projection: JigsawProjectionModel::from_id(tag.dest_proj)?,
        })
    }

    pub fn java_equals(&self, other: &Self) -> bool {
        self.source_x == other.source_x
            && self.source_z == other.source_z
            && self.delta_y == other.delta_y
            && self.dest_projection == other.dest_projection
    }

    pub fn java_hash_inputs(&self) -> (i32, i32, i32, i32, JigsawProjectionModel) {
        (
            self.source_x,
            self.source_ground_y,
            self.source_z,
            self.delta_y,
            self.dest_projection,
        )
    }
}

impl JigsawPoolAliasBindingModel {
    pub fn codec_id(&self) -> &'static str {
        match self {
            Self::Direct { .. } => "minecraft:direct",
            Self::Random { .. } => "minecraft:random",
            Self::RandomGroup { .. } => "minecraft:random_group",
        }
    }

    pub fn all_targets(&self) -> Vec<&'static str> {
        match self {
            Self::Direct { target, .. } => vec![*target],
            Self::Random { targets, .. } => targets.iter().map(|target| target.target).collect(),
            Self::RandomGroup { groups } => groups
                .iter()
                .flat_map(|group| group.bindings.iter())
                .flat_map(Self::all_targets)
                .collect(),
        }
    }

    pub fn for_each_resolved(
        &self,
        random: &mut RandomSourceKind,
        consumer: &mut impl FnMut(&'static str, &'static str),
    ) {
        match self {
            Self::Direct { alias, target } => consumer(alias, target),
            Self::Random { alias, targets } => {
                let target = select_weighted_pool_target(targets, random);
                consumer(alias, target);
            }
            Self::RandomGroup { groups } => {
                let bindings = select_weighted_pool_group(groups, random);
                for binding in bindings {
                    binding.for_each_resolved(random, consumer);
                }
            }
        }
    }
}

impl JigsawPoolAliasLookupModel {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn create(
        bindings: &[JigsawPoolAliasBindingModel],
        pos: (i32, i32, i32),
        seed: i64,
    ) -> Self {
        if bindings.is_empty() {
            return Self::empty();
        }

        let mut base = LegacyRandom::new(seed);
        let mut random = base.fork_positional().at(pos.0, pos.1, pos.2);
        let mut mappings = BTreeMap::new();
        for binding in bindings {
            binding.for_each_resolved(&mut random, &mut |alias, target| {
                mappings.insert(alias, target);
            });
        }
        Self { mappings }
    }

    pub fn lookup(&self, alias: &'static str) -> &'static str {
        self.mappings.get(alias).copied().unwrap_or(alias)
    }
}
