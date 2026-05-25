use crate::base_entity::{BaseEntity, EntityDimensions, RemovalReason, Vec3};
use crate::storage::nbt::Tag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectileOwner {
    pub uuid: String,
    pub entity_id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileState {
    pub base: BaseEntity,
    pub owner: Option<ProjectileOwner>,
    pub left_owner: bool,
    pub has_been_shot: bool,
    pub last_deflected_by: Option<i32>,
}

impl ProjectileState {
    pub fn new(id: i32, entity_type: &'static str, width: f32, height: f32) -> Self {
        Self {
            base: entity(id, entity_type, width, height),
            owner: None,
            left_owner: false,
            has_been_shot: false,
            last_deflected_by: None,
        }
    }

    pub fn tick_shot_and_owner_collision(&mut self, intersects_owner_vehicle: bool) -> bool {
        let fired_event = !self.has_been_shot;
        self.has_been_shot = true;
        if !self.left_owner {
            self.left_owner = self.owner.is_none() || !intersects_owner_vehicle;
        }
        fired_event
    }

    pub fn can_hit_owner_or_passenger(&self) -> bool {
        self.left_owner
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowPickup {
    Disallowed,
    Allowed,
    CreativeOnly,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrowState {
    pub projectile: ProjectileState,
    pub pickup: ArrowPickup,
    pub in_ground: bool,
    pub life: i32,
    pub shake_time: i32,
    pub base_damage: f64,
    pub crit: bool,
    pub pierce_level: u8,
    pub pierced_entity_ids: Vec<i32>,
    pub pickup_item: &'static str,
    pub fired_from_weapon: Option<&'static str>,
}

impl ArrowState {
    pub const BASE_DAMAGE: f64 = 2.0;
    pub const DESPAWN_LIFE: i32 = 1200;
    pub const SHAKE_TIME: i32 = 7;
    pub const WATER_INERTIA: f64 = 0.6;
    pub const AIR_INERTIA: f64 = 0.99;

    pub fn new(id: i32, kind: ArrowKind) -> Self {
        let (entity_type, pickup_item) = match kind {
            ArrowKind::Arrow => ("minecraft:arrow", "minecraft:arrow"),
            ArrowKind::SpectralArrow => ("minecraft:spectral_arrow", "minecraft:spectral_arrow"),
        };
        Self {
            projectile: ProjectileState::new(id, entity_type, 0.5, 0.5),
            pickup: ArrowPickup::Disallowed,
            in_ground: false,
            life: 0,
            shake_time: 0,
            base_damage: Self::BASE_DAMAGE,
            crit: false,
            pierce_level: 0,
            pierced_entity_ids: Vec::new(),
            pickup_item,
            fired_from_weapon: None,
        }
    }

    pub fn shoot(&mut self, velocity: Vec3) {
        self.projectile.base.velocity = velocity;
        self.life = 0;
        self.in_ground = false;
    }

    pub fn tick_in_ground(&mut self) {
        self.life += 1;
        if self.life >= Self::DESPAWN_LIFE {
            self.projectile.base.remove(RemovalReason::Discarded);
        }
    }

    pub fn hit_block(&mut self) {
        self.projectile.base.velocity = Vec3::ZERO;
        self.in_ground = true;
        self.shake_time = Self::SHAKE_TIME;
        self.crit = false;
        self.pierce_level = 0;
        self.pierced_entity_ids.clear();
    }

    pub fn hit_entity(&mut self, target_id: i32, speed: f64, target_hurt: bool) -> ArrowHitOutcome {
        let damage = (speed * self.base_damage).ceil().max(0.0) as i32;
        if self.pierce_level > 0 {
            if self.pierced_entity_ids.len() > usize::from(self.pierce_level) {
                self.projectile.base.remove(RemovalReason::Discarded);
                return ArrowHitOutcome::DiscardedBeforeDamage;
            }
            self.pierced_entity_ids.push(target_id);
        }
        if target_hurt {
            if self.pierce_level == 0 {
                self.projectile.base.remove(RemovalReason::Discarded);
            }
            ArrowHitOutcome::Damage { amount: damage }
        } else {
            self.projectile.base.velocity = scale(self.projectile.base.velocity, -0.2);
            ArrowHitOutcome::Deflected
        }
    }

    pub fn can_player_pickup(&self, is_owner: bool, creative: bool) -> bool {
        self.in_ground
            && match self.pickup {
                ArrowPickup::Allowed => true,
                ArrowPickup::CreativeOnly => creative || is_owner,
                ArrowPickup::Disallowed => false,
            }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowKind {
    Arrow,
    SpectralArrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowHitOutcome {
    Damage { amount: i32 },
    Deflected,
    DiscardedBeforeDamage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TridentState {
    pub arrow: ArrowState,
    pub loyalty: u8,
    pub foil: bool,
    pub dealt_damage: bool,
    pub return_ticks: i32,
}

impl TridentState {
    pub fn new(id: i32, loyalty: u8) -> Self {
        let mut arrow = ArrowState::new(id, ArrowKind::Arrow);
        arrow.projectile.base.entity_type = "minecraft:trident";
        arrow.pickup_item = "minecraft:trident";
        arrow.base_damage = 8.0;
        Self {
            arrow,
            loyalty: loyalty.min(127),
            foil: false,
            dealt_damage: false,
            return_ticks: 0,
        }
    }

    pub fn tick_return(&mut self, owner_alive: bool, owner_spectator: bool) -> TridentTickOutcome {
        if self.arrow.in_ground && self.arrow.life > 4 {
            self.dealt_damage = true;
        }
        if self.loyalty == 0 || !self.dealt_damage {
            return TridentTickOutcome::NormalArrowTick;
        }
        if !owner_alive || owner_spectator {
            if self.arrow.pickup == ArrowPickup::Allowed {
                self.arrow.projectile.base.remove(RemovalReason::Discarded);
                TridentTickOutcome::DropPickupItem
            } else {
                self.arrow.projectile.base.remove(RemovalReason::Discarded);
                TridentTickOutcome::Discard
            }
        } else {
            self.arrow.in_ground = false;
            self.return_ticks += 1;
            TridentTickOutcome::Returning {
                acceleration: 0.05 * f64::from(self.loyalty),
            }
        }
    }

    pub fn hit_entity(&mut self) -> i32 {
        self.dealt_damage = true;
        8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TridentTickOutcome {
    NormalArrowTick,
    Returning { acceleration: f64 },
    DropPickupItem,
    Discard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrowableKind {
    Snowball,
    Egg,
    SplashPotion,
    LingeringPotion,
    EnderPearl,
    ExperienceBottle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThrowableItemProjectile {
    pub projectile: ProjectileState,
    pub kind: ThrowableKind,
    pub item: &'static str,
}

impl ThrowableItemProjectile {
    pub fn new(id: i32, kind: ThrowableKind) -> Self {
        let (entity_type, item) = match kind {
            ThrowableKind::Snowball => ("minecraft:snowball", "minecraft:snowball"),
            ThrowableKind::Egg => ("minecraft:egg", "minecraft:egg"),
            ThrowableKind::SplashPotion => ("minecraft:potion", "minecraft:splash_potion"),
            ThrowableKind::LingeringPotion => ("minecraft:potion", "minecraft:lingering_potion"),
            ThrowableKind::EnderPearl => ("minecraft:ender_pearl", "minecraft:ender_pearl"),
            ThrowableKind::ExperienceBottle => {
                ("minecraft:experience_bottle", "minecraft:experience_bottle")
            }
        };
        Self {
            projectile: ProjectileState::new(id, entity_type, 0.25, 0.25),
            kind,
            item,
        }
    }

    pub fn hit(
        &mut self,
        random_roll_8: u8,
        random_roll_32: u8,
        owner_can_teleport: bool,
    ) -> ThrowableImpact {
        self.projectile.base.remove(RemovalReason::Discarded);
        match self.kind {
            ThrowableKind::Snowball => ThrowableImpact::DamageAndBreakParticles {
                damage: 0,
                blaze_damage: 3,
                particles: 8,
            },
            ThrowableKind::Egg => ThrowableImpact::HatchChickens {
                count: if random_roll_8 == 0 {
                    if random_roll_32 == 0 {
                        4
                    } else {
                        1
                    }
                } else {
                    0
                },
                particles: 8,
            },
            ThrowableKind::EnderPearl => ThrowableImpact::TeleportOwner {
                damage: if owner_can_teleport { 5.0 } else { 0.0 },
                endermite_chance: 0.05,
            },
            ThrowableKind::SplashPotion => ThrowableImpact::SplashPotionEffects,
            ThrowableKind::LingeringPotion => ThrowableImpact::LingeringPotionCloud {
                radius: 3.0,
                duration: 600,
            },
            ThrowableKind::ExperienceBottle => ThrowableImpact::ExperienceBottle {
                min_xp: 3,
                max_xp: 11,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThrowableImpact {
    DamageAndBreakParticles {
        damage: i32,
        blaze_damage: i32,
        particles: i32,
    },
    HatchChickens {
        count: i32,
        particles: i32,
    },
    TeleportOwner {
        damage: f32,
        endermite_chance: f32,
    },
    SplashPotionEffects,
    LingeringPotionCloud {
        radius: f32,
        duration: i32,
    },
    ExperienceBottle {
        min_xp: i32,
        max_xp: i32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct HurtingProjectileState {
    pub projectile: ProjectileState,
    pub kind: HurtingProjectileKind,
    pub acceleration_power: f64,
    pub item: Option<&'static str>,
    pub dangerous: bool,
}

impl HurtingProjectileState {
    pub fn new(id: i32, kind: HurtingProjectileKind) -> Self {
        let entity_type = kind.entity_type();
        Self {
            projectile: ProjectileState::new(id, entity_type, 1.0, 1.0),
            kind,
            acceleration_power: if kind.is_wind_charge() { 0.0 } else { 0.1 },
            item: if kind.uses_fire_charge_item() {
                Some("minecraft:fire_charge")
            } else {
                None
            },
            dangerous: false,
        }
    }

    pub fn inertia(&self, in_water: bool) -> f64 {
        if self.kind.is_wind_charge() {
            1.0
        } else if self.kind == HurtingProjectileKind::WitherSkull && self.dangerous {
            0.73
        } else if in_water {
            0.8
        } else {
            0.95
        }
    }

    pub fn wither_skull_explosion_resistance(
        &self,
        block_is_air: bool,
        block_is_wither_immune: bool,
        resistance: f32,
    ) -> f32 {
        if self.kind == HurtingProjectileKind::WitherSkull
            && self.dangerous
            && !block_is_air
            && !block_is_wither_immune
        {
            resistance.min(0.8)
        } else {
            resistance
        }
    }

    pub fn wither_skull_additional_save_data(&self) -> Tag {
        Tag::Compound(vec![(
            "dangerous".to_string(),
            Tag::Byte(
                if self.kind == HurtingProjectileKind::WitherSkull && self.dangerous {
                    1
                } else {
                    0
                },
            ),
        )])
    }

    pub fn read_wither_skull_additional_save_data(&mut self, tag: &Tag) {
        if self.kind != HurtingProjectileKind::WitherSkull {
            return;
        }
        let Tag::Compound(fields) = tag else {
            self.dangerous = false;
            return;
        };
        self.dangerous = fields
            .iter()
            .find_map(|(name, value)| {
                (name == "dangerous").then(|| match value {
                    Tag::Byte(value) => *value != 0,
                    _ => false,
                })
            })
            .unwrap_or(false);
    }

    pub fn wither_skull_effect_ticks(difficulty: ProjectileDifficulty) -> i32 {
        match difficulty {
            ProjectileDifficulty::Peaceful | ProjectileDifficulty::Easy => 0,
            ProjectileDifficulty::Normal => 200,
            ProjectileDifficulty::Hard => 800,
        }
    }

    pub fn deflect(&mut self, by_attack: bool, new_owner: Option<ProjectileOwner>) {
        self.projectile.owner = new_owner;
        self.acceleration_power = if by_attack {
            0.1
        } else {
            self.acceleration_power * 0.5
        };
    }

    pub fn impact(&mut self, target_is_owner: bool) -> HurtingImpact {
        match self.kind {
            HurtingProjectileKind::LargeFireball => {
                self.projectile.base.remove(RemovalReason::Discarded);
                HurtingImpact::Explosion {
                    power: 1.0,
                    interaction: ExplosionInteraction::Mob,
                }
            }
            HurtingProjectileKind::SmallFireball => {
                self.projectile.base.remove(RemovalReason::Discarded);
                HurtingImpact::IgniteAndDamage { damage: 5 }
            }
            HurtingProjectileKind::WitherSkull => {
                self.projectile.base.remove(RemovalReason::Discarded);
                HurtingImpact::WitherSkull {
                    owner_damage: 8,
                    magic_damage: 5,
                    owner_heal_on_kill: 5.0,
                    wither_effect_amplifier: 1,
                    explosion_power: 1.0,
                }
            }
            HurtingProjectileKind::DragonFireball => {
                if target_is_owner {
                    HurtingImpact::IgnoredOwner
                } else {
                    self.projectile.base.remove(RemovalReason::Discarded);
                    HurtingImpact::AreaEffectCloud {
                        radius: 3.0,
                        duration: 600,
                        splash_range: 4.0,
                    }
                }
            }
            HurtingProjectileKind::WindCharge | HurtingProjectileKind::BreezeWindCharge => {
                self.projectile.base.remove(RemovalReason::Discarded);
                HurtingImpact::Explosion {
                    power: if self.kind == HurtingProjectileKind::WindCharge {
                        1.2
                    } else {
                        3.0
                    },
                    interaction: ExplosionInteraction::Trigger,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HurtingProjectileKind {
    LargeFireball,
    SmallFireball,
    WitherSkull,
    DragonFireball,
    WindCharge,
    BreezeWindCharge,
}

impl HurtingProjectileKind {
    fn entity_type(self) -> &'static str {
        match self {
            Self::LargeFireball => "minecraft:fireball",
            Self::SmallFireball => "minecraft:small_fireball",
            Self::WitherSkull => "minecraft:wither_skull",
            Self::DragonFireball => "minecraft:dragon_fireball",
            Self::WindCharge => "minecraft:wind_charge",
            Self::BreezeWindCharge => "minecraft:breeze_wind_charge",
        }
    }

    fn uses_fire_charge_item(self) -> bool {
        matches!(self, Self::LargeFireball | Self::SmallFireball)
    }

    fn is_wind_charge(self) -> bool {
        matches!(self, Self::WindCharge | Self::BreezeWindCharge)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionInteraction {
    Mob,
    Trigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileDifficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HurtingImpact {
    Explosion {
        power: f32,
        interaction: ExplosionInteraction,
    },
    IgniteAndDamage {
        damage: i32,
    },
    WitherSkull {
        owner_damage: i32,
        magic_damage: i32,
        owner_heal_on_kill: f32,
        wither_effect_amplifier: i32,
        explosion_power: f32,
    },
    AreaEffectCloud {
        radius: f32,
        duration: i32,
        splash_range: f32,
    },
    IgnoredOwner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishingHookStateKind {
    Flying,
    HookedInEntity,
    Bobbing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FishingHookState {
    pub projectile: ProjectileState,
    pub hooked_entity: Option<i32>,
    pub biting: bool,
    pub life: i32,
    pub nibble: i32,
    pub time_until_lured: i32,
    pub time_until_hooked: i32,
    pub out_of_water_time: i32,
    pub open_water: bool,
    pub state: FishingHookStateKind,
    pub luck: i32,
    pub lure_speed: i32,
}

impl FishingHookState {
    pub const MAX_OUT_OF_WATER_TIME: i32 = 10;
    pub const GROUND_DESPAWN_LIFE: i32 = 1200;

    pub fn new(id: i32, luck: i32, lure_speed: i32) -> Self {
        Self {
            projectile: ProjectileState::new(id, "minecraft:fishing_bobber", 0.25, 0.25),
            hooked_entity: None,
            biting: false,
            life: 0,
            nibble: 0,
            time_until_lured: 0,
            time_until_hooked: 0,
            out_of_water_time: 0,
            open_water: true,
            state: FishingHookStateKind::Flying,
            luck: luck.max(0),
            lure_speed: lure_speed.max(0),
        }
    }

    pub fn tick_state(
        &mut self,
        owner_present: bool,
        on_ground: bool,
        in_water: bool,
        hooked_alive: bool,
    ) {
        if !owner_present {
            self.projectile.base.remove(RemovalReason::Discarded);
            return;
        }
        if on_ground {
            self.life += 1;
            if self.life >= Self::GROUND_DESPAWN_LIFE {
                self.projectile.base.remove(RemovalReason::Discarded);
                return;
            }
        } else {
            self.life = 0;
        }
        match self.state {
            FishingHookStateKind::Flying => {
                if self.hooked_entity.is_some() {
                    self.state = FishingHookStateKind::HookedInEntity;
                    self.projectile.base.velocity = Vec3::ZERO;
                } else if in_water {
                    self.state = FishingHookStateKind::Bobbing;
                    self.projectile.base.velocity =
                        scale_xyz(self.projectile.base.velocity, 0.3, 0.2, 0.3);
                }
            }
            FishingHookStateKind::HookedInEntity => {
                if !hooked_alive {
                    self.hooked_entity = None;
                    self.state = FishingHookStateKind::Flying;
                }
            }
            FishingHookStateKind::Bobbing => {
                if in_water {
                    self.out_of_water_time = (self.out_of_water_time - 1).max(0);
                } else {
                    self.out_of_water_time += 1;
                    if self.out_of_water_time >= Self::MAX_OUT_OF_WATER_TIME {
                        self.open_water = false;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlamaSpitState {
    pub projectile: ProjectileState,
}

impl LlamaSpitState {
    pub fn new(id: i32) -> Self {
        Self {
            projectile: ProjectileState::new(id, "minecraft:llama_spit", 0.25, 0.25),
        }
    }

    pub fn tick(&mut self, blocked_or_in_water: bool) -> Option<i32> {
        if blocked_or_in_water {
            self.projectile.base.remove(RemovalReason::Discarded);
            None
        } else {
            self.projectile.base.velocity = scale(self.projectile.base.velocity, 0.99);
            Some(1)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShulkerBulletState {
    pub projectile: ProjectileState,
    pub target: Option<i32>,
    pub current_move_direction: Option<Direction>,
    pub flight_steps: i32,
    pub target_delta: Vec3,
}

impl ShulkerBulletState {
    pub const SPEED: f64 = 0.15;

    pub fn new(id: i32, target: Option<i32>) -> Self {
        Self {
            projectile: ProjectileState::new(id, "minecraft:shulker_bullet", 0.3125, 0.3125),
            target,
            current_move_direction: Some(Direction::Up),
            flight_steps: 0,
            target_delta: Vec3::ZERO,
        }
    }

    pub fn check_despawn(&mut self, peaceful: bool) {
        if peaceful {
            self.projectile.base.remove(RemovalReason::Discarded);
        }
    }

    pub fn retarget(
        &mut self,
        direction: Option<Direction>,
        steps_random_0_to_4: i32,
        delta: Vec3,
    ) {
        self.current_move_direction = direction;
        self.target_delta = normalize_scale(delta, Self::SPEED);
        self.flight_steps = 10 + steps_random_0_to_4.clamp(0, 4) * 10;
    }

    pub fn tick_guidance(&mut self, target_alive: bool) {
        if target_alive {
            self.target_delta = Vec3 {
                x: (self.target_delta.x * 1.025).clamp(-1.0, 1.0),
                y: (self.target_delta.y * 1.025).clamp(-1.0, 1.0),
                z: (self.target_delta.z * 1.025).clamp(-1.0, 1.0),
            };
            let movement = self.projectile.base.velocity;
            self.projectile.base.velocity = Vec3 {
                x: movement.x + (self.target_delta.x - movement.x) * 0.2,
                y: movement.y + (self.target_delta.y - movement.y) * 0.2,
                z: movement.z + (self.target_delta.z - movement.z) * 0.2,
            };
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

pub const PROJECTILE_ENTITY_FAMILIES: &[&str] = &[
    "arrows",
    "spectral arrows",
    "tridents",
    "snowballs",
    "eggs",
    "fireballs",
    "wind charges",
    "potions",
    "ender pearls",
    "fishing hooks",
    "llama spit",
    "shulker bullets",
    "wither skulls",
    "dragon fireballs",
    "thrown items",
];

fn entity(id: i32, entity_type: &'static str, width: f32, height: f32) -> BaseEntity {
    BaseEntity::new(
        id,
        format!("00000000-0000-0000-0000-{id:012x}"),
        entity_type,
        EntityDimensions {
            width,
            height,
            eye_height: height / 2.0,
        },
    )
}

fn scale(v: Vec3, factor: f64) -> Vec3 {
    Vec3 {
        x: v.x * factor,
        y: v.y * factor,
        z: v.z * factor,
    }
}

fn scale_xyz(v: Vec3, x: f64, y: f64, z: f64) -> Vec3 {
    Vec3 {
        x: v.x * x,
        y: v.y * y,
        z: v.z * z,
    }
}

fn normalize_scale(v: Vec3, scale_by: f64) -> Vec3 {
    let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    if length == 0.0 {
        Vec3::ZERO
    } else {
        Vec3 {
            x: v.x / length * scale_by,
            y: v.y / length * scale_by,
            z: v.z / length * scale_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projectile_owner_state_fires_once_and_gates_owner_collision() {
        let mut projectile = ProjectileState::new(1, "minecraft:snowball", 0.25, 0.25);
        projectile.owner = Some(ProjectileOwner {
            uuid: "owner".to_string(),
            entity_id: 99,
        });
        assert!(projectile.tick_shot_and_owner_collision(true));
        assert!(!projectile.left_owner);
        assert!(!projectile.can_hit_owner_or_passenger());
        assert!(!projectile.tick_shot_and_owner_collision(false));
        assert!(projectile.left_owner);
        assert!(projectile.can_hit_owner_or_passenger());
    }

    #[test]
    fn arrows_reset_on_shoot_stick_in_blocks_pierce_and_despawn() {
        let mut arrow = ArrowState::new(2, ArrowKind::Arrow);
        arrow.life = 99;
        arrow.shoot(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 3.0,
        });
        assert_eq!(arrow.life, 0);
        arrow.hit_block();
        assert!(arrow.in_ground);
        assert_eq!(arrow.shake_time, ArrowState::SHAKE_TIME);

        arrow.pickup = ArrowPickup::Allowed;
        assert!(arrow.can_player_pickup(false, false));
        arrow.pickup = ArrowPickup::CreativeOnly;
        assert!(arrow.can_player_pickup(false, true));
        assert!(!arrow.can_player_pickup(false, false));
        assert_eq!(ArrowState::WATER_INERTIA, 0.6);
        assert_eq!(ArrowState::AIR_INERTIA, 0.99);
        arrow.life = ArrowState::DESPAWN_LIFE - 1;
        arrow.tick_in_ground();
        assert_eq!(
            arrow.projectile.base.removal_reason,
            Some(RemovalReason::Discarded)
        );
    }

    #[test]
    fn arrows_compute_damage_and_keep_piercing_until_limit() {
        let mut arrow = ArrowState::new(3, ArrowKind::SpectralArrow);
        arrow.pierce_level = 1;
        assert_eq!(
            arrow.hit_entity(10, 2.1, true),
            ArrowHitOutcome::Damage { amount: 5 }
        );
        assert_eq!(arrow.projectile.base.removal_reason, None);
        assert_eq!(
            arrow.hit_entity(11, 2.1, true),
            ArrowHitOutcome::Damage { amount: 5 }
        );
        assert_eq!(
            arrow.hit_entity(12, 2.1, true),
            ArrowHitOutcome::DiscardedBeforeDamage
        );
        assert_eq!(
            arrow.projectile.base.removal_reason,
            Some(RemovalReason::Discarded)
        );
    }

    #[test]
    fn tridents_return_with_loyalty_or_drop_when_owner_invalid() {
        let mut trident = TridentState::new(4, 3);
        assert_eq!(trident.hit_entity(), 8);
        match trident.tick_return(true, false) {
            TridentTickOutcome::Returning { acceleration } => {
                assert!((acceleration - 0.15).abs() < f64::EPSILON)
            }
            other => panic!("unexpected trident outcome: {other:?}"),
        }
        assert_eq!(trident.return_ticks, 1);

        let mut no_owner = TridentState::new(5, 2);
        no_owner.dealt_damage = true;
        no_owner.arrow.pickup = ArrowPickup::Allowed;
        assert_eq!(
            no_owner.tick_return(false, false),
            TridentTickOutcome::DropPickupItem
        );
    }

    #[test]
    fn throwable_items_apply_named_hit_behaviors() {
        let mut snowball = ThrowableItemProjectile::new(6, ThrowableKind::Snowball);
        assert_eq!(
            snowball.hit(1, 1, true),
            ThrowableImpact::DamageAndBreakParticles {
                damage: 0,
                blaze_damage: 3,
                particles: 8
            }
        );

        let mut egg = ThrowableItemProjectile::new(7, ThrowableKind::Egg);
        assert_eq!(
            egg.hit(0, 0, true),
            ThrowableImpact::HatchChickens {
                count: 4,
                particles: 8
            }
        );

        let mut pearl = ThrowableItemProjectile::new(8, ThrowableKind::EnderPearl);
        assert_eq!(
            pearl.hit(1, 1, true),
            ThrowableImpact::TeleportOwner {
                damage: 5.0,
                endermite_chance: 0.05
            }
        );
    }

    #[test]
    fn throwable_potions_and_experience_bottles_keep_server_impact_contracts() {
        assert_eq!(
            ThrowableItemProjectile::new(9, ThrowableKind::SplashPotion).hit(1, 1, true),
            ThrowableImpact::SplashPotionEffects
        );
        assert_eq!(
            ThrowableItemProjectile::new(10, ThrowableKind::LingeringPotion).hit(1, 1, true),
            ThrowableImpact::LingeringPotionCloud {
                radius: 3.0,
                duration: 600
            }
        );
        assert_eq!(
            ThrowableItemProjectile::new(11, ThrowableKind::ExperienceBottle).hit(1, 1, true),
            ThrowableImpact::ExperienceBottle {
                min_xp: 3,
                max_xp: 11
            }
        );
    }

    #[test]
    fn hurting_projectiles_track_inertia_deflection_items_and_impacts() {
        let mut fireball = HurtingProjectileState::new(12, HurtingProjectileKind::LargeFireball);
        assert_eq!(fireball.item, Some("minecraft:fire_charge"));
        assert_eq!(fireball.inertia(false), 0.95);
        fireball.deflect(false, None);
        assert_eq!(fireball.acceleration_power, 0.05);
        assert_eq!(
            fireball.impact(false),
            HurtingImpact::Explosion {
                power: 1.0,
                interaction: ExplosionInteraction::Mob
            }
        );

        let mut skull = HurtingProjectileState::new(13, HurtingProjectileKind::WitherSkull);
        assert_eq!(skull.inertia(false), 0.95);
        assert_eq!(
            skull.wither_skull_additional_save_data(),
            Tag::Compound(vec![("dangerous".to_string(), Tag::Byte(0))])
        );
        skull.dangerous = true;
        assert_eq!(skull.inertia(false), 0.73);
        assert_eq!(
            skull.wither_skull_explosion_resistance(false, false, 6.0),
            0.8
        );
        assert_eq!(
            skull.wither_skull_explosion_resistance(false, true, 6.0),
            6.0
        );
        assert_eq!(
            skull.wither_skull_explosion_resistance(true, false, 6.0),
            6.0
        );
        assert_eq!(
            skull.wither_skull_additional_save_data(),
            Tag::Compound(vec![("dangerous".to_string(), Tag::Byte(1))])
        );
        let mut loaded = HurtingProjectileState::new(14, HurtingProjectileKind::WitherSkull);
        loaded.read_wither_skull_additional_save_data(&skull.wither_skull_additional_save_data());
        assert!(loaded.dangerous);
        loaded.read_wither_skull_additional_save_data(&Tag::Compound(Vec::new()));
        assert!(!loaded.dangerous);
        assert_eq!(
            skull.impact(false),
            HurtingImpact::WitherSkull {
                owner_damage: 8,
                magic_damage: 5,
                owner_heal_on_kill: 5.0,
                wither_effect_amplifier: 1,
                explosion_power: 1.0
            }
        );
        assert_eq!(
            HurtingProjectileState::wither_skull_effect_ticks(ProjectileDifficulty::Peaceful),
            0
        );
        assert_eq!(
            HurtingProjectileState::wither_skull_effect_ticks(ProjectileDifficulty::Easy),
            0
        );
        assert_eq!(
            HurtingProjectileState::wither_skull_effect_ticks(ProjectileDifficulty::Normal),
            200
        );
        assert_eq!(
            HurtingProjectileState::wither_skull_effect_ticks(ProjectileDifficulty::Hard),
            800
        );

        let mut small = HurtingProjectileState::new(20, HurtingProjectileKind::SmallFireball);
        assert_eq!(
            small.impact(false),
            HurtingImpact::IgniteAndDamage { damage: 5 }
        );
    }

    #[test]
    fn dragon_fireballs_ignore_owner_hits_and_otherwise_spawn_clouds() {
        let mut dragon = HurtingProjectileState::new(14, HurtingProjectileKind::DragonFireball);
        assert_eq!(dragon.impact(true), HurtingImpact::IgnoredOwner);
        assert_eq!(dragon.projectile.base.removal_reason, None);
        assert_eq!(
            dragon.impact(false),
            HurtingImpact::AreaEffectCloud {
                radius: 3.0,
                duration: 600,
                splash_range: 4.0
            }
        );
    }

    #[test]
    fn wind_charges_have_no_acceleration_or_burn_and_trigger_explosions() {
        let mut wind = HurtingProjectileState::new(15, HurtingProjectileKind::WindCharge);
        assert_eq!(wind.acceleration_power, 0.0);
        assert_eq!(wind.inertia(true), 1.0);
        assert_eq!(
            wind.impact(false),
            HurtingImpact::Explosion {
                power: 1.2,
                interaction: ExplosionInteraction::Trigger
            }
        );

        let mut breeze = HurtingProjectileState::new(16, HurtingProjectileKind::BreezeWindCharge);
        assert_eq!(
            breeze.impact(false),
            HurtingImpact::Explosion {
                power: 3.0,
                interaction: ExplosionInteraction::Trigger
            }
        );
    }

    #[test]
    fn fishing_hooks_transition_between_flying_bobbing_hooked_and_despawn_states() {
        let mut hook = FishingHookState::new(17, -1, 2);
        assert_eq!(hook.luck, 0);
        assert_eq!(hook.lure_speed, 2);
        hook.projectile.base.velocity = Vec3 {
            x: 1.0,
            y: -1.0,
            z: 1.0,
        };
        hook.tick_state(true, false, true, true);
        assert_eq!(hook.state, FishingHookStateKind::Bobbing);
        assert_eq!(
            hook.projectile.base.velocity,
            Vec3 {
                x: 0.3,
                y: -0.2,
                z: 0.3
            }
        );

        hook.hooked_entity = Some(30);
        hook.state = FishingHookStateKind::HookedInEntity;
        hook.tick_state(true, false, false, false);
        assert_eq!(hook.state, FishingHookStateKind::Flying);
        assert_eq!(hook.hooked_entity, None);

        hook.life = FishingHookState::GROUND_DESPAWN_LIFE - 1;
        hook.tick_state(true, true, false, true);
        assert_eq!(
            hook.projectile.base.removal_reason,
            Some(RemovalReason::Discarded)
        );
    }

    #[test]
    fn llama_spit_discards_on_water_or_solid_collision_otherwise_applies_gravity() {
        let mut spit = LlamaSpitState::new(18);
        spit.projectile.base.velocity = Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(spit.tick(false), Some(1));
        assert_eq!(spit.projectile.base.velocity.x, 0.99);
        assert_eq!(spit.tick(true), None);
        assert_eq!(
            spit.projectile.base.removal_reason,
            Some(RemovalReason::Discarded)
        );
    }

    #[test]
    fn shulker_bullets_retarget_guide_and_despawn_in_peaceful() {
        let mut bullet = ShulkerBulletState::new(19, Some(20));
        let _covered_directions = [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ];
        bullet.retarget(
            Some(Direction::East),
            3,
            Vec3 {
                x: 3.0,
                y: 0.0,
                z: 4.0,
            },
        );
        assert_eq!(bullet.flight_steps, 40);
        assert_eq!(bullet.target_delta.x, 0.09);
        assert_eq!(bullet.target_delta.z, 0.12);
        bullet.tick_guidance(true);
        assert!(bullet.projectile.base.velocity.x > 0.0);
        bullet.check_despawn(true);
        assert_eq!(
            bullet.projectile.base.removal_reason,
            Some(RemovalReason::Discarded)
        );
    }

    #[test]
    fn checklist_projectile_families_are_represented() {
        for family in [
            "arrows",
            "spectral arrows",
            "tridents",
            "snowballs",
            "eggs",
            "fireballs",
            "wind charges",
            "potions",
            "ender pearls",
            "fishing hooks",
            "llama spit",
            "shulker bullets",
            "wither skulls",
            "dragon fireballs",
            "thrown items",
        ] {
            assert!(PROJECTILE_ENTITY_FAMILIES.contains(&family), "{family}");
        }
    }
}
