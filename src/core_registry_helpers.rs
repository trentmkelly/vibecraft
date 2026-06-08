use std::collections::BTreeMap;
use std::fmt;

use crate::core_misc::Vec3iModel;

pub const ID_MAP_DEFAULT: i32 = -1;

pub trait IdMapModel<T> {
    fn get_id(&self, thing: &T) -> i32;
    fn by_id(&self, id: i32) -> Option<&T>;
    fn size(&self) -> usize;

    fn by_id_or_throw(&self, id: i32) -> Result<&T, String> {
        self.by_id(id)
            .ok_or_else(|| format!("No value with id {id}"))
    }

    fn get_id_or_throw(&self, value: &T) -> Result<i32, String>
    where
        T: fmt::Display,
        Self: fmt::Display,
    {
        let id = self.get_id(value);
        if id == ID_MAP_DEFAULT {
            Err(format!("Can't find id for '{value}' in map {self}"))
        } else {
            Ok(id)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdMappedValue {
    identity: usize,
    label: String,
}

impl IdMappedValue {
    pub fn new(identity: usize, label: impl Into<String>) -> Self {
        Self {
            identity,
            label: label.into(),
        }
    }

    pub fn identity(&self) -> usize {
        self.identity
    }
}

impl fmt::Display for IdMappedValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdMapperModel {
    next_id: i32,
    value_to_id: BTreeMap<usize, i32>,
    id_to_value: Vec<Option<IdMappedValue>>,
}

impl IdMapperModel {
    pub fn new() -> Self {
        Self::with_expected_size(512)
    }

    pub fn with_expected_size(expected_size: usize) -> Self {
        Self {
            next_id: 0,
            value_to_id: BTreeMap::new(),
            id_to_value: Vec::with_capacity(expected_size),
        }
    }

    pub fn add_mapping(&mut self, thing: IdMappedValue, id: i32) {
        self.value_to_id.insert(thing.identity(), id);
        while self.id_to_value.len() <= id as usize {
            self.id_to_value.push(None);
        }
        self.id_to_value[id as usize] = Some(thing);
        if self.next_id <= id {
            self.next_id = id + 1;
        }
    }

    pub fn add(&mut self, thing: IdMappedValue) {
        self.add_mapping(thing, self.next_id);
    }

    pub fn contains(&self, id: i32) -> bool {
        self.by_id(id).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = &IdMappedValue> {
        self.id_to_value.iter().filter_map(Option::as_ref)
    }

    pub fn next_id(&self) -> i32 {
        self.next_id
    }
}

impl Default for IdMapperModel {
    fn default() -> Self {
        Self::new()
    }
}

impl IdMapModel<IdMappedValue> for IdMapperModel {
    fn get_id(&self, thing: &IdMappedValue) -> i32 {
        self.value_to_id
            .get(&thing.identity())
            .copied()
            .unwrap_or(ID_MAP_DEFAULT)
    }

    fn by_id(&self, id: i32) -> Option<&IdMappedValue> {
        if id < 0 {
            return None;
        }
        self.id_to_value.get(id as usize).and_then(Option::as_ref)
    }

    fn size(&self) -> usize {
        self.value_to_id.len()
    }
}

impl fmt::Display for IdMapperModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("IdMapperModel")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPosModel {
    dimension: String,
    pos: Vec3iModel,
}

impl GlobalPosModel {
    pub const MAP_CODEC_FIELDS: [&'static str; 2] = ["dimension", "pos"];
    pub const STREAM_CODEC_FIELDS: [&'static str; 2] = ["dimension", "pos"];

    pub fn of(dimension: impl Into<String>, pos: Vec3iModel) -> Self {
        Self {
            dimension: dimension.into(),
            pos,
        }
    }

    pub fn dimension(&self) -> &str {
        &self.dimension
    }

    pub fn pos(&self) -> Vec3iModel {
        self.pos
    }

    pub fn is_close_enough(&self, dimension: &str, pos: Vec3iModel, max_distance: i32) -> bool {
        self.dimension == dimension && self.pos.dist_chessboard(pos) <= max_distance
    }
}

impl fmt::Display for GlobalPosModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} BlockPos{{x={}, y={}, z={}}}",
            self.dimension,
            self.pos.get(crate::core_orientation::CoreAxisModel::X),
            self.pos.get(crate::core_orientation::CoreAxisModel::Y),
            self.pos.get(crate::core_orientation::CoreAxisModel::Z)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HolderOwnerModel {
    identity: usize,
}

impl HolderOwnerModel {
    pub fn new(identity: usize) -> Self {
        Self { identity }
    }

    pub fn can_serialize_in(self, context: Self) -> bool {
        context == self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(identity: usize, label: &str) -> IdMappedValue {
        IdMappedValue::new(identity, label)
    }

    #[test]
    fn id_map_defaults_throw_with_java_messages() {
        let mut mapper = IdMapperModel::new();
        mapper.add_mapping(value(1, "stone"), 4);

        assert_eq!(ID_MAP_DEFAULT, -1);
        assert_eq!(mapper.by_id_or_throw(4).unwrap().to_string(), "stone");
        assert_eq!(mapper.by_id_or_throw(3).unwrap_err(), "No value with id 3");
        assert_eq!(mapper.get_id_or_throw(&value(1, "stone")).unwrap(), 4);
        assert_eq!(
            mapper.get_id_or_throw(&value(2, "dirt")).unwrap_err(),
            "Can't find id for 'dirt' in map IdMapperModel"
        );
    }

    #[test]
    fn id_mapper_uses_reference_identity_not_value_equality() {
        let mut mapper = IdMapperModel::with_expected_size(1);
        let first_stone = value(10, "stone");
        let equal_label_different_identity = value(11, "stone");

        mapper.add_mapping(first_stone.clone(), 2);
        assert_eq!(mapper.get_id(&first_stone), 2);
        assert_eq!(
            mapper.get_id(&equal_label_different_identity),
            ID_MAP_DEFAULT
        );
        assert_eq!(mapper.by_id(2), Some(&first_stone));
        assert!(mapper.by_id(-1).is_none());
        assert!(mapper.by_id(3).is_none());
        assert!(mapper.contains(2));
        assert!(!mapper.contains(3));
        assert_eq!(mapper.size(), 1);
        assert_eq!(mapper.next_id(), 3);
    }

    #[test]
    fn id_mapper_adds_at_next_id_and_filters_empty_slots() {
        let mut mapper = IdMapperModel::new();
        mapper.add_mapping(value(1, "stone"), 4);
        mapper.add(value(2, "dirt"));
        mapper.add_mapping(value(3, "granite"), 1);

        assert_eq!(mapper.next_id(), 6);
        assert_eq!(mapper.by_id(0), None);
        assert_eq!(mapper.by_id(1).unwrap().to_string(), "granite");
        assert_eq!(mapper.by_id(4).unwrap().to_string(), "stone");
        assert_eq!(mapper.by_id(5).unwrap().to_string(), "dirt");
        assert_eq!(
            mapper.iter().map(ToString::to_string).collect::<Vec<_>>(),
            vec!["granite", "stone", "dirt"]
        );
    }

    #[test]
    fn id_mapper_overwrites_id_slot_without_removing_old_reverse_mapping() {
        let mut mapper = IdMapperModel::new();
        let old = value(1, "old");
        let new = value(2, "new");
        mapper.add_mapping(old.clone(), 7);
        mapper.add_mapping(new.clone(), 7);

        assert_eq!(mapper.by_id(7), Some(&new));
        assert_eq!(mapper.get_id(&old), 7);
        assert_eq!(mapper.get_id(&new), 7);
        assert_eq!(mapper.size(), 2);
        assert_eq!(
            mapper.iter().map(ToString::to_string).collect::<Vec<_>>(),
            vec!["new"]
        );
    }

    #[test]
    fn global_pos_matches_factory_codecs_string_and_distance_logic() {
        let pos = GlobalPosModel::of("minecraft:overworld", Vec3iModel::new(10, 64, -3));
        assert_eq!(GlobalPosModel::MAP_CODEC_FIELDS, ["dimension", "pos"]);
        assert_eq!(GlobalPosModel::STREAM_CODEC_FIELDS, ["dimension", "pos"]);
        assert_eq!(pos.dimension(), "minecraft:overworld");
        assert_eq!(pos.pos(), Vec3iModel::new(10, 64, -3));
        assert_eq!(
            pos.to_string(),
            "minecraft:overworld BlockPos{x=10, y=64, z=-3}"
        );
        assert!(pos.is_close_enough("minecraft:overworld", Vec3iModel::new(12, 60, -3), 4));
        assert!(!pos.is_close_enough("minecraft:overworld", Vec3iModel::new(15, 64, -3), 4));
        assert!(!pos.is_close_enough("minecraft:the_nether", Vec3iModel::new(10, 64, -3), 4));
    }

    #[test]
    fn holder_owner_serializes_only_in_same_context_identity() {
        let owner = HolderOwnerModel::new(1);
        let same = HolderOwnerModel::new(1);
        let other = HolderOwnerModel::new(2);

        assert!(owner.can_serialize_in(same));
        assert!(!owner.can_serialize_in(other));
    }
}
