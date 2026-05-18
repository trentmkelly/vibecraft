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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodState {
    pub food_level: i32,
    pub saturation: f32,
    pub exhaustion: f32,
}

impl Default for FoodState {
    fn default() -> Self {
        Self {
            food_level: 20,
            saturation: 5.0,
            exhaustion: 0.0,
        }
    }
}

impl FoodState {
    pub fn add_exhaustion(&mut self, amount: f32) {
        self.exhaustion += amount.max(0.0);
        while self.exhaustion >= 4.0 {
            self.exhaustion -= 4.0;
            if self.saturation > 0.0 {
                self.saturation = (self.saturation - 1.0).max(0.0);
            } else {
                self.food_level = (self.food_level - 1).max(0);
            }
        }
    }

    pub fn eat(&mut self, nutrition: i32, saturation_modifier: f32) {
        self.food_level = (self.food_level + nutrition).min(20);
        self.saturation = (self.saturation + nutrition as f32 * saturation_modifier * 2.0)
            .min(self.food_level as f32);
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

impl ExperienceState {
    pub fn set_level(&mut self, level: i32) {
        self.level = level.max(0);
    }

    pub fn set_progress_points(&mut self, amount: i32, needed_for_next_level: i32) {
        let limit = needed_for_next_level.max(1) as f32;
        self.progress = (amount as f32 / limit).clamp(0.0, 1.0);
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
        player.food.add_exhaustion(8.5);
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

    fn stat_value(player: &PlayerEntityState, stat: &'static str) -> i32 {
        player
            .stats
            .iter()
            .find_map(|(id, value)| (*id == stat).then_some(*value))
            .unwrap_or(0)
    }
}
