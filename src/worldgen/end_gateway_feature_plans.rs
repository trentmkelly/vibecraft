use super::*;

pub fn end_gateway_known_exit(exit: BlockPos, exact: bool) -> EndGatewayConfigurationModel {
    EndGatewayConfigurationModel {
        exit: Some(exit),
        exact,
    }
}

pub fn end_gateway_delayed_exit_search() -> EndGatewayConfigurationModel {
    EndGatewayConfigurationModel {
        exit: None,
        exact: false,
    }
}

pub fn end_gateway_blocks(origin: BlockPos) -> Vec<FeaturePlacementBlock> {
    let mut blocks = Vec::new();
    for dy in -2i32..=2 {
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let same_x = dx == 0;
                let same_y = dy == 0;
                let same_z = dz == 0;
                let end = dy.abs() == 2;
                let state = if same_x && same_y && same_z {
                    "minecraft:end_gateway"
                } else if same_y {
                    "minecraft:air"
                } else if (end && same_x && same_z) || ((same_x || same_z) && !end) {
                    "minecraft:bedrock"
                } else {
                    "minecraft:air"
                };
                blocks.push(FeaturePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + dy,
                        z: origin.z + dz,
                    },
                    state,
                });
            }
        }
    }
    blocks
}
