use super::super::*;

pub(super) fn assert_end_platform_and_gateway_support() {
    let end_platform = super::super::end_platform_blocks(BlockPos { x: 0, y: 64, z: 0 });
    assert_eq!(end_platform.len(), 100);
    assert_eq!(
        end_platform
            .iter()
            .filter(|block| block.state == "minecraft:obsidian")
            .count(),
        25
    );
    assert!(end_platform.contains(&super::super::FeaturePlacementBlock {
        pos: BlockPos {
            x: -2,
            y: 63,
            z: -2
        },
        state: "minecraft:obsidian",
    }));
    assert!(end_platform.contains(&super::super::FeaturePlacementBlock {
        pos: BlockPos { x: 2, y: 66, z: 2 },
        state: "minecraft:air",
    }));
    assert_eq!(
        super::super::void_start_platform_origin(64),
        BlockPos { x: 8, y: 67, z: 8 }
    );
    assert!(super::super::void_start_platform_applies_to_chunk(
        ChunkPos { x: 1, z: 1 }
    ));
    assert!(!super::super::void_start_platform_applies_to_chunk(
        ChunkPos { x: 2, z: 0 }
    ));
    let void_platform = super::super::void_start_platform_blocks(ChunkPos { x: 0, z: 0 }, 64);
    assert!(
        void_platform.contains(&super::super::FeaturePlacementBlock {
            pos: BlockPos { x: 8, y: 67, z: 8 },
            state: "minecraft:cobblestone",
        })
    );
    assert!(
        void_platform.contains(&super::super::FeaturePlacementBlock {
            pos: BlockPos { x: 0, y: 67, z: 0 },
            state: "minecraft:stone",
        })
    );
    assert_eq!(
        super::super::void_start_platform_blocks(ChunkPos { x: 2, z: 0 }, 64),
        Vec::new()
    );
    assert_eq!(
        super::super::end_gateway_known_exit(BlockPos { x: 1, y: 2, z: 3 }, true),
        super::super::EndGatewayConfigurationModel {
            exit: Some(BlockPos { x: 1, y: 2, z: 3 }),
            exact: true,
        }
    );
    assert_eq!(
        super::super::end_gateway_delayed_exit_search(),
        super::super::EndGatewayConfigurationModel {
            exit: None,
            exact: false,
        }
    );
    let gateway = super::super::end_gateway_blocks(BlockPos { x: 0, y: 64, z: 0 });
    assert_eq!(gateway.len(), 45);
    assert!(gateway.contains(&super::super::FeaturePlacementBlock {
        pos: BlockPos { x: 0, y: 64, z: 0 },
        state: "minecraft:end_gateway",
    }));
    assert!(gateway.contains(&super::super::FeaturePlacementBlock {
        pos: BlockPos { x: 0, y: 66, z: 0 },
        state: "minecraft:bedrock",
    }));
    assert!(gateway.contains(&super::super::FeaturePlacementBlock {
        pos: BlockPos { x: 1, y: 64, z: 0 },
        state: "minecraft:air",
    }));
    assert_eq!(
        gateway
            .iter()
            .filter(|block| block.state == "minecraft:bedrock")
            .count(),
        12
    );
}

pub(super) fn assert_chorus_and_end_podium_support() {
    assert_chorus_plant_support();
    assert_end_podium_support();
}

fn assert_chorus_plant_support() {
    assert!(super::super::chorus_plant_can_start(
        true,
        "minecraft:end_stone"
    ));
    assert!(!super::super::chorus_plant_can_start(
        false,
        "minecraft:end_stone"
    ));
    assert!(!super::super::chorus_plant_can_start(
        true,
        "minecraft:stone"
    ));
    assert!(super::super::chorus_all_horizontal_neighbors_empty(
        [true, false, true, true],
        Some(1),
    ));
    assert!(!super::super::chorus_all_horizontal_neighbors_empty(
        [true, false, true, true],
        None,
    ));
    assert!(super::super::chorus_branch_target_within_spread(
        BlockPos { x: 7, y: 68, z: -7 },
        BlockPos { x: 0, y: 64, z: 0 },
        8,
    ));
    assert!(!super::super::chorus_branch_target_within_spread(
        BlockPos { x: 8, y: 68, z: 0 },
        BlockPos { x: 0, y: 64, z: 0 },
        8,
    ));
    assert_eq!(super::super::chorus_trunk_height(0, 0), 2);
    assert_eq!(super::super::chorus_trunk_height(1, 3), 4);
    assert_eq!(super::super::chorus_stem_attempts(0, 0), 1);
    assert_eq!(super::super::chorus_stem_attempts(1, 3), 3);
    let chorus_trunk =
        super::super::chorus_trunk_and_terminal_flower(BlockPos { x: 0, y: 64, z: 0 }, 0, 0, false);
    assert_eq!(
        chorus_trunk.last(),
        Some(&super::super::ChorusPlantPlacementBlock {
            pos: BlockPos { x: 0, y: 66, z: 0 },
            kind: super::super::ChorusPlantPlacementKind::Flower,
            age: Some(5),
        })
    );
    assert_eq!(
        chorus_trunk
            .iter()
            .filter(|block| block.kind == super::super::ChorusPlantPlacementKind::Plant)
            .count(),
        3
    );
}

fn assert_end_podium_support() {
    assert_eq!(
        super::super::end_podium_location(BlockPos { x: 1, y: 2, z: 3 }),
        BlockPos { x: 1, y: 2, z: 3 }
    );
    assert!(super::super::end_podium_inside_rim(
        BlockPos { x: 2, y: 64, z: 0 },
        BlockPos { x: 0, y: 64, z: 0 },
    ));
    assert!(!super::super::end_podium_inside_rim(
        BlockPos { x: 3, y: 64, z: 0 },
        BlockPos { x: 0, y: 64, z: 0 },
    ));
    assert!(super::super::end_podium_inside_body(
        BlockPos { x: 3, y: 64, z: 0 },
        BlockPos { x: 0, y: 64, z: 0 },
    ));
    let inactive_podium = super::super::end_podium_blocks(BlockPos { x: 0, y: 64, z: 0 }, false);
    let active_podium = super::super::end_podium_blocks(BlockPos { x: 0, y: 64, z: 0 }, true);
    assert!(
        inactive_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 0, y: 63, z: 0 },
            kind: super::super::EndPodiumBlockKind::Bedrock,
        })
    );
    assert!(
        inactive_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 3, y: 63, z: 0 },
            kind: super::super::EndPodiumBlockKind::EndStone,
        })
    );
    assert!(
        inactive_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 3, y: 64, z: 0 },
            kind: super::super::EndPodiumBlockKind::Bedrock,
        })
    );
    assert!(
        inactive_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 0, y: 66, z: -1 },
            kind: super::super::EndPodiumBlockKind::WallTorch(
                super::super::HorizontalDirection::North
            ),
        })
    );
    assert!(
        active_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 1 },
            kind: super::super::EndPodiumBlockKind::EndPortal,
        })
    );
    assert!(
        inactive_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 1 },
            kind: super::super::EndPodiumBlockKind::Air,
        })
    );
    assert_eq!(
        active_podium
            .iter()
            .filter(|block| block.kind
                == super::super::EndPodiumBlockKind::WallTorch(
                    super::super::HorizontalDirection::East
                ))
            .count(),
        1
    );
}

pub(super) fn assert_end_spike_blocks_support() {
    let spike = super::super::end_spike_from_size(0, 2);
    assert_eq!(
        spike,
        super::super::EndSpikeModel {
            center_x: 42,
            center_z: 0,
            radius: 2,
            height: 82,
            guarded: true,
        }
    );
    assert!(super::super::end_spike_is_center_within_chunk(
        spike,
        BlockPos { x: 32, y: 0, z: 0 },
    ));
    assert!(!super::super::end_spike_is_center_within_chunk(
        spike,
        BlockPos { x: 16, y: 0, z: 0 },
    ));
    assert_eq!(
        super::super::end_spike_top_bounding_box(spike, -64, 320),
        (
            BlockPos {
                x: 40,
                y: -64,
                z: -2
            },
            BlockPos {
                x: 44,
                y: 320,
                z: 2
            },
        )
    );
    let spike_blocks = super::super::end_spike_cylinder_and_air_blocks(spike, 64);
    assert!(
        spike_blocks.contains(&super::super::EndSpikePlacementBlock {
            pos: BlockPos { x: 42, y: 64, z: 0 },
            kind: super::super::EndSpikeBlockKind::Obsidian,
        })
    );
    assert!(
        spike_blocks.contains(&super::super::EndSpikePlacementBlock {
            pos: BlockPos {
                x: 40,
                y: 82,
                z: -2
            },
            kind: super::super::EndSpikeBlockKind::Air,
        })
    );
    let cage_blocks = super::super::end_spike_guard_cage_blocks(spike);
    assert!(cage_blocks.contains(&super::super::EndSpikePlacementBlock {
        pos: BlockPos { x: 40, y: 82, z: 0 },
        kind: super::super::EndSpikeBlockKind::IronBars {
            north: true,
            south: true,
            west: false,
            east: false,
        },
    }));
    assert!(cage_blocks.contains(&super::super::EndSpikePlacementBlock {
        pos: BlockPos { x: 42, y: 85, z: 0 },
        kind: super::super::EndSpikeBlockKind::IronBars {
            north: true,
            south: true,
            west: true,
            east: true,
        },
    }));
    assert_eq!(
        super::super::end_spike_guard_cage_blocks(super::super::EndSpikeModel {
            guarded: false,
            ..spike
        }),
        Vec::new()
    );
}

pub(super) fn assert_end_crystal_support() {
    let spike = super::super::end_spike_from_size(0, 2);
    let spike_config = super::super::EndSpikeConfigurationModel {
        crystal_invulnerable: true,
        spikes: vec![spike],
        crystal_beam_target: Some(BlockPos { x: 0, y: 80, z: 0 }),
    };
    assert_eq!(
        super::super::end_crystal_for_spike(spike, &spike_config, 0.25),
        super::super::EndCrystalPlacement {
            x: 42.5,
            y: 83.0,
            z: 0.5,
            yaw: 90.0,
            beam_target: Some(BlockPos { x: 0, y: 80, z: 0 }),
            invulnerable: true,
        }
    );
    assert_eq!(
        super::super::end_spike_crystal_support_blocks(spike),
        vec![
            super::super::EndSpikePlacementBlock {
                pos: BlockPos { x: 42, y: 82, z: 0 },
                kind: super::super::EndSpikeBlockKind::Bedrock,
            },
            super::super::EndSpikePlacementBlock {
                pos: BlockPos { x: 42, y: 83, z: 0 },
                kind: super::super::EndSpikeBlockKind::Fire,
            },
        ]
    );
}
