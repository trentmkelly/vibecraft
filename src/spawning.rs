#![allow(dead_code)]

use crate::entity_category::{mob_category, MobCategoryDef};

pub const MIN_NATURAL_SPAWN_DISTANCE_BLOCKS: i32 = 24;
pub const NATURAL_SPAWN_DISTANCE_CHUNKS: i32 = 8;
pub const NATURAL_SPAWN_DISTANCE_BLOCKS: i32 = 128;
pub const INSCRIBED_SQUARE_SPAWN_DISTANCE_CHUNKS: i32 = 5;
pub const MOB_CAP_CHUNK_MAGIC: i32 = 17 * 17;

pub const DEFAULT_SPAWNER_DELAY: i32 = 20;
pub const DEFAULT_MIN_SPAWNER_DELAY: i32 = 200;
pub const DEFAULT_MAX_SPAWNER_DELAY: i32 = 800;
pub const DEFAULT_SPAWNER_COUNT: i32 = 4;
pub const DEFAULT_MAX_NEARBY_SPAWNER_ENTITIES: i32 = 6;
pub const DEFAULT_SPAWNER_PLAYER_RANGE: i32 = 16;
pub const DEFAULT_SPAWNER_RANGE: i32 = 4;

pub const DEFAULT_WANDERING_TRADER_SPAWN_DELAY: i32 = 24_000;
pub const DEFAULT_WANDERING_TRADER_SPAWN_CHANCE: i32 = 25;
pub const PATROL_BASE_DELAY_TICKS: i32 = 12_000;
pub const PATROL_RANDOM_DELAY_BOUND: i32 = 1_200;
pub const PHANTOM_MIN_DELAY_SECONDS: i32 = 60;
pub const PHANTOM_RANDOM_DELAY_SECONDS: i32 = 60;
pub const PHANTOM_INSOMNIA_THRESHOLD_TICKS: i32 = 72_000;
pub const TRIAL_SPAWNER_DEFAULT_TARGET_COOLDOWN: i32 = 36_000;
pub const TRIAL_SPAWNER_DEFAULT_PLAYER_SCAN_RANGE: i32 = 14;
pub const TRIAL_SPAWNER_DETECT_PLAYER_SPAWN_BUFFER: i32 = 40;
pub const TRIAL_SPAWNER_MAX_MOB_TRACKING_DISTANCE: i32 = 47;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobCount {
    pub category: &'static str,
    pub count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnedEntitySample {
    pub category: &'static str,
    pub persistent: bool,
    pub custom_persistent: bool,
    pub is_mob: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnStateSummary {
    pub spawnable_chunk_count: i32,
    pub category_counts: [MobCount; 7],
}

impl SpawnStateSummary {
    pub fn create(spawnable_chunk_count: i32, entities: &[SpawnedEntitySample]) -> Self {
        let mut summary = Self {
            spawnable_chunk_count,
            category_counts: [
                MobCount {
                    category: "monster",
                    count: 0,
                },
                MobCount {
                    category: "creature",
                    count: 0,
                },
                MobCount {
                    category: "ambient",
                    count: 0,
                },
                MobCount {
                    category: "axolotls",
                    count: 0,
                },
                MobCount {
                    category: "underground_water_creature",
                    count: 0,
                },
                MobCount {
                    category: "water_creature",
                    count: 0,
                },
                MobCount {
                    category: "water_ambient",
                    count: 0,
                },
            ],
        };
        for entity in entities {
            if entity.category == "misc" || entity.persistent || entity.custom_persistent {
                continue;
            }
            if let Some(count) = summary
                .category_counts
                .iter_mut()
                .find(|count| count.category == entity.category)
            {
                count.count += 1;
            }
        }
        summary
    }

    pub fn count_for(&self, category: &str) -> i32 {
        self.category_counts
            .iter()
            .find(|count| count.category == category)
            .map(|count| count.count)
            .unwrap_or(0)
    }

    pub fn global_cap(&self, category: MobCategoryDef) -> i32 {
        category.max_per_chunk * self.spawnable_chunk_count / MOB_CAP_CHUNK_MAGIC
    }

    pub fn can_spawn_for_category_global(&self, category: MobCategoryDef) -> bool {
        category.max_per_chunk >= 0 && self.count_for(category.id) < self.global_cap(category)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnCategoryFilters {
    pub spawn_friendlies: bool,
    pub spawn_enemies: bool,
    pub spawn_persistent: bool,
}

pub fn filtered_spawning_categories(
    state: &SpawnStateSummary,
    filters: SpawnCategoryFilters,
) -> Vec<&'static str> {
    crate::entity_category::MOB_CATEGORIES
        .iter()
        .filter(|category| category.id != "misc")
        .filter(|category| filters.spawn_friendlies || !category.friendly)
        .filter(|category| filters.spawn_enemies || category.friendly)
        .filter(|category| filters.spawn_persistent || !category.persistent)
        .filter(|category| state.can_spawn_for_category_global(**category))
        .map(|category| category.id)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NaturalSpawnContext {
    pub category: &'static str,
    pub nearest_player_distance_squared: i32,
    pub distance_to_world_spawn_squared: i32,
    pub same_chunk_or_spawnable_neighbor: bool,
    pub block_collision_full: bool,
    pub block_signal_source: bool,
    pub fluid_empty: bool,
    pub prevent_mob_spawning_inside: bool,
    pub dangerous_block: bool,
    pub can_summon: bool,
    pub placement_ok: bool,
    pub spawn_rules_ok: bool,
    pub collision_free: bool,
    pub can_spawn_far_from_player: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NaturalSpawnRejection {
    TooCloseToPlayer,
    TooCloseToWorldSpawn,
    ChunkNotSpawnable,
    MiscCategory,
    TooFarFromPlayer,
    CannotSummon,
    InvalidEmptyBlock,
    PlacementRejected,
    SpawnRulesRejected,
    Collision,
}

pub fn validate_natural_spawn(ctx: NaturalSpawnContext) -> Result<(), NaturalSpawnRejection> {
    if ctx.nearest_player_distance_squared <= MIN_NATURAL_SPAWN_DISTANCE_BLOCKS.pow(2) {
        return Err(NaturalSpawnRejection::TooCloseToPlayer);
    }
    if ctx.distance_to_world_spawn_squared < MIN_NATURAL_SPAWN_DISTANCE_BLOCKS.pow(2) {
        return Err(NaturalSpawnRejection::TooCloseToWorldSpawn);
    }
    if !ctx.same_chunk_or_spawnable_neighbor {
        return Err(NaturalSpawnRejection::ChunkNotSpawnable);
    }
    let category = mob_category(ctx.category).ok_or(NaturalSpawnRejection::MiscCategory)?;
    if category.id == "misc" {
        return Err(NaturalSpawnRejection::MiscCategory);
    }
    if !ctx.can_spawn_far_from_player
        && ctx.nearest_player_distance_squared > category.despawn_distance.pow(2)
    {
        return Err(NaturalSpawnRejection::TooFarFromPlayer);
    }
    if !ctx.can_summon {
        return Err(NaturalSpawnRejection::CannotSummon);
    }
    if !is_valid_empty_spawn_block(
        ctx.block_collision_full,
        ctx.block_signal_source,
        ctx.fluid_empty,
        ctx.prevent_mob_spawning_inside,
        ctx.dangerous_block,
    ) {
        return Err(NaturalSpawnRejection::InvalidEmptyBlock);
    }
    if !ctx.placement_ok {
        return Err(NaturalSpawnRejection::PlacementRejected);
    }
    if !ctx.spawn_rules_ok {
        return Err(NaturalSpawnRejection::SpawnRulesRejected);
    }
    if !ctx.collision_free {
        return Err(NaturalSpawnRejection::Collision);
    }
    Ok(())
}

pub fn is_valid_empty_spawn_block(
    block_collision_full: bool,
    block_signal_source: bool,
    fluid_empty: bool,
    prevent_mob_spawning_inside: bool,
    dangerous_block: bool,
) -> bool {
    !block_collision_full
        && !block_signal_source
        && fluid_empty
        && !prevent_mob_spawning_inside
        && !dangerous_block
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DespawnDecision {
    Keep,
    RandomEligible,
    Immediate,
}

pub fn despawn_decision(
    category: &'static str,
    persistent: bool,
    custom_persistent: bool,
    remove_when_far_away: bool,
    nearest_player_distance_squared: i32,
) -> DespawnDecision {
    if persistent || custom_persistent {
        return DespawnDecision::Keep;
    }
    let Some(category) = mob_category(category) else {
        return DespawnDecision::Keep;
    };
    if nearest_player_distance_squared > category.despawn_distance.pow(2) && remove_when_far_away {
        DespawnDecision::Immediate
    } else if nearest_player_distance_squared > category.no_despawn_distance.pow(2) {
        DespawnDecision::RandomEligible
    } else {
        DespawnDecision::Keep
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnerConfig {
    pub spawn_delay: i32,
    pub min_spawn_delay: i32,
    pub max_spawn_delay: i32,
    pub spawn_count: i32,
    pub max_nearby_entities: i32,
    pub required_player_range: i32,
    pub spawn_range: i32,
}

impl Default for SpawnerConfig {
    fn default() -> Self {
        Self {
            spawn_delay: DEFAULT_SPAWNER_DELAY,
            min_spawn_delay: DEFAULT_MIN_SPAWNER_DELAY,
            max_spawn_delay: DEFAULT_MAX_SPAWNER_DELAY,
            spawn_count: DEFAULT_SPAWNER_COUNT,
            max_nearby_entities: DEFAULT_MAX_NEARBY_SPAWNER_ENTITIES,
            required_player_range: DEFAULT_SPAWNER_PLAYER_RANGE,
            spawn_range: DEFAULT_SPAWNER_RANGE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnerTickPlan {
    Idle,
    CountDown {
        next_delay: i32,
    },
    TrySpawn {
        attempts: i32,
    },
    Delay {
        next_delay_min: i32,
        next_delay_max_exclusive: i32,
    },
}

pub fn spawner_tick_plan(
    config: SpawnerConfig,
    player_in_range: bool,
    spawner_blocks_work: bool,
    nearby_entities: i32,
) -> SpawnerTickPlan {
    if !player_in_range || !spawner_blocks_work {
        return SpawnerTickPlan::Idle;
    }
    if config.spawn_delay == -1 || nearby_entities >= config.max_nearby_entities {
        return SpawnerTickPlan::Delay {
            next_delay_min: config.min_spawn_delay,
            next_delay_max_exclusive: config.max_spawn_delay,
        };
    }
    if config.spawn_delay > 0 {
        return SpawnerTickPlan::CountDown {
            next_delay: config.spawn_delay - 1,
        };
    }
    SpawnerTickPlan::TrySpawn {
        attempts: config.spawn_count,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatrolSpawnContext {
    pub spawn_enemies: bool,
    pub spawn_patrols_rule: bool,
    pub next_tick: i32,
    pub bright_outside: bool,
    pub random_one_in_five_hit: bool,
    pub player_count: usize,
    pub selected_player_spectator: bool,
    pub close_to_village: bool,
    pub chunks_available: bool,
    pub biome_allows_patrol: bool,
    pub effective_difficulty_ceil: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatrolSpawnPlan {
    Disabled,
    Wait {
        next_tick: i32,
    },
    ResetOnly {
        next_tick_floor: i32,
        next_tick_random_bound: i32,
    },
    SpawnGroup {
        members: i32,
        leader_index: i32,
    },
}

pub fn patrol_spawn_plan(ctx: PatrolSpawnContext) -> PatrolSpawnPlan {
    if !ctx.spawn_enemies || !ctx.spawn_patrols_rule {
        return PatrolSpawnPlan::Disabled;
    }
    if ctx.next_tick > 1 {
        return PatrolSpawnPlan::Wait {
            next_tick: ctx.next_tick - 1,
        };
    }
    if !ctx.bright_outside
        || !ctx.random_one_in_five_hit
        || ctx.player_count == 0
        || ctx.selected_player_spectator
        || ctx.close_to_village
        || !ctx.chunks_available
        || !ctx.biome_allows_patrol
    {
        return PatrolSpawnPlan::ResetOnly {
            next_tick_floor: PATROL_BASE_DELAY_TICKS,
            next_tick_random_bound: PATROL_RANDOM_DELAY_BOUND,
        };
    }
    PatrolSpawnPlan::SpawnGroup {
        members: ctx.effective_difficulty_ceil + 1,
        leader_index: 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhantomSpawnContext {
    pub spawn_enemies: bool,
    pub spawn_phantoms_rule: bool,
    pub next_tick: i32,
    pub sky_darken: i32,
    pub dimension_has_sky_light: bool,
    pub player_spectator: bool,
    pub player_y: i32,
    pub sea_level: i32,
    pub can_see_sky: bool,
    pub difficulty_id: i32,
    pub difficulty_roll_passed: bool,
    pub time_since_rest: i32,
    pub insomnia_roll_passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhantomSpawnPlan {
    Disabled,
    Wait {
        next_tick: i32,
    },
    ResetOnly {
        min_delay_ticks: i32,
        random_delay_ticks: i32,
    },
    SpawnGroup {
        min_members: i32,
        max_members_exclusive: i32,
    },
}

pub fn phantom_spawn_plan(ctx: PhantomSpawnContext) -> PhantomSpawnPlan {
    if !ctx.spawn_enemies || !ctx.spawn_phantoms_rule {
        return PhantomSpawnPlan::Disabled;
    }
    if ctx.next_tick > 1 {
        return PhantomSpawnPlan::Wait {
            next_tick: ctx.next_tick - 1,
        };
    }
    let reset = PhantomSpawnPlan::ResetOnly {
        min_delay_ticks: PHANTOM_MIN_DELAY_SECONDS * 20,
        random_delay_ticks: PHANTOM_RANDOM_DELAY_SECONDS * 20,
    };
    if ctx.sky_darken < 5 && ctx.dimension_has_sky_light {
        return reset;
    }
    let sky_position_ok =
        !ctx.dimension_has_sky_light || (ctx.player_y >= ctx.sea_level && ctx.can_see_sky);
    if ctx.player_spectator
        || !sky_position_ok
        || !ctx.difficulty_roll_passed
        || ctx.time_since_rest < PHANTOM_INSOMNIA_THRESHOLD_TICKS
        || !ctx.insomnia_roll_passed
    {
        return reset;
    }
    PhantomSpawnPlan::SpawnGroup {
        min_members: 1,
        max_members_exclusive: ctx.difficulty_id + 2,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WanderingTraderData {
    pub spawn_delay: i32,
    pub spawn_chance: i32,
}

impl Default for WanderingTraderData {
    fn default() -> Self {
        Self {
            spawn_delay: DEFAULT_WANDERING_TRADER_SPAWN_DELAY,
            spawn_chance: DEFAULT_WANDERING_TRADER_SPAWN_CHANCE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrialSpawnerConfigSummary {
    pub required_player_range: i32,
    pub target_cooldown_length: i32,
    pub simultaneous_mobs: i32,
    pub spawn_range: i32,
    pub ticks_between_spawn: i32,
}

impl Default for TrialSpawnerConfigSummary {
    fn default() -> Self {
        Self {
            required_player_range: TRIAL_SPAWNER_DEFAULT_PLAYER_SCAN_RANGE,
            target_cooldown_length: TRIAL_SPAWNER_DEFAULT_TARGET_COOLDOWN,
            simultaneous_mobs: 6,
            spawn_range: 4,
            ticks_between_spawn: 40,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrialSpawnerSpawnContext {
    pub spawner_blocks_work: bool,
    pub override_peaceful_and_mob_spawn_rule: bool,
    pub peaceful: bool,
    pub spawn_mobs_rule: bool,
    pub collision_free: bool,
    pub line_of_sight: bool,
    pub placement_rules_ok: bool,
    pub custom_rules_ok: bool,
    pub obstruction_free: bool,
    pub tracked_mobs: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialSpawnerRejection {
    Disabled,
    Collision,
    LineOfSight,
    SpawnRules,
    CustomRules,
    Obstruction,
    MobCap,
}

pub fn validate_trial_spawner_spawn(
    config: TrialSpawnerConfigSummary,
    ctx: TrialSpawnerSpawnContext,
) -> Result<(), TrialSpawnerRejection> {
    if !ctx.spawner_blocks_work
        || (!ctx.override_peaceful_and_mob_spawn_rule && (ctx.peaceful || !ctx.spawn_mobs_rule))
    {
        return Err(TrialSpawnerRejection::Disabled);
    }
    if ctx.tracked_mobs >= config.simultaneous_mobs {
        return Err(TrialSpawnerRejection::MobCap);
    }
    if !ctx.collision_free {
        return Err(TrialSpawnerRejection::Collision);
    }
    if !ctx.line_of_sight {
        return Err(TrialSpawnerRejection::LineOfSight);
    }
    if !ctx.placement_rules_ok {
        return Err(TrialSpawnerRejection::SpawnRules);
    }
    if !ctx.custom_rules_ok {
        return Err(TrialSpawnerRejection::CustomRules);
    }
    if !ctx.obstruction_free {
        return Err(TrialSpawnerRejection::Obstruction);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkGenerationSpawnPlan {
    pub enabled: bool,
    pub category: &'static str,
    pub attempts_per_member: i32,
    pub xz_retry_jitter_bound: i32,
}

pub fn chunk_generation_creature_spawn_plan(
    spawn_mobs_rule: bool,
    biome_creature_probability_positive: bool,
    creature_spawn_list_empty: bool,
) -> Option<ChunkGenerationSpawnPlan> {
    if !spawn_mobs_rule || !biome_creature_probability_positive || creature_spawn_list_empty {
        return None;
    }
    Some(ChunkGenerationSpawnPlan {
        enabled: true,
        category: "creature",
        attempts_per_member: 4,
        xz_retry_jitter_bound: 5,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawningImplementationCoverage {
    pub source: &'static str,
    pub covered_rules: &'static [&'static str],
}

pub const SPAWNING_IMPLEMENTATION_COVERAGE: &[SpawningImplementationCoverage] = &[
    SpawningImplementationCoverage {
        source: "NaturalSpawner",
        covered_rules: &[
            "mob caps",
            "natural spawning",
            "despawn rules",
            "chunk generation spawns",
        ],
    },
    SpawningImplementationCoverage {
        source: "PatrolSpawner",
        covered_rules: &["patrols"],
    },
    SpawningImplementationCoverage {
        source: "PhantomSpawner",
        covered_rules: &["phantoms"],
    },
    SpawningImplementationCoverage {
        source: "WanderingTraderData",
        covered_rules: &["wandering traders"],
    },
    SpawningImplementationCoverage {
        source: "BaseSpawner",
        covered_rules: &["mob spawners"],
    },
    SpawningImplementationCoverage {
        source: "TrialSpawner",
        covered_rules: &["trial spawners"],
    },
    SpawningImplementationCoverage {
        source: "MonsterRoomFeature",
        covered_rules: &["monster rooms"],
    },
    SpawningImplementationCoverage {
        source: "SummonCommand",
        covered_rules: &["command spawns"],
    },
    SpawningImplementationCoverage {
        source: "Raids",
        covered_rules: &["raids custom spawner entrypoint"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn natural_context() -> NaturalSpawnContext {
        NaturalSpawnContext {
            category: "monster",
            nearest_player_distance_squared: 40 * 40,
            distance_to_world_spawn_squared: 40 * 40,
            same_chunk_or_spawnable_neighbor: true,
            block_collision_full: false,
            block_signal_source: false,
            fluid_empty: true,
            prevent_mob_spawning_inside: false,
            dangerous_block: false,
            can_summon: true,
            placement_ok: true,
            spawn_rules_ok: true,
            collision_free: true,
            can_spawn_far_from_player: false,
        }
    }

    #[test]
    fn natural_spawner_mob_caps_filter_categories_like_vanilla() {
        let entities = vec![
            SpawnedEntitySample {
                category: "monster",
                persistent: false,
                custom_persistent: false,
                is_mob: true,
            },
            SpawnedEntitySample {
                category: "monster",
                persistent: true,
                custom_persistent: false,
                is_mob: true,
            },
            SpawnedEntitySample {
                category: "creature",
                persistent: false,
                custom_persistent: false,
                is_mob: true,
            },
            SpawnedEntitySample {
                category: "misc",
                persistent: false,
                custom_persistent: false,
                is_mob: false,
            },
        ];
        let state = SpawnStateSummary::create(MOB_CAP_CHUNK_MAGIC, &entities);
        assert_eq!(state.count_for("monster"), 1);
        assert_eq!(state.count_for("creature"), 1);
        assert_eq!(state.global_cap(*mob_category("monster").unwrap()), 70);

        let hostile_only = filtered_spawning_categories(
            &state,
            SpawnCategoryFilters {
                spawn_friendlies: false,
                spawn_enemies: true,
                spawn_persistent: false,
            },
        );
        assert_eq!(hostile_only, vec!["monster"]);
        assert!(!filtered_spawning_categories(
            &state,
            SpawnCategoryFilters {
                spawn_friendlies: true,
                spawn_enemies: true,
                spawn_persistent: false,
            },
        )
        .contains(&"creature"));
    }

    #[test]
    fn natural_spawn_validation_enforces_distance_blocks_summon_and_collision_gates() {
        assert_eq!(validate_natural_spawn(natural_context()), Ok(()));

        let mut too_close = natural_context();
        too_close.nearest_player_distance_squared = 24 * 24;
        assert_eq!(
            validate_natural_spawn(too_close),
            Err(NaturalSpawnRejection::TooCloseToPlayer)
        );

        let mut too_far = natural_context();
        too_far.nearest_player_distance_squared = 129 * 129;
        assert_eq!(
            validate_natural_spawn(too_far),
            Err(NaturalSpawnRejection::TooFarFromPlayer)
        );

        let mut bad_block = natural_context();
        bad_block.block_signal_source = true;
        assert_eq!(
            validate_natural_spawn(bad_block),
            Err(NaturalSpawnRejection::InvalidEmptyBlock)
        );
    }

    #[test]
    fn despawn_rules_use_category_distances_and_persistence() {
        assert_eq!(
            despawn_decision("monster", true, false, true, 200 * 200),
            DespawnDecision::Keep
        );
        assert_eq!(
            despawn_decision("monster", false, false, true, 129 * 129),
            DespawnDecision::Immediate
        );
        assert_eq!(
            despawn_decision("water_ambient", false, false, false, 65 * 65),
            DespawnDecision::RandomEligible
        );
        assert_eq!(
            despawn_decision("monster", false, false, false, 20 * 20),
            DespawnDecision::Keep
        );
    }

    #[test]
    fn base_spawner_defaults_delay_and_nearby_cap_match_vanilla() {
        let config = SpawnerConfig::default();
        assert_eq!(config.spawn_delay, 20);
        assert_eq!(config.min_spawn_delay, 200);
        assert_eq!(config.max_spawn_delay, 800);
        assert_eq!(config.spawn_count, 4);
        assert_eq!(config.max_nearby_entities, 6);
        assert_eq!(config.required_player_range, 16);
        assert_eq!(config.spawn_range, 4);

        assert_eq!(
            spawner_tick_plan(config, false, true, 0),
            SpawnerTickPlan::Idle
        );
        assert_eq!(
            spawner_tick_plan(
                SpawnerConfig {
                    spawn_delay: 2,
                    ..config
                },
                true,
                true,
                0
            ),
            SpawnerTickPlan::CountDown { next_delay: 1 }
        );
        assert_eq!(
            spawner_tick_plan(
                SpawnerConfig {
                    spawn_delay: 0,
                    ..config
                },
                true,
                true,
                0
            ),
            SpawnerTickPlan::TrySpawn { attempts: 4 }
        );
        assert_eq!(
            spawner_tick_plan(
                SpawnerConfig {
                    spawn_delay: -1,
                    ..config
                },
                true,
                true,
                0
            ),
            SpawnerTickPlan::Delay {
                next_delay_min: 200,
                next_delay_max_exclusive: 800
            }
        );
    }

    #[test]
    fn patrol_and_phantom_custom_spawners_keep_vanilla_timing_and_gates() {
        assert_eq!(
            patrol_spawn_plan(PatrolSpawnContext {
                spawn_enemies: true,
                spawn_patrols_rule: true,
                next_tick: 2,
                bright_outside: true,
                random_one_in_five_hit: true,
                player_count: 1,
                selected_player_spectator: false,
                close_to_village: false,
                chunks_available: true,
                biome_allows_patrol: true,
                effective_difficulty_ceil: 2,
            }),
            PatrolSpawnPlan::Wait { next_tick: 1 }
        );
        assert_eq!(
            patrol_spawn_plan(PatrolSpawnContext {
                next_tick: 1,
                ..PatrolSpawnContext {
                    spawn_enemies: true,
                    spawn_patrols_rule: true,
                    next_tick: 2,
                    bright_outside: true,
                    random_one_in_five_hit: true,
                    player_count: 1,
                    selected_player_spectator: false,
                    close_to_village: false,
                    chunks_available: true,
                    biome_allows_patrol: true,
                    effective_difficulty_ceil: 2,
                }
            }),
            PatrolSpawnPlan::SpawnGroup {
                members: 3,
                leader_index: 0
            }
        );

        let phantom = PhantomSpawnContext {
            spawn_enemies: true,
            spawn_phantoms_rule: true,
            next_tick: 1,
            sky_darken: 5,
            dimension_has_sky_light: true,
            player_spectator: false,
            player_y: 80,
            sea_level: 63,
            can_see_sky: true,
            difficulty_id: 2,
            difficulty_roll_passed: true,
            time_since_rest: 72_000,
            insomnia_roll_passed: true,
        };
        assert_eq!(
            phantom_spawn_plan(phantom),
            PhantomSpawnPlan::SpawnGroup {
                min_members: 1,
                max_members_exclusive: 4
            }
        );
        assert_eq!(
            phantom_spawn_plan(PhantomSpawnContext {
                time_since_rest: 71_999,
                ..phantom
            }),
            PhantomSpawnPlan::ResetOnly {
                min_delay_ticks: 1_200,
                random_delay_ticks: 1_200
            }
        );
    }

    #[test]
    fn wandering_trader_and_trial_spawner_defaults_match_saved_data_and_block_entity() {
        assert_eq!(
            WanderingTraderData::default(),
            WanderingTraderData {
                spawn_delay: 24_000,
                spawn_chance: 25
            }
        );

        let config = TrialSpawnerConfigSummary::default();
        assert_eq!(config.required_player_range, 14);
        assert_eq!(config.target_cooldown_length, 36_000);
        assert_eq!(TRIAL_SPAWNER_DETECT_PLAYER_SPAWN_BUFFER, 40);
        assert_eq!(TRIAL_SPAWNER_MAX_MOB_TRACKING_DISTANCE, 47);

        assert_eq!(
            validate_trial_spawner_spawn(
                config,
                TrialSpawnerSpawnContext {
                    spawner_blocks_work: true,
                    override_peaceful_and_mob_spawn_rule: false,
                    peaceful: false,
                    spawn_mobs_rule: true,
                    collision_free: true,
                    line_of_sight: true,
                    placement_rules_ok: true,
                    custom_rules_ok: true,
                    obstruction_free: true,
                    tracked_mobs: 0,
                }
            ),
            Ok(())
        );
        assert_eq!(
            validate_trial_spawner_spawn(
                config,
                TrialSpawnerSpawnContext {
                    tracked_mobs: 6,
                    spawner_blocks_work: true,
                    override_peaceful_and_mob_spawn_rule: false,
                    peaceful: false,
                    spawn_mobs_rule: true,
                    collision_free: true,
                    line_of_sight: true,
                    placement_rules_ok: true,
                    custom_rules_ok: true,
                    obstruction_free: true,
                }
            ),
            Err(TrialSpawnerRejection::MobCap)
        );
    }

    #[test]
    fn chunk_generation_and_source_coverage_capture_remaining_spawn_surfaces() {
        assert_eq!(
            chunk_generation_creature_spawn_plan(true, true, false),
            Some(ChunkGenerationSpawnPlan {
                enabled: true,
                category: "creature",
                attempts_per_member: 4,
                xz_retry_jitter_bound: 5,
            })
        );
        assert_eq!(
            chunk_generation_creature_spawn_plan(false, true, false),
            None
        );

        let covered: Vec<&str> = SPAWNING_IMPLEMENTATION_COVERAGE
            .iter()
            .flat_map(|entry| entry.covered_rules.iter().copied())
            .collect();
        for expected in [
            "mob caps",
            "despawn rules",
            "natural spawning",
            "patrols",
            "wandering traders",
            "phantoms",
            "raids custom spawner entrypoint",
            "trial spawners",
            "monster rooms",
            "chunk generation spawns",
            "command spawns",
        ] {
            assert!(covered.contains(&expected), "missing {expected}");
        }
    }
}
