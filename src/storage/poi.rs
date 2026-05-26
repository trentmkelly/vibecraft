#![allow(dead_code)]

use std::collections::BTreeMap;

use super::nbt::Tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoiType {
    pub id: String,
    pub max_tickets: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoiRecord {
    pos: BlockPos,
    poi_type: PoiType,
    free_tickets: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoiSection {
    valid: bool,
    records: BTreeMap<u16, PoiRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Occupancy {
    HasSpace,
    IsOccupied,
    Any,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoiStore {
    sections: BTreeMap<SectionPos, PoiSection>,
}

impl BlockPos {
    pub fn section(self) -> SectionPos {
        SectionPos {
            x: self.x.div_euclid(16),
            y: self.y.div_euclid(16),
            z: self.z.div_euclid(16),
        }
    }

    pub fn dist_sqr(self, other: Self) -> i64 {
        let dx = i64::from(self.x - other.x);
        let dy = i64::from(self.y - other.y);
        let dz = i64::from(self.z - other.z);
        dx * dx + dy * dy + dz * dz
    }
}

fn section_relative_pos(pos: BlockPos) -> u16 {
    let x = pos.x.rem_euclid(16) as u16;
    let y = pos.y.rem_euclid(16) as u16;
    let z = pos.z.rem_euclid(16) as u16;
    (y << 8) | (z << 4) | x
}

impl PoiRecord {
    pub fn new(pos: BlockPos, poi_type: PoiType) -> Self {
        let free_tickets = poi_type.max_tickets;
        Self {
            pos,
            poi_type,
            free_tickets,
        }
    }

    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    pub fn poi_type(&self) -> &PoiType {
        &self.poi_type
    }

    pub fn free_tickets(&self) -> u32 {
        self.free_tickets
    }

    pub fn acquire_ticket(&mut self) -> bool {
        if self.free_tickets == 0 {
            false
        } else {
            self.free_tickets -= 1;
            true
        }
    }

    pub fn release_ticket(&mut self) -> bool {
        if self.free_tickets >= self.poi_type.max_tickets {
            false
        } else {
            self.free_tickets += 1;
            true
        }
    }

    pub fn has_space(&self) -> bool {
        self.free_tickets > 0
    }

    pub fn is_occupied(&self) -> bool {
        self.free_tickets != self.poi_type.max_tickets
    }

    pub fn to_nbt(&self) -> Tag {
        Tag::Compound(vec![
            (
                "pos".to_string(),
                Tag::IntArray(vec![self.pos.x, self.pos.y, self.pos.z]),
            ),
            ("type".to_string(), Tag::String(self.poi_type.id.clone())),
            (
                "free_tickets".to_string(),
                Tag::Int(self.free_tickets as i32),
            ),
            (
                "max_tickets".to_string(),
                Tag::Int(self.poi_type.max_tickets as i32),
            ),
        ])
    }

    pub fn from_nbt(tag: &Tag) -> Result<Self, String> {
        let compound = compound(tag)?;
        let pos = match field(compound, "pos")? {
            Tag::IntArray(values) if values.len() == 3 => BlockPos {
                x: values[0],
                y: values[1],
                z: values[2],
            },
            _ => return Err("POI pos must be an int array of length 3".to_string()),
        };
        let id = match field(compound, "type")? {
            Tag::String(value) => value.clone(),
            _ => return Err("POI type must be a string".to_string()),
        };
        let free_tickets = match field(compound, "free_tickets")? {
            Tag::Int(value) if *value >= 0 => *value as u32,
            _ => return Err("POI free_tickets must be a non-negative int".to_string()),
        };
        let max_tickets = match compound.iter().find(|(name, _)| name == "max_tickets") {
            Some((_, Tag::Int(value))) if *value > 0 => *value as u32,
            _ => free_tickets.max(1),
        };
        Ok(Self {
            pos,
            poi_type: PoiType { id, max_tickets },
            free_tickets,
        })
    }
}

impl PoiSection {
    pub fn new(valid: bool) -> Self {
        Self {
            valid,
            records: BTreeMap::new(),
        }
    }

    pub fn valid(&self) -> bool {
        self.valid
    }

    pub fn add(&mut self, record: PoiRecord) -> bool {
        let key = section_relative_pos(record.pos);
        if let std::collections::btree_map::Entry::Vacant(entry) = self.records.entry(key) {
            entry.insert(record);
            return true;
        }
        false
    }

    pub fn remove(&mut self, pos: BlockPos) -> Option<PoiRecord> {
        self.records.remove(&section_relative_pos(pos))
    }

    pub fn get(&self, pos: BlockPos) -> Option<&PoiRecord> {
        self.records.get(&section_relative_pos(pos))
    }

    pub fn get_mut(&mut self, pos: BlockPos) -> Option<&mut PoiRecord> {
        self.records.get_mut(&section_relative_pos(pos))
    }

    pub fn records(&self, occupancy: Occupancy) -> Vec<&PoiRecord> {
        self.records
            .values()
            .filter(|record| occupancy.matches(record))
            .collect()
    }

    pub fn refresh(&mut self, records: Vec<PoiRecord>) {
        if self.valid {
            return;
        }
        self.records.clear();
        for record in records {
            self.add(record);
        }
        self.valid = true;
    }

    pub fn to_nbt(&self) -> Tag {
        Tag::Compound(vec![
            (
                "Valid".to_string(),
                Tag::Byte(if self.valid { 1 } else { 0 }),
            ),
            (
                "Records".to_string(),
                Tag::List(self.records.values().map(PoiRecord::to_nbt).collect()),
            ),
        ])
    }

    pub fn from_nbt(tag: &Tag) -> Result<Self, String> {
        let compound = compound(tag)?;
        let valid = match field(compound, "Valid") {
            Ok(Tag::Byte(value)) => *value != 0,
            _ => false,
        };
        let records = match field(compound, "Records")? {
            Tag::List(records) => records,
            _ => return Err("POI Records must be a list".to_string()),
        };
        let mut section = Self::new(valid);
        for record in records {
            section.add(PoiRecord::from_nbt(record)?);
        }
        Ok(section)
    }
}

impl Occupancy {
    fn matches(self, record: &PoiRecord) -> bool {
        match self {
            Self::HasSpace => record.has_space(),
            Self::IsOccupied => record.is_occupied(),
            Self::Any => true,
        }
    }
}

impl PoiStore {
    pub fn add(&mut self, record: PoiRecord) -> bool {
        self.sections
            .entry(record.pos.section())
            .or_insert_with(|| PoiSection::new(true))
            .add(record)
    }

    pub fn remove(&mut self, pos: BlockPos) -> Option<PoiRecord> {
        self.sections
            .get_mut(&pos.section())
            .and_then(|section| section.remove(pos))
    }

    pub fn acquire(&mut self, pos: BlockPos) -> bool {
        self.sections
            .get_mut(&pos.section())
            .and_then(|section| section.get_mut(pos))
            .is_some_and(PoiRecord::acquire_ticket)
    }

    pub fn release(&mut self, pos: BlockPos) -> bool {
        self.sections
            .get_mut(&pos.section())
            .and_then(|section| section.get_mut(pos))
            .is_some_and(PoiRecord::release_ticket)
    }

    pub fn get_in_range(
        &self,
        center: BlockPos,
        radius: i32,
        occupancy: Occupancy,
    ) -> Vec<&PoiRecord> {
        let radius_sqr = i64::from(radius) * i64::from(radius);
        self.sections
            .values()
            .flat_map(|section| section.records(occupancy))
            .filter(|record| record.pos.dist_sqr(center) <= radius_sqr)
            .collect()
    }
}

fn compound(tag: &Tag) -> Result<&[(String, Tag)], String> {
    match tag {
        Tag::Compound(values) => Ok(values),
        _ => Err("expected NBT compound".to_string()),
    }
}

fn field<'a>(compound: &'a [(String, Tag)], name: &str) -> Result<&'a Tag, String> {
    compound
        .iter()
        .find(|(field_name, _)| field_name == name)
        .map(|(_, value)| value)
        .ok_or_else(|| format!("missing NBT field {name}"))
}

#[cfg(test)]
mod tests {
    use super::{BlockPos, Occupancy, PoiRecord, PoiSection, PoiStore, PoiType, SectionPos};

    fn bed() -> PoiType {
        PoiType {
            id: "minecraft:home".to_string(),
            max_tickets: 1,
        }
    }

    #[test]
    fn poi_record_ticket_semantics_match_vanilla() {
        let mut record = PoiRecord::new(BlockPos { x: 1, y: 2, z: 3 }, bed());
        assert!(record.has_space());
        assert!(!record.is_occupied());
        assert!(record.acquire_ticket());
        assert!(!record.has_space());
        assert!(record.is_occupied());
        assert!(!record.acquire_ticket());
        assert!(record.release_ticket());
        assert!(!record.release_ticket());
    }

    #[test]
    fn poi_section_uses_section_relative_position_keys() {
        let mut section = PoiSection::new(true);
        let pos = BlockPos {
            x: -1,
            y: 17,
            z: 31,
        };
        assert!(section.add(PoiRecord::new(pos, bed())));
        assert!(!section.add(PoiRecord::new(pos, bed())));
        assert!(section.get(pos).is_some());
        assert!(section.remove(pos).is_some());
        assert!(section.get(pos).is_none());
    }

    #[test]
    fn poi_store_queries_range_and_occupancy() {
        let mut store = PoiStore::default();
        let near = BlockPos { x: 0, y: 64, z: 0 };
        let far = BlockPos { x: 20, y: 64, z: 0 };
        store.add(PoiRecord::new(near, bed()));
        store.add(PoiRecord::new(far, bed()));
        assert!(store.acquire(near));

        assert_eq!(near.section(), SectionPos { x: 0, y: 4, z: 0 });
        assert_eq!(store.get_in_range(near, 6, Occupancy::Any).len(), 1);
        assert_eq!(
            store
                .get_in_range(near, 32, Occupancy::IsOccupied)
                .into_iter()
                .map(PoiRecord::pos)
                .collect::<Vec<_>>(),
            vec![near]
        );
        assert_eq!(store.get_in_range(near, 32, Occupancy::HasSpace).len(), 1);
        assert!(store.release(near));
    }

    #[test]
    fn poi_section_round_trips_vanilla_nbt_shape() {
        let mut section = PoiSection::new(true);
        let pos = BlockPos { x: 7, y: 8, z: 9 };
        section.add(PoiRecord::new(pos, bed()));

        let decoded = PoiSection::from_nbt(&section.to_nbt()).unwrap();

        assert!(decoded.valid());
        assert_eq!(decoded.get(pos).unwrap().poi_type().id, "minecraft:home");
        assert_eq!(decoded.get(pos).unwrap().free_tickets(), 1);
    }
}
