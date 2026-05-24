use super::*;

pub fn huge_fungus_can_start(config: &HugeFungusConfigurationModel, below_state: &str) -> bool {
    below_state == config.valid_base_state
}

pub fn huge_fungus_total_height(height_roll: i32, double_roll: i32) -> i32 {
    let mut total_height = 4 + height_roll.rem_euclid(10);
    if double_roll.rem_euclid(12) == 0 {
        total_height *= 2;
    }
    total_height
}

pub fn huge_fungus_fits_height(
    origin_y: i32,
    total_height: i32,
    gen_depth: i32,
    planted: bool,
) -> bool {
    planted || origin_y + total_height + 1 < gen_depth
}

pub fn huge_fungus_is_huge(planted: bool, roll: f32) -> bool {
    !planted && roll < 0.06
}

pub fn huge_fungus_stem_blocks(
    origin: BlockPos,
    total_height: i32,
    is_huge: bool,
    corner_rolls: &[f32],
) -> Vec<HugeFungusStemBlock> {
    let stem_radius: i32 = if is_huge { 1 } else { 0 };
    let mut blocks = Vec::new();
    let mut corner_index = 0usize;
    for dx in -stem_radius..=stem_radius {
        for dz in -stem_radius..=stem_radius {
            let corner = is_huge && dx.abs() == stem_radius && dz.abs() == stem_radius;
            let place_corner =
                !corner || corner_rolls.get(corner_index).copied().unwrap_or(1.0) < 0.1;
            if corner {
                corner_index += 1;
            }
            if !place_corner {
                continue;
            }
            for dy in 0..total_height {
                blocks.push(HugeFungusStemBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + dy,
                        z: origin.z + dz,
                    },
                    kind: if corner {
                        HugeFungusStemKind::CornerStem
                    } else {
                        HugeFungusStemKind::Stem
                    },
                });
            }
        }
    }
    blocks
}

pub fn huge_fungus_hat_height(total_height: i32, roll: i32) -> i32 {
    (roll.rem_euclid(1 + total_height / 3) + 5).min(total_height)
}

pub fn huge_fungus_hat_radius(
    dy: i32,
    total_height: i32,
    hat_height: i32,
    is_huge: bool,
    top_roll: i32,
) -> i32 {
    let hat_start_y = total_height - hat_height;
    let mut radius = if dy < total_height - top_roll.rem_euclid(3) {
        2
    } else {
        1
    };
    if hat_height > 8 && dy < hat_start_y + 4 {
        radius = 3;
    }
    if is_huge {
        radius += 1;
    }
    radius
}

pub fn huge_fungus_hat_cells(
    origin: BlockPos,
    total_height: i32,
    hat_height_roll: i32,
    top_rolls: &[i32],
    is_huge: bool,
) -> Vec<HugeFungusHatCell> {
    let hat_height = huge_fungus_hat_height(total_height, hat_height_roll);
    let hat_start_y = total_height - hat_height;
    let mut cells = Vec::new();
    for dy in hat_start_y..=total_height {
        let radius = huge_fungus_hat_radius(
            dy,
            total_height,
            hat_height,
            is_huge,
            top_rolls
                .get((dy - hat_start_y) as usize)
                .copied()
                .unwrap_or(0),
        );
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let edge_x = dx == -radius || dx == radius;
                let edge_z = dz == -radius || dz == radius;
                let inside = !edge_x && !edge_z && dy != total_height;
                let corner = edge_x && edge_z;
                let bottom = dy < hat_start_y + 3;
                let role = if bottom {
                    HugeFungusHatRole::Bottom
                } else if inside {
                    HugeFungusHatRole::Inside
                } else if corner {
                    HugeFungusHatRole::Corner
                } else {
                    HugeFungusHatRole::Edge
                };
                cells.push(HugeFungusHatCell {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + dy,
                        z: origin.z + dz,
                    },
                    role,
                    radius,
                });
            }
        }
    }
    cells
}

pub fn huge_fungus_hat_drop_outcome(
    below_same_hat: bool,
    hat_roll: f32,
    vine_roll: i32,
    place_vines: bool,
) -> HugeFungusHatPlacement {
    if below_same_hat {
        HugeFungusHatPlacement::Hat
    } else if hat_roll < 0.15 {
        if place_vines && vine_roll.rem_euclid(11) == 0 {
            HugeFungusHatPlacement::HatWithWeepingVines
        } else {
            HugeFungusHatPlacement::Hat
        }
    } else {
        HugeFungusHatPlacement::None
    }
}

pub fn huge_fungus_hat_block_outcome(
    decor_roll: f32,
    hat_roll: f32,
    vine_roll: f32,
    decor_probability: f32,
    hat_probability: f32,
    vines_probability: f32,
) -> HugeFungusHatPlacement {
    if decor_roll < decor_probability {
        HugeFungusHatPlacement::Decor
    } else if hat_roll < hat_probability {
        if vine_roll < vines_probability {
            HugeFungusHatPlacement::HatWithWeepingVines
        } else {
            HugeFungusHatPlacement::Hat
        }
    } else {
        HugeFungusHatPlacement::None
    }
}

pub fn huge_fungus_hat_probabilities(
    role: HugeFungusHatRole,
    place_vines: bool,
) -> Option<(f32, f32, f32)> {
    match role {
        HugeFungusHatRole::Bottom => None,
        HugeFungusHatRole::Inside => Some((0.1, 0.2, if place_vines { 0.1 } else { 0.0 })),
        HugeFungusHatRole::Corner => Some((0.01, 0.7, if place_vines { 0.083 } else { 0.0 })),
        HugeFungusHatRole::Edge => Some((0.0005, 0.98, if place_vines { 0.07 } else { 0.0 })),
    }
}

pub fn huge_fungus_weeping_vine_height(base_roll: i32, double_roll: i32) -> i32 {
    let mut height = 1 + base_roll.rem_euclid(5);
    if double_roll.rem_euclid(7) == 0 {
        height *= 2;
    }
    height
}
