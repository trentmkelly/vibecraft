#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonsterBaseSurface {
    pub xp_reward: i32,
    pub sound_source: &'static str,
    pub swim_sound: &'static str,
    pub swim_splash_sound: &'static str,
    pub hurt_sound: &'static str,
    pub death_sound: &'static str,
    pub small_fall_sound: &'static str,
    pub big_fall_sound: &'static str,
    pub attack_damage_attribute_present: bool,
    pub should_drop_experience: bool,
    pub prevents_player_rest: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterSpawnLightInput {
    pub sky_brightness: i32,
    pub random_sky_gate: i32,
    pub block_light_limit: i32,
    pub block_brightness: i32,
    pub thundering: bool,
    pub max_local_raw_brightness: i32,
    pub max_local_raw_brightness_thunder: i32,
    pub sampled_monster_spawn_light_test: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterSpawnRuleInput {
    pub peaceful_difficulty: bool,
    pub spawn_reason_ignores_light: bool,
    pub dark_enough_to_spawn: bool,
    pub mob_spawn_rules_pass: bool,
}

pub const MONSTER_BASE_XP_REWARD: i32 = super::ENEMY_XP_REWARD_MEDIUM;
pub const MONSTER_SOUND_SOURCE: &str = "hostile";
pub const MONSTER_SWIM_SOUND: &str = "minecraft:entity.hostile.swim";
pub const MONSTER_SWIM_SPLASH_SOUND: &str = "minecraft:entity.hostile.splash";
pub const MONSTER_HURT_SOUND: &str = "minecraft:entity.hostile.hurt";
pub const MONSTER_DEATH_SOUND: &str = "minecraft:entity.hostile.death";
pub const MONSTER_SMALL_FALL_SOUND: &str = "minecraft:entity.hostile.small_fall";
pub const MONSTER_BIG_FALL_SOUND: &str = "minecraft:entity.hostile.big_fall";
pub const MONSTER_DEFAULT_PROJECTILE: &str = "minecraft:arrow";

pub fn monster_base_surface() -> MonsterBaseSurface {
    MonsterBaseSurface {
        xp_reward: MONSTER_BASE_XP_REWARD,
        sound_source: MONSTER_SOUND_SOURCE,
        swim_sound: MONSTER_SWIM_SOUND,
        swim_splash_sound: MONSTER_SWIM_SPLASH_SOUND,
        hurt_sound: MONSTER_HURT_SOUND,
        death_sound: MONSTER_DEATH_SOUND,
        small_fall_sound: MONSTER_SMALL_FALL_SOUND,
        big_fall_sound: MONSTER_BIG_FALL_SOUND,
        attack_damage_attribute_present: true,
        should_drop_experience: true,
        prevents_player_rest: true,
    }
}

pub fn monster_update_no_action_time(no_action_time: i32, light_magic_value: f32) -> i32 {
    if light_magic_value > 0.5 {
        no_action_time + 2
    } else {
        no_action_time
    }
}

pub fn monster_walk_target_value(pathfinding_cost_from_light_levels: f32) -> f32 {
    -pathfinding_cost_from_light_levels
}

pub fn monster_is_dark_enough_to_spawn(input: MonsterSpawnLightInput) -> bool {
    if input.sky_brightness > input.random_sky_gate {
        return false;
    }

    if input.block_light_limit < 15 && input.block_brightness > input.block_light_limit {
        return false;
    }

    let brightness = if input.thundering {
        input.max_local_raw_brightness_thunder
    } else {
        input.max_local_raw_brightness
    };
    brightness <= input.sampled_monster_spawn_light_test
}

pub fn monster_spawn_rules(input: MonsterSpawnRuleInput) -> bool {
    !input.peaceful_difficulty
        && (input.spawn_reason_ignores_light || input.dark_enough_to_spawn)
        && input.mob_spawn_rules_pass
}

pub fn monster_any_light_spawn_rules(peaceful_difficulty: bool, mob_spawn_rules_pass: bool) -> bool {
    !peaceful_difficulty && mob_spawn_rules_pass
}

pub fn monster_surface_spawn_rules(
    base_monster_spawn_rules_pass: bool,
    spawn_reason_is_spawner: bool,
    can_see_sky: bool,
) -> bool {
    base_monster_spawn_rules_pass && (spawn_reason_is_spawner || can_see_sky)
}

pub fn monster_should_drop_loot(mob_drops_game_rule: bool) -> bool {
    mob_drops_game_rule
}

pub fn monster_projectile(
    held_weapon_is_projectile_weapon: bool,
    held_supported_projectile: Option<&str>,
) -> Option<&str> {
    if !held_weapon_is_projectile_weapon {
        return None;
    }

    Some(held_supported_projectile.unwrap_or(MONSTER_DEFAULT_PROJECTILE))
}
