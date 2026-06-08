use std::collections::{BTreeMap, BTreeSet};

use crate::criterion_distance_predicate::DoubleBoundsModel;
use crate::criterion_light_predicate::IntBoundsModel;
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct LocationPredicateModel {
    pub position: Option<PositionPredicateModel>,
    pub biomes: Option<HolderSetModel>,
    pub structures: Option<HolderSetModel>,
    pub dimension: Option<Identifier>,
    pub smokey: Option<bool>,
    pub light: Option<LightPredicateModel>,
    pub block: Option<BlockPredicateModel>,
    pub fluid: Option<FluidPredicateModel>,
    pub can_see_sky: Option<bool>,
}

impl LocationPredicateModel {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position: Option<PositionPredicateModel>,
        biomes: Option<HolderSetModel>,
        structures: Option<HolderSetModel>,
        dimension: Option<Identifier>,
        smokey: Option<bool>,
        light: Option<LightPredicateModel>,
        block: Option<BlockPredicateModel>,
        fluid: Option<FluidPredicateModel>,
        can_see_sky: Option<bool>,
    ) -> Self {
        Self {
            position,
            biomes,
            structures,
            dimension,
            smokey,
            light,
            block,
            fluid,
            can_see_sky,
        }
    }

    pub fn matches(&self, level: &ServerLevelLocationModel, x: f64, y: f64, z: f64) -> bool {
        if self
            .position
            .as_ref()
            .is_some_and(|position| !position.matches(x, y, z))
        {
            return false;
        }

        if self
            .dimension
            .as_ref()
            .is_some_and(|dimension| dimension != &level.dimension)
        {
            return false;
        }

        let pos = BlockPosModel::containing(x, y, z);
        let loaded = level.is_loaded(pos);

        if self
            .biomes
            .as_ref()
            .is_some_and(|biomes| !loaded || !biomes.contains(&level.get_biome(pos)))
        {
            return false;
        }

        if self.structures.as_ref().is_some_and(|structures| {
            !loaded || !level.has_structure_with_piece_at(pos, structures)
        }) {
            return false;
        }

        if self
            .smokey
            .is_some_and(|smokey| !loaded || smokey != level.is_smokey_pos(pos))
        {
            return false;
        }

        if self
            .light
            .as_ref()
            .is_some_and(|light| !light.matches(level, pos))
        {
            return false;
        }

        if self
            .block
            .as_ref()
            .is_some_and(|block| !block.matches(level, pos))
        {
            return false;
        }

        if self
            .fluid
            .as_ref()
            .is_some_and(|fluid| !fluid.matches(level, pos))
        {
            return false;
        }

        self.can_see_sky
            .is_none_or(|can_see_sky| can_see_sky == level.can_see_sky(pos))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocationPredicateBuilderModel {
    x: DoubleBoundsModel,
    y: DoubleBoundsModel,
    z: DoubleBoundsModel,
    biomes: Option<HolderSetModel>,
    structures: Option<HolderSetModel>,
    dimension: Option<Identifier>,
    smokey: Option<bool>,
    light: Option<LightPredicateModel>,
    block: Option<BlockPredicateModel>,
    fluid: Option<FluidPredicateModel>,
    can_see_sky: Option<bool>,
}

impl LocationPredicateBuilderModel {
    pub fn location() -> Self {
        Self::default()
    }

    pub fn in_biome(biome: Identifier) -> Self {
        Self::location().set_biomes(HolderSetModel::direct([biome]))
    }

    pub fn in_dimension(dimension: Identifier) -> Self {
        Self::location().set_dimension(dimension)
    }

    pub fn in_structure(structure: Identifier) -> Self {
        Self::location().set_structures(HolderSetModel::direct([structure]))
    }

    pub fn at_y_location(y_location: DoubleBoundsModel) -> Self {
        Self::location().set_y(y_location)
    }

    pub fn set_x(mut self, x: DoubleBoundsModel) -> Self {
        self.x = x;
        self
    }

    pub fn set_y(mut self, y: DoubleBoundsModel) -> Self {
        self.y = y;
        self
    }

    pub fn set_z(mut self, z: DoubleBoundsModel) -> Self {
        self.z = z;
        self
    }

    pub fn set_biomes(mut self, biomes: HolderSetModel) -> Self {
        self.biomes = Some(biomes);
        self
    }

    pub fn set_structures(mut self, structures: HolderSetModel) -> Self {
        self.structures = Some(structures);
        self
    }

    pub fn set_dimension(mut self, dimension: Identifier) -> Self {
        self.dimension = Some(dimension);
        self
    }

    pub fn set_light(mut self, light: LightPredicateBuilderModel) -> Self {
        self.light = Some(light.build());
        self
    }

    pub fn set_block(mut self, block: BlockPredicateBuilderModel) -> Self {
        self.block = Some(block.build());
        self
    }

    pub fn set_fluid(mut self, fluid: FluidPredicateBuilderModel) -> Self {
        self.fluid = Some(fluid.build());
        self
    }

    pub fn set_smokey(mut self, smokey: bool) -> Self {
        self.smokey = Some(smokey);
        self
    }

    pub fn set_can_see_sky(mut self, can_see_sky: bool) -> Self {
        self.can_see_sky = Some(can_see_sky);
        self
    }

    pub fn build(self) -> LocationPredicateModel {
        LocationPredicateModel::new(
            PositionPredicateModel::of(self.x, self.y, self.z),
            self.biomes,
            self.structures,
            self.dimension,
            self.smokey,
            self.light,
            self.block,
            self.fluid,
            self.can_see_sky,
        )
    }
}

impl Default for LocationPredicateBuilderModel {
    fn default() -> Self {
        Self {
            x: DoubleBoundsModel::any(),
            y: DoubleBoundsModel::any(),
            z: DoubleBoundsModel::any(),
            biomes: None,
            structures: None,
            dimension: None,
            smokey: None,
            light: None,
            block: None,
            fluid: None,
            can_see_sky: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionPredicateModel {
    x: DoubleBoundsModel,
    y: DoubleBoundsModel,
    z: DoubleBoundsModel,
}

impl PositionPredicateModel {
    pub fn of(
        x: DoubleBoundsModel,
        y: DoubleBoundsModel,
        z: DoubleBoundsModel,
    ) -> Option<PositionPredicateModel> {
        if double_bounds_is_any(x) && double_bounds_is_any(y) && double_bounds_is_any(z) {
            None
        } else {
            Some(Self { x, y, z })
        }
    }

    pub fn matches(&self, x: f64, y: f64, z: f64) -> bool {
        self.x.matches(x) && self.y.matches(y) && self.z.matches(z)
    }
}

fn double_bounds_is_any(bounds: DoubleBoundsModel) -> bool {
    bounds.min.is_none() && bounds.max.is_none()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSetModel {
    values: BTreeSet<Identifier>,
}

impl HolderSetModel {
    pub fn direct(values: impl IntoIterator<Item = Identifier>) -> Self {
        Self {
            values: values.into_iter().collect(),
        }
    }

    fn contains(&self, value: &Identifier) -> bool {
        self.values.contains(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPredicateModel {
    required_pos: BlockPosModel,
}

impl BlockPredicateModel {
    pub fn at(required_pos: BlockPosModel) -> Self {
        Self { required_pos }
    }

    fn matches(&self, _level: &ServerLevelLocationModel, pos: BlockPosModel) -> bool {
        self.required_pos == pos
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPredicateBuilderModel {
    required_pos: BlockPosModel,
}

impl BlockPredicateBuilderModel {
    pub fn at(required_pos: BlockPosModel) -> Self {
        Self { required_pos }
    }

    fn build(self) -> BlockPredicateModel {
        BlockPredicateModel::at(self.required_pos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidPredicateModel {
    required_pos: BlockPosModel,
}

impl FluidPredicateModel {
    pub fn at(required_pos: BlockPosModel) -> Self {
        Self { required_pos }
    }

    fn matches(&self, _level: &ServerLevelLocationModel, pos: BlockPosModel) -> bool {
        self.required_pos == pos
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidPredicateBuilderModel {
    required_pos: BlockPosModel,
}

impl FluidPredicateBuilderModel {
    pub fn at(required_pos: BlockPosModel) -> Self {
        Self { required_pos }
    }

    fn build(self) -> FluidPredicateModel {
        FluidPredicateModel::at(self.required_pos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightPredicateModel {
    composite: IntBoundsModel,
}

impl LightPredicateModel {
    fn matches(&self, level: &ServerLevelLocationModel, pos: BlockPosModel) -> bool {
        level.is_loaded(pos)
            && self
                .composite
                .matches(level.get_max_local_raw_brightness(pos))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightPredicateBuilderModel {
    composite: IntBoundsModel,
}

impl LightPredicateBuilderModel {
    pub fn light() -> Self {
        Self {
            composite: IntBoundsModel::ANY,
        }
    }

    pub fn set_composite(mut self, composite: IntBoundsModel) -> Self {
        self.composite = composite;
        self
    }

    fn build(self) -> LightPredicateModel {
        LightPredicateModel {
            composite: self.composite,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLevelLocationModel {
    dimension: Identifier,
    loaded_positions: BTreeSet<BlockPosModel>,
    biomes: BTreeMap<BlockPosModel, Identifier>,
    structures: BTreeMap<BlockPosModel, BTreeSet<Identifier>>,
    smokey_positions: BTreeSet<BlockPosModel>,
    sky_visible_positions: BTreeSet<BlockPosModel>,
    raw_brightness: BTreeMap<BlockPosModel, i32>,
}

impl ServerLevelLocationModel {
    pub fn new(dimension: Identifier) -> Self {
        Self {
            dimension,
            loaded_positions: BTreeSet::new(),
            biomes: BTreeMap::new(),
            structures: BTreeMap::new(),
            smokey_positions: BTreeSet::new(),
            sky_visible_positions: BTreeSet::new(),
            raw_brightness: BTreeMap::new(),
        }
    }

    pub fn with_loaded(mut self, pos: BlockPosModel) -> Self {
        self.loaded_positions.insert(pos);
        self
    }

    pub fn with_biome(mut self, pos: BlockPosModel, biome: Identifier) -> Self {
        self.biomes.insert(pos, biome);
        self
    }

    pub fn with_structure(mut self, pos: BlockPosModel, structure: Identifier) -> Self {
        self.structures.entry(pos).or_default().insert(structure);
        self
    }

    pub fn with_smokey(mut self, pos: BlockPosModel) -> Self {
        self.smokey_positions.insert(pos);
        self
    }

    pub fn with_sky_visible(mut self, pos: BlockPosModel) -> Self {
        self.sky_visible_positions.insert(pos);
        self
    }

    pub fn with_raw_brightness(mut self, pos: BlockPosModel, brightness: i32) -> Self {
        self.raw_brightness.insert(pos, brightness);
        self
    }

    fn get_biome(&self, pos: BlockPosModel) -> Identifier {
        self.biomes
            .get(&pos)
            .cloned()
            .unwrap_or_else(|| id("minecraft:plains"))
    }

    fn has_structure_with_piece_at(&self, pos: BlockPosModel, structures: &HolderSetModel) -> bool {
        self.structures.get(&pos).is_some_and(|present| {
            present
                .iter()
                .any(|structure| structures.contains(structure))
        })
    }

    fn is_smokey_pos(&self, pos: BlockPosModel) -> bool {
        self.smokey_positions.contains(&pos)
    }

    fn can_see_sky(&self, pos: BlockPosModel) -> bool {
        self.sky_visible_positions.contains(&pos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockPosModel {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPosModel {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn containing(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: x.floor() as i32,
            y: y.floor() as i32,
            z: z.floor() as i32,
        }
    }
}

impl ServerLevelLocationModel {
    fn is_loaded(&self, pos: BlockPosModel) -> bool {
        self.loaded_positions.contains(&pos)
    }

    fn get_max_local_raw_brightness(&self, pos: BlockPosModel) -> i32 {
        *self.raw_brightness.get(&pos).unwrap_or(&0)
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: i32, y: i32, z: i32) -> BlockPosModel {
        BlockPosModel::new(x, y, z)
    }

    fn level() -> ServerLevelLocationModel {
        let origin = pos(0, 64, 0);
        ServerLevelLocationModel::new(id("minecraft:overworld"))
            .with_loaded(origin)
            .with_loaded(pos(-2, 70, 3))
            .with_biome(origin, id("minecraft:plains"))
            .with_structure(origin, id("minecraft:village"))
            .with_smokey(origin)
            .with_sky_visible(origin)
            .with_raw_brightness(origin, 12)
    }

    #[test]
    fn omitted_fields_match_any_location_and_builder_omits_any_position() {
        let predicate = LocationPredicateBuilderModel::location().build();

        assert_eq!(predicate.position, None);
        assert!(predicate.matches(&level(), 0.25, 64.0, 0.75));
        assert!(predicate.matches(&level(), 99.0, -10.0, 99.0));
    }

    #[test]
    fn position_and_dimension_are_checked_before_block_position_predicates() {
        let predicate = LocationPredicateBuilderModel::location()
            .set_x(DoubleBoundsModel::between(-2.0, -1.0))
            .set_y(DoubleBoundsModel::at_least(70.0))
            .set_z(DoubleBoundsModel::exactly(3.25))
            .set_dimension(id("minecraft:overworld"))
            .build();

        assert!(predicate.matches(&level(), -1.2, 70.0, 3.25));
        assert!(!predicate.matches(&level(), -0.99, 70.0, 3.25));
        assert!(!predicate.matches(
            &ServerLevelLocationModel::new(id("minecraft:the_nether")),
            -1.2,
            70.0,
            3.25,
        ));
    }

    #[test]
    fn block_pos_uses_java_containing_floor_semantics() {
        assert_eq!(BlockPosModel::containing(0.99, 64.0, -0.01), pos(0, 64, -1));
    }

    #[test]
    fn biome_structure_and_smokey_predicates_require_loaded_position() {
        let predicate = LocationPredicateBuilderModel::location()
            .set_biomes(HolderSetModel::direct([id("minecraft:plains")]))
            .set_structures(HolderSetModel::direct([id("minecraft:village")]))
            .set_smokey(true)
            .build();

        assert!(predicate.matches(&level(), 0.25, 64.0, 0.75));
        assert!(!predicate.matches(&level(), 10.0, 64.0, 10.0));
        assert!(!LocationPredicateBuilderModel::location()
            .set_biomes(HolderSetModel::direct([id("minecraft:desert")]))
            .build()
            .matches(&level(), 0.25, 64.0, 0.75));
        assert!(!LocationPredicateBuilderModel::location()
            .set_structures(HolderSetModel::direct([id("minecraft:mineshaft")]))
            .build()
            .matches(&level(), 0.25, 64.0, 0.75));
        assert!(!LocationPredicateBuilderModel::location()
            .set_smokey(false)
            .build()
            .matches(&level(), 0.25, 64.0, 0.75));
    }

    #[test]
    fn light_block_fluid_and_sky_checks_match_java_delegate_order() {
        let origin = pos(0, 64, 0);
        let predicate = LocationPredicateBuilderModel::location()
            .set_light(
                LightPredicateBuilderModel::light().set_composite(IntBoundsModel::at_least(10)),
            )
            .set_block(BlockPredicateBuilderModel::at(origin))
            .set_fluid(FluidPredicateBuilderModel::at(origin))
            .set_can_see_sky(true)
            .build();

        assert!(predicate.matches(&level(), 0.25, 64.0, 0.75));
        assert!(!predicate.matches(&level(), -1.2, 70.0, 3.0));
        assert!(!LocationPredicateBuilderModel::location()
            .set_light(
                LightPredicateBuilderModel::light().set_composite(IntBoundsModel::exactly(15))
            )
            .build()
            .matches(&level(), 0.25, 64.0, 0.75));
        assert!(!LocationPredicateBuilderModel::location()
            .set_block(BlockPredicateBuilderModel::at(pos(1, 64, 0)))
            .build()
            .matches(&level(), 0.25, 64.0, 0.75));
        assert!(!LocationPredicateBuilderModel::location()
            .set_fluid(FluidPredicateBuilderModel::at(pos(1, 64, 0)))
            .build()
            .matches(&level(), 0.25, 64.0, 0.75));
        assert!(!LocationPredicateBuilderModel::location()
            .set_can_see_sky(false)
            .build()
            .matches(&level(), 0.25, 64.0, 0.75));
    }

    #[test]
    fn can_see_sky_is_not_gated_by_loaded_when_it_is_the_only_block_pos_predicate() {
        let sky_pos = pos(9, 80, 9);
        let sky_level =
            ServerLevelLocationModel::new(id("minecraft:overworld")).with_sky_visible(sky_pos);
        let predicate = LocationPredicateBuilderModel::location()
            .set_can_see_sky(true)
            .build();

        assert!(predicate.matches(&sky_level, 9.0, 80.0, 9.0));
    }

    #[test]
    fn static_builder_helpers_match_java_shapes() {
        let biome = LocationPredicateBuilderModel::in_biome(id("minecraft:plains")).build();
        let dimension =
            LocationPredicateBuilderModel::in_dimension(id("minecraft:overworld")).build();
        let structure =
            LocationPredicateBuilderModel::in_structure(id("minecraft:village")).build();
        let at_y =
            LocationPredicateBuilderModel::at_y_location(DoubleBoundsModel::at_most(80.0)).build();

        assert!(biome.biomes.is_some());
        assert_eq!(dimension.dimension, Some(id("minecraft:overworld")));
        assert!(structure.structures.is_some());
        assert!(at_y
            .position
            .is_some_and(|position| position.matches(100.0, 80.0, -100.0)));
        assert!(at_y.position.is_some());
    }
}
