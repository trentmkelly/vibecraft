use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnowGolemState {
    pumpkin_flags: u8,
    pub alive: bool,
}

impl SnowGolemState {
    pub fn new() -> Self {
        Self {
            pumpkin_flags: SNOW_GOLEM_PUMPKIN_FLAG,
            alive: true,
        }
    }

    pub fn from_saved_pumpkin(has_pumpkin: bool) -> Self {
        let mut state = Self::new();
        state.set_pumpkin(has_pumpkin);
        state
    }

    pub fn has_pumpkin(self) -> bool {
        (self.pumpkin_flags & SNOW_GOLEM_PUMPKIN_FLAG) != 0
    }

    pub fn set_pumpkin(&mut self, pumpkin: bool) {
        if pumpkin {
            self.pumpkin_flags |= SNOW_GOLEM_PUMPKIN_FLAG;
        } else {
            self.pumpkin_flags &= !SNOW_GOLEM_PUMPKIN_FLAG;
        }
    }

    pub fn saved_pumpkin(self) -> bool {
        self.has_pumpkin()
    }

    pub fn ready_for_shearing(self) -> bool {
        self.alive && self.has_pumpkin()
    }

    pub fn shear(&mut self) -> SnowGolemShearResult {
        if !self.ready_for_shearing() {
            return SnowGolemShearResult::Pass;
        }
        self.set_pumpkin(false);
        SnowGolemShearResult::Sheared {
            sound: "minecraft:entity.snow_golem.shear",
            game_event: "minecraft:shear",
            loot_table: "minecraft:entities/shear/snow_golem",
            tool_damage: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnowGolemShearResult {
    Pass,
    Sheared {
        sound: &'static str,
        game_event: &'static str,
        loot_table: &'static str,
        tool_damage: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnowGolemAiStepPlan {
    pub melt_damage: Option<i32>,
    pub snow_positions: [(i32, i32, i32); 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnowGolemSnowballPlan {
    pub item: &'static str,
    pub velocity: (f32, f32, f32),
    pub speed: f32,
    pub inaccuracy: f32,
    pub sound: &'static str,
}

pub fn snow_golem_ai_step_plan(
    x: f32,
    y: f32,
    z: f32,
    melts_in_environment: bool,
    mob_griefing: bool,
) -> SnowGolemAiStepPlan {
    let mut snow_positions = [(0, 0, 0); 4];
    if mob_griefing {
        for (i, slot) in snow_positions.iter_mut().enumerate() {
            let xx = (x + ((i % 2) as i32 * 2 - 1) as f32 * 0.25).floor() as i32;
            let yy = y.floor() as i32;
            let zz = (z + (((i / 2) % 2) as i32 * 2 - 1) as f32 * 0.25).floor() as i32;
            *slot = (xx, yy, zz);
        }
    }
    SnowGolemAiStepPlan {
        melt_damage: melts_in_environment.then_some(1),
        snow_positions,
    }
}

pub fn snow_golem_snow_placement_allowed(
    current_block: &'static str,
    snow_can_survive: bool,
) -> bool {
    current_block == "minecraft:air" && snow_can_survive
}

pub fn snow_golem_ranged_attack_plan(
    self_x: f32,
    self_z: f32,
    target_x: f32,
    target_eye_y: f32,
    target_z: f32,
    projectile_y: f32,
) -> SnowGolemSnowballPlan {
    let xd = target_x - self_x;
    let yd = target_eye_y - 1.1;
    let zd = target_z - self_z;
    let yo = (xd * xd + zd * zd).sqrt() * 0.2;
    SnowGolemSnowballPlan {
        item: "minecraft:snowball",
        velocity: (xd, yd + yo - projectile_y, zd),
        speed: SNOW_GOLEM_SNOWBALL_SPEED,
        inaccuracy: SNOW_GOLEM_SNOWBALL_INACCURACY,
        sound: "minecraft:entity.snow_golem.shoot",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IronGolemCrackiness {
    None,
    Low,
    Medium,
    High,
}

pub fn iron_golem_crackiness(health: f32, max_health: f32) -> IronGolemCrackiness {
    let fraction = health / max_health;
    if fraction < 0.25 {
        IronGolemCrackiness::High
    } else if fraction < 0.5 {
        IronGolemCrackiness::Medium
    } else if fraction < 0.75 {
        IronGolemCrackiness::Low
    } else {
        IronGolemCrackiness::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IronGolemState {
    flags: u8,
    pub health: f32,
    pub max_health: f32,
    pub attack_animation_tick: i32,
    pub offer_flower_tick: i32,
}

impl IronGolemState {
    pub fn new() -> Self {
        Self {
            flags: 0,
            health: IRON_GOLEM_MAX_HEALTH,
            max_health: IRON_GOLEM_MAX_HEALTH,
            attack_animation_tick: 0,
            offer_flower_tick: 0,
        }
    }

    pub fn from_saved_player_created(player_created: bool) -> Self {
        let mut state = Self::new();
        state.set_player_created(player_created);
        state
    }

    pub fn is_player_created(self) -> bool {
        (self.flags & IRON_GOLEM_PLAYER_CREATED_FLAG) != 0
    }

    pub fn set_player_created(&mut self, value: bool) {
        if value {
            self.flags |= IRON_GOLEM_PLAYER_CREATED_FLAG;
        } else {
            self.flags &= !IRON_GOLEM_PLAYER_CREATED_FLAG;
        }
    }

    pub fn saved_player_created(self) -> bool {
        self.is_player_created()
    }

    pub fn ai_step(&mut self) {
        self.attack_animation_tick = (self.attack_animation_tick - 1).max(0);
        self.offer_flower_tick = (self.offer_flower_tick - 1).max(0);
    }

    pub fn handle_entity_event(&mut self, event_id: u8) {
        match event_id {
            4 => self.attack_animation_tick = IRON_GOLEM_ATTACK_ANIMATION_TICKS,
            11 => self.offer_flower_tick = IRON_GOLEM_OFFER_FLOWER_TICKS,
            34 => self.offer_flower_tick = 0,
            _ => {}
        }
    }

    pub fn offer_flower(&mut self, offer: bool) -> u8 {
        if offer {
            self.offer_flower_tick = IRON_GOLEM_OFFER_FLOWER_TICKS;
            11
        } else {
            self.offer_flower_tick = 0;
            34
        }
    }

    pub fn repair_with_iron_ingot(&mut self, item: &'static str) -> IronGolemRepairResult {
        if item != "minecraft:iron_ingot" {
            return IronGolemRepairResult::Pass;
        }
        let before = self.health;
        self.health = (self.health + IRON_GOLEM_REPAIR_HEAL_AMOUNT).min(self.max_health);
        if self.health == before {
            IronGolemRepairResult::Pass
        } else {
            IronGolemRepairResult::Repaired {
                consumed: 1,
                healed: self.health - before,
                sound: "minecraft:entity.iron_golem.repair",
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IronGolemRepairResult {
    Pass,
    Repaired {
        consumed: i32,
        healed: f32,
        sound: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IronGolemAttackPlan {
    pub event_id: u8,
    pub attack_animation_tick: i32,
    pub damage: f32,
    pub target_delta_y: f32,
    pub sound: &'static str,
}

pub fn iron_golem_attack_plan(
    attack_damage: f32,
    random_next_int_attack_damage: i32,
    target_knockback_resistance: f32,
) -> IronGolemAttackPlan {
    let damage = if attack_damage as i32 > 0 {
        attack_damage / 2.0 + random_next_int_attack_damage as f32
    } else {
        attack_damage
    };
    IronGolemAttackPlan {
        event_id: 4,
        attack_animation_tick: IRON_GOLEM_ATTACK_ANIMATION_TICKS,
        damage,
        target_delta_y: 0.4 * (1.0 - target_knockback_resistance).max(0.0),
        sound: "minecraft:entity.iron_golem.attack",
    }
}

pub fn iron_golem_can_attack_target(
    player_created: bool,
    target_entity_type: &'static str,
) -> bool {
    if target_entity_type == "minecraft:creeper" {
        return false;
    }
    !(player_created && target_entity_type == "minecraft:player")
}

pub fn iron_golem_block_summon_sets_player_created(top_block: &'static str) -> bool {
    matches!(
        top_block,
        "minecraft:carved_pumpkin" | "minecraft:jack_o_lantern"
    )
}

