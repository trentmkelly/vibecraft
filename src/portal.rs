#![allow(dead_code)]

use crate::block_update::{BlockPos, Direction};

pub const NETHER_SCALE: f64 = 8.0;
pub const MIN_NETHER_PORTAL_WIDTH: u8 = 2;
pub const MAX_NETHER_PORTAL_WIDTH: u8 = 21;
pub const MIN_NETHER_PORTAL_HEIGHT: u8 = 3;
pub const MAX_NETHER_PORTAL_HEIGHT: u8 = 21;
pub const GATEWAY_COOLDOWN_TICKS: i32 = 40;
pub const END_GATEWAY_OUTER_RADIUS: f64 = 1024.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dimension {
    Overworld,
    Nether,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalAxis {
    X,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalRectangle {
    pub bottom_left: BlockPos,
    pub width: u8,
    pub height: u8,
    pub axis: PortalAxis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetherPortalAction {
    Invalid,
    Create {
        blocks: Vec<BlockPos>,
        axis: PortalAxis,
    },
    UseExisting(PortalRectangle),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortalTransition {
    pub target_dimension: Dimension,
    pub target_pos: (f64, f64, f64),
    pub play_sound: bool,
    pub place_portal_ticket: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndGatewayState {
    pub exit_portal: Option<BlockPos>,
    pub exact_teleport: bool,
    pub age: i64,
    pub teleport_cooldown: i32,
}

pub fn valid_nether_portal_rectangle(rect: PortalRectangle) -> bool {
    (MIN_NETHER_PORTAL_WIDTH..=MAX_NETHER_PORTAL_WIDTH).contains(&rect.width)
        && (MIN_NETHER_PORTAL_HEIGHT..=MAX_NETHER_PORTAL_HEIGHT).contains(&rect.height)
}

pub fn create_nether_portal(
    rect: PortalRectangle,
    frame_is_obsidian: bool,
    interior_replaceable: bool,
) -> NetherPortalAction {
    if !frame_is_obsidian || !interior_replaceable || !valid_nether_portal_rectangle(rect) {
        return NetherPortalAction::Invalid;
    }
    NetherPortalAction::Create {
        blocks: portal_interior_blocks(rect),
        axis: rect.axis,
    }
}

pub fn portal_interior_blocks(rect: PortalRectangle) -> Vec<BlockPos> {
    let mut blocks = Vec::new();
    for width in 0..rect.width {
        for height in 0..rect.height {
            blocks.push(offset_for_axis(
                rect.bottom_left,
                rect.axis,
                width as i32,
                height as i32,
            ));
        }
    }
    blocks
}

pub fn nether_scaled_exit(
    from_dimension: Dimension,
    x: f64,
    y: f64,
    z: f64,
    min_y: f64,
    max_y: f64,
) -> (Dimension, (f64, f64, f64)) {
    match from_dimension {
        Dimension::Overworld => (
            Dimension::Nether,
            (x / NETHER_SCALE, y.clamp(min_y, max_y), z / NETHER_SCALE),
        ),
        Dimension::Nether => (
            Dimension::Overworld,
            (x * NETHER_SCALE, y.clamp(min_y, max_y), z * NETHER_SCALE),
        ),
        Dimension::End => (Dimension::End, (x, y.clamp(min_y, max_y), z)),
    }
}

pub fn nether_portal_transition(
    from_dimension: Dimension,
    entry_pos: (f64, f64, f64),
    exit_rect: PortalRectangle,
) -> PortalTransition {
    let (target_dimension, scaled) = nether_scaled_exit(
        from_dimension,
        entry_pos.0,
        entry_pos.1,
        entry_pos.2,
        -64.0,
        320.0,
    );
    let center = portal_center(exit_rect);
    PortalTransition {
        target_dimension,
        target_pos: (center.0, scaled.1, center.2),
        play_sound: true,
        place_portal_ticket: true,
    }
}

pub fn end_portal_transition(
    from_dimension: Dimension,
    spawn_platform: BlockPos,
) -> PortalTransition {
    match from_dimension {
        Dimension::End => PortalTransition {
            target_dimension: Dimension::Overworld,
            target_pos: (
                spawn_platform.x as f64 + 0.5,
                spawn_platform.y as f64,
                spawn_platform.z as f64 + 0.5,
            ),
            play_sound: false,
            place_portal_ticket: false,
        },
        _ => PortalTransition {
            target_dimension: Dimension::End,
            target_pos: (100.5, 50.0, 0.5),
            play_sound: false,
            place_portal_ticket: false,
        },
    }
}

pub fn end_gateway_transition(
    state: EndGatewayState,
    gateway_pos: BlockPos,
    generated_exit: Option<BlockPos>,
) -> (EndGatewayState, Option<PortalTransition>) {
    if state.teleport_cooldown > 0 {
        return (state, None);
    }
    let exit = state
        .exit_portal
        .or(generated_exit)
        .unwrap_or_else(|| tentative_gateway_exit(gateway_pos));
    let target = if state.exact_teleport {
        exit
    } else {
        exit.relative(Direction::Up)
    };
    (
        EndGatewayState {
            exit_portal: Some(exit),
            teleport_cooldown: GATEWAY_COOLDOWN_TICKS,
            ..state
        },
        Some(PortalTransition {
            target_dimension: Dimension::End,
            target_pos: (
                target.x as f64 + 0.5,
                target.y as f64,
                target.z as f64 + 0.5,
            ),
            play_sound: false,
            place_portal_ticket: false,
        }),
    )
}

pub fn tick_gateway_cooldown(state: EndGatewayState) -> EndGatewayState {
    EndGatewayState {
        age: state.age + 1,
        teleport_cooldown: state.teleport_cooldown.saturating_sub(1),
        ..state
    }
}

pub fn tentative_gateway_exit(gateway_pos: BlockPos) -> BlockPos {
    let x = gateway_pos.x as f64;
    let z = gateway_pos.z as f64;
    let length = (x * x + z * z).sqrt().max(1.0);
    BlockPos {
        x: (x / length * END_GATEWAY_OUTER_RADIUS).round() as i32,
        y: 75,
        z: (z / length * END_GATEWAY_OUTER_RADIUS).round() as i32,
    }
}

fn portal_center(rect: PortalRectangle) -> (f64, f64, f64) {
    let far = offset_for_axis(
        rect.bottom_left,
        rect.axis,
        i32::from(rect.width) - 1,
        i32::from(rect.height) - 1,
    );
    (
        (rect.bottom_left.x + far.x) as f64 / 2.0 + 0.5,
        rect.bottom_left.y as f64,
        (rect.bottom_left.z + far.z) as f64 / 2.0 + 0.5,
    )
}

fn offset_for_axis(pos: BlockPos, axis: PortalAxis, width: i32, height: i32) -> BlockPos {
    match axis {
        PortalAxis::X => BlockPos {
            x: pos.x + width,
            y: pos.y + height,
            z: pos.z,
        },
        PortalAxis::Z => BlockPos {
            x: pos.x,
            y: pos.y + height,
            z: pos.z + width,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        create_nether_portal, end_gateway_transition, end_portal_transition,
        nether_portal_transition, nether_scaled_exit, portal_interior_blocks,
        tentative_gateway_exit, tick_gateway_cooldown, valid_nether_portal_rectangle, Dimension,
        EndGatewayState, NetherPortalAction, PortalAxis, PortalRectangle, END_GATEWAY_OUTER_RADIUS,
        GATEWAY_COOLDOWN_TICKS,
    };
    use crate::block_update::BlockPos;

    #[test]
    fn nether_portal_rectangles_obey_vanilla_size_limits() {
        assert!(valid_nether_portal_rectangle(PortalRectangle {
            bottom_left: BlockPos { x: 0, y: 64, z: 0 },
            width: 2,
            height: 3,
            axis: PortalAxis::X,
        }));
        assert!(!valid_nether_portal_rectangle(PortalRectangle {
            bottom_left: BlockPos { x: 0, y: 64, z: 0 },
            width: 1,
            height: 3,
            axis: PortalAxis::X,
        }));
    }

    #[test]
    fn nether_portal_creation_fills_interior_by_axis() {
        let rect = PortalRectangle {
            bottom_left: BlockPos {
                x: 10,
                y: 64,
                z: -2,
            },
            width: 2,
            height: 3,
            axis: PortalAxis::Z,
        };
        assert_eq!(
            create_nether_portal(rect, true, true),
            NetherPortalAction::Create {
                blocks: portal_interior_blocks(rect),
                axis: PortalAxis::Z
            }
        );
        assert_eq!(portal_interior_blocks(rect).len(), 6);
        assert_eq!(
            portal_interior_blocks(rect)[1],
            BlockPos {
                x: 10,
                y: 65,
                z: -2
            }
        );
        assert_eq!(
            portal_interior_blocks(rect)[3],
            BlockPos {
                x: 10,
                y: 64,
                z: -1
            }
        );
        assert_eq!(
            create_nether_portal(rect, false, true),
            NetherPortalAction::Invalid
        );
    }

    #[test]
    fn nether_coordinate_scaling_uses_eight_to_one_ratio_and_clamps_y() {
        assert_eq!(
            nether_scaled_exit(Dimension::Overworld, 80.0, 500.0, -40.0, -64.0, 320.0),
            (Dimension::Nether, (10.0, 320.0, -5.0))
        );
        assert_eq!(
            nether_scaled_exit(Dimension::Nether, 10.0, -100.0, -5.0, -64.0, 320.0),
            (Dimension::Overworld, (80.0, -64.0, -40.0))
        );
    }

    #[test]
    fn nether_transition_uses_exit_rectangle_center_and_places_ticket() {
        let transition = nether_portal_transition(
            Dimension::Overworld,
            (80.0, 70.0, 80.0),
            PortalRectangle {
                bottom_left: BlockPos {
                    x: 10,
                    y: 65,
                    z: 10,
                },
                width: 2,
                height: 3,
                axis: PortalAxis::X,
            },
        );
        assert_eq!(transition.target_dimension, Dimension::Nether);
        assert_eq!(transition.target_pos, (11.0, 70.0, 10.5));
        assert!(transition.play_sound);
        assert!(transition.place_portal_ticket);
    }

    #[test]
    fn end_portal_enters_end_or_returns_to_overworld_spawn_platform() {
        assert_eq!(
            end_portal_transition(Dimension::Overworld, BlockPos { x: 0, y: 64, z: 0 })
                .target_dimension,
            Dimension::End
        );
        let return_home = end_portal_transition(Dimension::End, BlockPos { x: 4, y: 70, z: -4 });
        assert_eq!(return_home.target_dimension, Dimension::Overworld);
        assert_eq!(return_home.target_pos, (4.5, 70.0, -3.5));
    }

    #[test]
    fn end_gateway_generates_or_uses_exit_and_sets_cooldown() {
        let gateway = EndGatewayState {
            exit_portal: None,
            exact_teleport: false,
            age: 10,
            teleport_cooldown: 0,
        };
        let (updated, transition) = end_gateway_transition(
            gateway,
            BlockPos {
                x: 100,
                y: 75,
                z: 0,
            },
            Some(BlockPos {
                x: 1024,
                y: 75,
                z: 0,
            }),
        );
        assert_eq!(
            updated.exit_portal,
            Some(BlockPos {
                x: 1024,
                y: 75,
                z: 0
            })
        );
        assert_eq!(updated.teleport_cooldown, GATEWAY_COOLDOWN_TICKS);
        assert_eq!(transition.unwrap().target_pos, (1024.5, 76.0, 0.5));

        let (_, blocked) = end_gateway_transition(
            updated,
            BlockPos {
                x: 100,
                y: 75,
                z: 0,
            },
            None,
        );
        assert!(blocked.is_none());
    }

    #[test]
    fn gateway_cooldown_ticks_down_and_tentative_exit_points_outward() {
        let state = EndGatewayState {
            exit_portal: None,
            exact_teleport: true,
            age: 4,
            teleport_cooldown: 3,
        };
        assert_eq!(tick_gateway_cooldown(state).teleport_cooldown, 2);
        assert_eq!(tick_gateway_cooldown(state).age, 5);

        let exit = tentative_gateway_exit(BlockPos {
            x: 100,
            y: 70,
            z: 0,
        });
        assert_eq!(exit.x, END_GATEWAY_OUTER_RADIUS as i32);
        assert_eq!(exit.y, 75);
        assert_eq!(exit.z, 0);
    }
}
