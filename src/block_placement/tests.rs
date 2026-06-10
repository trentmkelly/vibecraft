use std::collections::HashMap;

use super::*;

#[derive(Default)]
struct TestWorld {
    blocks: HashMap<(i32, i32, i32), BlockStateModel>,
    signal: bool,
}

impl TestWorld {
    fn with(mut self, pos: (i32, i32, i32), state: BlockStateModel) -> Self {
        self.blocks.insert(pos, state);
        self
    }

    fn powered(mut self) -> Self {
        self.signal = true;
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
        self.signal
    }
}

const POS: BlockPos = BlockPos { x: 0, y: 64, z: 0 };

/// A south-facing player clicking the top of the block below POS.
fn context() -> PlaceContext {
    PlaceContext {
        clicked_pos: POS,
        clicked_face: Direction::Up,
        click_location: [0.5, 64.0, 0.5],
        replacing_clicked_on_block: false,
        player_yaw: 0.0,
        player_pitch: 45.0,
        secondary_use_active: false,
    }
}

fn block(id: &str) -> BlockStateModel {
    BlockStateModel::default_for(id).unwrap_or_else(|| panic!("unknown block {id}"))
}

fn place(id: &str, ctx: &PlaceContext, world: &TestWorld) -> BlockStateModel {
    match state_for_placement(id, ctx, world) {
        Some(PlacementOutcome::Place(state)) => state,
        other => panic!("{id}: expected placement, got {other:?}"),
    }
}

#[test]
fn direction_helpers_match_java() {
    // Java Direction.fromYRot quadrants (yaw 0 = south).
    assert_eq!(direction_from_y_rot(0.0), Direction::South);
    assert_eq!(direction_from_y_rot(90.0), Direction::West);
    assert_eq!(direction_from_y_rot(180.0), Direction::North);
    assert_eq!(direction_from_y_rot(-90.0), Direction::East);
    assert_eq!(direction_from_y_rot(44.9), Direction::South);
    assert_eq!(direction_from_y_rot(45.1), Direction::West);

    // orderedByNearest: looking straight down puts DOWN first, straight up UP.
    assert_eq!(ordered_by_nearest(90.0, 0.0)[0], Direction::Down);
    assert_eq!(ordered_by_nearest(-90.0, 0.0)[0], Direction::Up);
    // Level look south (yaw 0) leads with SOUTH; second is vertical.
    assert_eq!(ordered_by_nearest(0.0, 0.0)[0], Direction::South);
    assert_eq!(ordered_by_nearest(0.0, 90.0)[0], Direction::West);
    // The array is the three axes then their opposites mirrored.
    let order = ordered_by_nearest(10.0, 0.0);
    assert_eq!(order[5], order[0].opposite());
    assert_eq!(order[4], order[1].opposite());
    assert_eq!(order[3], order[2].opposite());

    // RotationSegment.convertToSegment.
    assert_eq!(rotation_segment(0.0), 0);
    assert_eq!(rotation_segment(180.0), 8);
    assert_eq!(rotation_segment(348.75), 0);
    assert_eq!(rotation_segment(-22.5), 15);
}

#[test]
fn horizontal_facing_families_orient_like_java() {
    let world = TestWorld::default();
    // Player faces south (yaw 0): furnaces face the player -> north.
    assert_eq!(
        place("minecraft:furnace", &context(), &world).property("facing"),
        Some("north")
    );
    assert_eq!(
        place("minecraft:repeater", &context(), &world).property("facing"),
        Some("north")
    );
    // Glazed terracotta uses the player's own direction.
    assert_eq!(
        place("minecraft:white_glazed_terracotta", &context(), &world).property("facing"),
        Some("south")
    );
    // Anvils turn clockwise from the player direction.
    assert_eq!(
        place("minecraft:anvil", &context(), &world).property("facing"),
        Some("west")
    );
    // Barrels/dispensers point back at the player along the dominant look
    // axis; pitch 45 looking south -> nearest is DOWN or SOUTH; with pitch 45
    // exactly Java's tie-break picks the horizontal? Use steep pitch.
    let steep = PlaceContext {
        player_pitch: 80.0,
        ..context()
    };
    assert_eq!(
        place("minecraft:dispenser", &steep, &world).property("facing"),
        Some("up")
    );
    assert_eq!(
        place("minecraft:observer", &steep, &world).property("facing"),
        Some("down")
    );

    // Logs take their axis from the clicked face.
    let side = PlaceContext {
        clicked_face: Direction::East,
        ..context()
    };
    assert_eq!(
        place("minecraft:oak_log", &side, &world).property("axis"),
        Some("x")
    );
    assert_eq!(
        place("minecraft:oak_log", &context(), &world).property("axis"),
        Some("y")
    );
}

#[test]
fn stairs_and_slabs_pick_half_from_click() {
    let world = TestWorld::default();
    // Clicking the top face places bottom-half stairs facing the player's way.
    let stairs = place("minecraft:oak_stairs", &context(), &world);
    assert_eq!(stairs.property("facing"), Some("south"));
    assert_eq!(stairs.property("half"), Some("bottom"));
    assert_eq!(stairs.property("shape"), Some("straight"));
    assert_eq!(stairs.property("waterlogged"), Some("false"));

    // Side click above the midline -> top half.
    let high_side = PlaceContext {
        clicked_face: Direction::North,
        click_location: [0.5, 64.7, 0.0],
        ..context()
    };
    assert_eq!(
        place("minecraft:oak_stairs", &high_side, &world).property("half"),
        Some("top")
    );
    assert_eq!(
        place("minecraft:oak_slab", &high_side, &world).property("type"),
        Some("top")
    );

    // Clicking into an existing slab merges to double.
    let world_with_slab = TestWorld::default().with((0, 64, 0), block("minecraft:oak_slab"));
    let merged = place("minecraft:oak_slab", &context(), &world_with_slab);
    assert_eq!(merged.property("type"), Some("double"));
    assert_eq!(merged.property("waterlogged"), Some("false"));

    // Corner stairs: existing stairs to the south facing east makes an outer
    // corner on a new south-facing stair.
    let corner_world = TestWorld::default().with(
        (0, 64, 1),
        block("minecraft:oak_stairs").try_set_property("facing", "east"),
    );
    let cornered = place("minecraft:oak_stairs", &context(), &corner_world);
    assert_eq!(cornered.property("shape"), Some("outer_left"));
}

#[test]
fn doors_trapdoors_follow_java_orientation() {
    let world = TestWorld::default();
    let door = place("minecraft:oak_door", &context(), &world);
    assert_eq!(door.property("facing"), Some("south"));
    assert_eq!(door.property("half"), Some("lower"));
    assert_eq!(door.property("open"), Some("false"));
    // Hinge tie-break for a south-facing door clicked dead center: stepZ=1,
    // clickX=0.5 -> left.
    assert_eq!(door.property("hinge"), Some("left"));

    // Solid block on the right pushes the hinge right.
    let world_solid_right = TestWorld::default().with((-1, 64, 0), block("minecraft:stone"));
    let door = place("minecraft:oak_door", &context(), &world_solid_right);
    assert_eq!(door.property("hinge"), Some("right"));

    // Door placement fails when the upper half is blocked.
    let blocked = TestWorld::default().with((0, 65, 0), block("minecraft:stone"));
    assert_eq!(
        state_for_placement("minecraft:oak_door", &context(), &blocked),
        Some(PlacementOutcome::Reject)
    );

    // Redstone-powered placement opens the door.
    let powered = TestWorld::default().powered();
    let door = place("minecraft:oak_door", &context(), &powered);
    assert_eq!(door.property("open"), Some("true"));
    assert_eq!(door.property("powered"), Some("true"));

    // Trapdoor: horizontal click below midline -> bottom, facing the face.
    let side = PlaceContext {
        clicked_face: Direction::North,
        click_location: [0.5, 64.2, 0.0],
        ..context()
    };
    let trapdoor = place("minecraft:oak_trapdoor", &side, &world);
    assert_eq!(trapdoor.property("facing"), Some("north"));
    assert_eq!(trapdoor.property("half"), Some("bottom"));
    // Top-face click -> facing away from player, bottom half.
    let trapdoor = place("minecraft:oak_trapdoor", &context(), &world);
    assert_eq!(trapdoor.property("facing"), Some("north"));
    assert_eq!(trapdoor.property("half"), Some("bottom"));
}

#[test]
fn chests_pair_along_their_axis() {
    let world = TestWorld::default();
    let single = place("minecraft:chest", &context(), &world);
    assert_eq!(single.property("facing"), Some("north"));
    assert_eq!(single.property("type"), Some("single"));

    // A single chest with the same facing sits to the east (player's
    // clockwise from north facing is east): new chest becomes the LEFT half.
    let east_neighbor = TestWorld::default().with(
        (1, 64, 0),
        block("minecraft:chest").try_set_property("facing", "north"),
    );
    let paired = place("minecraft:chest", &context(), &east_neighbor);
    assert_eq!(paired.property("type"), Some("left"));

    let west_neighbor = TestWorld::default().with(
        (-1, 64, 0),
        block("minecraft:chest").try_set_property("facing", "north"),
    );
    let paired = place("minecraft:chest", &context(), &west_neighbor);
    assert_eq!(paired.property("type"), Some("right"));

    // Sneak-placing avoids pairing.
    let sneak = PlaceContext {
        secondary_use_active: true,
        ..context()
    };
    let lone = place("minecraft:chest", &sneak, &east_neighbor);
    assert_eq!(lone.property("type"), Some("single"));
}

#[test]
fn attached_blocks_probe_survival() {
    // Ladder against a wall: clicking the north face of a stone block to the
    // south attaches the ladder facing north.
    let world = TestWorld::default().with((0, 64, 1), block("minecraft:stone"));
    let side = PlaceContext {
        clicked_face: Direction::North,
        ..context()
    };
    let ladder = place("minecraft:ladder", &side, &world);
    assert_eq!(ladder.property("facing"), Some("north"));

    // No wall anywhere: ladder placement rejects.
    assert_eq!(
        state_for_placement("minecraft:ladder", &side, &TestWorld::default()),
        Some(PlacementOutcome::Reject)
    );

    // Lever on the floor.
    let floor_world = TestWorld::default().with((0, 63, 0), block("minecraft:stone"));
    let down_look = PlaceContext {
        player_pitch: 89.0,
        ..context()
    };
    let lever = place("minecraft:lever", &down_look, &floor_world);
    assert_eq!(lever.property("face"), Some("floor"));

    // Bell on the floor.
    let bell = place("minecraft:bell", &context(), &floor_world);
    assert_eq!(bell.property("attachment"), Some("floor"));

    // Standing lantern vs hanging lantern.
    let lantern = place("minecraft:lantern", &down_look, &floor_world);
    assert_eq!(lantern.property("hanging"), Some("false"));
    let ceiling_world = TestWorld::default().with((0, 65, 0), block("minecraft:stone"));
    let up_look = PlaceContext {
        player_pitch: -89.0,
        ..context()
    };
    let lantern = place("minecraft:lantern", &up_look, &ceiling_world);
    assert_eq!(lantern.property("hanging"), Some("true"));
}

#[test]
fn stacking_blocks_increment_their_segment_property() {
    let pickle_world = TestWorld::default().with((0, 64, 0), block("minecraft:sea_pickle"));
    let stacked = place("minecraft:sea_pickle", &context(), &pickle_world);
    assert_eq!(stacked.property("pickles"), Some("2"));

    let snow_world = TestWorld::default().with(
        (0, 64, 0),
        block("minecraft:snow").try_set_property("layers", "3"),
    );
    let stacked = place("minecraft:snow", &context(), &snow_world);
    assert_eq!(stacked.property("layers"), Some("4"));

    let candle_world = TestWorld::default().with((0, 64, 0), block("minecraft:candle"));
    let stacked = place("minecraft:candle", &context(), &candle_world);
    assert_eq!(stacked.property("candles"), Some("2"));

    let petal_world = TestWorld::default().with((0, 64, 0), block("minecraft:pink_petals"));
    let stacked = place("minecraft:pink_petals", &context(), &petal_world);
    assert_eq!(stacked.property("flower_amount"), Some("2"));
}

#[test]
fn waterlogged_placements_sample_replaced_fluid() {
    let water_world = TestWorld::default().with((0, 64, 0), block("minecraft:water"));
    for id in [
        "minecraft:oak_stairs",
        "minecraft:oak_slab",
        "minecraft:glass", // waterlogged_transparent has no waterlogged prop? skip below
    ] {
        if id == "minecraft:glass" {
            continue;
        }
        let placed = place(id, &context(), &water_world);
        assert_eq!(placed.property("waterlogged"), Some("true"), "{id}");
    }
    let chain = place("minecraft:iron_chain", &context(), &water_world);
    assert_eq!(chain.property("waterlogged"), Some("true"));
    assert_eq!(chain.property("axis"), Some("y"));
}

#[test]
fn unported_overrides_yield_none_and_plain_blocks_place_defaults() {
    let world = TestWorld::default();
    // Plain cube without an override: default state.
    assert_eq!(
        state_for_placement("minecraft:stone", &context(), &world),
        Some(PlacementOutcome::Place(block("minecraft:stone")))
    );
    // Unported overrides yield explicit None (caller keeps legacy).
    assert_eq!(
        state_for_placement("minecraft:kelp", &context(), &world),
        None
    );
    assert_eq!(
        state_for_placement("minecraft:big_dripleaf", &context(), &world),
        None
    );
    // Unknown block ids are not placeable.
    assert_eq!(
        state_for_placement("minecraft:not_a_block", &context(), &world),
        None
    );
}

#[test]
fn connection_scanning_families_link_like_java() {
    // Fence connects to a sturdy face and to fence gates rotated across it.
    let world = TestWorld::default().with((0, 64, 1), block("minecraft:stone"));
    let fence = place("minecraft:oak_fence", &context(), &world);
    assert_eq!(fence.property("south"), Some("true"));
    assert_eq!(fence.property("north"), Some("false"));
    // Leaves are connection exceptions.
    let leafy = TestWorld::default().with((0, 64, 1), block("minecraft:oak_leaves"));
    let fence = place("minecraft:oak_fence", &context(), &leafy);
    assert_eq!(fence.property("south"), Some("false"));
    // Wooden fences do not connect to nether brick fences.
    let nether = TestWorld::default().with((0, 64, 1), block("minecraft:nether_brick_fence"));
    let fence = place("minecraft:oak_fence", &context(), &nether);
    assert_eq!(fence.property("south"), Some("false"));

    // Panes attach to sturdy faces, other panes, and walls.
    let world = TestWorld::default()
        .with((0, 64, 1), block("minecraft:glass_pane"))
        .with((1, 64, 0), block("minecraft:cobblestone_wall"));
    let pane = place("minecraft:glass_pane", &context(), &world);
    assert_eq!(pane.property("south"), Some("true"));
    assert_eq!(pane.property("east"), Some("true"));
    assert_eq!(pane.property("west"), Some("false"));

    // Walls: a sturdy side gives a LOW arm and a free-standing post stays up.
    let world = TestWorld::default().with((0, 64, 1), block("minecraft:stone"));
    let wall = place("minecraft:cobblestone_wall", &context(), &world);
    assert_eq!(wall.property("south"), Some("low"));
    assert_eq!(wall.property("north"), Some("none"));
    assert_eq!(wall.property("up"), Some("true"));
    // A wall sandwiched between two sides with a full block above raises TALL
    // arms and drops the post.
    let world = TestWorld::default()
        .with((0, 64, 1), block("minecraft:stone"))
        .with((0, 64, -1), block("minecraft:stone"))
        .with((0, 65, 0), block("minecraft:stone"));
    let wall = place("minecraft:cobblestone_wall", &context(), &world);
    assert_eq!(wall.property("south"), Some("tall"));
    assert_eq!(wall.property("north"), Some("tall"));
    assert_eq!(wall.property("up"), Some("false"));

    // Fence gates notice flanking walls.
    let world = TestWorld::default().with((1, 64, 0), block("minecraft:cobblestone_wall"));
    let gate = place("minecraft:oak_fence_gate", &context(), &world);
    assert_eq!(gate.property("in_wall"), Some("true"));
    assert_eq!(gate.property("facing"), Some("south"));
}

#[test]
fn batch_two_simple_families_match_java() {
    let world = TestWorld::default();
    // Rails align with the player.
    assert_eq!(
        place("minecraft:rail", &context(), &world).property("shape"),
        Some("north_south")
    );
    let east = PlaceContext {
        player_yaw: 90.0,
        ..context()
    };
    assert_eq!(
        place("minecraft:rail", &east, &world).property("shape"),
        Some("east_west")
    );

    // Skulls use yaw segments without the banner 180-degree flip.
    assert_eq!(
        place("minecraft:skeleton_skull", &context(), &world).property("rotation"),
        Some("0")
    );

    // Fire over soul sand becomes soul fire; over stone it is plain fire.
    let soul = TestWorld::default().with((0, 63, 0), block("minecraft:soul_sand"));
    let placed = place("minecraft:fire", &context(), &soul);
    assert_eq!(placed.registry_id, "minecraft:soul_fire");
    let stone = TestWorld::default().with((0, 63, 0), block("minecraft:stone"));
    assert_eq!(
        place("minecraft:fire", &context(), &stone).registry_id,
        "minecraft:fire"
    );
    // Floating next to planks: fire keeps side faces toward burnables.
    let side_fuel = TestWorld::default().with((0, 64, 1), block("minecraft:oak_planks"));
    let placed = place("minecraft:fire", &context(), &side_fuel);
    assert_eq!(placed.property("south"), Some("true"));
    assert_eq!(placed.property("north"), Some("false"));

    // Leaves place persistent with recomputed distance.
    let lone = place("minecraft:oak_leaves", &context(), &world);
    assert_eq!(lone.property("persistent"), Some("true"));
    assert_eq!(lone.property("distance"), Some("7"));
    let logged = TestWorld::default().with((0, 64, 1), block("minecraft:oak_log"));
    assert_eq!(
        place("minecraft:oak_leaves", &context(), &logged).property("distance"),
        Some("1")
    );

    // Farmland/dirt path convert to dirt when unsupportable (solid above).
    let capped = TestWorld::default().with((0, 65, 0), block("minecraft:stone"));
    assert_eq!(
        place("minecraft:farmland", &context(), &capped).registry_id,
        "minecraft:dirt"
    );
    assert_eq!(
        place("minecraft:farmland", &context(), &world).registry_id,
        "minecraft:farmland"
    );

    // Concrete powder solidifies in water.
    let wet = TestWorld::default().with((0, 64, 0), block("minecraft:water"));
    assert_eq!(
        place("minecraft:red_concrete_powder", &context(), &wet).registry_id,
        "minecraft:red_concrete"
    );
    assert_eq!(
        place("minecraft:red_concrete_powder", &context(), &world).registry_id,
        "minecraft:red_concrete_powder"
    );

    // Scaffolding distance: grounded is 0, floating defaults to 7+bottom.
    let grounded = TestWorld::default().with((0, 63, 0), block("minecraft:stone"));
    let placed = place("minecraft:scaffolding", &context(), &grounded);
    assert_eq!(placed.property("distance"), Some("0"));
    assert_eq!(placed.property("bottom"), Some("false"));
    let floating = place("minecraft:scaffolding", &context(), &world);
    assert_eq!(floating.property("distance"), Some("7"));
    assert_eq!(floating.property("bottom"), Some("true"));

    // Grass blocks sample SNOWY from the block above.
    let snowy = TestWorld::default().with((0, 65, 0), block("minecraft:snow"));
    assert_eq!(
        place("minecraft:grass_block", &context(), &snowy).property("snowy"),
        Some("true")
    );
    assert_eq!(
        place("minecraft:grass_block", &context(), &world).property("snowy"),
        Some("false")
    );

    // Crafter orientation from front and top.
    let steep = PlaceContext {
        player_pitch: 80.0,
        ..context()
    };
    assert_eq!(
        place("minecraft:crafter", &steep, &world).property("orientation"),
        Some("up_south")
    );
    assert_eq!(
        place("minecraft:jigsaw", &context(), &world).property("orientation"),
        Some("up_north")
    );

    // Tripwire links to hooks facing back at it.
    let hooked = TestWorld::default().with(
        (0, 64, 1),
        block("minecraft:tripwire_hook").try_set_property("facing", "north"),
    );
    assert_eq!(
        place("minecraft:tripwire", &context(), &hooked).property("south"),
        Some("true")
    );
}
