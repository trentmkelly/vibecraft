use super::*;

pub fn snow_and_freeze_placement_plan(
    origin: BlockPos,
    columns: &[SnowAndFreezeColumn],
) -> Vec<SnowAndFreezePlacement> {
    let mut placements = Vec::new();
    for dx in 0..16 {
        for dz in 0..16 {
            let x = origin.x + dx;
            let z = origin.z + dz;
            let Some(column) = columns.iter().find(|column| column.x == x && column.z == z) else {
                continue;
            };
            let top = BlockPos {
                x,
                y: column.motion_blocking_height,
                z,
            };
            let below = BlockPos {
                x,
                y: column.motion_blocking_height - 1,
                z,
            };
            if column.should_freeze {
                placements.push(SnowAndFreezePlacement {
                    pos: below,
                    state: "minecraft:ice",
                });
            }
            if column.should_snow {
                placements.push(SnowAndFreezePlacement {
                    pos: top,
                    state: "minecraft:snow",
                });
                if column.below_has_snowy_property {
                    placements.push(SnowAndFreezePlacement {
                        pos: below,
                        state: "minecraft:snowy=true",
                    });
                }
            }
        }
    }
    placements
}
