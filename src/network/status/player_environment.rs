//! Environmental damage a player takes from the world around it.
//!
//! Java references (all under `net.minecraft.world`):
//! * `Entity.baseTick` — on-fire ticking (`remainingFireTicks`, `ON_FIRE` damage every 20 ticks
//!   outside lava), halved fall distance in lava, `checkBelowWorld`.
//! * `LivingEntity.baseTick` / `Entity.isInWall` — suffocation (`IN_WALL`).
//! * `LivingEntity.aiStep` (freezing) and `Entity.canFreeze` / `InsideBlockEffectType.FREEZE`.
//! * `Entity.applyEffectsFromBlocks` / `checkInsideBlocks` and the `entityInside` / `stepOn` of
//!   `BaseFireBlock`, `LavaFluid`, `WaterFluid`, `LavaCauldronBlock`, `CactusBlock`,
//!   `SweetBerryBushBlock`, `CampfireBlock`, `PowderSnowBlock`, `WebBlock` and `MagmaBlock`.
//! * `Entity.lavaIgnite` / `lavaHurt` / `igniteForTicks`, `Player.getFireImmuneTicks`,
//!   `Player.setRemainingFireTicks` and `LivingEntity.igniteForTicks` (`BURNING_TIME`).
//! * `Block.fallOn` overrides and `LivingEntity.causeFallDamage` — fall damage per landing block.
//!
//! Every damage source goes through [`super::player_damage::hurt_server`]. The world is read
//! through a `block_at` lookup returning `block[prop=value,...]` state strings, so the whole
//! module is testable without a chunk cache.

use std::collections::HashMap;

use super::player_damage::{
    effect_amplifier, hurt_server, queue_entity_event, worn_armor, HurtOrigin, ENTITY_EVENT_HONEY_SLIDE,
};
use super::player_death::{live_random_seed, simple_damage_source};
use super::*;
use crate::block_properties::{shape, state_physics_by_name, StateFluid, StatePhysics};
use crate::player_entity::{
    calculate_fall_damage, FallDamageInput, DEFAULT_FALL_DAMAGE_MULTIPLIER,
    DEFAULT_SAFE_FALL_DISTANCE,
};

/// `Level.getMinY()` of the overworld, the only dimension the live server simulates.
const OVERWORLD_MIN_Y: i32 = -64;
/// `Entity.checkBelowWorld`: distance below the lowest build height that is void.
const BELOW_WORLD_MARGIN: i32 = 64;
/// `LivingEntity.onBelowWorld`: `hurt(fellOutOfWorld, 4.0F)`.
const VOID_DAMAGE: f32 = 4.0;
/// `Entity.baseTick`: on-fire damage cadence and amount.
const ON_FIRE_INTERVAL_TICKS: i32 = 20;
const ON_FIRE_DAMAGE: f32 = 1.0;
/// `Player.getFireImmuneTicks()`.
const PLAYER_FIRE_IMMUNE_TICKS: i32 = 20;
/// `BaseFireBlock.fireIgnite`: `igniteForSeconds(8.0F)`.
const FIRE_IGNITE_TICKS: i32 = 8 * 20;
/// `Entity.lavaIgnite`: `igniteForSeconds(15.0F)`.
const LAVA_IGNITE_TICKS: i32 = 15 * 20;
/// `Entity.lavaHurt`: `hurtServer(lava, 4.0F)`.
const LAVA_DAMAGE: f32 = 4.0;
/// `Entity.getTicksRequiredToFreeze()`.
const TICKS_REQUIRED_TO_FREEZE: i32 = 140;
/// `LivingEntity.aiStep`: freeze damage cadence.
const FREEZE_DAMAGE_INTERVAL_TICKS: i32 = 40;
/// `Entity.checkInsideBlocks` deflates the bounding box by `1.0E-5F`.
const INSIDE_BLOCK_DEFLATE: f64 = 1.0e-5;
/// `Entity.getOnPosLegacy()` offset used by `stepOn` and `fallOn`.
const ON_POS_OFFSET: f64 = 0.2;
/// `SweetBerryBushBlock.entityInside`: minimum horizontal movement that hurts (`0.003F`).
const BERRY_MOVEMENT_THRESHOLD: f64 = 0.003_f32 as f64;
/// Standing / sneaking pose dimensions (`EntityType.PLAYER`, `Pose.CROUCHING`).
const STANDING_HEIGHT: f64 = 1.8;
const CROUCHING_HEIGHT: f64 = 1.5;
const STANDING_EYE_HEIGHT: f64 = 1.62;
const CROUCHING_EYE_HEIGHT: f64 = 1.27;
const PLAYER_WIDTH_F: f64 = 0.6;

/// Fire / freeze bookkeeping Java keeps on `Entity`.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct EnvironmentState {
    /// `Entity.remainingFireTicks` (negative = fire-immune grace period).
    pub remaining_fire_ticks: i32,
    /// `Entity.DATA_TICKS_FROZEN`.
    pub ticks_frozen: i32,
    /// `Entity.isInPowderSnow`, set by the `FREEZE` inside-block effect.
    pub in_powder_snow: bool,
    /// Position at the end of the previous tick (`Entity.oldPosition`).
    pub previous_position: Option<[f64; 3]>,
    /// splitmix64 state for the few `Entity.random` draws (0 = seed on first use).
    pub rng_state: u64,
}

/// `Entity.isOnFire()` for a player (never fire-immune).
pub(super) fn is_on_fire(state: &PlaySessionState) -> bool {
    state.combat.environment.remaining_fire_ticks > 0
}

/// `RandomSource.nextInt(bound)` from the state's splitmix64 stream.
fn next_random_int(env: &mut EnvironmentState, bound: i32) -> i32 {
    if env.rng_state == 0 {
        env.rng_state = live_random_seed() | 1;
    }
    env.rng_state = env.rng_state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = env.rng_state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (((z >> 11) as f64 / (1u64 << 53) as f64) * f64::from(bound)) as i32
}

/// `RandomSource.nextDouble()` from the same stream.
pub(super) fn next_random_double(env: &mut EnvironmentState) -> f64 {
    f64::from(next_random_int(env, 1 << 24)) / f64::from(1 << 24)
}

/// `Player.setRemainingFireTicks`: an invulnerable player never burns longer than one tick.
fn set_remaining_fire_ticks(state: &mut PlaySessionState, ticks: i32) {
    state.combat.environment.remaining_fire_ticks =
        if state.abilities.invulnerable { ticks.min(1) } else { ticks };
}

/// `Entity.clearFire`.
fn clear_fire(state: &mut PlaySessionState) {
    let current = state.combat.environment.remaining_fire_ticks;
    set_remaining_fire_ticks(state, current.min(0));
}

/// `Entity.clearFreeze`.
fn clear_freeze(state: &mut PlaySessionState) {
    state.combat.environment.ticks_frozen = 0;
}

/// The `BURNING_TIME` attribute of the player: 1.0 plus `add_multiplied_base` of
/// Fire Protection's `-0.15` per level on every worn piece
/// (`data/minecraft/enchantment/fire_protection.json`), clamped to `[0, 1024]`.
fn burning_time(state: &PlaySessionState) -> f64 {
    let levels: i32 = worn_armor(state)
        .iter()
        .filter_map(|(_, _, _, stack)| match stack.component("minecraft:enchantments") {
            Some(ItemComponent::Enchantments(map)) => map.get("minecraft:fire_protection").copied(),
            _ => None,
        })
        .sum();
    (1.0 - 0.15 * f64::from(levels)).clamp(0.0, 1024.0)
}

/// `LivingEntity.igniteForTicks` (which scales by `BURNING_TIME`) -> `Entity.igniteForTicks`.
fn ignite_for_ticks(state: &mut PlaySessionState, ticks: i32) {
    let ticks = (f64::from(ticks) * burning_time(state)).ceil() as i32;
    if state.combat.environment.remaining_fire_ticks < ticks {
        set_remaining_fire_ticks(state, ticks);
    }
    clear_freeze(state);
}

/// Bounding box of the player in its current pose.
#[derive(Debug, Clone, Copy)]
struct Bbox {
    min: [f64; 3],
    max: [f64; 3],
}

impl Bbox {
    fn of_player(state: &PlaySessionState) -> Self {
        let height = if state.input_shift { CROUCHING_HEIGHT } else { STANDING_HEIGHT };
        let half = PLAYER_WIDTH_F / 2.0;
        Self {
            min: [state.x - half, state.y, state.z - half],
            max: [state.x + half, state.y + height, state.z + half],
        }
    }

    fn deflate(self, amount: f64) -> Self {
        Self {
            min: self.min.map(|v| v + amount),
            max: self.max.map(|v| v - amount),
        }
    }

    /// Cells `BlockPos.betweenClosed(this)` covers.
    fn cells(self) -> impl Iterator<Item = (i32, i32, i32)> {
        let (lo, hi) = (self.min.map(|v| v.floor() as i32), self.max.map(|v| v.floor() as i32));
        (lo[0]..=hi[0]).flat_map(move |x| {
            (lo[1]..=hi[1]).flat_map(move |y| (lo[2]..=hi[2]).map(move |z| (x, y, z)))
        })
    }

    /// Positive-volume intersection with `other`.
    fn overlaps(self, other: Bbox) -> bool {
        (0..3).all(|axis| self.min[axis] < other.max[axis] && other.min[axis] < self.max[axis])
    }
}

/// A parsed `block[prop=value,...]` state with its physics.
struct BlockAt {
    name: String,
    properties: Vec<(String, String)>,
    physics: Option<&'static StatePhysics>,
}

impl BlockAt {
    fn parse(state: &str) -> Self {
        let (name, raw) = state.split_once('[').map_or((state, ""), |(n, r)| (n, r.trim_end_matches(']')));
        let properties = raw
            .split(',')
            .filter_map(|pair| pair.split_once('='))
            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            .collect();
        Self { name: name.to_string(), properties, physics: state_physics_by_name(state) }
    }

    fn property(&self, key: &str) -> Option<&str> {
        self.properties.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    fn is_air(&self) -> bool {
        self.physics.is_none_or(|physics| physics.is_air)
    }

    fn fluid(&self) -> StateFluid {
        self.physics.map_or(StateFluid::Empty, |physics| physics.fluid)
    }

    /// `BlockState.getCollisionShape` boxes, block-local.
    fn collision_boxes(&self) -> &'static [[f64; 6]] {
        self.physics.map_or(&[], |physics| shape(physics.collision_shape))
    }
}

/// Cached block lookups for one tick.
struct Probe<'a> {
    lookup: &'a mut dyn FnMut(i32, i32, i32) -> Option<String>,
    cache: HashMap<(i32, i32, i32), std::rc::Rc<BlockAt>>,
}

impl<'a> Probe<'a> {
    fn new(lookup: &'a mut dyn FnMut(i32, i32, i32) -> Option<String>) -> Self {
        Self { lookup, cache: HashMap::new() }
    }

    fn at(&mut self, x: i32, y: i32, z: i32) -> std::rc::Rc<BlockAt> {
        let lookup = &mut self.lookup;
        self.cache
            .entry((x, y, z))
            .or_insert_with(|| {
                let state = lookup(x, y, z).unwrap_or_else(|| "minecraft:air".to_string());
                std::rc::Rc::new(BlockAt::parse(&state))
            })
            .clone()
    }
}

/// `FluidState.getHeight`: `amount / 9`, or a full block when the same fluid sits above.
fn fluid_height(probe: &mut Probe<'_>, (x, y, z): (i32, i32, i32), fluid: StateFluid) -> f64 {
    let amount = match fluid {
        StateFluid::Water { amount, .. } | StateFluid::Lava { amount, .. } => amount,
        StateFluid::Empty => return 0.0,
    };
    let above = probe.at(x, y + 1, z).fluid();
    let same_above = matches!(
        (fluid, above),
        (StateFluid::Water { .. }, StateFluid::Water { .. })
            | (StateFluid::Lava { .. }, StateFluid::Lava { .. })
    );
    if same_above { 1.0 } else { f64::from(amount) / 9.0 }
}

/// The cell's fluid box `[y, y + height]` overlapped with the (deflated) player box
/// (`Entity.collidedWithFluid`).
fn touches_fluid(
    probe: &mut Probe<'_>,
    cell: (i32, i32, i32),
    fluid: StateFluid,
    bbox: Bbox,
) -> bool {
    let height = fluid_height(probe, cell, fluid);
    let (x, y, z) = (f64::from(cell.0), f64::from(cell.1), f64::from(cell.2));
    height > 0.0 && bbox.overlaps(Bbox { min: [x, y, z], max: [x + 1.0, y + height, z + 1.0] })
}

/// `Entity.isInLava` via `EntityFluidInteraction`: the box shrunk by `0.001` touches lava whose
/// surface reaches the feet.
fn is_in_lava(state: &PlaySessionState, probe: &mut Probe<'_>) -> bool {
    let bbox = Bbox::of_player(state).deflate(0.001);
    bbox.cells().any(|cell| {
        let block = probe.at(cell.0, cell.1, cell.2);
        matches!(block.fluid(), StateFluid::Lava { .. })
            && f64::from(cell.1) + fluid_height(probe, cell, block.fluid()) >= bbox.min[1]
    })
}

/// `Entity.baseTick` fire section.
fn tick_fire(state: &mut PlaySessionState, in_lava: bool) {
    let remaining = state.combat.environment.remaining_fire_ticks;
    if remaining <= 0 {
        return;
    }
    if remaining % ON_FIRE_INTERVAL_TICKS == 0 && !in_lava {
        hurt_server(state, simple_damage_source("minecraft:on_fire"), ON_FIRE_DAMAGE, HurtOrigin::default());
    }
    let current = state.combat.environment.remaining_fire_ticks;
    set_remaining_fire_ticks(state, current - 1);
}

/// `Entity.checkBelowWorld` -> `LivingEntity.onBelowWorld`.
fn check_below_world(state: &mut PlaySessionState, min_y: i32) {
    if state.y < f64::from(min_y - BELOW_WORLD_MARGIN) {
        hurt_server(state, simple_damage_source("minecraft:out_of_world"), VOID_DAMAGE, HurtOrigin::default());
    }
}

/// `Entity.isInWall` for a non-sleeping player: the eye box overlaps a suffocating block.
fn is_in_wall(state: &PlaySessionState, probe: &mut Probe<'_>) -> bool {
    if state.game_mode == GameMode::Spectator {
        return false;
    }
    let eye_height = if state.input_shift { CROUCHING_EYE_HEIGHT } else { STANDING_EYE_HEIGHT };
    let half = f64::from(0.6_f32 * 0.8_f32) / 2.0;
    let eye_y = state.y + eye_height;
    let eye = Bbox {
        min: [state.x - half, eye_y - 5.0e-7, state.z - half],
        max: [state.x + half, eye_y + 5.0e-7, state.z + half],
    };
    eye.cells().any(|(x, y, z)| {
        let block = probe.at(x, y, z);
        let Some(physics) = block.physics else { return false };
        !physics.is_air
            && physics.suffocating
            && block.collision_boxes().iter().any(|b| {
                let (ox, oy, oz) = (f64::from(x), f64::from(y), f64::from(z));
                eye.overlaps(Bbox { min: [ox + b[0], oy + b[1], oz + b[2]], max: [ox + b[3], oy + b[4], oz + b[5]] })
            })
    })
}

/// Inside-block effects gathered by one scan (`InsideBlockEffectApplier.StepBasedCollector`),
/// applied in `InsideBlockEffectType` order: FREEZE, CLEAR_FREEZE, FIRE_IGNITE, LAVA_IGNITE,
/// EXTINGUISH.
#[derive(Debug, Default)]
struct InsideEffects {
    freeze: bool,
    clear_freeze: bool,
    /// `FIRE_IGNITE` requested, each with the `runAfter` `inFire` damage of its fire block.
    fire_damage: Vec<f32>,
    fire_ignite: bool,
    lava_ignite: bool,
    lava_hurts: usize,
    extinguish: bool,
}

/// `BaseFireBlock.fireIgnite` for a server player.
fn fire_ignite(state: &mut PlaySessionState) {
    let remaining = state.combat.environment.remaining_fire_ticks;
    if remaining < 0 {
        set_remaining_fire_ticks(state, remaining + 1);
    } else {
        let added = next_random_int(&mut state.combat.environment, 2) + 1;
        set_remaining_fire_ticks(state, remaining + added);
    }
    if state.combat.environment.remaining_fire_ticks >= 0 {
        ignite_for_ticks(state, FIRE_IGNITE_TICKS);
    }
}

impl InsideEffects {
    /// `InsideBlockEffectApplier.applyAndClear`.
    fn apply(self, state: &mut PlaySessionState) {
        if self.freeze {
            state.combat.environment.in_powder_snow = true;
            if can_freeze(state) {
                let env = &mut state.combat.environment;
                env.ticks_frozen = (env.ticks_frozen + 1).min(TICKS_REQUIRED_TO_FREEZE);
            }
        }
        if self.clear_freeze {
            clear_freeze(state);
        }
        if self.fire_ignite {
            fire_ignite(state);
            for damage in self.fire_damage {
                hurt_server(state, simple_damage_source("minecraft:in_fire"), damage, HurtOrigin::default());
            }
        }
        if self.lava_ignite {
            ignite_for_ticks(state, LAVA_IGNITE_TICKS);
            for _ in 0..self.lava_hurts {
                hurt_server(state, simple_damage_source("minecraft:lava"), LAVA_DAMAGE, HurtOrigin::default());
            }
        }
        if self.extinguish {
            clear_fire(state);
        }
    }
}

/// `LivingEntity.canFreeze` / `Entity.canFreeze`: not a spectator and no leather armor
/// (`#minecraft:freeze_immune_wearables`).
fn can_freeze(state: &PlaySessionState) -> bool {
    state.game_mode != GameMode::Spectator
        && !worn_armor(state).iter().any(|(_, _, _, stack)| stack.item_id().starts_with("minecraft:leather_"))
}

/// The `entityInside` of the block (and the fluid) in one cell.
fn scan_cell(
    state: &mut PlaySessionState,
    probe: &mut Probe<'_>,
    cell: (i32, i32, i32),
    bbox: Bbox,
    effects: &mut InsideEffects,
) {
    let block = probe.at(cell.0, cell.1, cell.2);
    if block.is_air() {
        return;
    }
    let inside_fluid = touches_fluid(probe, cell, block.fluid(), bbox);
    let hurt = |state: &mut PlaySessionState, id: &str, damage: f32| {
        hurt_server(state, simple_damage_source(id), damage, HurtOrigin::default());
    };
    match block.name.as_str() {
        "minecraft:fire" | "minecraft:soul_fire" => {
            let damage = if block.name == "minecraft:fire" { 1.0 } else { 2.0 };
            effects.clear_freeze = true;
            effects.fire_ignite = true;
            effects.fire_damage.push(damage);
        }
        "minecraft:campfire" | "minecraft:soul_campfire" if block.property("lit") == Some("true") => {
            hurt(state, "minecraft:campfire", if block.name == "minecraft:campfire" { 1.0 } else { 2.0 });
        }
        "minecraft:cactus" => hurt(state, "minecraft:cactus", 1.0),
        "minecraft:sweet_berry_bush" => {
            // makeStuckInBlock resets the fall distance.
            state.fall_distance = 0.0;
            if block.property("age") != Some("0") && moved_horizontally(state) {
                hurt(state, "minecraft:sweet_berry_bush", 1.0);
            }
        }
        "minecraft:cobweb" => state.fall_distance = 0.0,
        "minecraft:powder_snow" => {
            let feet = probe.at(state.x.floor() as i32, state.y.floor() as i32, state.z.floor() as i32);
            if feet.name == "minecraft:powder_snow" {
                state.fall_distance = 0.0;
            }
            effects.freeze = true;
            effects.extinguish = true;
        }
        "minecraft:lava_cauldron" if lava_cauldron_contains(&block, cell, bbox) => {
            effects.clear_freeze = true;
            effects.lava_ignite = true;
            effects.lava_hurts += 1;
        }
        _ => {}
    }
    match block.fluid() {
        StateFluid::Lava { .. } if inside_fluid => {
            effects.clear_freeze = true;
            effects.lava_ignite = true;
            effects.lava_hurts += 1;
        }
        StateFluid::Water { .. } if inside_fluid => effects.extinguish = true,
        _ => {}
    }
}

/// `LavaCauldronBlock.getEntityInsideCollisionShape`: the cauldron shape plus its 12x11 pixel
/// lava column (`Block.column(12.0, 4.0, 15.0)`).
fn lava_cauldron_contains(block: &BlockAt, cell: (i32, i32, i32), bbox: Bbox) -> bool {
    let (x, y, z) = (f64::from(cell.0), f64::from(cell.1), f64::from(cell.2));
    let column = [0.125, 0.25, 0.125, 0.875, 0.9375, 0.875];
    block
        .collision_boxes()
        .iter()
        .chain(std::iter::once(&column))
        .any(|b| {
            bbox.overlaps(Bbox {
                min: [x + b[0], y + b[1], z + b[2]],
                max: [x + b[3], y + b[4], z + b[5]],
            })
        })
}

/// `SweetBerryBushBlock`: `oldPosition - position` moved by at least `0.003` on an axis.
fn moved_horizontally(state: &PlaySessionState) -> bool {
    state.combat.environment.previous_position.is_some_and(|previous| {
        let (dx, dz) = ((previous[0] - state.x).abs(), (previous[2] - state.z).abs());
        (dx > 0.0 || dz > 0.0) && (dx >= BERRY_MOVEMENT_THRESHOLD || dz >= BERRY_MOVEMENT_THRESHOLD)
    })
}

/// `Entity.applyEffectsFromBlocks`: `stepOn`, then the inside-block scan and its effects, then
/// the `remainingFireTicks = -fireImmuneTicks` grace rule.
fn apply_effects_from_blocks(state: &mut PlaySessionState, probe: &mut Probe<'_>) {
    if state.on_ground {
        let below = probe.at(state.x.floor() as i32, (state.y - ON_POS_OFFSET).floor() as i32, state.z.floor() as i32);
        // MagmaBlock.stepOn: isSteppingCarefully() is the shift key.
        if below.name == "minecraft:magma_block" && !state.input_shift {
            hurt_server(state, simple_damage_source("minecraft:hot_floor"), 1.0, HurtOrigin::default());
        }
    }
    let previous_fire = state.combat.environment.remaining_fire_ticks;
    let bbox = Bbox::of_player(state).deflate(INSIDE_BLOCK_DEFLATE);
    let mut effects = InsideEffects::default();
    for cell in bbox.cells() {
        if state.health <= 0.0 {
            break;
        }
        scan_cell(state, probe, cell, bbox, &mut effects);
    }
    effects.apply(state);
    // TODO(rain-extinguish): `Entity.isInRain` -> clearFire needs precipitation at the player.
    let ignited = state.combat.environment.remaining_fire_ticks > previous_fire;
    if !is_on_fire(state) && !ignited {
        set_remaining_fire_ticks(state, -PLAYER_FIRE_IMMUNE_TICKS);
    }
}

/// `LivingEntity.aiStep` freezing section.
fn tick_freezing(state: &mut PlaySessionState) {
    let in_snow = std::mem::take(&mut state.combat.environment.in_powder_snow);
    let can_freeze = can_freeze(state);
    if !in_snow || !can_freeze {
        let env = &mut state.combat.environment;
        env.ticks_frozen = (env.ticks_frozen - 2).max(0);
    }
    let fully_frozen = state.combat.environment.ticks_frozen >= TICKS_REQUIRED_TO_FREEZE;
    if state.combat.tick_count % FREEZE_DAMAGE_INTERVAL_TICKS == 0 && fully_frozen && can_freeze {
        hurt_server(state, simple_damage_source("minecraft:freeze"), 1.0, HurtOrigin::default());
    }
}

/// `LivingEntity.causeFallDamage`: `floor((distance + 1e-6 - safeFallDistance) * modifier *
/// fallDamageMultiplier)`, hurting with `damage_type` when positive. `Player.causeFallDamage`
/// ignores players that may fly.
fn cause_fall_damage(state: &mut PlaySessionState, distance: f32, modifier: f32, damage_type: &str) -> bool {
    let damage = calculate_fall_damage(FallDamageInput {
        fall_distance: distance,
        damage_modifier: modifier,
        safe_fall_distance: DEFAULT_SAFE_FALL_DISTANCE,
        fall_damage_multiplier: DEFAULT_FALL_DAMAGE_MULTIPLIER,
        fall_damage_enabled: true,
        may_fly: state.abilities.mayfly,
    });
    if damage <= 0 {
        return false;
    }
    hurt_server(state, simple_damage_source(damage_type), damage as f32, HurtOrigin::default());
    true
}

/// `Block.fallOn` of the block the player landed on, then the resulting fall damage.
///
/// TODO(fall-block-side-effects): farmland turning to dirt and turtle eggs breaking need world
/// mutation; the fall damage of those blocks is the default one.
fn fall_on(state: &mut PlaySessionState, block: &BlockAt, distance: f32) {
    let name = block.name.as_str();
    match name {
        "minecraft:hay_block" => {
            cause_fall_damage(state, distance, 0.2, "minecraft:fall");
        }
        // SlimeBlock.fallOn: a sneaking player is not protected in 26.1.2, it just skips damage.
        "minecraft:slime_block" => {
            if !state.input_shift {
                cause_fall_damage(state, distance, 0.0, "minecraft:fall");
            }
        }
        "minecraft:honey_block" => {
            queue_entity_event(state, ENTITY_EVENT_HONEY_SLIDE);
            cause_fall_damage(state, distance, 0.2, "minecraft:fall");
        }
        "minecraft:powder_snow" => {}
        "minecraft:pointed_dripstone"
            if block.property("vertical_direction") == Some("up")
                && block.property("thickness") == Some("tip") =>
        {
            cause_fall_damage(state, distance + 2.5, 2.0, "minecraft:stalagmite");
        }
        _ if name.ends_with("_bed") => {
            cause_fall_damage(state, distance * 0.5, 1.0, "minecraft:fall");
        }
        _ => {
            cause_fall_damage(state, distance, 1.0, "minecraft:fall");
        }
    }
}

/// Resolves a landing recorded by the movement handler against the block below the player
/// (`Entity.checkFallDamage` -> `Block.fallOn`).
fn resolve_landing(state: &mut PlaySessionState, probe: &mut Probe<'_>) {
    let Some(distance) = state.combat.hurt.pending_landing.take() else {
        return;
    };
    let below = probe.at(state.x.floor() as i32, (state.y - ON_POS_OFFSET).floor() as i32, state.z.floor() as i32);
    fall_on(state, &below, distance);
}

/// One player tick of environmental damage. Returns whether health changed.
///
/// `min_y` is the dimension's lowest build height (`Level.getMinY`); `lookup` returns the
/// `block[prop=value,...]` state at a position.
pub(super) fn tick_player_environment(
    state: &mut PlaySessionState,
    min_y: i32,
    lookup: &mut dyn FnMut(i32, i32, i32) -> Option<String>,
) -> bool {
    let health_before = state.health;
    let mut probe = Probe::new(lookup);
    resolve_landing(state, &mut probe);
    let in_lava = is_in_lava(state, &mut probe);
    tick_fire(state, in_lava);
    if in_lava {
        state.fall_distance *= 0.5;
    }
    check_below_world(state, min_y);
    if state.health > 0.0 && is_in_wall(state, &mut probe) {
        hurt_server(state, simple_damage_source("minecraft:in_wall"), 1.0, HurtOrigin::default());
    }
    if state.game_mode != GameMode::Spectator && state.health > 0.0 {
        apply_effects_from_blocks(state, &mut probe);
    }
    tick_freezing(state);
    state.combat.environment.previous_position = Some([state.x, state.y, state.z]);
    state.health != health_before
}

/// `MobEffectUtil.hasWaterBreathing`.
pub(super) fn has_water_breathing(state: &PlaySessionState) -> bool {
    ["minecraft:water_breathing", "minecraft:conduit_power", "minecraft:breath_of_the_nautilus"]
        .iter()
        .any(|effect| effect_amplifier(state, effect).is_some())
}

/// `MobEffectUtil.shouldEffectsRefillAirsupply`.
pub(super) fn effects_refill_air_supply(state: &PlaySessionState) -> bool {
    effect_amplifier(state, "minecraft:breath_of_the_nautilus").is_none()
        || effect_amplifier(state, "minecraft:water_breathing").is_some()
        || effect_amplifier(state, "minecraft:conduit_power").is_some()
}

/// `LivingEntity.decreaseAirSupply`: Respiration (`OXYGEN_BONUS`, +1 per level on every worn
/// piece) skips the decrement with probability `bonus / (bonus + 1)`.
pub(super) fn decrease_air_supply(state: &mut PlaySessionState) -> i32 {
    let bonus: i32 = worn_armor(state)
        .iter()
        .filter_map(|(_, _, _, stack)| match stack.component("minecraft:enchantments") {
            Some(ItemComponent::Enchantments(map)) => map.get("minecraft:respiration").copied(),
            _ => None,
        })
        .sum();
    let current = state.air_supply;
    if bonus > 0 && next_random_double(&mut state.combat.environment) >= 1.0 / (f64::from(bonus) + 1.0) {
        current
    } else {
        current - 1
    }
}

/// `LevelChunk` block state at a position as a `block[prop=value,...]` string.
fn chunk_block_state(
    chunk_cache: &GeneratedChunkCache,
    world_root: &Path,
    world_seed: i64,
    (x, y, z): (i32, i32, i32),
) -> Option<String> {
    let chunk = chunk_cache.get_or_load(x.div_euclid(16), z.div_euclid(16), world_root, world_seed);
    let entry = chunk.get_block_state_model(x, y, z)?;
    if entry.properties.is_empty() {
        return Some(entry.name);
    }
    let properties: Vec<String> = entry.properties.iter().map(|(k, v)| format!("{k}={v}")).collect();
    Some(format!("{}[{}]", entry.name, properties.join(",")))
}

/// Start-of-tick hazards: refreshes the damage rules, runs `ServerPlayer.tick`'s hurt-cooldown
/// countdown and then the environmental damage of [`tick_player_environment`] against the live
/// chunks. Returns whether health changed.
pub(super) fn tick_player_hazards(
    state: &mut PlaySessionState,
    game_rules: &SharedGameRules,
    difficulty: FoodDifficulty,
    world_root: &Path,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
) -> bool {
    super::player_damage::refresh_damage_rules(state, game_rules, difficulty);
    super::player_damage::tick_hurt_cooldown(state);
    tick_player_environment(state, OVERWORLD_MIN_Y, &mut |x, y, z| {
        chunk_block_state(chunk_cache, world_root, world_seed, (x, y, z))
    })
}

#[cfg(test)]
mod tests;
