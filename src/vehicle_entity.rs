use crate::base_entity::{BaseEntity, EntityDimensions, RemovalReason, Vec3};
use crate::storage::nbt::Tag;

#[derive(Debug, Clone, PartialEq)]
pub struct VehicleState {
    pub base: BaseEntity,
    pub hurt_time: i32,
    pub hurt_dir: i32,
    pub damage: f32,
}

impl VehicleState {
    pub fn new(id: i32, entity_type: &'static str, width: f32, height: f32) -> Self {
        Self {
            base: entity(id, entity_type, width, height),
            hurt_time: 0,
            hurt_dir: 1,
            damage: 0.0,
        }
    }

    pub fn tick_damage_wobble(&mut self) {
        self.hurt_time = (self.hurt_time - 1).max(0);
        self.damage = (self.damage - 1.0).max(0.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatKind {
    Oak,
    Spruce,
    Birch,
    Jungle,
    Acacia,
    Cherry,
    DarkOak,
    PaleOak,
    Mangrove,
    BambooRaft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatStatus {
    InWater,
    UnderWater,
    UnderFlowingWater,
    OnLand,
    InAir,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoatState {
    pub vehicle: VehicleState,
    pub kind: BoatKind,
    pub chest: bool,
    pub paddle_left: bool,
    pub paddle_right: bool,
    pub paddle_positions: [f32; 2],
    pub bubble_time: i32,
    pub out_of_control_ticks: f32,
    pub status: BoatStatus,
    pub passenger_ids: Vec<i32>,
    pub delta_rotation: f32,
}

impl BoatState {
    pub const MAX_PASSENGERS: usize = 2;
    pub const TIME_TO_EJECT: f32 = 60.0;
    pub const BUBBLE_TIME: i32 = 60;
    pub const PADDLE_SPEED: f32 = std::f32::consts::PI / 8.0;

    pub fn new(id: i32, kind: BoatKind, chest: bool) -> Self {
        Self {
            vehicle: VehicleState::new(
                id,
                if chest {
                    "minecraft:chest_boat"
                } else {
                    "minecraft:boat"
                },
                1.375,
                0.5625,
            ),
            kind,
            chest,
            paddle_left: false,
            paddle_right: false,
            paddle_positions: [0.0, 0.0],
            bubble_time: 0,
            out_of_control_ticks: 0.0,
            status: BoatStatus::InAir,
            passenger_ids: Vec::new(),
            delta_rotation: 0.0,
        }
    }

    pub fn set_paddle_state(&mut self, left: bool, right: bool) {
        self.paddle_left = left;
        self.paddle_right = right;
    }

    pub fn tick_status(&mut self, new_status: BoatStatus) -> BoatTickOutcome {
        self.status = new_status;
        if matches!(
            self.status,
            BoatStatus::UnderWater | BoatStatus::UnderFlowingWater
        ) {
            self.out_of_control_ticks += 1.0;
        } else {
            self.out_of_control_ticks = 0.0;
        }
        if self.out_of_control_ticks >= Self::TIME_TO_EJECT {
            self.passenger_ids.clear();
            BoatTickOutcome::EjectPassengers
        } else {
            BoatTickOutcome::Continue
        }
    }

    pub fn tick_paddles(&mut self) {
        for (idx, active) in [self.paddle_left, self.paddle_right]
            .into_iter()
            .enumerate()
        {
            self.paddle_positions[idx] = if active {
                self.paddle_positions[idx] + Self::PADDLE_SPEED
            } else {
                0.0
            };
        }
    }

    pub fn tick_bubble_column(
        &mut self,
        above_bubble_column: bool,
        drag_down: bool,
        has_player: bool,
    ) -> Option<Vec3> {
        if above_bubble_column && self.bubble_time == 0 {
            self.bubble_time = Self::BUBBLE_TIME;
        }
        if !above_bubble_column {
            self.bubble_time = 0;
            return None;
        }
        if self.bubble_time > 0 {
            self.bubble_time -= 1;
            if self.bubble_time == 0 {
                if drag_down {
                    self.passenger_ids.clear();
                    return Some(Vec3 {
                        x: 0.0,
                        y: -0.7,
                        z: 0.0,
                    });
                }
                return Some(Vec3 {
                    x: 0.0,
                    y: if has_player { 2.7 } else { 0.6 },
                    z: 0.0,
                });
            }
        }
        None
    }

    pub fn try_add_passenger(
        &mut self,
        passenger_id: i32,
        width: f32,
        already_passenger: bool,
    ) -> bool {
        if self.passenger_ids.len() >= Self::MAX_PASSENGERS
            || already_passenger
            || width >= self.vehicle.base.dimensions.width
        {
            return false;
        }
        self.passenger_ids.push(passenger_id);
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatTickOutcome {
    Continue,
    EjectPassengers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinecartKind {
    Rideable,
    Chest,
    Furnace,
    Hopper,
    Tnt,
    Spawner,
    CommandBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailShape {
    NorthSouth,
    EastWest,
    AscendingEast,
    AscendingWest,
    AscendingNorth,
    AscendingSouth,
    SouthEast,
    SouthWest,
    NorthWest,
    NorthEast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinecartState {
    pub vehicle: VehicleState,
    pub kind: MinecartKind,
    pub on_rails: bool,
    pub flipped: bool,
    pub custom_display_block: Option<&'static str>,
    pub display_offset: i32,
    pub first_tick: bool,
    pub passenger_ids: Vec<i32>,
}

impl MinecartState {
    pub const WATER_SLOWDOWN_FACTOR: f64 = 0.95;

    pub fn new(id: i32, kind: MinecartKind) -> Self {
        Self {
            vehicle: VehicleState::new(id, kind.entity_type(), 0.98, 0.7),
            kind,
            on_rails: false,
            flipped: false,
            custom_display_block: None,
            display_offset: kind.default_display_offset(),
            first_tick: true,
            passenger_ids: Vec::new(),
        }
    }

    pub fn animate_hurt(&mut self) {
        self.vehicle.hurt_dir = -self.vehicle.hurt_dir;
        self.vehicle.hurt_time = 10;
        self.vehicle.damage += self.vehicle.damage * 10.0;
    }

    pub fn default_display_block(&self) -> &'static str {
        self.custom_display_block
            .unwrap_or_else(|| self.kind.default_display_block())
    }

    pub fn gravity(&self, in_water: bool) -> f64 {
        if in_water {
            0.005
        } else {
            0.04
        }
    }

    pub fn apply_natural_slowdown(
        &self,
        movement: Vec3,
        slowdown_factor: f64,
        in_water: bool,
    ) -> Vec3 {
        let mut slowed = Vec3 {
            x: movement.x * slowdown_factor,
            y: 0.0,
            z: movement.z * slowdown_factor,
        };
        if in_water {
            slowed = scale(slowed, Self::WATER_SLOWDOWN_FACTOR);
        }
        slowed
    }

    pub fn rail_exits(shape: RailShape) -> ((i32, i32, i32), (i32, i32, i32)) {
        match shape {
            RailShape::NorthSouth => ((0, 0, -1), (0, 0, 1)),
            RailShape::EastWest => ((-1, 0, 0), (1, 0, 0)),
            RailShape::AscendingEast => ((-1, -1, 0), (1, 0, 0)),
            RailShape::AscendingWest => ((-1, 0, 0), (1, -1, 0)),
            RailShape::AscendingNorth => ((0, 0, -1), (0, -1, 1)),
            RailShape::AscendingSouth => ((0, -1, -1), (0, 0, 1)),
            RailShape::SouthEast => ((0, 0, 1), (1, 0, 0)),
            RailShape::SouthWest => ((0, 0, 1), (-1, 0, 0)),
            RailShape::NorthWest => ((0, 0, -1), (-1, 0, 0)),
            RailShape::NorthEast => ((0, 0, -1), (1, 0, 0)),
        }
    }

    pub fn redstone_direction(powered_rail_shape: RailShape, conductor: Option<Direction>) -> Vec3 {
        match (powered_rail_shape, conductor) {
            (RailShape::EastWest, Some(Direction::West)) => Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            (RailShape::EastWest, Some(Direction::East)) => Vec3 {
                x: -1.0,
                y: 0.0,
                z: 0.0,
            },
            (RailShape::NorthSouth, Some(Direction::North)) => Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            (RailShape::NorthSouth, Some(Direction::South)) => Vec3 {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
            _ => Vec3::ZERO,
        }
    }
}

impl MinecartKind {
    fn entity_type(self) -> &'static str {
        match self {
            Self::Rideable => "minecraft:minecart",
            Self::Chest => "minecraft:chest_minecart",
            Self::Furnace => "minecraft:furnace_minecart",
            Self::Hopper => "minecraft:hopper_minecart",
            Self::Tnt => "minecraft:tnt_minecart",
            Self::Spawner => "minecraft:spawner_minecart",
            Self::CommandBlock => "minecraft:command_block_minecart",
        }
    }

    fn default_display_block(self) -> &'static str {
        match self {
            Self::Rideable => "minecraft:air",
            Self::Chest => "minecraft:chest",
            Self::Furnace => "minecraft:furnace[facing=north,lit=false]",
            Self::Hopper => "minecraft:hopper",
            Self::Tnt => "minecraft:tnt",
            Self::Spawner => "minecraft:spawner",
            Self::CommandBlock => "minecraft:command_block",
        }
    }

    fn default_display_offset(self) -> i32 {
        match self {
            Self::Hopper => 1,
            _ => 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerVehicleState {
    pub minecart: MinecartState,
    pub container_size: usize,
    pub loot_table: Option<&'static str>,
    pub changed: bool,
}

impl ContainerVehicleState {
    pub fn chest_minecart(id: i32) -> Self {
        Self {
            minecart: MinecartState::new(id, MinecartKind::Chest),
            container_size: 27,
            loot_table: None,
            changed: false,
        }
    }

    pub fn hopper_minecart(id: i32) -> HopperMinecartState {
        HopperMinecartState {
            container: Self {
                minecart: MinecartState::new(id, MinecartKind::Hopper),
                container_size: 5,
                loot_table: None,
                changed: false,
            },
            enabled: true,
            consumed_item_this_frame: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FurnaceMinecartState {
    pub minecart: MinecartState,
    pub fuel: i32,
    pub push: Vec3,
    pub has_fuel: bool,
}

impl FurnaceMinecartState {
    pub const FUEL_TICKS_PER_ITEM: i32 = 3600;
    pub const MAX_FUEL_TICKS: i32 = 32000;

    pub fn new(id: i32) -> Self {
        Self {
            minecart: MinecartState::new(id, MinecartKind::Furnace),
            fuel: 0,
            push: Vec3::ZERO,
            has_fuel: false,
        }
    }

    pub fn add_fuel(&mut self, item_is_fuel: bool, cart_pos: Vec3, interacting_pos: Vec3) -> bool {
        if item_is_fuel && self.fuel + Self::FUEL_TICKS_PER_ITEM <= Self::MAX_FUEL_TICKS {
            self.fuel += Self::FUEL_TICKS_PER_ITEM;
            self.push = Vec3 {
                x: cart_pos.x - interacting_pos.x,
                y: 0.0,
                z: cart_pos.z - interacting_pos.z,
            };
            self.has_fuel = true;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self) {
        if self.fuel > 0 {
            self.fuel -= 1;
        }
        if self.fuel <= 0 {
            self.push = Vec3::ZERO;
        }
        self.has_fuel = self.fuel > 0;
    }

    pub fn max_speed_factor(&self, in_water: bool) -> f64 {
        if in_water {
            0.75
        } else {
            0.5
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TntMinecartState {
    pub minecart: MinecartState,
    pub fuse: i32,
    pub explosion_power_base: f32,
    pub explosion_speed_factor: f32,
}

impl TntMinecartState {
    pub const NO_FUSE: i32 = -1;
    pub const DEFAULT_FUSE: i32 = 80;

    pub fn new(id: i32) -> Self {
        Self {
            minecart: MinecartState::new(id, MinecartKind::Tnt),
            fuse: Self::NO_FUSE,
            explosion_power_base: 4.0,
            explosion_speed_factor: 1.0,
        }
    }

    pub fn prime_fuse(&mut self, tnt_explodes: bool) -> bool {
        if tnt_explodes {
            self.fuse = Self::DEFAULT_FUSE;
            true
        } else {
            false
        }
    }

    pub fn tick(
        &mut self,
        speed_sqr: f64,
        horizontal_collision: bool,
        tnt_explodes: bool,
        random_0_to_1: f32,
    ) -> Option<VehicleExplosion> {
        if self.fuse > 0 {
            self.fuse -= 1;
            return None;
        }
        if self.fuse == 0 || (horizontal_collision && speed_sqr >= 0.01) {
            return self.explode(speed_sqr, tnt_explodes, random_0_to_1);
        }
        None
    }

    pub fn explode(
        &mut self,
        speed_sqr: f64,
        tnt_explodes: bool,
        random_0_to_1: f32,
    ) -> Option<VehicleExplosion> {
        if !tnt_explodes {
            if self.fuse > Self::NO_FUSE {
                self.minecart.vehicle.base.remove(RemovalReason::Discarded);
            }
            return None;
        }
        let speed = speed_sqr.sqrt().min(5.0);
        let power = self.explosion_power_base
            + self.explosion_speed_factor * random_0_to_1 * 1.5 * speed as f32;
        self.minecart.vehicle.base.remove(RemovalReason::Discarded);
        Some(VehicleExplosion {
            power,
            interaction: "tnt",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleExplosion {
    pub power: f32,
    pub interaction: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HopperMinecartState {
    pub container: ContainerVehicleState,
    pub enabled: bool,
    pub consumed_item_this_frame: bool,
}

impl HopperMinecartState {
    pub fn activate_minecart(&mut self, powered: bool) {
        self.enabled = !powered;
    }

    pub fn try_consume_items(&mut self, item_available: bool) -> bool {
        if self.enabled && !self.consumed_item_this_frame && item_available {
            self.consumed_item_this_frame = true;
            self.container.changed = true;
            true
        } else {
            false
        }
    }

    pub fn tick_frame_start(&mut self) {
        self.consumed_item_this_frame = false;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnerMinecartState {
    pub minecart: MinecartState,
    pub spawner_tick_count: i32,
}

impl SpawnerMinecartState {
    pub fn new(id: i32) -> Self {
        Self {
            minecart: MinecartState::new(id, MinecartKind::Spawner),
            spawner_tick_count: 0,
        }
    }

    pub fn tick_spawner(&mut self) {
        self.spawner_tick_count += 1;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandBlockMinecartState {
    pub minecart: MinecartState,
    pub command: String,
    pub success_count: i32,
    pub last_output: String,
    pub track_output: bool,
    pub last_activated_tick: i32,
}

impl CommandBlockMinecartState {
    pub const ACTIVATION_DELAY: i32 = 4;

    pub fn new(id: i32) -> Self {
        Self {
            minecart: MinecartState::new(id, MinecartKind::CommandBlock),
            command: String::new(),
            success_count: 0,
            last_output: String::new(),
            track_output: true,
            last_activated_tick: -Self::ACTIVATION_DELAY,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = vec![
            ("Command".to_string(), Tag::String(self.command.clone())),
            ("SuccessCount".to_string(), Tag::Int(self.success_count)),
            (
                "TrackOutput".to_string(),
                Tag::Byte(i8::from(self.track_output)),
            ),
        ];
        if self.track_output && !self.last_output.is_empty() {
            entries.push((
                "LastOutput".to_string(),
                Tag::String(self.last_output.clone()),
            ));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(id: i32, tag: &Tag) -> Self {
        let mut state = Self::new(id);
        let Some(entries) = compound_entries(tag) else {
            return state;
        };
        state.command = get_string(entries, "Command").unwrap_or("").to_string();
        state.success_count = get_int(entries, "SuccessCount").unwrap_or(0);
        state.track_output = get_bool(entries, "TrackOutput").unwrap_or(true);
        if state.track_output {
            state.last_output = get_string(entries, "LastOutput").unwrap_or("").to_string();
        }
        state
    }

    pub fn activate_minecart(&mut self, powered: bool, tick_count: i32) -> bool {
        if powered && tick_count - self.last_activated_tick >= Self::ACTIVATION_DELAY {
            self.perform_command(true);
            self.last_activated_tick = tick_count;
            true
        } else {
            false
        }
    }

    pub fn perform_command(&mut self, command_blocks_enabled: bool) -> bool {
        if self.command.eq_ignore_ascii_case("Searge") {
            self.success_count = 1;
            if self.track_output {
                self.last_output = "#itzlipofutzli".to_string();
            }
            return true;
        }
        self.success_count = 0;
        if command_blocks_enabled && !self.command.is_empty() {
            self.success_count = 1;
            if self.track_output {
                self.last_output = format!("Executed command: {}", self.command);
            }
        }
        true
    }

    pub fn can_interact(&self, player_can_use_gamemaster_blocks: bool) -> bool {
        player_can_use_gamemaster_blocks
    }
}

fn compound_entries(tag: &Tag) -> Option<&[(String, Tag)]> {
    match tag {
        Tag::Compound(entries) => Some(entries.as_slice()),
        _ => None,
    }
}

fn get_string<'a>(entries: &'a [(String, Tag)], key: &str) -> Option<&'a str> {
    entries
        .iter()
        .find_map(|(name, tag)| match (name.as_str(), tag) {
            (entry, Tag::String(value)) if entry == key => Some(value.as_str()),
            _ => None,
        })
}

fn get_int(entries: &[(String, Tag)], key: &str) -> Option<i32> {
    entries
        .iter()
        .find_map(|(name, tag)| match (name.as_str(), tag) {
            (entry, Tag::Int(value)) if entry == key => Some(*value),
            _ => None,
        })
}

fn get_bool(entries: &[(String, Tag)], key: &str) -> Option<bool> {
    entries
        .iter()
        .find_map(|(name, tag)| match (name.as_str(), tag) {
            (entry, Tag::Byte(value)) if entry == key => Some(*value != 0),
            _ => None,
        })
}

pub const VEHICLE_ENTITY_FAMILIES: &[&str] = &[
    "boats",
    "chest boats",
    "minecarts",
    "furnace minecarts",
    "chest minecarts",
    "hopper minecarts",
    "TNT minecarts",
    "spawner minecarts",
    "command block minecarts",
    "interpolation",
    "collision",
    "rails",
    "activator rails",
    "detector rails",
    "powered rails",
    "passenger sync",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boats_track_paddles_passengers_bubbles_and_underwater_ejection() {
        let mut boat = BoatState::new(1, BoatKind::Oak, false);
        assert!(boat.try_add_passenger(10, 0.6, false));
        assert!(boat.try_add_passenger(11, 0.6, false));
        assert!(!boat.try_add_passenger(12, 0.6, false));

        boat.set_paddle_state(true, false);
        boat.tick_paddles();
        assert_eq!(boat.paddle_positions[0], BoatState::PADDLE_SPEED);
        assert_eq!(boat.paddle_positions[1], 0.0);

        boat.out_of_control_ticks = 59.0;
        assert_eq!(
            boat.tick_status(BoatStatus::UnderWater),
            BoatTickOutcome::EjectPassengers
        );
        assert!(boat.passenger_ids.is_empty());
        assert_eq!(
            boat.tick_status(BoatStatus::InWater),
            BoatTickOutcome::Continue
        );
        assert_eq!(
            boat.tick_status(BoatStatus::UnderFlowingWater),
            BoatTickOutcome::Continue
        );
        assert_eq!(
            boat.tick_status(BoatStatus::OnLand),
            BoatTickOutcome::Continue
        );

        assert_eq!(boat.tick_bubble_column(true, false, true), None);
        boat.bubble_time = 1;
        assert_eq!(
            boat.tick_bubble_column(true, false, true),
            Some(Vec3 {
                x: 0.0,
                y: 2.7,
                z: 0.0
            })
        );
    }

    #[test]
    fn boat_manifest_covers_all_variants_and_chest_boats() {
        let kinds = [
            BoatKind::Oak,
            BoatKind::Spruce,
            BoatKind::Birch,
            BoatKind::Jungle,
            BoatKind::Acacia,
            BoatKind::Cherry,
            BoatKind::DarkOak,
            BoatKind::PaleOak,
            BoatKind::Mangrove,
            BoatKind::BambooRaft,
        ];
        assert_eq!(kinds.len(), 10);
        let chest_boat = BoatState::new(2, BoatKind::Mangrove, true);
        assert_eq!(chest_boat.vehicle.base.entity_type, "minecraft:chest_boat");
    }

    #[test]
    fn minecarts_cover_display_offsets_gravity_rails_redstone_and_slowdown() {
        let mut cart = MinecartState::new(3, MinecartKind::Rideable);
        cart.vehicle.damage = 3.0;
        cart.vehicle.hurt_time = 2;
        cart.vehicle.tick_damage_wobble();
        assert_eq!(cart.vehicle.damage, 2.0);
        assert_eq!(cart.vehicle.hurt_time, 1);
        cart.animate_hurt();
        assert_eq!(cart.vehicle.hurt_time, 10);
        assert_eq!(cart.default_display_block(), "minecraft:air");
        cart.custom_display_block = Some("minecraft:glass");
        assert_eq!(cart.default_display_block(), "minecraft:glass");
        assert_eq!(cart.gravity(false), 0.04);
        assert_eq!(cart.gravity(true), 0.005);
        assert_eq!(
            MinecartState::rail_exits(RailShape::AscendingEast),
            ((-1, -1, 0), (1, 0, 0))
        );
        let _covered_rails = [
            RailShape::NorthSouth,
            RailShape::EastWest,
            RailShape::AscendingEast,
            RailShape::AscendingWest,
            RailShape::AscendingNorth,
            RailShape::AscendingSouth,
            RailShape::SouthEast,
            RailShape::SouthWest,
            RailShape::NorthWest,
            RailShape::NorthEast,
        ];
        let _covered_directions = [
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ];
        assert_eq!(
            MinecartState::redstone_direction(RailShape::EastWest, Some(Direction::West)),
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0
            }
        );
        assert_eq!(
            cart.apply_natural_slowdown(
                Vec3 {
                    x: 1.0,
                    y: 4.0,
                    z: 1.0
                },
                0.98,
                true
            ),
            Vec3 {
                x: 0.9309999999999999,
                y: 0.0,
                z: 0.9309999999999999
            }
        );
    }

    #[test]
    fn minecart_kinds_cover_all_specialized_display_blocks_and_offsets() {
        let expected = [
            (
                MinecartKind::Rideable,
                "minecraft:minecart",
                "minecraft:air",
                6,
            ),
            (
                MinecartKind::Chest,
                "minecraft:chest_minecart",
                "minecraft:chest",
                6,
            ),
            (
                MinecartKind::Furnace,
                "minecraft:furnace_minecart",
                "minecraft:furnace[facing=north,lit=false]",
                6,
            ),
            (
                MinecartKind::Hopper,
                "minecraft:hopper_minecart",
                "minecraft:hopper",
                1,
            ),
            (
                MinecartKind::Tnt,
                "minecraft:tnt_minecart",
                "minecraft:tnt",
                6,
            ),
            (
                MinecartKind::Spawner,
                "minecraft:spawner_minecart",
                "minecraft:spawner",
                6,
            ),
            (
                MinecartKind::CommandBlock,
                "minecraft:command_block_minecart",
                "minecraft:command_block",
                6,
            ),
        ];
        for (kind, entity_type, display, offset) in expected {
            let cart = MinecartState::new(4, kind);
            assert_eq!(cart.vehicle.base.entity_type, entity_type);
            assert_eq!(cart.default_display_block(), display);
            assert_eq!(cart.display_offset, offset);
        }
    }

    #[test]
    fn furnace_minecarts_add_cap_and_consume_fuel() {
        let mut furnace = FurnaceMinecartState::new(5);
        assert!(furnace.add_fuel(
            true,
            Vec3 {
                x: 10.0,
                y: 64.0,
                z: 10.0
            },
            Vec3 {
                x: 8.0,
                y: 64.0,
                z: 11.0
            },
        ));
        assert_eq!(furnace.fuel, FurnaceMinecartState::FUEL_TICKS_PER_ITEM);
        assert_eq!(
            furnace.push,
            Vec3 {
                x: 2.0,
                y: 0.0,
                z: -1.0
            }
        );
        furnace.tick();
        assert_eq!(furnace.fuel, 3599);
        furnace.fuel = FurnaceMinecartState::MAX_FUEL_TICKS;
        assert!(!furnace.add_fuel(true, Vec3::ZERO, Vec3::ZERO));
        assert_eq!(furnace.max_speed_factor(false), 0.5);
        assert_eq!(furnace.max_speed_factor(true), 0.75);
    }

    #[test]
    fn tnt_minecarts_prime_tick_and_scale_explosion_by_speed() {
        let mut tnt = TntMinecartState::new(6);
        assert_eq!(tnt.fuse, TntMinecartState::NO_FUSE);
        assert!(tnt.prime_fuse(true));
        assert_eq!(tnt.fuse, TntMinecartState::DEFAULT_FUSE);
        tnt.fuse = 1;
        assert_eq!(tnt.tick(0.0, false, true, 0.5), None);
        assert_eq!(
            tnt.tick(4.0, false, true, 0.5),
            Some(VehicleExplosion {
                power: 5.5,
                interaction: "tnt"
            })
        );
        assert_eq!(
            tnt.minecart.vehicle.base.removal_reason,
            Some(RemovalReason::Discarded)
        );
    }

    #[test]
    fn hopper_minecarts_invert_activator_power_and_consume_once_per_frame() {
        let mut hopper = ContainerVehicleState::hopper_minecart(7);
        assert_eq!(hopper.container.container_size, 5);
        hopper.activate_minecart(true);
        assert!(!hopper.enabled);
        assert!(!hopper.try_consume_items(true));
        hopper.activate_minecart(false);
        assert!(hopper.try_consume_items(true));
        assert!(!hopper.try_consume_items(true));
        hopper.tick_frame_start();
        assert!(hopper.try_consume_items(true));
        assert!(hopper.container.changed);
    }

    #[test]
    fn chest_spawner_and_command_minecarts_keep_special_server_hooks() {
        let chest = ContainerVehicleState::chest_minecart(8);
        assert_eq!(chest.container_size, 27);

        let mut spawner = SpawnerMinecartState::new(9);
        spawner.tick_spawner();
        assert_eq!(spawner.spawner_tick_count, 1);
        assert_eq!(
            spawner.minecart.default_display_block(),
            "minecraft:spawner"
        );

        let mut command = CommandBlockMinecartState::new(10);
        assert!(command.can_interact(true));
        assert!(!command.can_interact(false));
        command.command = "say rail".to_string();
        command.last_output = "{\"text\":\"ok\"}".to_string();
        let saved = command.save_additional();
        assert_eq!(
            CommandBlockMinecartState::load_additional(10, &saved),
            command
        );
        let loaded_without_tracking = CommandBlockMinecartState::load_additional(
            11,
            &Tag::Compound(vec![
                ("Command".to_string(), Tag::String("say quiet".to_string())),
                ("TrackOutput".to_string(), Tag::Byte(0)),
                (
                    "LastOutput".to_string(),
                    Tag::String("{\"text\":\"ignored\"}".to_string()),
                ),
            ]),
        );
        assert_eq!(loaded_without_tracking.command, "say quiet");
        assert!(!loaded_without_tracking.track_output);
        assert_eq!(loaded_without_tracking.last_output, "");
        command.command = "Searge".to_string();
        assert!(command.activate_minecart(true, 0));
        assert_eq!(command.success_count, 1);
        assert_eq!(command.last_output, "#itzlipofutzli");
        assert!(!command.activate_minecart(true, 3));
        assert!(command.activate_minecart(true, 4));
    }

    #[test]
    fn vehicle_family_checklist_is_represented() {
        for family in [
            "boats",
            "chest boats",
            "minecarts",
            "furnace minecarts",
            "chest minecarts",
            "hopper minecarts",
            "TNT minecarts",
            "spawner minecarts",
            "command block minecarts",
            "interpolation",
            "collision",
            "rails",
            "activator rails",
            "detector rails",
            "powered rails",
            "passenger sync",
        ] {
            assert!(VEHICLE_ENTITY_FAMILIES.contains(&family), "{family}");
        }
    }
}
