use std::collections::{BTreeMap, BTreeSet};

use crate::registry::Identifier;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityPredicateModel {
    pub entity_type: Option<EntityTypePredicateModel>,
    pub distance_to_player: Option<DistancePredicateModel>,
    pub movement: Option<MovementPredicateModel>,
    pub location: LocationWrapperModel,
    pub effects: Option<MobEffectsPredicateModel>,
    pub nbt: Option<NbtPredicateModel>,
    pub flags: Option<EntityFlagsPredicateModel>,
    pub equipment: Option<EntityEquipmentPredicateModel>,
    pub sub_predicate: Option<EntitySubPredicateModel>,
    pub periodic_tick: Option<i32>,
    pub vehicle: Option<Box<EntityPredicateModel>>,
    pub passenger: Option<Box<EntityPredicateModel>>,
    pub targeted_entity: Option<Box<EntityPredicateModel>>,
    pub team: Option<String>,
    pub slots: Option<SlotsPredicateModel>,
    pub components: DataComponentMatchersModel,
}

impl EntityPredicateModel {
    pub fn builder() -> EntityPredicateBuilder {
        EntityPredicateBuilder::entity()
    }

    pub fn wrap(predicate: EntityPredicateModel) -> ContextAwarePredicateModel {
        ContextAwarePredicateModel {
            condition_count: 1,
            entity_predicate: predicate,
        }
    }

    pub fn wrap_optional(
        predicate: Option<EntityPredicateModel>,
    ) -> Option<ContextAwarePredicateModel> {
        predicate.map(Self::wrap)
    }

    pub fn wrap_builders(builders: Vec<EntityPredicateBuilder>) -> Vec<ContextAwarePredicateModel> {
        builders
            .into_iter()
            .map(|builder| Self::wrap(builder.build()))
            .collect()
    }

    #[allow(clippy::too_many_lines)]
    pub fn matches(
        &self,
        level: &LevelContextModel,
        position: Option<Vec3Model>,
        entity: Option<&EntityContextModel>,
    ) -> bool {
        let Some(entity) = entity else {
            return false;
        };

        if self
            .entity_type
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(entity))
        {
            return false;
        }

        if let Some(distance_to_player) = &self.distance_to_player {
            let Some(position) = position else {
                return false;
            };

            if !distance_to_player.matches(position, entity.position) {
                return false;
            }
        }

        if self.movement.as_ref().is_some_and(|predicate| {
            !predicate.matches(entity.known_movement.scale(20), entity.fall_distance)
        }) {
            return false;
        }

        if self
            .location
            .located
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(level, entity.position))
        {
            return false;
        }

        if let Some(stepping_on) = &self.location.stepping_on {
            if !entity.on_ground || !stepping_on.matches(level, entity.on_pos.center()) {
                return false;
            }
        }

        if self
            .location
            .affects_movement
            .as_ref()
            .is_some_and(|predicate| {
                !predicate.matches(level, entity.movement_affecting_pos.center())
            })
        {
            return false;
        }

        if self
            .effects
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(entity))
        {
            return false;
        }

        if self
            .flags
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(entity))
        {
            return false;
        }

        if self
            .equipment
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(entity))
        {
            return false;
        }

        if self
            .sub_predicate
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(entity, level, position))
        {
            return false;
        }

        if self
            .vehicle
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(level, position, entity.vehicle.as_deref()))
        {
            return false;
        }

        if self.passenger.as_ref().is_some_and(|predicate| {
            entity
                .passengers
                .iter()
                .all(|passenger| !predicate.matches(level, position, Some(passenger)))
        }) {
            return false;
        }

        if self.targeted_entity.as_ref().is_some_and(|predicate| {
            !predicate.matches(
                level,
                position,
                if entity.mob {
                    entity.targeted_entity.as_deref()
                } else {
                    None
                },
            )
        }) {
            return false;
        }

        if self
            .periodic_tick
            .is_some_and(|period| entity.tick_count % period != 0)
        {
            return false;
        }

        if self.team.as_ref().is_some_and(|team| {
            entity
                .team
                .as_ref()
                .is_none_or(|entity_team| entity_team != team)
        }) {
            return false;
        }

        if self
            .slots
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(entity))
        {
            return false;
        }

        if !self.components.test(entity) {
            return false;
        }

        self.nbt
            .as_ref()
            .is_none_or(|predicate| predicate.matches(entity))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityPredicateBuilder {
    entity_type: Option<EntityTypePredicateModel>,
    distance_to_player: Option<DistancePredicateModel>,
    movement: Option<MovementPredicateModel>,
    located: Option<LocationPredicateModel>,
    stepping_on_location: Option<LocationPredicateModel>,
    movement_affected_by: Option<LocationPredicateModel>,
    effects: Option<MobEffectsPredicateModel>,
    nbt: Option<NbtPredicateModel>,
    flags: Option<EntityFlagsPredicateModel>,
    equipment: Option<EntityEquipmentPredicateModel>,
    sub_predicate: Option<EntitySubPredicateModel>,
    periodic_tick: Option<i32>,
    vehicle: Option<EntityPredicateModel>,
    passenger: Option<EntityPredicateModel>,
    targeted_entity: Option<EntityPredicateModel>,
    team: Option<String>,
    slots: Option<SlotsPredicateModel>,
    components: DataComponentMatchersModel,
}

impl EntityPredicateBuilder {
    pub fn entity() -> Self {
        Self::default()
    }

    pub fn of(mut self, entity_type: Identifier) -> Self {
        self.entity_type = Some(EntityTypePredicateModel::entity_type(entity_type));
        self
    }

    pub fn of_tag(mut self, tag: Identifier) -> Self {
        self.entity_type = Some(EntityTypePredicateModel::tag(tag));
        self
    }

    pub fn entity_type(mut self, entity_type: EntityTypePredicateModel) -> Self {
        self.entity_type = Some(entity_type);
        self
    }

    pub fn distance(mut self, distance_to_player: DistancePredicateModel) -> Self {
        self.distance_to_player = Some(distance_to_player);
        self
    }

    pub fn moving(mut self, movement: MovementPredicateModel) -> Self {
        self.movement = Some(movement);
        self
    }

    pub fn located(mut self, location: LocationPredicateBuilder) -> Self {
        self.located = Some(location.build());
        self
    }

    pub fn stepping_on(mut self, location: LocationPredicateBuilder) -> Self {
        self.stepping_on_location = Some(location.build());
        self
    }

    pub fn movement_affected_by(mut self, location: LocationPredicateBuilder) -> Self {
        self.movement_affected_by = Some(location.build());
        self
    }

    pub fn effects(mut self, effects: MobEffectsPredicateModel) -> Self {
        self.effects = Some(effects);
        self
    }

    pub fn nbt(mut self, nbt: NbtPredicateModel) -> Self {
        self.nbt = Some(nbt);
        self
    }

    pub fn flags(mut self, flags: EntityFlagsPredicateModel) -> Self {
        self.flags = Some(flags);
        self
    }

    pub fn equipment(mut self, equipment: EntityEquipmentPredicateModel) -> Self {
        self.equipment = Some(equipment);
        self
    }

    pub fn sub_predicate(mut self, sub_predicate: EntitySubPredicateModel) -> Self {
        self.sub_predicate = Some(sub_predicate);
        self
    }

    pub fn periodic_tick(mut self, period: i32) -> Self {
        self.periodic_tick = Some(period);
        self
    }

    pub fn vehicle(mut self, vehicle: EntityPredicateBuilder) -> Self {
        self.vehicle = Some(vehicle.build());
        self
    }

    pub fn passenger(mut self, passenger: EntityPredicateBuilder) -> Self {
        self.passenger = Some(passenger.build());
        self
    }

    pub fn targeted_entity(mut self, targeted_entity: EntityPredicateBuilder) -> Self {
        self.targeted_entity = Some(targeted_entity.build());
        self
    }

    pub fn team(mut self, team: &str) -> Self {
        self.team = Some(team.to_string());
        self
    }

    pub fn slots(mut self, slots: SlotsPredicateModel) -> Self {
        self.slots = Some(slots);
        self
    }

    pub fn components(mut self, components: DataComponentMatchersModel) -> Self {
        self.components = components;
        self
    }

    pub fn build(self) -> EntityPredicateModel {
        EntityPredicateModel {
            entity_type: self.entity_type,
            distance_to_player: self.distance_to_player,
            movement: self.movement,
            location: LocationWrapperModel {
                located: self.located,
                stepping_on: self.stepping_on_location,
                affects_movement: self.movement_affected_by,
            },
            effects: self.effects,
            nbt: self.nbt,
            flags: self.flags,
            equipment: self.equipment,
            sub_predicate: self.sub_predicate,
            periodic_tick: self.periodic_tick,
            vehicle: self.vehicle.map(Box::new),
            passenger: self.passenger.map(Box::new),
            targeted_entity: self.targeted_entity.map(Box::new),
            team: self.team,
            slots: self.slots,
            components: self.components,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    condition_count: usize,
    entity_predicate: EntityPredicateModel,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocationWrapperModel {
    pub located: Option<LocationPredicateModel>,
    pub stepping_on: Option<LocationPredicateModel>,
    pub affects_movement: Option<LocationPredicateModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityContextModel {
    pub entity_type: Identifier,
    pub type_tags: BTreeSet<Identifier>,
    pub position: Vec3Model,
    pub known_movement: Vec3Model,
    pub fall_distance: i32,
    pub on_ground: bool,
    pub on_pos: BlockPosModel,
    pub movement_affecting_pos: BlockPosModel,
    pub effects: BTreeSet<Identifier>,
    pub flags: BTreeSet<EntityFlagModel>,
    pub equipment: BTreeMap<EquipmentSlotModel, Identifier>,
    pub sub_kind: Option<Identifier>,
    pub tick_count: i32,
    pub team: Option<String>,
    pub slots: BTreeMap<String, Identifier>,
    pub components: BTreeMap<Identifier, String>,
    pub nbt: BTreeMap<String, String>,
    pub vehicle: Option<Box<EntityContextModel>>,
    pub passengers: Vec<EntityContextModel>,
    pub mob: bool,
    pub targeted_entity: Option<Box<EntityContextModel>>,
}

impl EntityContextModel {
    pub fn new(entity_type: Identifier) -> Self {
        Self {
            entity_type,
            type_tags: BTreeSet::new(),
            position: Vec3Model::new(0, 0, 0),
            known_movement: Vec3Model::new(0, 0, 0),
            fall_distance: 0,
            on_ground: false,
            on_pos: BlockPosModel::new(0, 0, 0),
            movement_affecting_pos: BlockPosModel::new(0, 0, 0),
            effects: BTreeSet::new(),
            flags: BTreeSet::new(),
            equipment: BTreeMap::new(),
            sub_kind: None,
            tick_count: 0,
            team: None,
            slots: BTreeMap::new(),
            components: BTreeMap::new(),
            nbt: BTreeMap::new(),
            vehicle: None,
            passengers: Vec::new(),
            mob: false,
            targeted_entity: None,
        }
    }

    pub fn with_position(mut self, position: Vec3Model) -> Self {
        self.position = position;
        self
    }

    pub fn with_type_tag(mut self, tag: Identifier) -> Self {
        self.type_tags.insert(tag);
        self
    }

    pub fn with_known_movement(mut self, movement: Vec3Model) -> Self {
        self.known_movement = movement;
        self
    }

    pub fn with_fall_distance(mut self, fall_distance: i32) -> Self {
        self.fall_distance = fall_distance;
        self
    }

    pub fn on_ground_at(mut self, on_pos: BlockPosModel) -> Self {
        self.on_ground = true;
        self.on_pos = on_pos;
        self
    }

    pub fn with_movement_affecting_pos(mut self, pos: BlockPosModel) -> Self {
        self.movement_affecting_pos = pos;
        self
    }

    pub fn with_effect(mut self, effect: Identifier) -> Self {
        self.effects.insert(effect);
        self
    }

    pub fn with_flag(mut self, flag: EntityFlagModel) -> Self {
        self.flags.insert(flag);
        self
    }

    pub fn with_equipment(mut self, slot: EquipmentSlotModel, item: Identifier) -> Self {
        self.equipment.insert(slot, item);
        self
    }

    pub fn with_sub_kind(mut self, sub_kind: Identifier) -> Self {
        self.sub_kind = Some(sub_kind);
        self
    }

    pub fn with_tick_count(mut self, tick_count: i32) -> Self {
        self.tick_count = tick_count;
        self
    }

    pub fn with_team(mut self, team: &str) -> Self {
        self.team = Some(team.to_string());
        self
    }

    pub fn with_slot(mut self, slot: &str, item: Identifier) -> Self {
        self.slots.insert(slot.to_string(), item);
        self
    }

    pub fn with_component(mut self, component: Identifier, value: &str) -> Self {
        self.components.insert(component, value.to_string());
        self
    }

    pub fn with_nbt(mut self, key: &str, value: &str) -> Self {
        self.nbt.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_vehicle(mut self, vehicle: EntityContextModel) -> Self {
        self.vehicle = Some(Box::new(vehicle));
        self
    }

    pub fn with_passenger(mut self, passenger: EntityContextModel) -> Self {
        self.passengers.push(passenger);
        self
    }

    pub fn with_targeted_entity(mut self, target: EntityContextModel) -> Self {
        self.mob = true;
        self.targeted_entity = Some(Box::new(target));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vec3Model {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3Model {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    fn scale(self, factor: i32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPosModel {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPosModel {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    fn center(self) -> Vec3Model {
        Vec3Model::new(self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LevelContextModel {
    matching_locations: BTreeSet<Vec3Model>,
}

impl LevelContextModel {
    pub fn with_matching_location(mut self, pos: Vec3Model) -> Self {
        self.matching_locations.insert(pos);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityTypePredicateModel {
    Type(Identifier),
    Tag(Identifier),
}

impl EntityTypePredicateModel {
    pub fn entity_type(entity_type: Identifier) -> Self {
        Self::Type(entity_type)
    }

    pub fn tag(tag: Identifier) -> Self {
        Self::Tag(tag)
    }

    fn matches(&self, entity: &EntityContextModel) -> bool {
        match self {
            Self::Type(entity_type) => entity_type == &entity.entity_type,
            Self::Tag(tag) => entity.type_tags.contains(tag),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistancePredicateModel {
    max_absolute: i32,
}

impl DistancePredicateModel {
    pub fn at_most(max_absolute: i32) -> Self {
        Self { max_absolute }
    }

    fn matches(&self, start: Vec3Model, end: Vec3Model) -> bool {
        let dx = (start.x - end.x).abs();
        let dy = (start.y - end.y).abs();
        let dz = (start.z - end.z).abs();
        (dx * dx + dy * dy + dz * dz) <= self.max_absolute * self.max_absolute
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovementPredicateModel {
    velocity: Vec3Model,
    fall_distance: i32,
}

impl MovementPredicateModel {
    pub fn new(velocity: Vec3Model, fall_distance: i32) -> Self {
        Self {
            velocity,
            fall_distance,
        }
    }

    fn matches(&self, velocity: Vec3Model, fall_distance: i32) -> bool {
        self.velocity == velocity && self.fall_distance == fall_distance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocationPredicateModel;

impl LocationPredicateModel {
    pub fn matching() -> Self {
        Self
    }

    fn matches(&self, level: &LevelContextModel, pos: Vec3Model) -> bool {
        level.matching_locations.contains(&pos)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocationPredicateBuilder;

impl LocationPredicateBuilder {
    pub fn location() -> Self {
        Self
    }

    fn build(self) -> LocationPredicateModel {
        LocationPredicateModel::matching()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffectsPredicateModel {
    required_effect: Identifier,
}

impl MobEffectsPredicateModel {
    pub fn requiring(required_effect: Identifier) -> Self {
        Self { required_effect }
    }

    fn matches(&self, entity: &EntityContextModel) -> bool {
        entity.effects.contains(&self.required_effect)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtPredicateModel {
    key: String,
    value: String,
}

impl NbtPredicateModel {
    pub fn requiring(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    fn matches(&self, entity: &EntityContextModel) -> bool {
        entity.nbt.get(&self.key) == Some(&self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityFlagModel {
    OnFire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityFlagsPredicateModel {
    required_flag: EntityFlagModel,
}

impl EntityFlagsPredicateModel {
    pub fn requiring(required_flag: EntityFlagModel) -> Self {
        Self { required_flag }
    }

    fn matches(&self, entity: &EntityContextModel) -> bool {
        entity.flags.contains(&self.required_flag)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EquipmentSlotModel {
    Head,
    Mainhand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityEquipmentPredicateModel {
    slot: EquipmentSlotModel,
    item: Identifier,
}

impl EntityEquipmentPredicateModel {
    pub fn requiring(slot: EquipmentSlotModel, item: Identifier) -> Self {
        Self { slot, item }
    }

    fn matches(&self, entity: &EntityContextModel) -> bool {
        entity.equipment.get(&self.slot) == Some(&self.item)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitySubPredicateModel {
    sub_kind: Identifier,
}

impl EntitySubPredicateModel {
    pub fn requiring(sub_kind: Identifier) -> Self {
        Self { sub_kind }
    }

    fn matches(
        &self,
        entity: &EntityContextModel,
        _level: &LevelContextModel,
        _position: Option<Vec3Model>,
    ) -> bool {
        entity.sub_kind.as_ref() == Some(&self.sub_kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotsPredicateModel {
    slot: String,
    item: Identifier,
}

impl SlotsPredicateModel {
    pub fn requiring(slot: &str, item: Identifier) -> Self {
        Self {
            slot: slot.to_string(),
            item,
        }
    }

    fn matches(&self, entity: &EntityContextModel) -> bool {
        entity.slots.get(&self.slot) == Some(&self.item)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataComponentMatchersModel {
    component: Option<(Identifier, String)>,
}

impl DataComponentMatchersModel {
    pub fn requiring(component: Identifier, value: &str) -> Self {
        Self {
            component: Some((component, value.to_string())),
        }
    }

    fn test(&self, entity: &EntityContextModel) -> bool {
        self.component
            .as_ref()
            .is_none_or(|(component, value)| entity.components.get(component) == Some(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn level() -> LevelContextModel {
        LevelContextModel::default()
            .with_matching_location(Vec3Model::new(10, 64, 10))
            .with_matching_location(Vec3Model::new(1, 63, 1))
            .with_matching_location(Vec3Model::new(2, 62, 2))
    }

    fn base_entity() -> EntityContextModel {
        EntityContextModel::new(id("minecraft:zombie"))
            .with_type_tag(id("minecraft:undead"))
            .with_position(Vec3Model::new(10, 64, 10))
            .with_known_movement(Vec3Model::new(1, 0, 0))
            .with_fall_distance(3)
            .on_ground_at(BlockPosModel::new(1, 63, 1))
            .with_movement_affecting_pos(BlockPosModel::new(2, 62, 2))
            .with_effect(id("minecraft:speed"))
            .with_flag(EntityFlagModel::OnFire)
            .with_equipment(EquipmentSlotModel::Mainhand, id("minecraft:iron_sword"))
            .with_sub_kind(id("minecraft:zombie"))
            .with_tick_count(40)
            .with_team("red")
            .with_slot("weapon.mainhand", id("minecraft:iron_sword"))
            .with_component(id("minecraft:custom_name"), "Dinnerbone")
            .with_nbt("Health", "20")
            .with_vehicle(EntityContextModel::new(id("minecraft:boat")))
            .with_passenger(EntityContextModel::new(id("minecraft:skeleton")))
            .with_targeted_entity(EntityContextModel::new(id("minecraft:villager")))
    }

    #[test]
    fn null_entity_fails_and_empty_predicate_matches_present_entity() {
        let predicate = EntityPredicateModel::builder().build();

        assert!(!predicate.matches(&level(), Some(Vec3Model::new(0, 0, 0)), None));
        assert!(predicate.matches(
            &level(),
            Some(Vec3Model::new(0, 0, 0)),
            Some(&base_entity())
        ));
    }

    #[test]
    fn entity_type_distance_and_movement_follow_java_order() {
        let predicate = EntityPredicateModel::builder()
            .entity_type(EntityTypePredicateModel::tag(id("minecraft:undead")))
            .distance(DistancePredicateModel::at_most(20))
            .moving(MovementPredicateModel::new(Vec3Model::new(20, 0, 0), 3))
            .build();

        assert!(predicate.matches(
            &level(),
            Some(Vec3Model::new(0, 64, 10)),
            Some(&base_entity())
        ));
        assert!(!predicate.matches(&level(), None, Some(&base_entity())));
        assert!(!EntityPredicateModel::builder()
            .of(id("minecraft:skeleton"))
            .distance(DistancePredicateModel::at_most(20))
            .build()
            .matches(
                &level(),
                Some(Vec3Model::new(0, 64, 10)),
                Some(&base_entity())
            ));
    }

    #[test]
    fn location_wrapper_checks_located_stepping_and_movement_affecting_positions() {
        let predicate = EntityPredicateModel::builder()
            .located(LocationPredicateBuilder::location())
            .stepping_on(LocationPredicateBuilder::location())
            .movement_affected_by(LocationPredicateBuilder::location())
            .build();

        assert!(predicate.matches(
            &level(),
            Some(Vec3Model::new(0, 0, 0)),
            Some(&base_entity())
        ));

        let not_on_ground = EntityContextModel {
            on_ground: false,
            ..base_entity()
        };
        assert!(!predicate.matches(
            &level(),
            Some(Vec3Model::new(0, 0, 0)),
            Some(&not_on_ground)
        ));
    }

    #[test]
    fn direct_composed_predicates_match_before_recursive_relationships() {
        let predicate = EntityPredicateModel::builder()
            .effects(MobEffectsPredicateModel::requiring(id("minecraft:speed")))
            .flags(EntityFlagsPredicateModel::requiring(
                EntityFlagModel::OnFire,
            ))
            .equipment(EntityEquipmentPredicateModel::requiring(
                EquipmentSlotModel::Mainhand,
                id("minecraft:iron_sword"),
            ))
            .sub_predicate(EntitySubPredicateModel::requiring(id("minecraft:zombie")))
            .build();

        assert!(predicate.matches(
            &level(),
            Some(Vec3Model::new(0, 0, 0)),
            Some(&base_entity())
        ));
        assert!(!EntityPredicateModel::builder()
            .effects(MobEffectsPredicateModel::requiring(id(
                "minecraft:strength"
            )))
            .vehicle(EntityPredicateModel::builder().of(id("minecraft:boat")))
            .build()
            .matches(
                &level(),
                Some(Vec3Model::new(0, 0, 0)),
                Some(&base_entity())
            ));
    }

    #[test]
    fn recursive_vehicle_passenger_and_targeted_entity_match_like_java() {
        let predicate = EntityPredicateModel::builder()
            .vehicle(EntityPredicateModel::builder().of(id("minecraft:boat")))
            .passenger(EntityPredicateModel::builder().of(id("minecraft:skeleton")))
            .targeted_entity(EntityPredicateModel::builder().of(id("minecraft:villager")))
            .build();

        assert!(predicate.matches(
            &level(),
            Some(Vec3Model::new(0, 0, 0)),
            Some(&base_entity())
        ));

        let non_mob = EntityContextModel {
            mob: false,
            ..base_entity()
        };
        assert!(!predicate.matches(&level(), Some(Vec3Model::new(0, 0, 0)), Some(&non_mob)));
    }

    #[test]
    fn periodic_tick_team_slots_components_and_nbt_are_checked_at_end() {
        let predicate = EntityPredicateModel::builder()
            .periodic_tick(20)
            .team("red")
            .slots(SlotsPredicateModel::requiring(
                "weapon.mainhand",
                id("minecraft:iron_sword"),
            ))
            .components(DataComponentMatchersModel::requiring(
                id("minecraft:custom_name"),
                "Dinnerbone",
            ))
            .nbt(NbtPredicateModel::requiring("Health", "20"))
            .build();

        assert!(predicate.matches(
            &level(),
            Some(Vec3Model::new(0, 0, 0)),
            Some(&base_entity())
        ));
        assert!(!EntityPredicateModel::builder()
            .periodic_tick(30)
            .team("red")
            .build()
            .matches(
                &level(),
                Some(Vec3Model::new(0, 0, 0)),
                Some(&base_entity())
            ));
        assert!(!EntityPredicateModel::builder()
            .components(DataComponentMatchersModel::requiring(
                id("minecraft:custom_name"),
                "Dinnerbone",
            ))
            .nbt(NbtPredicateModel::requiring("Health", "10"))
            .build()
            .matches(
                &level(),
                Some(Vec3Model::new(0, 0, 0)),
                Some(&base_entity())
            ));
    }

    #[test]
    fn builder_preserves_all_java_fields() {
        let predicate = EntityPredicateModel::builder()
            .of_tag(id("minecraft:undead"))
            .distance(DistancePredicateModel::at_most(10))
            .moving(MovementPredicateModel::new(Vec3Model::new(0, 0, 0), 0))
            .located(LocationPredicateBuilder::location())
            .stepping_on(LocationPredicateBuilder::location())
            .movement_affected_by(LocationPredicateBuilder::location())
            .effects(MobEffectsPredicateModel::requiring(id("minecraft:speed")))
            .nbt(NbtPredicateModel::requiring("Health", "20"))
            .flags(EntityFlagsPredicateModel::requiring(
                EntityFlagModel::OnFire,
            ))
            .equipment(EntityEquipmentPredicateModel::requiring(
                EquipmentSlotModel::Head,
                id("minecraft:iron_helmet"),
            ))
            .sub_predicate(EntitySubPredicateModel::requiring(id("minecraft:zombie")))
            .periodic_tick(5)
            .vehicle(EntityPredicateModel::builder().of(id("minecraft:boat")))
            .passenger(EntityPredicateModel::builder().of(id("minecraft:skeleton")))
            .targeted_entity(EntityPredicateModel::builder().of(id("minecraft:villager")))
            .team("red")
            .slots(SlotsPredicateModel::requiring(
                "weapon.mainhand",
                id("minecraft:iron_sword"),
            ))
            .components(DataComponentMatchersModel::requiring(
                id("minecraft:custom_name"),
                "Dinnerbone",
            ))
            .build();

        assert!(predicate.entity_type.is_some());
        assert!(predicate.distance_to_player.is_some());
        assert!(predicate.movement.is_some());
        assert!(predicate.location.located.is_some());
        assert!(predicate.location.stepping_on.is_some());
        assert!(predicate.location.affects_movement.is_some());
        assert!(predicate.effects.is_some());
        assert!(predicate.nbt.is_some());
        assert!(predicate.flags.is_some());
        assert!(predicate.equipment.is_some());
        assert!(predicate.sub_predicate.is_some());
        assert_eq!(predicate.periodic_tick, Some(5));
        assert!(predicate.vehicle.is_some());
        assert!(predicate.passenger.is_some());
        assert!(predicate.targeted_entity.is_some());
        assert_eq!(predicate.team.as_deref(), Some("red"));
        assert!(predicate.slots.is_some());
    }

    #[test]
    fn wrap_helpers_create_context_aware_predicates() {
        let predicate = EntityPredicateModel::builder()
            .of(id("minecraft:zombie"))
            .build();
        let wrapped = EntityPredicateModel::wrap(predicate.clone());
        let optional = EntityPredicateModel::wrap_optional(Some(predicate));
        let list = EntityPredicateModel::wrap_builders(vec![
            EntityPredicateModel::builder().of(id("minecraft:zombie")),
            EntityPredicateModel::builder().of(id("minecraft:skeleton")),
        ]);

        assert_eq!(wrapped.condition_count, 1);
        assert!(optional.is_some());
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].condition_count, 1);
    }
}
