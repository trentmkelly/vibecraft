#![allow(dead_code)]

use crate::block_update::{BlockPos, Direction};
use crate::portal::Dimension;

pub const BAD_RESPAWN_EXPLOSION_POWER: f32 = 5.0;
pub const MAX_RESPAWN_ANCHOR_CHARGES: u8 = 4;
/// Ticks of damage invulnerability applied on (re)spawn.
/// Matches Java `LivingEntity.handleDamageEvent` which sets `invulnerableTime = 20`.
pub const RESPAWN_INVULNERABILITY_TICKS: i32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BedPart {
    Head,
    Foot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BedState {
    pub facing: Direction,
    pub part: BedPart,
    pub occupied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RespawnAnchorState {
    pub charges: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnPoint {
    pub dimension: Dimension,
    pub pos: BlockPos,
    pub angle: i16,
    pub forced: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RespawnAction {
    Sleep { bed_head: BlockPos },
    SetSpawn(SpawnPoint),
    WakeExistingSleeper,
    ChargeAnchor(RespawnAnchorState),
    ConsumeAnchorCharge(RespawnAnchorState),
    Explode { pos: BlockPos, power: f32 },
    Invalid,
}

pub fn bed_head_pos(pos: BlockPos, bed: BedState) -> BlockPos {
    match bed.part {
        BedPart::Head => pos,
        BedPart::Foot => pos.relative(bed.facing),
    }
}

pub fn use_bed(
    pos: BlockPos,
    bed: BedState,
    dimension: Dimension,
    bed_works: bool,
    is_night_or_thundering: bool,
) -> RespawnAction {
    if !bed_works || matches!(dimension, Dimension::Nether | Dimension::End) {
        return RespawnAction::Explode {
            pos: bed_head_pos(pos, bed),
            power: BAD_RESPAWN_EXPLOSION_POWER,
        };
    }
    if bed.occupied {
        return RespawnAction::WakeExistingSleeper;
    }
    if is_night_or_thundering {
        RespawnAction::Sleep {
            bed_head: bed_head_pos(pos, bed),
        }
    } else {
        RespawnAction::SetSpawn(SpawnPoint {
            dimension,
            pos: bed_head_pos(pos, bed),
            angle: facing_angle(bed.facing),
            forced: false,
        })
    }
}

pub fn use_respawn_anchor(
    pos: BlockPos,
    state: RespawnAnchorState,
    dimension: Dimension,
    holding_glowstone: bool,
) -> RespawnAction {
    if holding_glowstone && state.charges < MAX_RESPAWN_ANCHOR_CHARGES {
        return RespawnAction::ChargeAnchor(RespawnAnchorState {
            charges: state.charges + 1,
        });
    }

    if !can_set_anchor_spawn(dimension) {
        return RespawnAction::Explode {
            pos,
            power: BAD_RESPAWN_EXPLOSION_POWER,
        };
    }

    if state.charges == 0 {
        RespawnAction::Invalid
    } else {
        RespawnAction::SetSpawn(SpawnPoint {
            dimension,
            pos,
            angle: 0,
            forced: false,
        })
    }
}

pub fn consume_respawn_anchor_charge(state: RespawnAnchorState, forced: bool) -> RespawnAction {
    if state.charges == 0 && !forced {
        return RespawnAction::Invalid;
    }
    RespawnAction::ConsumeAnchorCharge(RespawnAnchorState {
        charges: state.charges.saturating_sub(1),
    })
}

pub fn validate_spawn_point(
    spawn: SpawnPoint,
    block_still_valid: bool,
    stand_up_positions: &[BlockPos],
    consume_anchor: Option<RespawnAnchorState>,
) -> Option<(BlockPos, Option<RespawnAction>)> {
    if !spawn.forced && !block_still_valid {
        return None;
    }
    let stand = stand_up_positions.first().copied()?;
    let consume = consume_anchor.map(|anchor| consume_respawn_anchor_charge(anchor, spawn.forced));
    Some((stand, consume))
}

pub fn can_set_anchor_spawn(dimension: Dimension) -> bool {
    matches!(dimension, Dimension::Nether)
}

pub fn anchor_light_level(state: RespawnAnchorState) -> u8 {
    ((u16::from(state.charges.min(MAX_RESPAWN_ANCHOR_CHARGES)) * 15) / 4) as u8
}

fn facing_angle(direction: Direction) -> i16 {
    match direction {
        Direction::South => 0,
        Direction::West => 90,
        Direction::North => 180,
        Direction::East => -90,
        Direction::Up | Direction::Down => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        anchor_light_level, bed_head_pos, can_set_anchor_spawn, consume_respawn_anchor_charge,
        use_bed, use_respawn_anchor, validate_spawn_point, BedPart, BedState, RespawnAction,
        RespawnAnchorState, SpawnPoint, BAD_RESPAWN_EXPLOSION_POWER, MAX_RESPAWN_ANCHOR_CHARGES,
    };
    use crate::block_update::{BlockPos, Direction};
    use crate::portal::Dimension;

    #[test]
    fn bed_head_position_uses_facing_from_foot_part() {
        let foot = BlockPos { x: 0, y: 64, z: 0 };
        let bed = BedState {
            facing: Direction::South,
            part: BedPart::Foot,
            occupied: false,
        };
        assert_eq!(bed_head_pos(foot, bed), BlockPos { x: 0, y: 64, z: 1 });
        assert_eq!(
            bed_head_pos(
                foot,
                BedState {
                    part: BedPart::Head,
                    ..bed
                }
            ),
            foot
        );
    }

    #[test]
    fn bed_use_sleeps_at_night_sets_spawn_by_day_or_reports_occupied() {
        let pos = BlockPos { x: 1, y: 65, z: 1 };
        let bed = BedState {
            facing: Direction::East,
            part: BedPart::Head,
            occupied: false,
        };
        assert_eq!(
            use_bed(pos, bed, Dimension::Overworld, true, true),
            RespawnAction::Sleep { bed_head: pos }
        );
        assert!(matches!(
            use_bed(pos, bed, Dimension::Overworld, true, false),
            RespawnAction::SetSpawn(SpawnPoint { pos: spawn, angle: -90, .. }) if spawn == pos
        ));
        assert_eq!(
            use_bed(
                pos,
                BedState {
                    occupied: true,
                    ..bed
                },
                Dimension::Overworld,
                true,
                true
            ),
            RespawnAction::WakeExistingSleeper
        );
    }

    #[test]
    fn beds_explode_in_invalid_dimensions_or_when_bed_rule_disallows_sleep() {
        let pos = BlockPos { x: 0, y: 70, z: 0 };
        let bed = BedState {
            facing: Direction::North,
            part: BedPart::Head,
            occupied: false,
        };
        assert_eq!(
            use_bed(pos, bed, Dimension::Nether, true, true),
            RespawnAction::Explode {
                pos,
                power: BAD_RESPAWN_EXPLOSION_POWER
            }
        );
        assert_eq!(
            use_bed(pos, bed, Dimension::Overworld, false, true),
            RespawnAction::Explode {
                pos,
                power: BAD_RESPAWN_EXPLOSION_POWER
            }
        );
    }

    #[test]
    fn respawn_anchor_charges_sets_spawn_in_nether_and_explodes_elsewhere() {
        let pos = BlockPos { x: 5, y: 66, z: 5 };
        assert_eq!(
            use_respawn_anchor(
                pos,
                RespawnAnchorState { charges: 0 },
                Dimension::Nether,
                true
            ),
            RespawnAction::ChargeAnchor(RespawnAnchorState { charges: 1 })
        );
        assert_eq!(
            use_respawn_anchor(
                pos,
                RespawnAnchorState {
                    charges: MAX_RESPAWN_ANCHOR_CHARGES
                },
                Dimension::Nether,
                true
            ),
            RespawnAction::SetSpawn(SpawnPoint {
                dimension: Dimension::Nether,
                pos,
                angle: 0,
                forced: false
            })
        );
        assert_eq!(
            use_respawn_anchor(
                pos,
                RespawnAnchorState { charges: 1 },
                Dimension::Overworld,
                false
            ),
            RespawnAction::Explode {
                pos,
                power: BAD_RESPAWN_EXPLOSION_POWER
            }
        );
    }

    #[test]
    fn anchor_charge_consumption_and_light_level_match_charge_count() {
        assert_eq!(
            consume_respawn_anchor_charge(RespawnAnchorState { charges: 3 }, false),
            RespawnAction::ConsumeAnchorCharge(RespawnAnchorState { charges: 2 })
        );
        assert_eq!(
            consume_respawn_anchor_charge(RespawnAnchorState { charges: 0 }, false),
            RespawnAction::Invalid
        );
        assert_eq!(anchor_light_level(RespawnAnchorState { charges: 0 }), 0);
        assert_eq!(anchor_light_level(RespawnAnchorState { charges: 4 }), 15);
    }

    #[test]
    fn spawn_validation_requires_valid_block_unless_forced_and_consumes_anchor() {
        let spawn = SpawnPoint {
            dimension: Dimension::Nether,
            pos: BlockPos { x: 0, y: 64, z: 0 },
            angle: 0,
            forced: false,
        };
        assert_eq!(
            validate_spawn_point(spawn, false, &[BlockPos { x: 1, y: 64, z: 0 }], None),
            None
        );
        assert_eq!(
            validate_spawn_point(
                spawn,
                true,
                &[BlockPos { x: 1, y: 64, z: 0 }],
                Some(RespawnAnchorState { charges: 2 })
            ),
            Some((
                BlockPos { x: 1, y: 64, z: 0 },
                Some(RespawnAction::ConsumeAnchorCharge(RespawnAnchorState {
                    charges: 1
                }))
            ))
        );
        assert!(can_set_anchor_spawn(Dimension::Nether));
        assert!(!can_set_anchor_spawn(Dimension::Overworld));
    }
}
