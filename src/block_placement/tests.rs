use std::collections::HashMap;

use super::*;

const USE_ON_CONTEXT_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/world/item/context/UseOnContext.java"
);
const BLOCK_PLACE_CONTEXT_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/world/item/context/BlockPlaceContext.java"
);
const DIRECTIONAL_PLACE_CONTEXT_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/world/item/context/DirectionalPlaceContext.java"
);

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
        random_age_roll: 0,
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
fn item_context_construction_and_relocation_match_java() {
    for sentinel in [
        "public BlockPos getClickedPos()",
        "public Direction getClickedFace()",
        "public Vec3 getClickLocation()",
        "public boolean isInside()",
        "public Direction getHorizontalDirection()",
        "public boolean isSecondaryUseActive()",
        "public float getRotation()",
    ] {
        assert!(
            USE_ON_CONTEXT_JAVA.contains(sentinel),
            "missing UseOnContext sentinel {sentinel}"
        );
    }
    for sentinel in [
        "this.relativePos = hitResult.getBlockPos().relative(hitResult.getDirection());",
        "this.replaceClicked = level.getBlockState(hitResult.getBlockPos()).canBeReplaced(this);",
        "return this.replaceClicked ? super.getClickedPos() : this.relativePos;",
        "return this.replaceClicked || this.getLevel().getBlockState(this.getClickedPos()).canBeReplaced(this);",
        "System.arraycopy(directions, 0, directions, 1, index);",
        "directions[0] = clickedFace.getOpposite();",
    ] {
        assert!(
            BLOCK_PLACE_CONTEXT_JAVA.contains(sentinel),
            "missing BlockPlaceContext sentinel {sentinel}"
        );
    }

    let world = TestWorld::default().with((0, 64, 0), block("minecraft:stone"));
    let context = PlaceContext::from_use_on(
        &world,
        POS,
        Direction::Up,
        [0.25, 64.75, 0.25],
        90.0,
        10.0,
        true,
        7,
    );
    assert_eq!(context.clicked_pos, POS.relative(Direction::Up));
    assert_eq!(context.clicked_face, Direction::Up);
    assert_eq!(context.click_location, [0.25, 64.75, 0.25]);
    assert!(!context.replacing_clicked_on_block);
    assert_eq!(context.horizontal_direction(), Direction::West);
    assert!(context.secondary_use_active);
    assert_eq!(context.player_yaw, 90.0);

    let replaceable = PlaceContext::from_use_on(
        &TestWorld::default(),
        POS,
        Direction::North,
        [0.5, 64.5, 0.0],
        0.0,
        0.0,
        false,
        0,
    );
    assert_eq!(replaceable.clicked_pos, POS);
    assert!(replaceable.replacing_clicked_on_block);

    let redirected_world = TestWorld::default().with((4, 70, -2), block("minecraft:stone"));
    let redirected = context.at(
        &redirected_world,
        BlockPos { x: 4, y: 70, z: -2 },
        Direction::East,
    );
    assert_eq!(redirected.click_location, [5.0, 70.5, -1.5]);
    assert_eq!(redirected.clicked_pos, BlockPos { x: 5, y: 70, z: -2 });
}

#[test]
fn block_place_context_direction_orders_match_java_non_replace_branch() {
    let context = PlaceContext {
        clicked_face: Direction::North,
        replacing_clicked_on_block: false,
        player_pitch: 0.0,
        player_yaw: 0.0,
        ..context()
    };
    assert_eq!(
        context.block_place_nearest_looking_directions()[0],
        Direction::South,
        "clicked face opposite must move to the front when clicked block is not replaced"
    );

    let replace = PlaceContext {
        replacing_clicked_on_block: true,
        ..context
    };
    assert_eq!(
        replace.block_place_nearest_looking_directions(),
        replace.nearest_looking_directions()
    );
}

#[test]
fn directional_place_context_orders_rotation_and_flags_match_java() {
    for sentinel in [
        "new BlockHitResult(Vec3.atBottomCenterOf(pos), clickedFace, pos, false)",
        "return this.getHitResult().getBlockPos();",
        "return this.getLevel().getBlockState(this.getHitResult().getBlockPos()).canBeReplaced(this);",
        "return Direction.DOWN;",
        "return new Direction[]{Direction.DOWN, Direction.EAST, Direction.SOUTH, Direction.UP, Direction.NORTH, Direction.WEST};",
        "return this.direction.getAxis() == Direction.Axis.Y ? Direction.NORTH : this.direction;",
        "return false;",
        "return this.direction.get2DDataValue() * 90;",
    ] {
        assert!(
            DIRECTIONAL_PLACE_CONTEXT_JAVA.contains(sentinel),
            "missing DirectionalPlaceContext sentinel {sentinel}"
        );
    }

    assert_eq!(
        PlaceContext::directional_nearest_looking_directions(Direction::East),
        [
            Direction::Down,
            Direction::East,
            Direction::South,
            Direction::Up,
            Direction::North,
            Direction::West,
        ]
    );
    assert_eq!(
        PlaceContext::directional_nearest_looking_directions(Direction::Up),
        [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    );

    let context = PlaceContext::directional(&TestWorld::default(), POS, Direction::East, Direction::Up);
    assert_eq!(context.clicked_pos, POS);
    assert_eq!(context.click_location, [0.5, 64.0, 0.5]);
    assert!(context.replacing_clicked_on_block);
    assert_eq!(context.horizontal_direction(), Direction::East);
    assert!(!context.secondary_use_active);
    assert_eq!(context.player_yaw, 270.0);

    let vertical = PlaceContext::directional(&TestWorld::default(), POS, Direction::Up, Direction::North);
    assert_eq!(vertical.horizontal_direction(), Direction::North);
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
    assert_eq!(
        state_for_placement("minecraft:crafting_table", &context(), &world),
        Some(PlacementOutcome::Place(block("minecraft:crafting_table")))
    );
    // Unported overrides yield explicit None (caller keeps legacy).
    assert_eq!(
        state_for_placement("minecraft:redstone_wire", &context(), &world),
        None
    );
    assert_eq!(
        state_for_placement("minecraft:copper_chest", &context(), &world),
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

#[test]
#[allow(clippy::cognitive_complexity, clippy::too_many_lines)] // sequential family pins
fn final_batch_plant_families_match_java() {
    let world = TestWorld::default();

    // Bamboo: rejects without supports_bamboo ground, grows from existing.
    assert_eq!(
        state_for_placement("minecraft:bamboo", &context(), &world),
        Some(PlacementOutcome::Reject)
    );
    let sandy = TestWorld::default().with((0, 63, 0), block("minecraft:sand"));
    assert_eq!(
        place("minecraft:bamboo", &context(), &sandy).registry_id,
        "minecraft:bamboo_sapling"
    );
    let stalked = TestWorld::default().with(
        (0, 63, 0),
        block("minecraft:bamboo").try_set_property("age", "1"),
    );
    let placed = place("minecraft:bamboo", &context(), &stalked);
    assert_eq!(placed.registry_id, "minecraft:bamboo");
    assert_eq!(placed.property("age"), Some("1"));

    // Corals waterlog only in full water.
    let dry = place("minecraft:tube_coral", &context(), &world);
    assert_eq!(dry.property("waterlogged"), Some("false"));
    let wet = TestWorld::default().with((0, 64, 0), block("minecraft:water"));
    assert_eq!(
        place("minecraft:tube_coral", &context(), &wet).property("waterlogged"),
        Some("true")
    );

    // Chorus plant connects toward flowers and end stone below.
    let chorus_world = TestWorld::default()
        .with((0, 63, 0), block("minecraft:end_stone"))
        .with((0, 65, 0), block("minecraft:chorus_flower"));
    let chorus = place("minecraft:chorus_plant", &context(), &chorus_world);
    assert_eq!(chorus.property("down"), Some("true"));
    assert_eq!(chorus.property("up"), Some("true"));
    assert_eq!(chorus.property("north"), Some("false"));

    // Double plants reject when the upper half is blocked.
    let blocked = TestWorld::default().with((0, 65, 0), block("minecraft:stone"));
    assert_eq!(
        state_for_placement("minecraft:sunflower", &context(), &blocked),
        Some(PlacementOutcome::Reject)
    );
    assert_eq!(
        place("minecraft:sunflower", &context(), &world).property("half"),
        Some("lower")
    );

    // Huge mushroom blocks face away from their own kind.
    let capped = TestWorld::default().with((0, 65, 0), block("minecraft:red_mushroom_block"));
    let mushroom = place("minecraft:red_mushroom_block", &context(), &capped);
    assert_eq!(mushroom.property("up"), Some("false"));
    assert_eq!(mushroom.property("down"), Some("true"));

    // Vines: attach the looked-at face when supported; reject in the open.
    let walled = TestWorld::default().with((0, 64, 1), block("minecraft:stone"));
    let vine = place("minecraft:vine", &context(), &walled);
    assert_eq!(vine.property("south"), Some("true"));
    assert_eq!(
        state_for_placement("minecraft:vine", &context(), &world),
        Some(PlacementOutcome::Reject)
    );

    // Glow lichen merges new faces into an existing block and waterlogs.
    let lichen_world = TestWorld::default()
        .with((0, 64, 1), block("minecraft:stone"))
        .with(
            (0, 64, 0),
            block("minecraft:glow_lichen").try_set_property("down", "true"),
        );
    let lichen = place("minecraft:glow_lichen", &context(), &lichen_world);
    assert_eq!(lichen.property("down"), Some("true"));
    assert_eq!(lichen.property("south"), Some("true"));

    // Growing-plant heads take the rolled age.
    let rolled = PlaceContext {
        random_age_roll: 13,
        ..context()
    };
    assert_eq!(
        place("minecraft:kelp", &rolled, &world).property("age"),
        Some("13")
    );

    // Pointed dripstone: looking down at a ceiling-less floor grows a
    // stalagmite (tip up); from a ceiling it hangs (tip down).
    let floor = TestWorld::default().with((0, 63, 0), block("minecraft:stone"));
    let looking_down = PlaceContext {
        player_pitch: 60.0,
        ..context()
    };
    let stalagmite = place("minecraft:pointed_dripstone", &looking_down, &floor);
    assert_eq!(stalagmite.property("vertical_direction"), Some("up"));
    assert_eq!(stalagmite.property("thickness"), Some("tip"));
    let ceiling = TestWorld::default().with((0, 65, 0), block("minecraft:stone"));
    let looking_up = PlaceContext {
        player_pitch: -60.0,
        ..context()
    };
    let stalactite = place("minecraft:pointed_dripstone", &looking_up, &ceiling);
    assert_eq!(stalactite.property("vertical_direction"), Some("down"));
    // Stacking onto an existing stalagmite tip makes the base a frustum chain.
    let stacked_world = TestWorld::default()
        .with((0, 63, 0), block("minecraft:stone"))
        .with(
            (0, 65, 0),
            block("minecraft:pointed_dripstone")
                .try_set_property("vertical_direction", "up")
                .try_set_property("thickness", "tip"),
        );
    let middle = place("minecraft:pointed_dripstone", &looking_down, &stacked_world);
    assert_eq!(middle.property("thickness"), Some("frustum"));

    // Wall hanging signs need a sturdy bar end perpendicular to the view
    // (a west-facing sign bar runs north-south).
    let barred = TestWorld::default().with((0, 64, -1), block("minecraft:stone"));
    let east_look = PlaceContext {
        player_yaw: -90.0,
        clicked_face: Direction::North,
        ..context()
    };
    let sign = place("minecraft:oak_wall_hanging_sign", &east_look, &barred);
    assert_eq!(sign.property("facing"), Some("west"));
    assert_eq!(
        state_for_placement("minecraft:oak_wall_hanging_sign", &east_look, &world),
        Some(PlacementOutcome::Reject)
    );

    // Ceiling hanging signs hang attached under narrow ceilings.
    let chains = TestWorld::default().with((0, 65, 0), block("minecraft:iron_chain"));
    let hung = place("minecraft:oak_hanging_sign", &context(), &chains);
    assert_eq!(hung.property("attached"), Some("true"));
    let slab_ceiling = TestWorld::default().with((0, 65, 0), block("minecraft:stone"));
    let hung = place("minecraft:oak_hanging_sign", &context(), &slab_ceiling);
    assert_eq!(hung.property("attached"), Some("false"));
}
