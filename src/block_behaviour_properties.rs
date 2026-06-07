#![allow(dead_code)]

use crate::block_update::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockOffsetType {
    None,
    Xz,
    Xyz,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushReactionModel {
    Normal,
    Destroy,
    Block,
    Ignore,
    PushOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteBlockInstrumentModel {
    Harp,
    Basedrum,
    Snare,
    Hat,
    Bass,
    Flute,
    Bell,
    Guitar,
    Chime,
    Xylophone,
    IronXylophone,
    CowBell,
    Didgeridoo,
    Bit,
    Banjo,
    Pling,
    Zombie,
    Skeleton,
    Creeper,
    Dragon,
    WitherSkeleton,
    Piglin,
    CustomHead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootTableSource {
    DefaultBlock,
    Fixed(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptionSource {
    DefaultBlock,
    Fixed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatePredicateModel {
    CollisionShapeFullBlock,
    BlocksMotionAndCollisionShapeFullBlock,
    False,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateArgumentPredicateModel {
    FaceSturdyUpAndLightBelow14,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostProcessModel {
    None,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockBehaviourPropertiesModel {
    pub map_color: String,
    pub has_collision: bool,
    pub sound_type: String,
    pub light_emission: u8,
    pub explosion_resistance: f32,
    pub destroy_time: f32,
    pub requires_correct_tool_for_drops: bool,
    pub is_randomly_ticking: bool,
    pub friction: f32,
    pub speed_factor: f32,
    pub jump_factor: f32,
    pub id: Option<String>,
    pub drops: LootTableSource,
    pub description_id: DescriptionSource,
    pub can_occlude: bool,
    pub is_air: bool,
    pub ignited_by_lava: bool,
    pub liquid: bool,
    pub force_solid_off: bool,
    pub force_solid_on: bool,
    pub push_reaction: PushReactionModel,
    pub spawn_terrain_particles: bool,
    pub instrument: NoteBlockInstrumentModel,
    pub replaceable: bool,
    pub is_valid_spawn: StateArgumentPredicateModel,
    pub is_redstone_conductor: StatePredicateModel,
    pub is_suffocating: StatePredicateModel,
    pub is_view_blocking: StatePredicateModel,
    pub post_process: PostProcessModel,
    pub emissive_rendering: StatePredicateModel,
    pub dynamic_shape: bool,
    pub required_features: Vec<String>,
    pub offset_type: BlockOffsetType,
}

impl Default for BlockBehaviourPropertiesModel {
    fn default() -> Self {
        Self {
            map_color: "none".to_string(),
            has_collision: true,
            sound_type: "stone".to_string(),
            light_emission: 0,
            explosion_resistance: 0.0,
            destroy_time: 0.0,
            requires_correct_tool_for_drops: false,
            is_randomly_ticking: false,
            friction: 0.6,
            speed_factor: 1.0,
            jump_factor: 1.0,
            id: None,
            drops: LootTableSource::DefaultBlock,
            description_id: DescriptionSource::DefaultBlock,
            can_occlude: true,
            is_air: false,
            ignited_by_lava: false,
            liquid: false,
            force_solid_off: false,
            force_solid_on: false,
            push_reaction: PushReactionModel::Normal,
            spawn_terrain_particles: true,
            instrument: NoteBlockInstrumentModel::Harp,
            replaceable: false,
            is_valid_spawn: StateArgumentPredicateModel::FaceSturdyUpAndLightBelow14,
            is_redstone_conductor: StatePredicateModel::CollisionShapeFullBlock,
            is_suffocating: StatePredicateModel::BlocksMotionAndCollisionShapeFullBlock,
            is_view_blocking: StatePredicateModel::BlocksMotionAndCollisionShapeFullBlock,
            post_process: PostProcessModel::None,
            emissive_rendering: StatePredicateModel::False,
            dynamic_shape: false,
            required_features: vec!["minecraft:vanilla".to_string()],
            offset_type: BlockOffsetType::None,
        }
    }
}

impl BlockBehaviourPropertiesModel {
    pub fn of() -> Self {
        Self::default()
    }

    pub fn of_legacy_copy(block: &Self) -> Self {
        Self {
            destroy_time: block.destroy_time,
            explosion_resistance: block.explosion_resistance,
            has_collision: block.has_collision,
            is_randomly_ticking: block.is_randomly_ticking,
            light_emission: block.light_emission,
            map_color: block.map_color.clone(),
            sound_type: block.sound_type.clone(),
            friction: block.friction,
            speed_factor: block.speed_factor,
            dynamic_shape: block.dynamic_shape,
            can_occlude: block.can_occlude,
            is_air: block.is_air,
            ignited_by_lava: block.ignited_by_lava,
            liquid: block.liquid,
            force_solid_off: block.force_solid_off,
            force_solid_on: block.force_solid_on,
            push_reaction: block.push_reaction,
            requires_correct_tool_for_drops: block.requires_correct_tool_for_drops,
            offset_type: block.offset_type,
            spawn_terrain_particles: block.spawn_terrain_particles,
            required_features: block.required_features.clone(),
            emissive_rendering: block.emissive_rendering,
            instrument: block.instrument,
            replaceable: block.replaceable,
            ..Self::default()
        }
    }

    pub fn of_full_copy(block: &Self) -> Self {
        let mut copy = Self::of_legacy_copy(block);
        copy.jump_factor = block.jump_factor;
        copy.is_redstone_conductor = block.is_redstone_conductor;
        copy.is_valid_spawn = block.is_valid_spawn;
        copy.post_process = block.post_process;
        copy.is_suffocating = block.is_suffocating;
        copy.is_view_blocking = block.is_view_blocking;
        copy.drops.clone_from(&block.drops);
        copy.description_id.clone_from(&block.description_id);
        copy
    }

    pub fn map_color(mut self, map_color: impl Into<String>) -> Self {
        self.map_color = map_color.into();
        self
    }

    pub fn no_collision(mut self) -> Self {
        self.has_collision = false;
        self.can_occlude = false;
        self
    }

    pub fn no_occlusion(mut self) -> Self {
        self.can_occlude = false;
        self
    }

    pub fn friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    pub fn speed_factor(mut self, speed_factor: f32) -> Self {
        self.speed_factor = speed_factor;
        self
    }

    pub fn jump_factor(mut self, jump_factor: f32) -> Self {
        self.jump_factor = jump_factor;
        self
    }

    pub fn sound(mut self, sound_type: impl Into<String>) -> Self {
        self.sound_type = sound_type.into();
        self
    }

    pub fn light_level(mut self, light_emission: u8) -> Self {
        self.light_emission = light_emission;
        self
    }

    pub fn strength(self, destroy_time: f32, explosion_resistance: f32) -> Self {
        self.destroy_time(destroy_time)
            .explosion_resistance(explosion_resistance)
    }

    pub fn instabreak(self) -> Self {
        self.strength_same(0.0)
    }

    pub fn strength_same(self, destroy_time: f32) -> Self {
        self.strength(destroy_time, destroy_time)
    }

    pub fn random_ticks(mut self) -> Self {
        self.is_randomly_ticking = true;
        self
    }

    pub fn dynamic_shape(mut self) -> Self {
        self.dynamic_shape = true;
        self
    }

    pub fn no_loot_table(mut self) -> Self {
        self.drops = LootTableSource::Fixed(None);
        self
    }

    pub fn override_loot_table(mut self, table: Option<impl Into<String>>) -> Self {
        self.drops = LootTableSource::Fixed(table.map(Into::into));
        self
    }

    pub fn ignited_by_lava(mut self) -> Self {
        self.ignited_by_lava = true;
        self
    }

    pub fn liquid(mut self) -> Self {
        self.liquid = true;
        self
    }

    pub fn force_solid_on(mut self) -> Self {
        self.force_solid_on = true;
        self
    }

    pub fn force_solid_off(mut self) -> Self {
        self.force_solid_off = true;
        self
    }

    pub fn push_reaction(mut self, push_reaction: PushReactionModel) -> Self {
        self.push_reaction = push_reaction;
        self
    }

    pub fn air(mut self) -> Self {
        self.is_air = true;
        self
    }

    pub fn valid_spawn_predicate(mut self, predicate: StateArgumentPredicateModel) -> Self {
        self.is_valid_spawn = predicate;
        self
    }

    pub fn redstone_conductor_predicate(mut self, predicate: StatePredicateModel) -> Self {
        self.is_redstone_conductor = predicate;
        self
    }

    pub fn suffocating_predicate(mut self, predicate: StatePredicateModel) -> Self {
        self.is_suffocating = predicate;
        self
    }

    pub fn view_blocking_predicate(mut self, predicate: StatePredicateModel) -> Self {
        self.is_view_blocking = predicate;
        self
    }

    pub fn post_process(mut self, post_process: PostProcessModel) -> Self {
        self.post_process = post_process;
        self
    }

    pub fn emissive_rendering(mut self, predicate: StatePredicateModel) -> Self {
        self.emissive_rendering = predicate;
        self
    }

    pub fn requires_correct_tool_for_drops(mut self) -> Self {
        self.requires_correct_tool_for_drops = true;
        self
    }

    pub fn destroy_time(mut self, destroy_time: f32) -> Self {
        self.destroy_time = destroy_time;
        self
    }

    pub fn explosion_resistance(mut self, explosion_resistance: f32) -> Self {
        self.explosion_resistance = explosion_resistance.max(0.0);
        self
    }

    pub fn offset_type(mut self, offset_type: BlockOffsetType) -> Self {
        self.offset_type = offset_type;
        self
    }

    pub fn no_terrain_particles(mut self) -> Self {
        self.spawn_terrain_particles = false;
        self
    }

    pub fn required_features(mut self, flags: &[&str]) -> Self {
        self.required_features = flags.iter().map(|flag| (*flag).to_string()).collect();
        self
    }

    pub fn instrument(mut self, instrument: NoteBlockInstrumentModel) -> Self {
        self.instrument = instrument;
        self
    }

    pub fn replaceable(mut self) -> Self {
        self.replaceable = true;
        self
    }

    pub fn set_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn override_description(mut self, description_id: impl Into<String>) -> Self {
        self.description_id = DescriptionSource::Fixed(description_id.into());
        self
    }

    pub fn effective_drops(&self) -> Result<Option<String>, String> {
        match &self.drops {
            LootTableSource::DefaultBlock => {
                let id = self
                    .id
                    .as_deref()
                    .ok_or_else(|| "Block id not set".to_string())?;
                let (namespace, path) = split_resource_id(id);
                Ok(Some(format!("{namespace}:blocks/{path}")))
            }
            LootTableSource::Fixed(table) => Ok(table.clone()),
        }
    }

    pub fn effective_description_id(&self) -> Result<String, String> {
        match &self.description_id {
            DescriptionSource::DefaultBlock => {
                let id = self
                    .id
                    .as_deref()
                    .ok_or_else(|| "Block id not set".to_string())?;
                let (namespace, path) = split_resource_id(id);
                Ok(format!("block.{namespace}.{}", path.replace('/', ".")))
            }
            DescriptionSource::Fixed(description) => Ok(description.clone()),
        }
    }

    pub fn offset_at(&self, pos: BlockPos) -> (f64, f64, f64) {
        offset_for_type(self.offset_type, pos, 0.25, 0.2)
    }
}

pub fn offset_for_type(
    offset_type: BlockOffsetType,
    pos: BlockPos,
    max_horizontal_offset: f64,
    max_vertical_offset: f64,
) -> (f64, f64, f64) {
    match offset_type {
        BlockOffsetType::None => (0.0, 0.0, 0.0),
        BlockOffsetType::Xz | BlockOffsetType::Xyz => {
            let seed = mth_get_seed(pos.x, 0, pos.z);
            let x = ((((seed & 15) as f64) / 15.0 - 0.5) * 0.5)
                .clamp(-max_horizontal_offset, max_horizontal_offset);
            let z = (((((seed >> 8) & 15) as f64) / 15.0 - 0.5) * 0.5)
                .clamp(-max_horizontal_offset, max_horizontal_offset);
            let y = if offset_type == BlockOffsetType::Xyz {
                ((((seed >> 4) & 15) as f64) / 15.0 - 1.0) * max_vertical_offset
            } else {
                0.0
            };
            (x, y, z)
        }
    }
}

pub fn mth_get_seed(x: i32, y: i32, z: i32) -> i64 {
    let mut seed = i64::from(x.wrapping_mul(3_129_871))
        ^ i64::from(z).wrapping_mul(116_129_781)
        ^ i64::from(y);
    seed = seed
        .wrapping_mul(seed)
        .wrapping_mul(42_317_861)
        .wrapping_add(seed.wrapping_mul(11));
    seed >> 16
}

fn split_resource_id(id: &str) -> (&str, &str) {
    id.split_once(':').unwrap_or(("minecraft", id))
}

#[cfg(test)]
mod tests {
    use super::{
        mth_get_seed, offset_for_type, BlockBehaviourPropertiesModel, BlockOffsetType,
        DescriptionSource, LootTableSource, NoteBlockInstrumentModel, PostProcessModel,
        PushReactionModel, StateArgumentPredicateModel, StatePredicateModel,
    };
    use crate::block_update::BlockPos;

    #[test]
    fn block_behaviour_properties_physical_defaults_match_java_properties_of() {
        let properties = BlockBehaviourPropertiesModel::of();
        assert_eq!(properties.map_color, "none");
        assert!(properties.has_collision);
        assert_eq!(properties.sound_type, "stone");
        assert_eq!(properties.light_emission, 0);
        assert_eq!(properties.explosion_resistance, 0.0);
        assert_eq!(properties.destroy_time, 0.0);
        assert!(!properties.requires_correct_tool_for_drops);
        assert!(!properties.is_randomly_ticking);
        assert_eq!(properties.friction, 0.6);
        assert_eq!(properties.speed_factor, 1.0);
        assert_eq!(properties.jump_factor, 1.0);
        assert_eq!(properties.drops, LootTableSource::DefaultBlock);
        assert_eq!(properties.description_id, DescriptionSource::DefaultBlock);
        assert!(properties.effective_drops().is_err());
        assert!(properties.effective_description_id().is_err());
    }

    #[test]
    fn block_behaviour_properties_flag_defaults_match_java_properties_of() {
        let properties = BlockBehaviourPropertiesModel::of();
        assert!(properties.can_occlude);
        assert!(!properties.is_air);
        assert!(!properties.ignited_by_lava);
        assert!(!properties.liquid);
        assert!(!properties.force_solid_off);
        assert!(!properties.force_solid_on);
        assert_eq!(properties.push_reaction, PushReactionModel::Normal);
        assert!(properties.spawn_terrain_particles);
        assert_eq!(properties.instrument, NoteBlockInstrumentModel::Harp);
        assert!(!properties.replaceable);
        assert!(!properties.dynamic_shape);
        assert_eq!(properties.required_features, vec!["minecraft:vanilla"]);
        assert_eq!(properties.offset_type, BlockOffsetType::None);
    }

    #[test]
    fn block_behaviour_properties_predicate_defaults_match_java_properties_of() {
        let properties = BlockBehaviourPropertiesModel::of();
        assert_eq!(
            properties.is_valid_spawn,
            StateArgumentPredicateModel::FaceSturdyUpAndLightBelow14
        );
        assert_eq!(
            properties.is_redstone_conductor,
            StatePredicateModel::CollisionShapeFullBlock
        );
        assert_eq!(
            properties.is_suffocating,
            StatePredicateModel::BlocksMotionAndCollisionShapeFullBlock
        );
        assert_eq!(
            properties.is_view_blocking,
            StatePredicateModel::BlocksMotionAndCollisionShapeFullBlock
        );
        assert_eq!(properties.post_process, PostProcessModel::None);
        assert_eq!(properties.emissive_rendering, StatePredicateModel::False);
    }

    fn mutated_properties() -> BlockBehaviourPropertiesModel {
        BlockBehaviourPropertiesModel::of()
            .map_color("grass")
            .no_collision()
            .friction(0.8)
            .speed_factor(0.4)
            .jump_factor(1.2)
            .sound("wood")
            .light_level(14)
            .strength(2.0, -5.0)
            .random_ticks()
            .dynamic_shape()
            .ignited_by_lava()
            .liquid()
            .force_solid_on()
            .force_solid_off()
            .push_reaction(PushReactionModel::Destroy)
            .air()
            .valid_spawn_predicate(StateArgumentPredicateModel::Custom)
            .redstone_conductor_predicate(StatePredicateModel::Custom)
            .suffocating_predicate(StatePredicateModel::False)
            .view_blocking_predicate(StatePredicateModel::False)
            .post_process(PostProcessModel::Custom)
            .emissive_rendering(StatePredicateModel::Custom)
            .requires_correct_tool_for_drops()
            .offset_type(BlockOffsetType::Xyz)
            .no_terrain_particles()
            .required_features(&["minecraft:update_1_21"])
            .instrument(NoteBlockInstrumentModel::Bell)
            .replaceable()
            .set_id("minecraft:test_block")
            .override_description("block.minecraft.custom")
    }

    #[test]
    fn block_behaviour_properties_physical_builder_methods_match_java_mutations() {
        let properties = mutated_properties();
        assert_eq!(properties.map_color, "grass");
        assert!(!properties.has_collision);
        assert!(!properties.can_occlude);
        assert_eq!(properties.friction, 0.8);
        assert_eq!(properties.speed_factor, 0.4);
        assert_eq!(properties.jump_factor, 1.2);
        assert_eq!(properties.sound_type, "wood");
        assert_eq!(properties.light_emission, 14);
        assert_eq!(properties.destroy_time, 2.0);
        assert_eq!(properties.explosion_resistance, 0.0);
    }

    #[test]
    fn block_behaviour_properties_flag_builder_methods_match_java_mutations() {
        let properties = mutated_properties();
        assert!(properties.is_randomly_ticking);
        assert!(properties.dynamic_shape);
        assert!(properties.ignited_by_lava);
        assert!(properties.liquid);
        assert!(properties.force_solid_on);
        assert!(properties.force_solid_off);
        assert_eq!(properties.push_reaction, PushReactionModel::Destroy);
        assert!(properties.is_air);
        assert!(properties.requires_correct_tool_for_drops);
        assert_eq!(properties.offset_type, BlockOffsetType::Xyz);
        assert!(!properties.spawn_terrain_particles);
        assert_eq!(properties.required_features, vec!["minecraft:update_1_21"]);
        assert_eq!(properties.instrument, NoteBlockInstrumentModel::Bell);
        assert!(properties.replaceable);
    }

    #[test]
    fn block_behaviour_properties_predicate_builder_methods_match_java_mutations() {
        let properties = mutated_properties();
        assert_eq!(
            properties.is_valid_spawn,
            StateArgumentPredicateModel::Custom
        );
        assert_eq!(
            properties.is_redstone_conductor,
            StatePredicateModel::Custom
        );
        assert_eq!(properties.is_suffocating, StatePredicateModel::False);
        assert_eq!(properties.is_view_blocking, StatePredicateModel::False);
        assert_eq!(properties.post_process, PostProcessModel::Custom);
        assert_eq!(properties.emissive_rendering, StatePredicateModel::Custom);
        assert_eq!(
            properties.effective_description_id().unwrap(),
            "block.minecraft.custom"
        );
    }

    #[test]
    fn block_behaviour_properties_loot_and_description_follow_java_dependant_names() {
        let defaulted = BlockBehaviourPropertiesModel::of().set_id("minecraft:oak_log");
        assert_eq!(
            defaulted.effective_drops().unwrap(),
            Some("minecraft:blocks/oak_log".to_string())
        );
        assert_eq!(
            defaulted.effective_description_id().unwrap(),
            "block.minecraft.oak_log"
        );

        let modded = BlockBehaviourPropertiesModel::of().set_id("example:foo/bar");
        assert_eq!(
            modded.effective_drops().unwrap(),
            Some("example:blocks/foo/bar".to_string())
        );
        assert_eq!(
            modded.effective_description_id().unwrap(),
            "block.example.foo.bar"
        );

        assert_eq!(
            BlockBehaviourPropertiesModel::of()
                .no_loot_table()
                .effective_drops()
                .unwrap(),
            None
        );
        assert_eq!(
            BlockBehaviourPropertiesModel::of()
                .override_loot_table(Some("minecraft:blocks/custom"))
                .effective_drops()
                .unwrap(),
            Some("minecraft:blocks/custom".to_string())
        );
        assert_eq!(
            BlockBehaviourPropertiesModel::of()
                .override_loot_table(None::<&str>)
                .effective_drops()
                .unwrap(),
            None
        );
    }

    #[test]
    fn block_behaviour_properties_copy_modes_match_java_legacy_and_full_copy() {
        let source = BlockBehaviourPropertiesModel::of()
            .strength(3.0, 4.0)
            .no_collision()
            .random_ticks()
            .light_level(7)
            .map_color("deepslate")
            .sound("deepslate")
            .friction(0.9)
            .speed_factor(0.5)
            .jump_factor(0.25)
            .dynamic_shape()
            .air()
            .ignited_by_lava()
            .liquid()
            .force_solid_off()
            .push_reaction(PushReactionModel::Block)
            .requires_correct_tool_for_drops()
            .offset_type(BlockOffsetType::Xz)
            .no_terrain_particles()
            .required_features(&["minecraft:feature"])
            .emissive_rendering(StatePredicateModel::Custom)
            .instrument(NoteBlockInstrumentModel::Xylophone)
            .replaceable()
            .valid_spawn_predicate(StateArgumentPredicateModel::Custom)
            .redstone_conductor_predicate(StatePredicateModel::Custom)
            .suffocating_predicate(StatePredicateModel::False)
            .view_blocking_predicate(StatePredicateModel::False)
            .post_process(PostProcessModel::Custom)
            .no_loot_table()
            .override_description("block.minecraft.source");

        let legacy = BlockBehaviourPropertiesModel::of_legacy_copy(&source);
        assert_eq!(legacy.destroy_time, 3.0);
        assert_eq!(legacy.explosion_resistance, 4.0);
        assert_eq!(legacy.has_collision, source.has_collision);
        assert_eq!(legacy.light_emission, 7);
        assert_eq!(legacy.map_color, "deepslate");
        assert_eq!(legacy.sound_type, "deepslate");
        assert_eq!(legacy.speed_factor, 0.5);
        assert_eq!(
            legacy.jump_factor, 1.0,
            "ofLegacyCopy does not copy jumpFactor"
        );
        assert_eq!(legacy.drops, LootTableSource::DefaultBlock);
        assert_eq!(legacy.description_id, DescriptionSource::DefaultBlock);
        assert_eq!(
            legacy.is_valid_spawn,
            StateArgumentPredicateModel::FaceSturdyUpAndLightBelow14
        );
        assert_eq!(
            legacy.is_suffocating,
            StatePredicateModel::BlocksMotionAndCollisionShapeFullBlock
        );

        let full = BlockBehaviourPropertiesModel::of_full_copy(&source);
        assert_eq!(full.jump_factor, 0.25);
        assert_eq!(full.drops, LootTableSource::Fixed(None));
        assert_eq!(
            full.description_id,
            DescriptionSource::Fixed("block.minecraft.source".to_string())
        );
        assert_eq!(full.is_valid_spawn, StateArgumentPredicateModel::Custom);
        assert_eq!(full.is_redstone_conductor, StatePredicateModel::Custom);
        assert_eq!(full.is_suffocating, StatePredicateModel::False);
        assert_eq!(full.is_view_blocking, StatePredicateModel::False);
        assert_eq!(full.post_process, PostProcessModel::Custom);
    }

    #[test]
    fn block_behaviour_offset_type_matches_java_mth_get_seed_math() {
        let pos = BlockPos { x: 1, y: 64, z: 2 };
        assert_eq!(mth_get_seed(pos.x, 0, pos.z), -98_563_738_688_972);
        assert_eq!(
            offset_for_type(BlockOffsetType::None, pos, 0.25, 0.2),
            (0.0, 0.0, 0.0)
        );
        assert_eq!(
            offset_for_type(BlockOffsetType::Xz, pos, 0.25, 0.2),
            (-0.11666666666666667, 0.0, -0.18333333333333335)
        );
        assert_eq!(
            offset_for_type(BlockOffsetType::Xyz, pos, 0.25, 0.2),
            (
                -0.11666666666666667,
                -0.16000000000000003,
                -0.18333333333333335
            )
        );
    }

    #[test]
    fn block_behaviour_strength_helpers_match_java_chaining() {
        let same = BlockBehaviourPropertiesModel::of().strength_same(1.5);
        assert_eq!(same.destroy_time, 1.5);
        assert_eq!(same.explosion_resistance, 1.5);

        let instant = BlockBehaviourPropertiesModel::of().instabreak();
        assert_eq!(instant.destroy_time, 0.0);
        assert_eq!(instant.explosion_resistance, 0.0);

        let no_occlusion = BlockBehaviourPropertiesModel::of().no_occlusion();
        assert!(no_occlusion.has_collision);
        assert!(!no_occlusion.can_occlude);
    }
}
