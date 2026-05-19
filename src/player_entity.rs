#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerGameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerAbilitiesState {
    pub invulnerable: bool,
    pub flying: bool,
    pub may_fly: bool,
    pub instabuild: bool,
    pub may_build: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

impl PlayerAbilitiesState {
    pub fn for_game_mode(mode: PlayerGameMode) -> Self {
        match mode {
            PlayerGameMode::Creative => Self {
                invulnerable: true,
                flying: false,
                may_fly: true,
                instabuild: true,
                may_build: true,
                flying_speed: 0.05,
                walking_speed: 0.1,
            },
            PlayerGameMode::Spectator => Self {
                invulnerable: true,
                flying: true,
                may_fly: true,
                instabuild: false,
                may_build: false,
                flying_speed: 0.05,
                walking_speed: 0.1,
            },
            PlayerGameMode::Adventure => Self {
                invulnerable: false,
                flying: false,
                may_fly: false,
                instabuild: false,
                may_build: false,
                flying_speed: 0.05,
                walking_speed: 0.1,
            },
            PlayerGameMode::Survival => Self {
                invulnerable: false,
                flying: false,
                may_fly: false,
                instabuild: false,
                may_build: true,
                flying_speed: 0.05,
                walking_speed: 0.1,
            },
        }
    }
}

/// Exhaustion cost per metre sprinted on ground (ServerPlayer.checkMovementStatistics).
pub const SPRINT_EXHAUSTION_PER_METER: f32 = 0.1;
/// Exhaustion cost per metre walked/crouched on ground (0.0F in vanilla — explicit no-op).
pub const WALK_EXHAUSTION_PER_METER: f32 = 0.0;
/// Exhaustion cost per metre swum, walked under/on water (ServerPlayer.checkMovementStatistics).
pub const SWIM_EXHAUSTION_PER_METER: f32 = 0.01;
/// Exhaustion cost for a non-sprint jump (ServerPlayer.jumpFromGround).
pub const JUMP_EXHAUSTION: f32 = 0.05;
/// Exhaustion cost for a sprint jump (ServerPlayer.jumpFromGround).
pub const SPRINT_JUMP_EXHAUSTION: f32 = 0.2;

/// Returns the exhaustion for a distance-based movement action.
/// `exhaustion_per_meter` is the per-metre cost; `distance_cm` is centimetres
/// (vanilla stats track movement in 1/100-metre units).
pub fn movement_exhaustion(exhaustion_per_meter: f32, distance_cm: i32) -> f32 {
    exhaustion_per_meter * distance_cm as f32 * 0.01
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

/// Whether starvation should deal damage given difficulty and current health.
/// Matches Java FoodData.tick: always on Hard, above 1 HP on Normal, above 10 HP on Easy.
pub fn starvation_damages(difficulty: Difficulty, health: f32) -> bool {
    match difficulty {
        Difficulty::Peaceful => false,
        Difficulty::Easy => health > 10.0,
        Difficulty::Normal => health > 1.0,
        Difficulty::Hard => true,
    }
}

/// Outcome produced by one call to `FoodState::tick_food`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FoodTickOutcome {
    /// No action needed this tick.
    None,
    /// Saturation-based fast regeneration: heal `amount` HP and accumulate `exhaustion_cost`.
    FastHeal { amount: f32, exhaustion_cost: f32 },
    /// Food-level slow regeneration (food ≥ 18): heal 1 HP and accumulate 6.0 exhaustion.
    SlowHeal,
    /// Starvation attempt (food = 0): caller checks `starvation_damages()` and applies 1 damage.
    StarveAttempt,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodState {
    pub food_level: i32,
    pub saturation: f32,
    pub exhaustion: f32,
    pub tick_timer: i32,
}

impl Default for FoodState {
    fn default() -> Self {
        Self {
            food_level: 20,
            saturation: 5.0,
            exhaustion: 0.0,
            tick_timer: 0,
        }
    }
}

impl FoodState {
    /// Accumulate exhaustion, capped at 40.0.  Drain (saturation / food) happens
    /// once per tick in `tick_food`, matching Java `FoodData.addExhaustion` + `FoodData.tick`.
    pub fn add_exhaustion(&mut self, amount: f32) {
        self.exhaustion = (self.exhaustion + amount.max(0.0)).min(40.0);
    }

    pub fn eat(&mut self, nutrition: i32, saturation_modifier: f32) {
        self.food_level = (self.food_level + nutrition).min(20);
        self.saturation = (self.saturation + nutrition as f32 * saturation_modifier * 2.0)
            .min(self.food_level as f32);
    }

    /// Tick the food system.  Call once per server tick per player.
    ///
    /// * `is_hurt` — player health < max health.
    /// * `natural_regen` — `naturalHealthRegeneration` gamerule is on.
    /// * `difficulty` — current world difficulty.
    ///
    /// Matches Java `FoodData.tick` exactly: drain exhaustion first, then decide regen/starvation.
    pub fn tick_food(
        &mut self,
        is_hurt: bool,
        natural_regen: bool,
        difficulty: Difficulty,
    ) -> FoodTickOutcome {
        // Step 1: drain exhaustion (one drain per tick when exhaustion > 4.0).
        if self.exhaustion > 4.0 {
            self.exhaustion -= 4.0;
            if self.saturation > 0.0 {
                self.saturation = (self.saturation - 1.0).max(0.0);
            } else if difficulty != Difficulty::Peaceful {
                self.food_level = (self.food_level - 1).max(0);
            }
        }

        // Step 2: regen or starvation.
        if natural_regen && self.saturation > 0.0 && is_hurt && self.food_level >= 20 {
            self.tick_timer += 1;
            if self.tick_timer >= 10 {
                let spent = self.saturation.min(6.0);
                let heal = spent / 6.0;
                self.add_exhaustion(spent);
                self.tick_timer = 0;
                FoodTickOutcome::FastHeal {
                    amount: heal,
                    exhaustion_cost: spent,
                }
            } else {
                FoodTickOutcome::None
            }
        } else if natural_regen && self.food_level >= 18 && is_hurt {
            self.tick_timer += 1;
            if self.tick_timer >= 80 {
                self.add_exhaustion(6.0);
                self.tick_timer = 0;
                FoodTickOutcome::SlowHeal
            } else {
                FoodTickOutcome::None
            }
        } else if self.food_level <= 0 {
            self.tick_timer += 1;
            if self.tick_timer >= 80 {
                self.tick_timer = 0;
                FoodTickOutcome::StarveAttempt
            } else {
                FoodTickOutcome::None
            }
        } else {
            self.tick_timer = 0;
            FoodTickOutcome::None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExperienceState {
    pub level: i32,
    pub progress: f32,
    pub total: i32,
}

impl Default for ExperienceState {
    fn default() -> Self {
        Self {
            level: 0,
            progress: 0.0,
            total: 0,
        }
    }
}

/// XP required to advance from `level` to `level + 1`.
/// Matches Java `Player.getXpNeededForNextLevel()`.
pub fn xp_needed_for_next_level(level: i32) -> i32 {
    if level >= 30 {
        112 + (level - 30) * 9
    } else if level >= 15 {
        37 + (level - 15) * 5
    } else {
        7 + level * 2
    }
}

/// XP dropped as orbs when a player dies.
/// Matches Java `Player.getBaseExperienceReward()`: 0 if keepInventory or spectator.
pub fn player_xp_reward_on_death(level: i32, keep_inventory: bool, is_spectator: bool) -> i32 {
    if !keep_inventory && !is_spectator {
        (level * 7).min(100)
    } else {
        0
    }
}

impl ExperienceState {
    pub fn set_level(&mut self, level: i32) {
        self.level = level.max(0);
    }

    pub fn set_progress_points(&mut self, amount: i32, needed_for_next_level: i32) {
        let limit = needed_for_next_level.max(1) as f32;
        self.progress = (amount as f32 / limit).clamp(0.0, 1.0);
    }

    /// Award or remove XP points, advancing/retreating levels as needed.
    /// Matches Java `Player.giveExperiencePoints()` exactly including level
    /// threshold recomputation between level changes.
    pub fn add_experience(&mut self, points: i32) {
        self.progress += points as f32 / xp_needed_for_next_level(self.level) as f32;
        self.total = self.total.saturating_add(points).max(0);

        while self.progress < 0.0 {
            let needed_before = xp_needed_for_next_level(self.level) as f32;
            let remaining = self.progress * needed_before;
            if self.level > 0 {
                self.level -= 1;
                self.progress = 1.0 + remaining / xp_needed_for_next_level(self.level) as f32;
            } else {
                self.level = 0;
                self.progress = 0.0;
                self.total = 0;
                break;
            }
        }

        while self.progress >= 1.0 {
            let needed_before = xp_needed_for_next_level(self.level) as f32;
            self.progress = (self.progress - 1.0) * needed_before;
            self.level = self.level.saturating_add(1);
            self.progress /= xp_needed_for_next_level(self.level) as f32;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespawnConfig {
    pub dimension: &'static str,
    pub pos: (i32, i32, i32),
    pub yaw: i16,
    pub pitch: i16,
    pub forced: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerEntityState {
    pub name: String,
    pub game_mode: PlayerGameMode,
    pub previous_game_mode: Option<PlayerGameMode>,
    pub abilities: PlayerAbilitiesState,
    pub food: FoodState,
    pub experience: ExperienceState,
    pub stats: Vec<(&'static str, i32)>,
    pub advancements_dirty: bool,
    pub known_recipes: Vec<&'static str>,
    pub recipe_book_open: bool,
    pub recipe_book_filtering: bool,
    pub respawn: Option<RespawnConfig>,
    pub sleeping: bool,
    pub sleep_counter: i32,
    pub permission_level: u8,
    pub interaction_mode: PlayerInteractionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerInteractionMode {
    Normal,
    NoBuild,
    Spectator,
}

impl PlayerEntityState {
    pub fn new(name: impl Into<String>, mode: PlayerGameMode, permission_level: u8) -> Self {
        Self {
            name: name.into(),
            game_mode: mode,
            previous_game_mode: None,
            abilities: PlayerAbilitiesState::for_game_mode(mode),
            food: FoodState::default(),
            experience: ExperienceState::default(),
            stats: Vec::new(),
            advancements_dirty: false,
            known_recipes: Vec::new(),
            recipe_book_open: false,
            recipe_book_filtering: false,
            respawn: None,
            sleeping: false,
            sleep_counter: 0,
            permission_level,
            interaction_mode: interaction_mode_for(mode),
        }
    }

    pub fn set_game_mode(&mut self, mode: PlayerGameMode) -> bool {
        if self.game_mode == mode {
            return false;
        }
        self.previous_game_mode = Some(self.game_mode);
        self.game_mode = mode;
        self.abilities = PlayerAbilitiesState::for_game_mode(mode);
        self.interaction_mode = interaction_mode_for(mode);
        true
    }

    pub fn award_stat(&mut self, stat: &'static str, amount: i32) {
        if let Some((_, value)) = self.stats.iter_mut().find(|(id, _)| *id == stat) {
            *value += amount;
        } else {
            self.stats.push((stat, amount));
        }
    }

    pub fn reset_stat(&mut self, stat: &'static str) {
        self.stats.retain(|(id, _)| *id != stat);
    }

    pub fn award_recipe(&mut self, recipe: &'static str) -> bool {
        if self.known_recipes.contains(&recipe) {
            false
        } else {
            self.known_recipes.push(recipe);
            true
        }
    }

    pub fn remove_recipe(&mut self, recipe: &'static str) -> bool {
        let before = self.known_recipes.len();
        self.known_recipes.retain(|known| *known != recipe);
        before != self.known_recipes.len()
    }

    pub fn set_respawn_position(&mut self, respawn: Option<RespawnConfig>) -> bool {
        let changed = self.respawn != respawn;
        self.respawn = respawn;
        changed
    }

    pub fn start_sleeping(&mut self) {
        self.sleeping = true;
        self.sleep_counter = 0;
        self.award_stat("minecraft:sleep_in_bed", 1);
        self.reset_stat("minecraft:time_since_rest");
    }

    pub fn tick_sleep(&mut self) {
        if self.sleeping {
            self.sleep_counter = (self.sleep_counter + 1).min(100);
        } else if self.sleep_counter > 0 {
            self.sleep_counter += 1;
            if self.sleep_counter >= 110 {
                self.sleep_counter = 0;
            }
        }
    }

    pub fn stop_sleeping(&mut self) {
        self.sleeping = false;
    }

    pub fn can_use_command_level(&self, level: u8) -> bool {
        self.permission_level >= level
    }

    pub fn can_interact_with_block(&self) -> bool {
        self.interaction_mode == PlayerInteractionMode::Normal
    }

    pub fn die(&mut self) {
        self.award_stat("minecraft:deaths", 1);
        self.reset_stat("minecraft:time_since_death");
        self.reset_stat("minecraft:time_since_rest");
    }

    pub fn tick_server(&mut self, crouching: bool) {
        self.award_stat("minecraft:play_time", 1);
        self.award_stat("minecraft:total_world_time", 1);
        self.award_stat("minecraft:time_since_death", 1);
        if crouching {
            self.award_stat("minecraft:crouch_time", 1);
        }
    }

    pub fn sync_plan(&self) -> PlayerSyncPlan {
        PlayerSyncPlan {
            abilities: self.abilities,
            game_mode: self.game_mode,
            previous_game_mode: self.previous_game_mode,
            health_food_packet: (self.food.food_level, self.food.saturation),
            experience_packet: self.experience,
            stats: self.stats.clone(),
            advancements_dirty: self.advancements_dirty,
            recipes: self.known_recipes.clone(),
            recipe_book_open: self.recipe_book_open,
            recipe_book_filtering: self.recipe_book_filtering,
            respawn: self.respawn.clone(),
            permission_level: self.permission_level,
        }
    }

    pub fn save(&self) -> SavedPlayerEntity {
        SavedPlayerEntity {
            game_mode: self.game_mode,
            previous_game_mode: self.previous_game_mode,
            food: self.food,
            experience: self.experience,
            recipes: self.known_recipes.clone(),
            recipe_book_open: self.recipe_book_open,
            recipe_book_filtering: self.recipe_book_filtering,
            respawn: self.respawn.clone(),
        }
    }

    pub fn load(&mut self, saved: SavedPlayerEntity) {
        self.game_mode = saved.game_mode;
        self.previous_game_mode = saved.previous_game_mode;
        self.abilities = PlayerAbilitiesState::for_game_mode(saved.game_mode);
        self.interaction_mode = interaction_mode_for(saved.game_mode);
        self.food = saved.food;
        self.experience = saved.experience;
        self.known_recipes = saved.recipes;
        self.recipe_book_open = saved.recipe_book_open;
        self.recipe_book_filtering = saved.recipe_book_filtering;
        self.respawn = saved.respawn;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerSyncPlan {
    pub abilities: PlayerAbilitiesState,
    pub game_mode: PlayerGameMode,
    pub previous_game_mode: Option<PlayerGameMode>,
    pub health_food_packet: (i32, f32),
    pub experience_packet: ExperienceState,
    pub stats: Vec<(&'static str, i32)>,
    pub advancements_dirty: bool,
    pub recipes: Vec<&'static str>,
    pub recipe_book_open: bool,
    pub recipe_book_filtering: bool,
    pub respawn: Option<RespawnConfig>,
    pub permission_level: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SavedPlayerEntity {
    pub game_mode: PlayerGameMode,
    pub previous_game_mode: Option<PlayerGameMode>,
    pub food: FoodState,
    pub experience: ExperienceState,
    pub recipes: Vec<&'static str>,
    pub recipe_book_open: bool,
    pub recipe_book_filtering: bool,
    pub respawn: Option<RespawnConfig>,
}

fn interaction_mode_for(mode: PlayerGameMode) -> PlayerInteractionMode {
    match mode {
        PlayerGameMode::Survival | PlayerGameMode::Creative => PlayerInteractionMode::Normal,
        PlayerGameMode::Adventure => PlayerInteractionMode::NoBuild,
        PlayerGameMode::Spectator => PlayerInteractionMode::Spectator,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_modes_update_abilities_previous_mode_and_interaction_mode() {
        let mut player = PlayerEntityState::new("Steve", PlayerGameMode::Survival, 2);
        assert!(player.abilities.may_build);
        assert!(player.set_game_mode(PlayerGameMode::Creative));
        assert_eq!(player.previous_game_mode, Some(PlayerGameMode::Survival));
        assert!(player.abilities.instabuild);
        assert!(player.abilities.may_fly);
        assert_eq!(player.interaction_mode, PlayerInteractionMode::Normal);

        player.set_game_mode(PlayerGameMode::Spectator);
        assert!(player.abilities.flying);
        assert!(!player.abilities.may_build);
        assert_eq!(player.interaction_mode, PlayerInteractionMode::Spectator);

        player.set_game_mode(PlayerGameMode::Adventure);
        assert_eq!(player.interaction_mode, PlayerInteractionMode::NoBuild);
        assert!(!player.can_interact_with_block());
    }

    #[test]
    fn hunger_saturation_exhaustion_experience_and_tick_stats_follow_server_paths() {
        let mut player = PlayerEntityState::new("Alex", PlayerGameMode::Survival, 0);
        // add_exhaustion now only accumulates (Java parity); drain happens in tick_food.
        // Two ticks at 4.0-per-tick: exhaustion 8.5→4.5→0.5, saturation 5.0→4.0→3.0.
        player.food.add_exhaustion(8.5);
        player.food.tick_food(false, false, Difficulty::Normal);
        player.food.tick_food(false, false, Difficulty::Normal);
        assert_eq!(player.food.saturation, 3.0);
        assert_eq!(player.food.food_level, 20);
        player.food.eat(4, 0.3);
        assert_eq!(player.food.food_level, 20);
        assert!(player.food.saturation > 3.0);

        player.experience.set_level(7);
        player.experience.set_progress_points(5, 10);
        assert_eq!(player.experience.level, 7);
        assert_eq!(player.experience.progress, 0.5);

        player.tick_server(true);
        assert_eq!(stat_value(&player, "minecraft:play_time"), 1);
        assert_eq!(stat_value(&player, "minecraft:total_world_time"), 1);
        assert_eq!(stat_value(&player, "minecraft:crouch_time"), 1);
    }

    #[test]
    fn stats_advancements_recipe_book_spawn_sleep_and_permissions_are_tracked() {
        let mut player = PlayerEntityState::new("Steve", PlayerGameMode::Survival, 3);
        player.advancements_dirty = true;
        assert!(player.award_recipe("minecraft:planks"));
        assert!(!player.award_recipe("minecraft:planks"));
        assert!(player.remove_recipe("minecraft:planks"));

        let respawn = RespawnConfig {
            dimension: "minecraft:overworld",
            pos: (1, 70, 2),
            yaw: 90,
            pitch: 0,
            forced: false,
        };
        assert!(player.set_respawn_position(Some(respawn.clone())));
        assert!(!player.set_respawn_position(Some(respawn)));

        player.start_sleeping();
        for _ in 0..150 {
            player.tick_sleep();
        }
        assert_eq!(player.sleep_counter, 100);
        assert_eq!(stat_value(&player, "minecraft:sleep_in_bed"), 1);
        player.stop_sleeping();
        player.tick_sleep();
        assert_eq!(player.sleep_counter, 101);

        assert!(player.can_use_command_level(2));
        assert!(!player.can_use_command_level(4));
    }

    #[test]
    fn death_save_load_and_sync_plan_cover_server_player_packet_state() {
        let mut player = PlayerEntityState::new("Steve", PlayerGameMode::Creative, 4);
        player.food.food_level = 17;
        player.food.saturation = 0.0;
        player.experience = ExperienceState {
            level: 3,
            progress: 0.25,
            total: 42,
        };
        player.award_recipe("minecraft:stick");
        player.recipe_book_open = true;
        player.recipe_book_filtering = true;
        player.advancements_dirty = true;
        player.die();

        let sync = player.sync_plan();
        assert!(sync.abilities.instabuild);
        assert_eq!(sync.health_food_packet, (17, 0.0));
        assert_eq!(sync.experience_packet.level, 3);
        assert_eq!(sync.recipes, vec!["minecraft:stick"]);
        assert!(sync.recipe_book_open);
        assert!(sync.recipe_book_filtering);
        assert!(sync.advancements_dirty);
        assert_eq!(stat_value(&player, "minecraft:deaths"), 1);
        assert_eq!(stat_value(&player, "minecraft:time_since_death"), 0);

        let saved = player.save();
        let mut loaded = PlayerEntityState::new("Steve", PlayerGameMode::Survival, 0);
        loaded.load(saved);
        assert_eq!(loaded.game_mode, PlayerGameMode::Creative);
        assert!(loaded.abilities.instabuild);
        assert_eq!(loaded.food.food_level, 17);
        assert_eq!(loaded.known_recipes, vec!["minecraft:stick"]);
        assert!(loaded.recipe_book_open);
        assert!(loaded.recipe_book_filtering);
    }

    #[test]
    fn respawn_position_round_trips_bed_anchor_and_missing_fallback_state() {
        let mut player = PlayerEntityState::new("Steve", PlayerGameMode::Survival, 0);
        let bed_respawn = RespawnConfig {
            dimension: "minecraft:overworld",
            pos: (12, 64, -8),
            yaw: 180,
            pitch: 0,
            forced: false,
        };
        assert!(player.set_respawn_position(Some(bed_respawn.clone())));
        assert_eq!(player.sync_plan().respawn, Some(bed_respawn.clone()));

        let mut loaded = PlayerEntityState::new("Steve", PlayerGameMode::Survival, 0);
        loaded.load(player.save());
        assert_eq!(loaded.respawn, Some(bed_respawn));

        let anchor_respawn = RespawnConfig {
            dimension: "minecraft:the_nether",
            pos: (-4, 71, 9),
            yaw: -45,
            pitch: 0,
            forced: true,
        };
        assert!(loaded.set_respawn_position(Some(anchor_respawn.clone())));
        assert_eq!(loaded.save().respawn, Some(anchor_respawn.clone()));
        assert_eq!(loaded.sync_plan().respawn, Some(anchor_respawn));

        assert!(loaded.set_respawn_position(None));
        loaded.die();
        assert_eq!(loaded.save().respawn, None);
        assert_eq!(loaded.sync_plan().respawn, None);
        assert_eq!(stat_value(&loaded, "minecraft:deaths"), 1);
        assert_eq!(stat_value(&loaded, "minecraft:time_since_death"), 0);
    }

    #[test]
    fn all_game_modes_are_represented() {
        let modes = [
            PlayerGameMode::Survival,
            PlayerGameMode::Creative,
            PlayerGameMode::Adventure,
            PlayerGameMode::Spectator,
        ];
        assert_eq!(modes.len(), 4);
        assert_eq!(
            modes.map(interaction_mode_for),
            [
                PlayerInteractionMode::Normal,
                PlayerInteractionMode::Normal,
                PlayerInteractionMode::NoBuild,
                PlayerInteractionMode::Spectator
            ]
        );
    }

    #[test]
    fn xp_threshold_formula_and_add_experience_match_java() {
        // xp_needed_for_next_level thresholds from Player.getXpNeededForNextLevel()
        assert_eq!(xp_needed_for_next_level(0), 7);
        assert_eq!(xp_needed_for_next_level(14), 35);
        assert_eq!(xp_needed_for_next_level(15), 37);
        assert_eq!(xp_needed_for_next_level(29), 107); // 37 + (29-15)*5
        assert_eq!(xp_needed_for_next_level(30), 112);
        assert_eq!(xp_needed_for_next_level(31), 121);

        // Gaining enough XP to cross a level boundary.
        let mut xp = ExperienceState::default();
        xp.add_experience(7); // exactly one level from 0
        assert_eq!(xp.level, 1);
        assert_eq!(xp.progress, 0.0);
        assert_eq!(xp.total, 7);

        // xp_reward_on_death
        assert_eq!(player_xp_reward_on_death(0, false, false), 0);
        assert_eq!(player_xp_reward_on_death(1, false, false), 7);
        assert_eq!(player_xp_reward_on_death(14, false, false), 98);
        assert_eq!(player_xp_reward_on_death(15, false, false), 100);
        assert_eq!(player_xp_reward_on_death(100, false, false), 100);
        assert_eq!(player_xp_reward_on_death(10, true, false), 0);
        assert_eq!(player_xp_reward_on_death(10, false, true), 0);

        // Taking XP away bottoms out at level 0, progress 0, total 0.
        let mut xp2 = ExperienceState::default();
        xp2.add_experience(-50);
        assert_eq!(xp2.level, 0);
        assert_eq!(xp2.progress, 0.0);
        assert_eq!(xp2.total, 0);
    }

    #[test]
    fn starvation_damages_matches_java_difficulty_rules() {
        assert!(!starvation_damages(Difficulty::Peaceful, 1.0));
        assert!(!starvation_damages(Difficulty::Easy, 10.0));
        assert!(starvation_damages(Difficulty::Easy, 10.1));
        assert!(!starvation_damages(Difficulty::Normal, 1.0));
        assert!(starvation_damages(Difficulty::Normal, 1.1));
        assert!(starvation_damages(Difficulty::Hard, 0.5));
    }

    fn stat_value(player: &PlayerEntityState, stat: &'static str) -> i32 {
        player
            .stats
            .iter()
            .find_map(|(id, value)| (*id == stat).then_some(*value))
            .unwrap_or(0)
    }
}
