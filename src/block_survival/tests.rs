use std::collections::HashMap;

use super::*;

/// HashMap-backed world: unspecified positions are air, brightness defaults to
/// full daylight unless overridden.
#[derive(Default)]
struct TestWorld {
    blocks: HashMap<(i32, i32, i32), BlockStateModel>,
    brightness: Option<i32>,
}

impl TestWorld {
    fn with(mut self, pos: (i32, i32, i32), state: BlockStateModel) -> Self {
        self.blocks.insert(pos, state);
        self
    }

    fn dark(mut self) -> Self {
        self.brightness = Some(0);
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
        self.brightness.unwrap_or(15)
    }
}

const POS: BlockPos = BlockPos { x: 0, y: 64, z: 0 };
const BELOW: (i32, i32, i32) = (0, 63, 0);
const ABOVE: (i32, i32, i32) = (0, 65, 0);

fn block(id: &str) -> BlockStateModel {
    BlockStateModel::default_for(id).unwrap_or_else(|| panic!("unknown block {id}"))
}

fn survives_on(state: &BlockStateModel, ground: &str) -> bool {
    let world = TestWorld::default().with(BELOW, block(ground));
    can_survive(state, POS, &world)
}

#[test]
fn torch_family_requires_center_support_below() {
    // Java BaseTorchBlock: canSupportCenter(below, UP).
    let torch = block("minecraft:torch");
    assert!(survives_on(&torch, "minecraft:stone"));
    // Hopper tops are only RIGID-sturdy (the bowl breaks CENTER support), so
    // torches cannot stand on hoppers in vanilla.
    assert!(!survives_on(&torch, "minecraft:hopper"));
    assert!(!can_survive(&torch, POS, &TestWorld::default()));
    // A bottom slab's top face supports nothing; a top slab's does.
    assert!(!survives_on(
        &torch,
        "minecraft:oak_slab" // default type=bottom
    ));
    let world = TestWorld::default().with(
        BELOW,
        BlockStateModel::default_for("minecraft:oak_slab")
            .unwrap()
            .try_set_property("type", "top"),
    );
    assert!(can_survive(&torch, POS, &world));

    assert!(survives_on(&block("minecraft:candle"), "minecraft:stone"));
    assert!(!can_survive(
        &block("minecraft:candle"),
        POS,
        &TestWorld::default()
    ));
}

#[test]
fn wall_attached_blocks_require_sturdy_backing_face() {
    // Wall torch facing north hangs on the block to its south.
    let wall_torch = BlockStateModel::default_for("minecraft:wall_torch").unwrap();
    let facing = facing_property(&wall_torch).unwrap();
    let behind = POS.relative(facing.opposite());
    let world = TestWorld::default().with((behind.x, behind.y, behind.z), block("minecraft:stone"));
    assert!(can_survive(&wall_torch, POS, &world));
    assert!(!can_survive(&wall_torch, POS, &TestWorld::default()));

    // Ladders and tripwire hooks follow the same shape.
    let ladder = BlockStateModel::default_for("minecraft:ladder").unwrap();
    assert!(can_survive(&ladder, POS, &world));
    assert!(!can_survive(&ladder, POS, &TestWorld::default()));
    let hook = BlockStateModel::default_for("minecraft:tripwire_hook").unwrap();
    assert!(can_survive(&hook, POS, &world));
    assert!(!can_survive(&hook, POS, &TestWorld::default()));
}

#[test]
fn ground_cover_families_follow_java_rules() {
    // Carpet: anything non-air below.
    let carpet = block("minecraft:white_carpet");
    assert!(survives_on(&carpet, "minecraft:stone"));
    assert!(survives_on(&carpet, "minecraft:oak_slab"));
    assert!(!can_survive(&carpet, POS, &TestWorld::default()));

    // Banner/sign/cake: below isSolid.
    for id in [
        "minecraft:white_banner",
        "minecraft:oak_sign",
        "minecraft:cake",
    ] {
        let state = block(id);
        assert!(survives_on(&state, "minecraft:stone"), "{id}");
        assert!(!survives_on(&state, "minecraft:torch"), "{id}");
    }

    // Pressure plates: rigid or center support.
    let plate = block("minecraft:stone_pressure_plate");
    assert!(survives_on(&plate, "minecraft:stone"));
    assert!(survives_on(&plate, "minecraft:hopper")); // rigid top ring
    assert!(!can_survive(&plate, POS, &TestWorld::default()));

    // Rails: rigid only.
    let rail = block("minecraft:rail");
    assert!(survives_on(&rail, "minecraft:stone"));
    assert!(survives_on(&rail, "minecraft:hopper"));
    assert!(!survives_on(&rail, "minecraft:torch"));

    // Redstone wire: sturdy top OR a hopper.
    let wire = block("minecraft:redstone_wire");
    assert!(survives_on(&wire, "minecraft:stone"));
    assert!(survives_on(&wire, "minecraft:hopper"));
    assert!(!survives_on(&wire, "minecraft:torch"));
}

#[test]
fn vegetation_and_crops_use_data_driven_tags() {
    let grass = block("minecraft:short_grass");
    assert!(survives_on(&grass, "minecraft:grass_block"));
    assert!(survives_on(&grass, "minecraft:dirt"));
    assert!(!survives_on(&grass, "minecraft:stone"));

    // Crops need light >= 8 and supports_crops ground.
    let wheat = block("minecraft:wheat");
    assert!(survives_on(&wheat, "minecraft:farmland"));
    assert!(!survives_on(&wheat, "minecraft:dirt"));
    let dark = TestWorld::default()
        .with(BELOW, block("minecraft:farmland"))
        .dark();
    assert!(!can_survive(&wheat, POS, &dark));

    // Nether vegetation tags.
    assert!(survives_on(
        &block("minecraft:nether_wart"),
        "minecraft:soul_sand"
    ));
    assert!(!survives_on(
        &block("minecraft:nether_wart"),
        "minecraft:dirt"
    ));
    assert!(survives_on(
        &block("minecraft:crimson_fungus"),
        "minecraft:crimson_nylium"
    ));

    // Dry vegetation.
    assert!(survives_on(&block("minecraft:dead_bush"), "minecraft:sand"));

    // Wither rose has its own support tag (includes netherrack).
    assert!(survives_on(
        &block("minecraft:wither_rose"),
        "minecraft:netherrack"
    ));
    assert!(!survives_on(
        &block("minecraft:short_grass"),
        "minecraft:netherrack"
    ));

    // Bamboo.
    assert!(survives_on(&block("minecraft:bamboo"), "minecraft:gravel"));
    assert!(!survives_on(&block("minecraft:bamboo"), "minecraft:stone"));

    // Soul fire.
    assert!(survives_on(
        &block("minecraft:soul_fire"),
        "minecraft:soul_sand"
    ));
    assert!(!survives_on(
        &block("minecraft:soul_fire"),
        "minecraft:stone"
    ));
}

#[test]
fn door_and_double_plant_halves_check_their_counterparts() {
    let lower_door = block("minecraft:oak_door");
    assert_eq!(lower_door.property("half"), Some("lower"));
    assert!(survives_on(&lower_door, "minecraft:stone"));
    assert!(!survives_on(&lower_door, "minecraft:torch"));

    let upper_door = lower_door.clone().try_set_property("half", "upper");
    let world = TestWorld::default().with(BELOW, lower_door.clone());
    assert!(can_survive(&upper_door, POS, &world));
    let world = TestWorld::default().with(BELOW, block("minecraft:stone"));
    assert!(!can_survive(&upper_door, POS, &world));

    let lower_plant = block("minecraft:sunflower");
    let upper_plant = lower_plant.clone().try_set_property("half", "upper");
    let world = TestWorld::default().with(BELOW, lower_plant.clone());
    assert!(can_survive(&upper_plant, POS, &world));
    let world = TestWorld::default().with(BELOW, block("minecraft:grass_block"));
    assert!(can_survive(&lower_plant, POS, &world));
    assert!(!can_survive(&upper_plant, POS, &world));
}

#[test]
fn snow_layers_follow_collision_face_and_override_tags() {
    let snow = block("minecraft:snow");
    assert!(survives_on(&snow, "minecraft:stone"));
    // Top slab seals upward... a BOTTOM slab does not support snow.
    assert!(!survives_on(&snow, "minecraft:oak_slab"));
    // support_override_snow_layer: honey/soul sand/mud despite partial tops.
    assert!(survives_on(&snow, "minecraft:honey_block"));
    assert!(survives_on(&snow, "minecraft:soul_sand"));
    // cannot_support_snow_layer: both ice variants reject snow in 26.1.2.
    assert!(!survives_on(&snow, "minecraft:ice"));
    assert!(!survives_on(&snow, "minecraft:packed_ice"));
    // Stacking on full eight-layer snow.
    let full = block("minecraft:snow").try_set_property("layers", "8");
    let world = TestWorld::default().with(BELOW, full);
    assert!(can_survive(&snow, POS, &world));
}

#[test]
fn aquatic_blocks_check_fluids_and_support_faces() {
    // Sea pickle on any upward-facing collision surface.
    assert!(survives_on(
        &block("minecraft:sea_pickle"),
        "minecraft:stone"
    ));
    assert!(!survives_on(
        &block("minecraft:sea_pickle"),
        "minecraft:torch"
    ));

    // Lily pad over source water with nothing in its own cell.
    let pad = block("minecraft:lily_pad");
    assert!(survives_on(&pad, "minecraft:water"));
    assert!(!survives_on(&pad, "minecraft:stone"));

    // Frogspawn needs source water below.
    assert!(survives_on(
        &block("minecraft:frogspawn"),
        "minecraft:water"
    ));
    assert!(!survives_on(
        &block("minecraft:frogspawn"),
        "minecraft:stone"
    ));

    // Seagrass: sturdy non-excluded ground.
    assert!(survives_on(
        &block("minecraft:seagrass"),
        "minecraft:gravel"
    ));
    assert!(!survives_on(
        &block("minecraft:seagrass"),
        "minecraft:magma_block"
    ));

    // Kelp attaches to sturdy ground or its own plant, never magma.
    assert!(survives_on(&block("minecraft:kelp"), "minecraft:stone"));
    assert!(survives_on(
        &block("minecraft:kelp"),
        "minecraft:kelp_plant"
    ));
    assert!(!survives_on(
        &block("minecraft:kelp"),
        "minecraft:magma_block"
    ));
}

#[test]
fn hanging_and_attached_blocks_check_their_anchors() {
    // Hanging roots / ceiling hanging signs / spore blossom anchor above.
    for (id, ground) in [
        ("minecraft:hanging_roots", "minecraft:stone"),
        ("minecraft:oak_hanging_sign", "minecraft:stone"),
        ("minecraft:spore_blossom", "minecraft:stone"),
    ] {
        let state = block(id);
        let world = TestWorld::default().with(ABOVE, block(ground));
        assert!(can_survive(&state, POS, &world), "{id}");
        assert!(!can_survive(&state, POS, &TestWorld::default()), "{id}");
    }

    // Cocoa pods hang on jungle logs along their facing.
    let cocoa = block("minecraft:cocoa");
    let facing = facing_property(&cocoa).unwrap();
    let anchor = POS.relative(facing);
    let world = TestWorld::default().with(
        (anchor.x, anchor.y, anchor.z),
        block("minecraft:jungle_log"),
    );
    assert!(can_survive(&cocoa, POS, &world));
    let world =
        TestWorld::default().with((anchor.x, anchor.y, anchor.z), block("minecraft:oak_log"));
    assert!(!can_survive(&cocoa, POS, &world));

    // Amethyst clusters attach along FACING (default up -> needs block below).
    let cluster = block("minecraft:amethyst_cluster");
    assert!(survives_on(&cluster, "minecraft:budding_amethyst"));
    assert!(!can_survive(&cluster, POS, &TestWorld::default()));

    // Lanterns: standing needs center support below, hanging needs it above.
    let lantern = block("minecraft:lantern");
    assert!(survives_on(&lantern, "minecraft:stone"));
    let hanging = lantern.clone().try_set_property("hanging", "true");
    let world = TestWorld::default().with(ABOVE, block("minecraft:stone"));
    assert!(can_survive(&hanging, POS, &world));
    assert!(!can_survive(&hanging, POS, &TestWorld::default()));
}

#[test]
fn multiface_scaffolding_and_special_blocks() {
    // Glow lichen: every set face must attach to a full face.
    let lichen = BlockStateModel::default_for("minecraft:glow_lichen")
        .unwrap()
        .try_set_property("down", "true");
    assert!(survives_on(&lichen, "minecraft:stone"));
    assert!(!can_survive(&lichen, POS, &TestWorld::default()));
    // Default lichen has no faces set -> cannot survive anywhere.
    let no_face = BlockStateModel::default_for("minecraft:glow_lichen").unwrap();
    assert!(!survives_on(&no_face, "minecraft:stone"));

    // Scaffolding by distance property (the DEFAULT state is distance=7,
    // which does not survive; grounded scaffolding has distance 0).
    let far = block("minecraft:scaffolding");
    assert!(!can_survive(&far, POS, &TestWorld::default()));
    let near = far.clone().try_set_property("distance", "0");
    assert!(can_survive(&near, POS, &TestWorld::default()));

    // Grindstones always survive.
    assert!(can_survive(
        &block("minecraft:grindstone"),
        POS,
        &TestWorld::default()
    ));

    // Mushrooms: light-gated unless on mycelium-like ground.
    let mushroom = block("minecraft:red_mushroom");
    assert!(survives_on(&mushroom, "minecraft:mycelium"));
    assert!(!survives_on(&mushroom, "minecraft:stone")); // brightness 15
    let dark = TestWorld::default()
        .with(BELOW, block("minecraft:stone"))
        .dark();
    assert!(can_survive(&mushroom, POS, &dark));

    // Cactus: clear horizontal neighbors + supports_cactus ground.
    let cactus = block("minecraft:cactus");
    assert!(survives_on(&cactus, "minecraft:sand"));
    assert!(survives_on(&cactus, "minecraft:cactus"));
    assert!(!survives_on(&cactus, "minecraft:dirt"));
    let crowded = TestWorld::default()
        .with(BELOW, block("minecraft:sand"))
        .with((1, 64, 0), block("minecraft:stone"));
    assert!(!can_survive(&cactus, POS, &crowded));

    // Sugar cane: own column, or supported ground with adjacent water.
    let cane = block("minecraft:sugar_cane");
    assert!(survives_on(&cane, "minecraft:sugar_cane"));
    assert!(!survives_on(&cane, "minecraft:sand"));
    let watered = TestWorld::default()
        .with(BELOW, block("minecraft:sand"))
        .with((1, 63, 0), block("minecraft:water"));
    assert!(can_survive(&cane, POS, &watered));

    // Fire: sturdy ground or a burnable neighbor.
    let fire = block("minecraft:fire");
    assert!(survives_on(&fire, "minecraft:stone"));
    assert!(!can_survive(&fire, POS, &TestWorld::default()));
    let beside_planks = TestWorld::default().with((1, 64, 0), block("minecraft:oak_planks"));
    assert!(can_survive(&fire, POS, &beside_planks));

    // Default rule: ordinary blocks always survive.
    assert!(can_survive(
        &block("minecraft:stone"),
        POS,
        &TestWorld::default()
    ));
}

#[test]
#[allow(clippy::too_many_lines)] // one entry per Java canSurvive override
fn every_java_can_survive_override_has_a_dispatch_rule() {
    // One entry per Java class with a canSurvive override in 26.1.2 (grep
    // `boolean canSurvive` under net/minecraft/world/level/block), mapped to
    // the block-type keys our dispatch handles. GrindstoneBlock overrides to
    // a constant `true` and BlockBehaviour is the default.
    let java_overrides: &[(&str, &[&str])] = &[
        ("AmethystClusterBlock", &["amethyst_cluster"]),
        ("BambooSaplingBlock", &["bamboo_sapling"]),
        ("BambooStalkBlock", &["bamboo_stalk"]),
        ("BannerBlock", &["banner"]),
        (
            "BaseCoralPlantTypeBlock",
            &[
                "base_coral_plant",
                "coral",
                "coral_plant",
                "base_coral_fan",
                "coral_fan",
            ],
        ),
        (
            "BaseCoralWallFanBlock",
            &["base_coral_wall_fan", "coral_wall_fan"],
        ),
        (
            "BasePressurePlateBlock",
            &["pressure_plate", "weighted_pressure_plate"],
        ),
        ("BaseRailBlock", &["rail", "powered_rail", "detector_rail"]),
        ("BaseTorchBlock", &["torch", "redstone_torch"]),
        ("BellBlock", &["bell"]),
        ("BigDripleafBlock", &["big_dripleaf"]),
        ("BigDripleafStemBlock", &["big_dripleaf_stem"]),
        ("BubbleColumnBlock", &["bubble_column"]),
        ("CactusBlock", &["cactus"]),
        ("CakeBlock", &["cake"]),
        ("CandleBlock", &["candle"]),
        ("CandleCakeBlock", &["candle_cake"]),
        ("CarpetBlock", &["carpet", "wool_carpet"]),
        ("CeilingHangingSignBlock", &["ceiling_hanging_sign"]),
        ("ChorusFlowerBlock", &["chorus_flower"]),
        ("ChorusPlantBlock", &["chorus_plant"]),
        ("CocoaBlock", &["cocoa"]),
        (
            "CropBlock",
            &["crop", "carrot", "potato", "beetroot", "torchflower_crop"],
        ),
        ("DiodeBlock", &["repeater", "comparator"]),
        ("DirtPathBlock", &["dirt_path"]),
        ("DoorBlock", &["door", "weathering_copper_door"]),
        (
            "DoublePlantBlock",
            &["double_plant", "tall_flower", "tall_grass"],
        ),
        (
            "FaceAttachedHorizontalDirectionalBlock",
            &["button", "lever"],
        ),
        ("FarmlandBlock", &["farmland"]),
        ("FireBlock", &["fire"]),
        ("FrogspawnBlock", &["frogspawn"]),
        ("GrindstoneBlock", &["grindstone"]),
        (
            "GrowingPlantBlock",
            &[
                "kelp",
                "kelp_plant",
                "twisting_vines",
                "twisting_vines_plant",
                "cave_vines",
                "cave_vines_plant",
                "weeping_vines",
                "weeping_vines_plant",
            ],
        ),
        ("HangingMossBlock", &["hanging_moss"]),
        ("HangingRootsBlock", &["hanging_roots"]),
        ("LadderBlock", &["ladder"]),
        ("LanternBlock", &["lantern", "weathering_lantern"]),
        ("LeafLitterBlock", &["leaf_litter"]),
        ("MangrovePropaguleBlock", &["mangrove_propagule"]),
        ("MossyCarpetBlock", &["mossy_carpet"]),
        (
            "MultifaceBlock",
            &["multiface", "glow_lichen", "sculk_vein"],
        ),
        ("MushroomBlock", &["mushroom"]),
        ("PistonHeadBlock", &["piston_head"]),
        ("PitcherCropBlock", &["pitcher_crop"]),
        ("PointedDripstoneBlock", &["pointed_dripstone"]),
        ("RedStoneWireBlock", &["redstone_wire"]),
        ("RedstoneWallTorchBlock", &["redstone_wall_torch"]),
        ("ScaffoldingBlock", &["scaffolding"]),
        ("SeaPickleBlock", &["sea_pickle"]),
        ("SmallDripleafBlock", &["small_dripleaf"]),
        ("SnowLayerBlock", &["snow_layer"]),
        ("SoulFireBlock", &["soul_fire"]),
        ("SporeBlossomBlock", &["spore_blossom"]),
        ("StandingSignBlock", &["standing_sign"]),
        ("SugarCaneBlock", &["sugar_cane"]),
        ("TallSeagrassBlock", &["tall_seagrass"]),
        ("TripWireHookBlock", &["trip_wire_hook"]),
        (
            "VegetationBlock",
            &[
                "grass",
                "bush",
                "flower",
                "flower_bed",
                "cactus_flower",
                "eyeblossom",
                "firefly_bush",
                "sapling",
                "azalea",
                "lily_pad",
                "mangrove_roots",
                "sweet_berry_bush",
                "attached_stem",
                "dry_vegetation",
                "short_dry_grass",
                "tall_dry_grass",
                "nether_roots",
                "nether_sprouts",
                "nether_fungus",
                "nether_wart",
                "seagrass",
                "stem",
                "wither_rose",
            ],
        ),
        ("VineBlock", &["vine"]),
        ("WallBannerBlock", &["wall_banner"]),
        ("WallSignBlock", &["wall_sign"]),
        ("WallTorchBlock", &["wall_torch"]),
    ];
    assert_eq!(java_overrides.len(), 62);

    // Every block type named above must actually exist in the registry.
    let known_types: std::collections::HashSet<&str> = crate::block_states::block_state_entries()
        .iter()
        .map(|entry| entry.block_type)
        .collect();
    for (java_class, block_types) in java_overrides {
        for block_type in *block_types {
            assert!(
                known_types.contains(block_type),
                "{java_class}: unknown block type {block_type}"
            );
        }
    }
}
