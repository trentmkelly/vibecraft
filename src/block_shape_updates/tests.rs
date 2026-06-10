use std::collections::HashMap;

use super::*;
use crate::block_placement::PlacementWorld;

#[derive(Default)]
struct TestWorld {
    blocks: HashMap<(i32, i32, i32), BlockStateModel>,
}

impl TestWorld {
    fn with(mut self, pos: (i32, i32, i32), state: BlockStateModel) -> Self {
        self.blocks.insert(pos, state);
        self
    }
}

impl SurvivalWorld for TestWorld {
    fn state_at(&self, pos: BlockPos) -> BlockStateModel {
        self.blocks
            .get(&(pos.x, pos.y, pos.z))
            .cloned()
            .unwrap_or_else(BlockStateModel::air)
    }

    fn raw_brightness(&self, _pos: BlockPos) -> i32 {
        15
    }
}

impl PlacementWorld for TestWorld {
    fn has_neighbor_signal(&self, _pos: BlockPos) -> bool {
        false
    }
}

const POS: BlockPos = BlockPos { x: 0, y: 64, z: 0 };

fn block(id: &str) -> BlockStateModel {
    BlockStateModel::default_for(id).unwrap_or_else(|| panic!("unknown block {id}"))
}

fn run(
    state: &BlockStateModel,
    direction: Direction,
    neighbour: &BlockStateModel,
    world: &TestWorld,
) -> ShapeUpdate {
    let neighbour_pos = POS.relative(direction);
    update_shape(state, POS, direction, neighbour_pos, neighbour, 0, world)
        .unwrap_or_else(|| panic!("unported update for {}", state.registry_id))
}

#[test]
fn unsupported_blocks_pop_to_air_or_schedule_break_ticks() {
    let empty = TestWorld::default();
    let stone_floor = TestWorld::default().with((0, 63, 0), block("minecraft:stone"));

    // Vegetation dies immediately when its ground is gone.
    let grass = block("minecraft:short_grass");
    assert_eq!(
        run(&grass, Direction::Down, &BlockStateModel::air(), &empty).state,
        BlockStateModel::air()
    );

    // Torches only react to their supporting face.
    let torch = block("minecraft:torch");
    assert_eq!(
        run(&torch, Direction::Down, &BlockStateModel::air(), &empty).state,
        BlockStateModel::air()
    );
    assert_eq!(
        run(&torch, Direction::North, &BlockStateModel::air(), &empty).state,
        torch
    );
    let on_floor = run(
        &torch,
        Direction::Down,
        &block("minecraft:stone"),
        &stone_floor,
    );
    assert_eq!(on_floor.state, torch);

    // Cactus defers its death to a scheduled tick.
    let cactus = block("minecraft:cactus");
    let update = run(&cactus, Direction::Down, &BlockStateModel::air(), &empty);
    assert_eq!(update.state, cactus);
    assert_eq!(update.schedule_block_tick, Some(1));

    // Wall torches pop only when the wall behind breaks.
    let wall_torch = block("minecraft:wall_torch"); // facing north -> wall south
    assert_eq!(
        run(
            &wall_torch,
            Direction::South,
            &BlockStateModel::air(),
            &empty
        )
        .state,
        BlockStateModel::air()
    );
    assert_eq!(
        run(
            &wall_torch,
            Direction::North,
            &BlockStateModel::air(),
            &empty
        )
        .state,
        wall_torch
    );
}

#[test]
fn waterlogged_blocks_schedule_fluid_ticks() {
    let world = TestWorld::default();
    let dry_slab = block("minecraft:oak_slab");
    assert!(!run(&dry_slab, Direction::Up, &BlockStateModel::air(), &world).schedule_fluid_tick);
    let wet_slab = dry_slab.try_set_property("waterlogged", "true");
    assert!(run(&wet_slab, Direction::Up, &BlockStateModel::air(), &world).schedule_fluid_tick);

    let wet_fence = block("minecraft:oak_fence").try_set_property("waterlogged", "true");
    assert!(run(&wet_fence, Direction::Up, &BlockStateModel::air(), &world).schedule_fluid_tick);
}

#[test]
fn connection_blocks_recompute_the_touched_side() {
    let world = TestWorld::default().with((0, 64, 1), block("minecraft:stone"));
    let fence = block("minecraft:oak_fence");
    let update = run(&fence, Direction::South, &block("minecraft:stone"), &world);
    assert_eq!(update.state.property("south"), Some("true"));
    let update = run(
        &update.state,
        Direction::South,
        &BlockStateModel::air(),
        &TestWorld::default(),
    );
    assert_eq!(update.state.property("south"), Some("false"));

    // Stairs re-derive their corner shape.
    let stairs = block("minecraft:oak_stairs"); // facing north (default)
    let corner_world = TestWorld::default().with(
        (0, 64, -1),
        block("minecraft:oak_stairs").try_set_property("facing", "east"),
    );
    let update = run(
        &stairs,
        Direction::North,
        &block("minecraft:oak_stairs").try_set_property("facing", "east"),
        &corner_world,
    );
    assert_eq!(update.state.property("shape"), Some("outer_right"));

    // Chests unpair when their partner vanishes.
    let left_chest = block("minecraft:chest").try_set_property("type", "left");
    // facing north, type left -> connected toward clockwise(north) = east.
    let update = run(
        &left_chest,
        Direction::East,
        &BlockStateModel::air(),
        &TestWorld::default(),
    );
    assert_eq!(update.state.property("type"), Some("single"));

    // Snowy dirt reacts to snow placed above.
    let grass_block = block("minecraft:grass_block");
    let update = run(
        &grass_block,
        Direction::Up,
        &block("minecraft:snow"),
        &TestWorld::default(),
    );
    assert_eq!(update.state.property("snowy"), Some("true"));
}

#[test]
fn paired_halves_sync_and_break_together() {
    let world = TestWorld::default();

    // Doors: lower half dies when the upper half is no longer the same door.
    let lower_door = block("minecraft:oak_door");
    let update = run(&lower_door, Direction::Up, &BlockStateModel::air(), &world);
    assert_eq!(update.state, BlockStateModel::air());
    // The upper half copies hinge/open changes from the lower half.
    let upper_door = lower_door.clone().try_set_property("half", "upper");
    let opened_lower = lower_door.clone().try_set_property("open", "true");
    let update = run(&upper_door, Direction::Down, &opened_lower, &world);
    assert_eq!(update.state.property("open"), Some("true"));
    assert_eq!(update.state.property("half"), Some("upper"));

    // Beds: head pops when the foot disappears and copies OCCUPIED otherwise.
    let head = block("minecraft:red_bed").try_set_property("part", "head");
    // facing north (default): head's partner is to the south (opposite).
    let update = run(&head, Direction::South, &BlockStateModel::air(), &world);
    assert_eq!(update.state, BlockStateModel::air());
    let foot_occupied = block("minecraft:red_bed").try_set_property("occupied", "true");
    let update = run(&head, Direction::South, &foot_occupied, &world);
    assert_eq!(update.state.property("occupied"), Some("true"));

    // Sunflowers break both ways.
    let lower = block("minecraft:sunflower");
    let update = run(&lower, Direction::Up, &BlockStateModel::air(), &world);
    assert_eq!(update.state, BlockStateModel::air());
}

#[test]
fn special_families_follow_java_quirks() {
    let world = TestWorld::default();

    // Falling blocks schedule their fall check.
    let sand = block("minecraft:sand");
    assert_eq!(
        run(&sand, Direction::Down, &BlockStateModel::air(), &world).schedule_block_tick,
        Some(2)
    );

    // Leaves re-tick when their distance is stale.
    let leaves = block("minecraft:oak_leaves");
    let update = run(
        &leaves,
        Direction::North,
        &block("minecraft:oak_log"),
        &world,
    );
    assert_eq!(update.schedule_block_tick, Some(1));

    // Concrete powder solidifies when water arrives.
    let powder = block("minecraft:red_concrete_powder");
    let wet = TestWorld::default().with((0, 64, 1), block("minecraft:water"));
    let update = run(&powder, Direction::South, &block("minecraft:water"), &wet);
    assert_eq!(update.state.registry_id, "minecraft:red_concrete");

    // Huge mushrooms close the face toward a same-block neighbour.
    let cap = block("minecraft:red_mushroom_block");
    let update = run(
        &cap,
        Direction::Up,
        &block("minecraft:red_mushroom_block"),
        &world,
    );
    assert_eq!(update.state.property("up"), Some("false"));

    // Attached stems revert to grown stems when the fruit is gone.
    let attached = block("minecraft:attached_melon_stem");
    let direction = super::facing_of(&attached);
    let update = run(&attached, direction, &BlockStateModel::air(), &world);
    assert_eq!(update.state.registry_id, "minecraft:melon_stem");
    assert_eq!(update.state.property("age"), Some("7"));

    // Kelp heads turn into plant bodies when grown past.
    let kelp = block("minecraft:kelp");
    let grown = TestWorld::default()
        .with((0, 63, 0), block("minecraft:stone"))
        .with((0, 65, 0), block("minecraft:kelp"));
    let update = run(&kelp, Direction::Down, &block("minecraft:stone"), &grown);
    assert_eq!(update.state.registry_id, "minecraft:kelp_plant");
    // And bodies become heads when the chain above is cut.
    let body = block("minecraft:kelp_plant");
    let update = run(&body, Direction::Up, &BlockStateModel::air(), &world);
    assert_eq!(update.state.registry_id, "minecraft:kelp");

    // Glow lichen drops the face toward a lost wall and dies on the last one.
    let lichen = block("minecraft:glow_lichen").try_set_property("south", "true");
    let update = run(&lichen, Direction::South, &BlockStateModel::air(), &world);
    assert_eq!(update.state, BlockStateModel::air());
    let two_faced = block("minecraft:glow_lichen")
        .try_set_property("south", "true")
        .try_set_property("down", "true");
    let update = run(
        &two_faced,
        Direction::South,
        &BlockStateModel::air(),
        &world,
    );
    assert_eq!(update.state.property("south"), Some("false"));
    assert_eq!(update.state.property("down"), Some("true"));

    // Corals without water schedule the 60..99-tick die check (the default
    // state is waterlogged, which counts as contained water).
    let coral = block("minecraft:tube_coral").try_set_property("waterlogged", "false");
    let update = run(&coral, Direction::Up, &BlockStateModel::air(), &world);
    assert_eq!(update.schedule_block_tick, Some(60));
    let watered = TestWorld::default().with((0, 65, 0), block("minecraft:water"));
    let update = run(&coral, Direction::Up, &block("minecraft:water"), &watered);
    assert_eq!(update.schedule_block_tick, None);

    // Unported overrides report None.
    assert!(update_shape(
        &block("minecraft:redstone_wire"),
        POS,
        Direction::Down,
        POS.relative(Direction::Down),
        &BlockStateModel::air(),
        0,
        &world,
    )
    .is_none());

    // Blocks without an override stay untouched.
    let stone = block("minecraft:stone");
    let update = run(&stone, Direction::Up, &BlockStateModel::air(), &world);
    assert_eq!(update.state, stone);
    assert!(!update.schedule_fluid_tick);
    assert_eq!(update.schedule_block_tick, None);
}
