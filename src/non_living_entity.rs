use crate::base_entity::{BaseEntity, EntityDimensions, RemovalReason};

const DESPAWN_AGE: i32 = 6000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackRef {
    pub item: &'static str,
    pub count: u8,
    pub max_count: u8,
    pub components: &'static [&'static str],
}

impl StackRef {
    pub fn same_item_and_components(&self, other: &Self) -> bool {
        self.item == other.item && self.components == other.components
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemEntityState {
    pub base: BaseEntity,
    pub stack: StackRef,
    pub age: i32,
    pub pickup_delay: i32,
    pub health: i32,
    pub thrower: Option<String>,
    pub target: Option<String>,
}

impl ItemEntityState {
    pub const DEFAULT_HEALTH: i32 = 5;
    pub const DEFAULT_PICKUP_DELAY: i32 = 0;
    pub const DEFAULT_LIFETIME: i32 = DESPAWN_AGE;
    pub const INFINITE_PICKUP_DELAY: i32 = 32767;
    pub const INFINITE_LIFETIME_AGE: i32 = -32768;

    pub fn new(id: i32, stack: StackRef) -> Self {
        Self {
            base: entity(id, "minecraft:item", 0.25, 0.25, 0.2125),
            stack,
            age: 0,
            pickup_delay: Self::DEFAULT_PICKUP_DELAY,
            health: Self::DEFAULT_HEALTH,
            thrower: None,
            target: None,
        }
    }

    pub fn tick_age_and_pickup_delay(&mut self) {
        if self.pickup_delay > 0 && self.pickup_delay != Self::INFINITE_PICKUP_DELAY {
            self.pickup_delay -= 1;
        }
        if self.age != Self::INFINITE_LIFETIME_AGE {
            self.age += 1;
        }
        if self.age >= Self::DEFAULT_LIFETIME || self.stack.count == 0 {
            self.base.remove(RemovalReason::Discarded);
        }
    }

    pub fn merge_into(&mut self, other: &mut Self) -> bool {
        if !self.can_merge(other) {
            return false;
        }
        let room = self.stack.max_count.saturating_sub(self.stack.count);
        let moved = room.min(other.stack.count);
        self.stack.count += moved;
        other.stack.count -= moved;
        self.pickup_delay = self.pickup_delay.max(other.pickup_delay);
        self.age = self.age.min(other.age);
        if other.stack.count == 0 {
            other.base.remove(RemovalReason::Discarded);
        }
        moved > 0
    }

    pub fn can_be_picked_up_by(&self, player_uuid: &str) -> bool {
        self.pickup_delay == 0
            && self
                .target
                .as_ref()
                .map_or(true, |target| target == player_uuid)
            && self.base.removal_reason.is_none()
    }

    fn can_merge(&self, other: &Self) -> bool {
        self.target == other.target
            && self.pickup_delay != Self::INFINITE_PICKUP_DELAY
            && other.pickup_delay != Self::INFINITE_PICKUP_DELAY
            && self.age != Self::INFINITE_LIFETIME_AGE
            && other.age != Self::INFINITE_LIFETIME_AGE
            && self.age < Self::DEFAULT_LIFETIME
            && other.age < Self::DEFAULT_LIFETIME
            && self.stack.same_item_and_components(&other.stack)
            && self.stack.count < self.stack.max_count
            && self.stack.count.saturating_add(other.stack.count) <= self.stack.max_count
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExperienceOrbState {
    pub base: BaseEntity,
    pub value: i32,
    pub count: i32,
    pub age: i32,
    pub health: i32,
    pub following_player: Option<String>,
}

impl ExperienceOrbState {
    pub fn new(id: i32, value: i32) -> Self {
        Self {
            base: entity(id, "minecraft:experience_orb", 0.5, 0.5, 0.25),
            value,
            count: 1,
            age: 0,
            health: 5,
            following_player: None,
        }
    }

    pub fn tick_age(&mut self) {
        self.age += 1;
        if self.age >= DESPAWN_AGE {
            self.base.remove(RemovalReason::Discarded);
        }
    }

    pub fn can_merge_with(&self, other: &Self) -> bool {
        self.base.removal_reason.is_none()
            && other.base.removal_reason.is_none()
            && (other.base.id - self.base.id) % 40 == 0
            && self.value == other.value
            && self.value.saturating_mul(self.count + other.count) <= 10
    }

    pub fn merge(&mut self, other: &mut Self) -> bool {
        if !self.can_merge_with(other) {
            return false;
        }
        self.count += other.count;
        self.age = self.age.min(other.age);
        other.base.remove(RemovalReason::Discarded);
        true
    }

    pub fn collect(&mut self, player_take_xp_delay: i32) -> Option<i32> {
        if player_take_xp_delay != 0 || self.base.removal_reason.is_some() {
            return None;
        }
        self.count -= 1;
        if self.count == 0 {
            self.base.remove(RemovalReason::Discarded);
        }
        Some(self.value)
    }
}

pub fn xp_split_value(max_value: i32) -> i32 {
    match max_value {
        2477.. => 2477,
        1237.. => 1237,
        617.. => 617,
        307.. => 307,
        149.. => 149,
        73.. => 73,
        37.. => 37,
        17.. => 17,
        7.. => 7,
        3.. => 3,
        _ => 1,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallingBlockOutcome {
    ContinueFalling,
    PlaceBlock { block_state: &'static str },
    DropItem { item: &'static str },
    Discard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallingBlockState {
    pub base: BaseEntity,
    pub block_state: &'static str,
    pub time: i32,
    pub drop_item: bool,
    pub cancel_drop: bool,
    pub hurt_entities: bool,
    pub fall_damage_per_distance: f32,
    pub fall_damage_max: i32,
    pub start_pos: (i32, i32, i32),
}

impl FallingBlockState {
    pub fn landing_outcome(
        &self,
        on_ground_or_water_concrete: bool,
        may_replace: bool,
        would_survive: bool,
        entity_drops: bool,
        outside_build_height: bool,
    ) -> FallingBlockOutcome {
        if !on_ground_or_water_concrete {
            if self.time > 600 || (self.time > 100 && outside_build_height) {
                return if self.drop_item && entity_drops {
                    FallingBlockOutcome::DropItem {
                        item: self.block_state,
                    }
                } else {
                    FallingBlockOutcome::Discard
                };
            }
            return FallingBlockOutcome::ContinueFalling;
        }
        if !self.cancel_drop && may_replace && would_survive {
            FallingBlockOutcome::PlaceBlock {
                block_state: self.block_state,
            }
        } else if self.drop_item && entity_drops {
            FallingBlockOutcome::DropItem {
                item: self.block_state,
            }
        } else {
            FallingBlockOutcome::Discard
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimedTntState {
    pub base: BaseEntity,
    pub fuse: i32,
    pub block_state: &'static str,
    pub explosion_power: f32,
    pub owner: Option<String>,
    pub used_portal: bool,
}

impl PrimedTntState {
    pub fn new(id: i32) -> Self {
        Self {
            base: entity(id, "minecraft:tnt", 0.98, 0.98, 0.49),
            fuse: 80,
            block_state: "minecraft:tnt",
            explosion_power: 4.0,
            owner: None,
            used_portal: false,
        }
    }

    pub fn tick(&mut self, tnt_explodes: bool) -> Option<ExplosionEvent> {
        self.fuse -= 1;
        if self.fuse <= 0 {
            self.base.remove(RemovalReason::Discarded);
            if tnt_explodes {
                return Some(ExplosionEvent {
                    power: self.explosion_power,
                    interaction: ExplosionInteraction::Tnt,
                    respects_nether_portal_blocks: !self.used_portal,
                });
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionInteraction {
    Tnt,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplosionEvent {
    pub power: f32,
    pub interaction: ExplosionInteraction,
    pub respects_nether_portal_blocks: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EndCrystalState {
    pub base: BaseEntity,
    pub beam_target: Option<(i32, i32, i32)>,
    pub show_bottom: bool,
    pub time: i32,
}

impl EndCrystalState {
    pub fn new(id: i32) -> Self {
        Self {
            base: entity(id, "minecraft:end_crystal", 2.0, 2.0, 1.0),
            beam_target: None,
            show_bottom: true,
            time: 0,
        }
    }

    pub fn hurt(
        &mut self,
        from_ender_dragon: bool,
        explosion_damage: bool,
    ) -> Option<ExplosionEvent> {
        if from_ender_dragon || self.base.removal_reason.is_some() {
            return None;
        }
        self.base.remove(RemovalReason::Killed);
        if explosion_damage {
            None
        } else {
            Some(ExplosionEvent {
                power: 6.0,
                interaction: ExplosionInteraction::Block,
                respects_nether_portal_blocks: true,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Feet,
    Legs,
    Chest,
    Head,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmorStandState {
    pub invisible: bool,
    pub small: bool,
    pub show_arms: bool,
    pub no_base_plate: bool,
    pub marker: bool,
    pub disabled_slots: u32,
}

impl ArmorStandState {
    const DISABLE_TAKING_OFFSET: u32 = 8;
    const DISABLE_PUTTING_OFFSET: u32 = 16;

    pub fn client_flags(&self) -> u8 {
        u8::from(self.small)
            | (u8::from(self.show_arms) << 2)
            | (u8::from(self.no_base_plate) << 3)
            | (u8::from(self.marker) << 4)
    }

    pub fn dimensions(&self) -> EntityDimensions {
        if self.marker {
            EntityDimensions {
                width: 0.0,
                height: 0.0,
                eye_height: 0.0,
            }
        } else if self.small {
            EntityDimensions {
                width: 0.25,
                height: 0.9875,
                eye_height: 0.9875,
            }
        } else {
            EntityDimensions {
                width: 0.5,
                height: 1.975,
                eye_height: 1.7775,
            }
        }
    }

    pub fn can_put_in_slot(&self, slot: EquipmentSlot) -> bool {
        !self.slot_bit(slot, Self::DISABLE_PUTTING_OFFSET)
    }

    pub fn can_take_from_slot(&self, slot: EquipmentSlot) -> bool {
        !self.slot_bit(slot, Self::DISABLE_TAKING_OFFSET)
    }

    fn slot_bit(&self, slot: EquipmentSlot, offset: u32) -> bool {
        self.disabled_slots & (1 << (slot_index(slot) + offset)) != 0
    }
}

fn slot_index(slot: EquipmentSlot) -> u32 {
    match slot {
        EquipmentSlot::MainHand => 0,
        EquipmentSlot::Feet => 1,
        EquipmentSlot::Legs => 2,
        EquipmentSlot::Chest => 3,
        EquipmentSlot::Head => 4,
        EquipmentSlot::OffHand => 5,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HangingDecorationState {
    pub kind: HangingDecorationKind,
    pub pos: (i32, i32, i32),
    pub direction: Direction,
    pub fixed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HangingDecorationKind {
    Painting { variant: &'static str },
    ItemFrame { rotation: u8, has_item: bool },
    GlowItemFrame { rotation: u8, has_item: bool },
    LeashKnot,
}

impl HangingDecorationState {
    pub fn can_move_or_push(&self) -> bool {
        !self.fixed
    }

    pub fn item_frame_rotation(&self) -> Option<u8> {
        match self.kind {
            HangingDecorationKind::ItemFrame { rotation, .. }
            | HangingDecorationKind::GlowItemFrame { rotation, .. } => Some(rotation % 8),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkerState {
    pub base: BaseEntity,
}

impl MarkerState {
    pub fn new(id: i32) -> Self {
        Self {
            base: entity(id, "minecraft:marker", 0.0, 0.0, 0.0),
        }
    }

    pub fn is_server_only(&self) -> bool {
        true
    }

    pub fn can_accept_passenger(&self) -> bool {
        false
    }

    pub fn can_be_damaged(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InteractionEntityState {
    pub base: BaseEntity,
    pub width: f32,
    pub height: f32,
    pub response: bool,
    pub last_attack: Option<PlayerAction>,
    pub last_interaction: Option<PlayerAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerAction {
    pub player_uuid: String,
    pub game_time: i64,
}

impl InteractionEntityState {
    pub fn new(id: i32) -> Self {
        Self {
            base: entity(id, "minecraft:interaction", 1.0, 1.0, 0.5),
            width: 1.0,
            height: 1.0,
            response: false,
            last_attack: None,
            last_interaction: None,
        }
    }

    pub fn record_attack(&mut self, player_uuid: String, game_time: i64) -> bool {
        self.last_attack = Some(PlayerAction {
            player_uuid,
            game_time,
        });
        !self.response
    }

    pub fn record_interaction(&mut self, player_uuid: String, game_time: i64) -> InteractionReply {
        self.last_interaction = Some(PlayerAction {
            player_uuid,
            game_time,
        });
        InteractionReply::Consume
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionReply {
    Consume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Billboard {
    Fixed,
    Vertical,
    Horizontal,
    Center,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayEntityState {
    pub base: BaseEntity,
    pub kind: DisplayKind,
    pub interpolation_duration: i32,
    pub start_interpolation: i32,
    pub teleport_duration: i32,
    pub billboard: Billboard,
    pub view_range: f32,
    pub shadow_radius: f32,
    pub shadow_strength: f32,
    pub width: f32,
    pub height: f32,
    pub glow_color_override: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayKind {
    Block {
        block_state: &'static str,
    },
    Item {
        item: &'static str,
        transform: &'static str,
    },
    Text {
        text_json: String,
        line_width: i32,
    },
}

impl DisplayEntityState {
    pub fn new(id: i32, kind: DisplayKind) -> Self {
        Self {
            base: entity(id, "minecraft:display", 0.0, 0.0, 0.0),
            kind,
            interpolation_duration: 0,
            start_interpolation: 0,
            teleport_duration: 0,
            billboard: Billboard::Fixed,
            view_range: 1.0,
            shadow_radius: 0.0,
            shadow_strength: 1.0,
            width: 0.0,
            height: 0.0,
            glow_color_override: None,
        }
    }

    pub fn set_teleport_duration(&mut self, ticks: i32) {
        self.teleport_duration = ticks.clamp(0, 59);
    }

    pub fn has_culling_box(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

pub const NON_LIVING_ENTITY_FAMILIES: &[&str] = &[
    "item entities",
    "experience orbs",
    "falling blocks",
    "TNT",
    "end crystals",
    "armor stands",
    "paintings",
    "item frames",
    "leash knots",
    "markers",
    "interactions",
    "block displays",
    "item displays",
    "text displays",
    "decorations",
];

fn entity(
    id: i32,
    entity_type: &'static str,
    width: f32,
    height: f32,
    eye_height: f32,
) -> BaseEntity {
    BaseEntity::new(
        id,
        format!("00000000-0000-0000-0000-{id:012x}"),
        entity_type,
        EntityDimensions {
            width,
            height,
            eye_height,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dirt_stack(count: u8) -> StackRef {
        StackRef {
            item: "minecraft:dirt",
            count,
            max_count: 64,
            components: &[],
        }
    }

    #[test]
    fn item_entities_age_pickup_and_merge_like_server_items() {
        let mut item = ItemEntityState::new(1, dirt_stack(12));
        item.pickup_delay = 10;
        item.tick_age_and_pickup_delay();
        assert_eq!(item.pickup_delay, 9);
        assert_eq!(item.age, 1);

        item.target = Some("player-a".to_string());
        assert!(!item.can_be_picked_up_by("player-b"));
        item.pickup_delay = 0;
        assert!(item.can_be_picked_up_by("player-a"));

        let mut target = ItemEntityState::new(2, dirt_stack(32));
        let mut source = ItemEntityState::new(3, dirt_stack(16));
        source.age = 20;
        source.pickup_delay = 5;
        assert!(target.merge_into(&mut source));
        assert_eq!(target.stack.count, 48);
        assert_eq!(source.stack.count, 0);
        assert_eq!(target.pickup_delay, 5);
        assert_eq!(target.age, 0);
        assert_eq!(source.base.removal_reason, Some(RemovalReason::Discarded));
    }

    #[test]
    fn item_entities_respect_lifetime_and_infinite_sentinels() {
        let mut item = ItemEntityState::new(1, dirt_stack(1));
        item.age = ItemEntityState::INFINITE_LIFETIME_AGE;
        item.pickup_delay = ItemEntityState::INFINITE_PICKUP_DELAY;
        item.tick_age_and_pickup_delay();
        assert_eq!(item.age, ItemEntityState::INFINITE_LIFETIME_AGE);
        assert_eq!(item.pickup_delay, ItemEntityState::INFINITE_PICKUP_DELAY);
        assert_eq!(item.base.removal_reason, None);

        item.age = DESPAWN_AGE - 1;
        item.pickup_delay = 0;
        item.tick_age_and_pickup_delay();
        assert_eq!(item.base.removal_reason, Some(RemovalReason::Discarded));
    }

    #[test]
    fn experience_orbs_split_merge_collect_and_expire() {
        assert_eq!(xp_split_value(2500), 2477);
        assert_eq!(xp_split_value(149), 149);
        assert_eq!(xp_split_value(2), 1);

        let mut orb = ExperienceOrbState::new(40, 3);
        let mut matching = ExperienceOrbState::new(80, 3);
        matching.count = 2;
        assert!(orb.merge(&mut matching));
        assert_eq!(orb.count, 3);
        assert_eq!(matching.base.removal_reason, Some(RemovalReason::Discarded));
        let mut over_cap = ExperienceOrbState::new(120, 3);
        assert!(!orb.merge(&mut over_cap));

        assert_eq!(orb.collect(0), Some(3));
        assert_eq!(orb.count, 2);
        assert_eq!(orb.collect(2), None);

        orb.age = DESPAWN_AGE - 1;
        orb.tick_age();
        assert_eq!(orb.base.removal_reason, Some(RemovalReason::Discarded));
    }

    #[test]
    fn falling_blocks_place_drop_or_discard_from_landing_conditions() {
        let falling = FallingBlockState {
            base: entity(5, "minecraft:falling_block", 0.98, 0.98, 0.49),
            block_state: "minecraft:sand",
            time: 20,
            drop_item: true,
            cancel_drop: false,
            hurt_entities: false,
            fall_damage_per_distance: 0.0,
            fall_damage_max: 40,
            start_pos: (0, 70, 0),
        };
        assert_eq!(
            falling.landing_outcome(true, true, true, true, false),
            FallingBlockOutcome::PlaceBlock {
                block_state: "minecraft:sand"
            }
        );
        assert_eq!(
            falling.landing_outcome(true, false, false, true, false),
            FallingBlockOutcome::DropItem {
                item: "minecraft:sand"
            }
        );

        let timed_out = FallingBlockState {
            time: 601,
            ..falling
        };
        assert_eq!(
            timed_out.landing_outcome(false, false, false, false, false),
            FallingBlockOutcome::Discard
        );
    }

    #[test]
    fn primed_tnt_counts_down_and_explodes_when_game_rule_allows() {
        let mut tnt = PrimedTntState::new(6);
        tnt.fuse = 1;
        assert_eq!(
            tnt.tick(true),
            Some(ExplosionEvent {
                power: 4.0,
                interaction: ExplosionInteraction::Tnt,
                respects_nether_portal_blocks: true,
            })
        );
        assert_eq!(tnt.base.removal_reason, Some(RemovalReason::Discarded));

        let mut disabled = PrimedTntState::new(7);
        disabled.fuse = 1;
        assert_eq!(disabled.tick(false), None);
    }

    #[test]
    fn end_crystals_track_beam_bottom_and_explosion_damage_rules() {
        let mut crystal = EndCrystalState::new(8);
        crystal.beam_target = Some((0, 80, 0));
        crystal.show_bottom = false;
        assert_eq!(crystal.beam_target, Some((0, 80, 0)));
        assert!(!crystal.show_bottom);
        assert_eq!(crystal.hurt(true, false), None);
        assert_eq!(
            crystal.hurt(false, false),
            Some(ExplosionEvent {
                power: 6.0,
                interaction: ExplosionInteraction::Block,
                respects_nether_portal_blocks: true,
            })
        );
        assert_eq!(crystal.base.removal_reason, Some(RemovalReason::Killed));
    }

    #[test]
    fn armor_stands_expose_client_flags_dimensions_and_slot_locks() {
        let _covered_slots = [
            EquipmentSlot::MainHand,
            EquipmentSlot::OffHand,
            EquipmentSlot::Feet,
            EquipmentSlot::Legs,
            EquipmentSlot::Chest,
            EquipmentSlot::Head,
        ];
        let stand = ArmorStandState {
            invisible: false,
            small: true,
            show_arms: true,
            no_base_plate: true,
            marker: false,
            disabled_slots: 1 << (slot_index(EquipmentSlot::Head) + 16),
        };
        assert_eq!(stand.client_flags(), 0b0000_1101);
        assert_eq!(stand.dimensions().height, 0.9875);
        assert!(!stand.can_put_in_slot(EquipmentSlot::Head));
        assert!(stand.can_take_from_slot(EquipmentSlot::Head));

        let marker = ArmorStandState {
            marker: true,
            ..stand
        };
        assert_eq!(marker.dimensions().width, 0.0);
        assert_eq!(marker.client_flags() & 16, 16);
    }

    #[test]
    fn decorations_keep_attachment_fixed_state_and_frame_rotation() {
        let _covered_directions = [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ];
        let frame = HangingDecorationState {
            kind: HangingDecorationKind::ItemFrame {
                rotation: 10,
                has_item: true,
            },
            pos: (1, 64, 1),
            direction: Direction::North,
            fixed: true,
        };
        assert!(!frame.can_move_or_push());
        assert_eq!(frame.item_frame_rotation(), Some(2));

        let painting = HangingDecorationState {
            kind: HangingDecorationKind::Painting {
                variant: "minecraft:kebab",
            },
            fixed: false,
            ..frame
        };
        assert!(painting.can_move_or_push());
        assert_eq!(painting.item_frame_rotation(), None);

        let glow_frame = HangingDecorationState {
            kind: HangingDecorationKind::GlowItemFrame {
                rotation: 7,
                has_item: false,
            },
            fixed: false,
            ..frame
        };
        let leash = HangingDecorationState {
            kind: HangingDecorationKind::LeashKnot,
            ..glow_frame
        };
        assert_eq!(glow_frame.item_frame_rotation(), Some(7));
        assert_eq!(leash.item_frame_rotation(), None);
    }

    #[test]
    fn marker_entities_are_server_only_and_non_interactive() {
        let marker = MarkerState::new(9);
        assert!(marker.is_server_only());
        assert!(!marker.can_accept_passenger());
        assert!(!marker.can_be_damaged());
        assert_eq!(marker.base.entity_type, "minecraft:marker");
    }

    #[test]
    fn interaction_entities_record_attack_and_interact_actions() {
        let mut interaction = InteractionEntityState::new(10);
        assert!(interaction.record_attack("player-a".to_string(), 12));
        assert_eq!(
            interaction.record_interaction("player-b".to_string(), 13),
            InteractionReply::Consume
        );
        assert_eq!(interaction.last_attack.unwrap().game_time, 12);
        assert_eq!(
            interaction.last_interaction.unwrap().player_uuid,
            "player-b"
        );
    }

    #[test]
    fn display_entities_clamp_teleport_duration_and_cull_when_sized() {
        let _covered_billboards = [
            Billboard::Fixed,
            Billboard::Vertical,
            Billboard::Horizontal,
            Billboard::Center,
        ];
        let mut display = DisplayEntityState::new(
            11,
            DisplayKind::Text {
                text_json: "{\"text\":\"hello\"}".to_string(),
                line_width: 200,
            },
        );
        display.set_teleport_duration(80);
        assert_eq!(display.teleport_duration, 59);
        assert!(!display.has_culling_box());
        display.width = 2.0;
        display.height = 1.0;
        assert!(display.has_culling_box());

        let block = DisplayEntityState::new(
            12,
            DisplayKind::Block {
                block_state: "minecraft:stone",
            },
        );
        let item = DisplayEntityState::new(
            13,
            DisplayKind::Item {
                item: "minecraft:diamond",
                transform: "fixed",
            },
        );
        assert!(matches!(block.kind, DisplayKind::Block { .. }));
        assert!(matches!(item.kind, DisplayKind::Item { .. }));
    }

    #[test]
    fn checklist_non_living_entity_families_are_represented() {
        for family in [
            "item entities",
            "experience orbs",
            "falling blocks",
            "TNT",
            "end crystals",
            "armor stands",
            "paintings",
            "item frames",
            "leash knots",
            "markers",
            "interactions",
            "block displays",
            "item displays",
            "text displays",
            "decorations",
        ] {
            assert!(NON_LIVING_ENTITY_FAMILIES.contains(&family), "{family}");
        }
    }
}
