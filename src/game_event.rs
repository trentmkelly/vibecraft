#![allow(dead_code)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::block_behavior::BlockStateModel;
use crate::block_update::BlockPos;
use crate::entity_physics::{explosion_plan, ExplosionBlockInteraction, Vec3};
use crate::fire::{explosion_affects_block, ExplosionBlockResult, ExplosionInput};

pub const DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS: i32 = 16;
pub const SHRIEK_NOTIFICATION_RADIUS: i32 = 32;
pub const JUKEBOX_NOTIFICATION_RADIUS: i32 = 10;
pub const EXPLOSION_GRID_SIZE: i32 = 16;
pub const EXPLOSION_RAY_STEP: f64 = 0.3;
pub const MAX_DROPS_PER_COMBINED_STACK: u8 = 16;
pub const EXPLODE_GAME_EVENT: GameEventDefinition = GameEventDefinition {
    id: "minecraft:explode",
    notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameEventDefinition {
    pub id: &'static str,
    pub notification_radius: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEventContext {
    pub source_entity: Option<String>,
    pub affected_state: Option<String>,
}

impl GameEventContext {
    pub fn empty() -> Self {
        Self {
            source_entity: None,
            affected_state: None,
        }
    }

    pub fn source(source_entity: impl Into<String>) -> Self {
        Self {
            source_entity: Some(source_entity.into()),
            affected_state: None,
        }
    }

    pub fn state(affected_state: impl Into<String>) -> Self {
        Self {
            source_entity: None,
            affected_state: Some(affected_state.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEventDeliveryMode {
    Immediate,
    ByDistance,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameEventListener {
    pub id: String,
    pub position: Vec3,
    pub radius: i32,
    pub delivery_mode: GameEventDeliveryMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameEventDelivery {
    pub listener_id: String,
    pub event_id: &'static str,
    pub distance_sqr: f64,
    pub delivery_mode: GameEventDeliveryMode,
}

pub const BUILTIN_GAME_EVENTS: &[GameEventDefinition] = &[
    GameEventDefinition {
        id: "minecraft:block_activate",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:block_attach",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:block_change",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:block_close",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:block_deactivate",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:block_destroy",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:block_detach",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:block_open",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:block_place",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:container_close",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:container_open",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:drink",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:eat",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:elytra_glide",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:entity_damage",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:entity_die",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:entity_dismount",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:entity_interact",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:entity_mount",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:entity_place",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:entity_action",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:equip",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:explode",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:flap",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:fluid_pickup",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:fluid_place",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:hit_ground",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:instrument_play",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:item_interact_finish",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:item_interact_start",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:jukebox_play",
        notification_radius: JUKEBOX_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:jukebox_stop_play",
        notification_radius: JUKEBOX_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:lightning_strike",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:note_block_play",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:prime_fuse",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:projectile_land",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:projectile_shoot",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:sculk_sensor_tendrils_clicking",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:shear",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:shriek",
        notification_radius: SHRIEK_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:splash",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:step",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:swim",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:teleport",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:unequip",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_1",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_2",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_3",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_4",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_5",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_6",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_7",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_8",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_9",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_10",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_11",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_12",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_13",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_14",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
    GameEventDefinition {
        id: "minecraft:resonate_15",
        notification_radius: DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS,
    },
];

pub fn builtin_game_events() -> Vec<GameEventDefinition> {
    BUILTIN_GAME_EVENTS.to_vec()
}

pub fn game_event_by_id(id: &str) -> Option<GameEventDefinition> {
    BUILTIN_GAME_EVENTS
        .iter()
        .copied()
        .find(|event| event.id == id)
}

pub fn dispatch_game_event(
    event: GameEventDefinition,
    source: Vec3,
    _context: GameEventContext,
    listeners: &[GameEventListener],
) -> Vec<GameEventDelivery> {
    let mut immediate = Vec::new();
    let mut by_distance = Vec::new();
    let event_radius_sqr = f64::from(event.notification_radius * event.notification_radius);

    for listener in listeners {
        let listener_radius_sqr = f64::from(listener.radius * listener.radius);
        let distance = distance_sqr(source, listener.position);
        if distance > event_radius_sqr || distance > listener_radius_sqr {
            continue;
        }
        let delivery = GameEventDelivery {
            listener_id: listener.id.clone(),
            event_id: event.id,
            distance_sqr: distance,
            delivery_mode: listener.delivery_mode,
        };
        match listener.delivery_mode {
            GameEventDeliveryMode::Immediate => immediate.push(delivery),
            GameEventDeliveryMode::ByDistance => by_distance.push(delivery),
        }
    }

    by_distance.sort_by(|left, right| {
        left.distance_sqr
            .partial_cmp(&right.distance_sqr)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.listener_id.cmp(&right.listener_id))
    });
    immediate.extend(by_distance);
    immediate
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionBlockCandidate {
    pub pos: BlockPos,
    pub block: BlockStateModel,
    pub distance: f32,
    pub exposure: f32,
    pub initial_power: f32,
    /// `random.nextInt(3)` rolled for this position when placing post-explosion
    /// fire; vanilla places fire only when this is `0`.
    pub fire_random_roll: i32,
    /// Whether the block below this position is `isSolidRender()` (fire needs a
    /// solid floor). The destroyed position itself is air after the blast.
    pub below_is_solid_render: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionEntityCandidate {
    pub id: String,
    pub eye_position: Vec3,
    pub exposure: f32,
    pub protection: f32,
    pub knockback_multiplier: f32,
    pub ignored: bool,
    pub spectator: bool,
    pub creative_flying: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionEntityResult {
    pub id: String,
    pub damage: f32,
    pub knockback: Vec3,
    pub record_hit_player: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerExplosionInput {
    pub center: Vec3,
    pub radius: f32,
    pub fire: bool,
    pub block_interaction: ExplosionBlockInteraction,
    pub source_entity: Option<String>,
    pub mob_griefing: bool,
    pub source_is_wind_charge: bool,
    pub blocks: Vec<ExplosionBlockCandidate>,
    pub entities: Vec<ExplosionEntityCandidate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerExplosionPlan {
    pub game_event: GameEventDefinition,
    pub destroyed_blocks: Vec<ExplosionBlockResult>,
    pub entity_hits: Vec<ExplosionEntityResult>,
    pub fire_positions: Vec<BlockPos>,
    pub can_trigger_blocks: bool,
    pub should_affect_blocklike_entities: bool,
    pub is_small: bool,
    pub combined_drop_limit: u8,
}

pub fn plan_server_explosion(input: ServerExplosionInput) -> ServerExplosionPlan {
    let can_trigger_blocks = input.block_interaction == ExplosionBlockInteraction::TriggerBlock
        && (!input.source_is_wind_charge || input.mob_griefing);
    let should_affect_blocklike_entities = if input.mob_griefing {
        !input.source_is_wind_charge
    } else {
        matches!(
            input.block_interaction,
            ExplosionBlockInteraction::Destroy | ExplosionBlockInteraction::DestroyWithDecay
        ) && !input.source_is_wind_charge
    };
    let interacts_with_blocks = input.block_interaction != ExplosionBlockInteraction::Keep;
    let mut seen_blocks = BTreeSet::new();
    let mut destroyed_blocks = Vec::new();
    // 1:1 with ServerExplosion: when `fire` is set, each destroyed position
    // places fire if `random.nextInt(3) == 0 && getBlockState(pos).isAir() &&
    // getBlockState(pos.below()).isSolidRender()`. The destroyed position is air
    // after the blast, so only the per-position roll and the below-solid check
    // (supplied per candidate) remain.
    let mut fire_positions = Vec::new();

    if interacts_with_blocks {
        for block in input.blocks {
            if !seen_blocks.insert((block.pos.x, block.pos.y, block.pos.z)) {
                continue;
            }
            let fire_eligible =
                input.fire && block.fire_random_roll == 0 && block.below_is_solid_render;
            let result = explosion_affects_block(
                ExplosionInput {
                    pos: block.pos,
                    block: block.block,
                    distance: block.distance,
                    exposure: block.exposure,
                },
                block.initial_power,
            );
            if result.destroyed {
                if fire_eligible {
                    fire_positions.push(result.pos);
                }
                destroyed_blocks.push(result);
            }
        }
    }

    let double_radius = input.radius * 2.0;
    let entity_hits = if input.radius < 1.0E-5 {
        Vec::new()
    } else {
        input
            .entities
            .iter()
            .filter_map(|entity| {
                if entity.ignored {
                    return None;
                }
                let distance_ratio =
                    (vec_sub(entity.eye_position, input.center).length() as f32) / double_radius;
                if distance_ratio > 1.0 {
                    return None;
                }
                let physics = explosion_plan(
                    input.center,
                    entity.eye_position,
                    input.radius,
                    entity.exposure,
                    input.block_interaction,
                    input.fire,
                );
                let damage = physics.damage * (1.0 - entity.protection.clamp(0.0, 1.0));
                let knockback = scale_vec(
                    physics.knockback,
                    f64::from(entity.knockback_multiplier)
                        * f64::from(1.0 - entity.protection.clamp(0.0, 1.0)),
                );
                Some(ExplosionEntityResult {
                    id: entity.id.clone(),
                    damage,
                    knockback,
                    record_hit_player: !entity.spectator && !entity.creative_flying,
                })
            })
            .collect()
    };

    ServerExplosionPlan {
        game_event: EXPLODE_GAME_EVENT,
        destroyed_blocks,
        entity_hits,
        fire_positions,
        can_trigger_blocks,
        should_affect_blocklike_entities,
        is_small: input.radius < 2.0 || !interacts_with_blocks,
        combined_drop_limit: MAX_DROPS_PER_COMBINED_STACK,
    }
}

pub fn explosion_ray_directions() -> Vec<(f64, f64, f64)> {
    let mut directions = Vec::new();
    let mut seen = BTreeMap::new();
    for xx in 0..EXPLOSION_GRID_SIZE {
        for yy in 0..EXPLOSION_GRID_SIZE {
            for zz in 0..EXPLOSION_GRID_SIZE {
                if xx == 0
                    || xx == EXPLOSION_GRID_SIZE - 1
                    || yy == 0
                    || yy == EXPLOSION_GRID_SIZE - 1
                    || zz == 0
                    || zz == EXPLOSION_GRID_SIZE - 1
                {
                    let mut x = f64::from(xx) / 15.0 * 2.0 - 1.0;
                    let mut y = f64::from(yy) / 15.0 * 2.0 - 1.0;
                    let mut z = f64::from(zz) / 15.0 * 2.0 - 1.0;
                    let length = (x * x + y * y + z * z).sqrt();
                    x /= length;
                    y /= length;
                    z /= length;
                    let key = (
                        (x * 1_000_000.0).round() as i64,
                        (y * 1_000_000.0).round() as i64,
                        (z * 1_000_000.0).round() as i64,
                    );
                    seen.entry(key).or_insert((x, y, z));
                }
            }
        }
    }
    directions.extend(seen.into_values());
    directions
}

fn distance_sqr(left: Vec3, right: Vec3) -> f64 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    let dz = left.z - right.z;
    dx * dx + dy * dy + dz * dz
}

fn vec_sub(left: Vec3, right: Vec3) -> Vec3 {
    Vec3 {
        x: left.x - right.x,
        y: left.y - right.y,
        z: left.z - right.z,
    }
}

fn scale_vec(vec: Vec3, scalar: f64) -> Vec3 {
    Vec3 {
        x: vec.x * scalar,
        y: vec.y * scalar,
        z: vec.z * scalar,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        builtin_game_events, dispatch_game_event, explosion_ray_directions, game_event_by_id,
        plan_server_explosion, ExplosionBlockCandidate, ExplosionEntityCandidate, GameEventContext,
        GameEventDeliveryMode, GameEventListener, ServerExplosionInput,
        DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS, EXPLOSION_GRID_SIZE, EXPLOSION_RAY_STEP,
        JUKEBOX_NOTIFICATION_RADIUS, MAX_DROPS_PER_COMBINED_STACK, SHRIEK_NOTIFICATION_RADIUS,
    };
    use crate::block_behavior::BlockStateModel;
    use crate::block_update::BlockPos;
    use crate::entity_physics::{ExplosionBlockInteraction, Vec3};

    fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos { x, y, z }
    }

    #[test]
    fn builtin_game_event_registry_matches_vanilla_surface_and_radii() {
        let events = builtin_game_events();
        assert_eq!(events.first().unwrap().id, "minecraft:block_activate");
        assert_eq!(events.last().unwrap().id, "minecraft:resonate_15");
        assert_eq!(events.len(), 60);
        assert_eq!(
            game_event_by_id("minecraft:explode")
                .unwrap()
                .notification_radius,
            DEFAULT_GAME_EVENT_NOTIFICATION_RADIUS
        );
        assert_eq!(
            game_event_by_id("minecraft:shriek")
                .unwrap()
                .notification_radius,
            SHRIEK_NOTIFICATION_RADIUS
        );
        assert_eq!(
            game_event_by_id("minecraft:jukebox_play")
                .unwrap()
                .notification_radius,
            JUKEBOX_NOTIFICATION_RADIUS
        );
    }

    #[test]
    fn dispatcher_filters_by_event_and_listener_radius_then_sorts_distance_delivery() {
        let event = game_event_by_id("minecraft:explode").unwrap();
        let deliveries = dispatch_game_event(
            event,
            Vec3 {
                x: 0.0,
                y: 64.0,
                z: 0.0,
            },
            GameEventContext::empty(),
            &[
                GameEventListener {
                    id: "far".to_string(),
                    position: Vec3 {
                        x: 15.0,
                        y: 64.0,
                        z: 0.0,
                    },
                    radius: 16,
                    delivery_mode: GameEventDeliveryMode::ByDistance,
                },
                GameEventListener {
                    id: "near".to_string(),
                    position: Vec3 {
                        x: 3.0,
                        y: 64.0,
                        z: 0.0,
                    },
                    radius: 16,
                    delivery_mode: GameEventDeliveryMode::ByDistance,
                },
                GameEventListener {
                    id: "immediate".to_string(),
                    position: Vec3 {
                        x: 10.0,
                        y: 64.0,
                        z: 0.0,
                    },
                    radius: 16,
                    delivery_mode: GameEventDeliveryMode::Immediate,
                },
                GameEventListener {
                    id: "outside".to_string(),
                    position: Vec3 {
                        x: 17.0,
                        y: 64.0,
                        z: 0.0,
                    },
                    radius: 32,
                    delivery_mode: GameEventDeliveryMode::ByDistance,
                },
            ],
        );

        assert_eq!(
            deliveries
                .iter()
                .map(|delivery| delivery.listener_id.as_str())
                .collect::<Vec<_>>(),
            vec!["immediate", "near", "far"]
        );
    }

    #[test]
    fn explosion_ray_grid_uses_vanilla_sixteen_cube_boundary_and_step() {
        let total_boundary_cells = EXPLOSION_GRID_SIZE.pow(3) - (EXPLOSION_GRID_SIZE - 2).pow(3);
        assert_eq!(total_boundary_cells, 1352);
        assert_eq!(EXPLOSION_RAY_STEP, 0.3);
        assert_eq!(
            explosion_ray_directions().len(),
            total_boundary_cells as usize
        );
    }

    #[test]
    fn server_explosion_posts_event_destroys_blocks_hurts_entities_and_creates_fire() {
        let plan = plan_server_explosion(ServerExplosionInput {
            center: Vec3 {
                x: 0.0,
                y: 64.0,
                z: 0.0,
            },
            radius: 4.0,
            fire: true,
            block_interaction: ExplosionBlockInteraction::Destroy,
            source_entity: Some("tnt".to_string()),
            mob_griefing: true,
            source_is_wind_charge: false,
            blocks: vec![
                ExplosionBlockCandidate {
                    pos: pos(1, 64, 0),
                    block: BlockStateModel::new("minecraft:stone"),
                    distance: 0.0,
                    exposure: 1.0,
                    initial_power: 4.0,
                    fire_random_roll: 0,
                    below_is_solid_render: true,
                },
                ExplosionBlockCandidate {
                    pos: pos(1, 64, 0),
                    block: BlockStateModel::new("minecraft:stone"),
                    distance: 0.0,
                    exposure: 1.0,
                    initial_power: 4.0,
                    fire_random_roll: 0,
                    below_is_solid_render: true,
                },
            ],
            entities: vec![ExplosionEntityCandidate {
                id: "player".to_string(),
                eye_position: Vec3 {
                    x: 2.0,
                    y: 64.0,
                    z: 0.0,
                },
                exposure: 1.0,
                protection: 0.0,
                knockback_multiplier: 1.0,
                ignored: false,
                spectator: false,
                creative_flying: false,
            }],
        });

        assert_eq!(plan.game_event.id, "minecraft:explode");
        assert_eq!(plan.destroyed_blocks.len(), 1);
        assert_eq!(plan.entity_hits.len(), 1);
        assert!(plan.entity_hits[0].damage > 0.0);
        assert!(plan.entity_hits[0].record_hit_player);
        assert_eq!(plan.fire_positions, vec![pos(1, 64, 0)]);
        assert_eq!(plan.combined_drop_limit, MAX_DROPS_PER_COMBINED_STACK);
        assert!(!plan.is_small);
    }

    #[test]
    fn explosion_fire_requires_roll_zero_solid_below_and_fire_flag() {
        let candidate = |fire_random_roll: i32, below_is_solid_render: bool| ExplosionBlockCandidate {
            pos: pos(2, 64, 2),
            block: BlockStateModel::new("minecraft:stone"),
            distance: 0.0,
            exposure: 1.0,
            initial_power: 4.0,
            fire_random_roll,
            below_is_solid_render,
        };
        let input = |fire: bool, roll: i32, solid: bool| ServerExplosionInput {
            center: Vec3 {
                x: 2.0,
                y: 64.0,
                z: 2.0,
            },
            radius: 4.0,
            fire,
            block_interaction: ExplosionBlockInteraction::Destroy,
            source_entity: Some("tnt".to_string()),
            mob_griefing: true,
            source_is_wind_charge: false,
            blocks: vec![candidate(roll, solid)],
            entities: Vec::new(),
        };

        // nextInt(3)==0, solid floor, fire on → fire placed.
        assert_eq!(
            plan_server_explosion(input(true, 0, true)).fire_positions,
            vec![pos(2, 64, 2)]
        );
        // nextInt(3)!=0 → no fire.
        assert!(plan_server_explosion(input(true, 1, true))
            .fire_positions
            .is_empty());
        // No solid block below → no fire.
        assert!(plan_server_explosion(input(true, 0, false))
            .fire_positions
            .is_empty());
        // Explosion doesn't cause fire → no fire even with a zero roll.
        assert!(plan_server_explosion(input(false, 0, true))
            .fire_positions
            .is_empty());
    }

    #[test]
    fn explosion_block_interaction_gates_blocks_triggers_and_wind_charge_effects() {
        let base = ServerExplosionInput {
            center: Vec3 {
                x: 0.0,
                y: 64.0,
                z: 0.0,
            },
            radius: 1.5,
            fire: false,
            block_interaction: ExplosionBlockInteraction::Keep,
            source_entity: None,
            mob_griefing: false,
            source_is_wind_charge: false,
            blocks: vec![ExplosionBlockCandidate {
                pos: pos(0, 64, 0),
                block: BlockStateModel::new("minecraft:dirt"),
                distance: 0.0,
                exposure: 1.0,
                initial_power: 4.0,
                fire_random_roll: 0,
                below_is_solid_render: true,
            }],
            entities: Vec::new(),
        };

        let keep = plan_server_explosion(base.clone());
        assert!(keep.destroyed_blocks.is_empty());
        assert!(keep.is_small);

        let mut trigger = base.clone();
        trigger.block_interaction = ExplosionBlockInteraction::TriggerBlock;
        let trigger = plan_server_explosion(trigger);
        assert!(trigger.can_trigger_blocks);
        assert!(!trigger.should_affect_blocklike_entities);

        let mut wind_charge = base;
        wind_charge.block_interaction = ExplosionBlockInteraction::TriggerBlock;
        wind_charge.source_is_wind_charge = true;
        wind_charge.mob_griefing = false;
        let wind_charge = plan_server_explosion(wind_charge);
        assert!(!wind_charge.can_trigger_blocks);
        assert!(!wind_charge.should_affect_blocklike_entities);
    }
}
